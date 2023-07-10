use crate::instr;
use crate::pinout::{DataDirectionMode, Pinout};
use crate::registers::{IndexRegister, RegisterFile, SelectedRegister8, StatusRegFlags};
use std::{slice, time};

#[cfg(feature = "logging")]
use log::{debug, trace};

#[derive(PartialEq, Debug)]
enum WaitingInterrupt {
    NonMaskableInterrupt,
    Interrupt,
}

#[derive(PartialEq, Debug)]
enum ClockHalf {
    BeforeMemory,
    AfterMemory,
}

#[derive(Default, Debug)]
pub struct Cpu {
    regs: RegisterFile,
    pins: Pinout,
    current_sequence: Option<slice::Iter<'static, instr::MicroInstruction>>,
    current_op: Option<slice::Iter<'static, instr::MicroInstruction>>,
    data_destination: Option<SelectedRegister8>,
    index_register: Option<IndexRegister>,
    waiting_interrupt: Option<WaitingInterrupt>,
    cycle_count_since_reset: u128,
    instr_count_since_reset: u128,
    clock_half: ClockHalf,
    instr_ready: bool,
    running_op: bool,

    #[cfg(feature = "integration_test")]
    has_decoded: bool,
}

#[derive(PartialEq, Eq, Debug)]
pub enum YieldStatus {
    ClockFinished,
    WaitingMemory,
}

impl Default for ClockHalf {
    fn default() -> Self {
        ClockHalf::BeforeMemory
    }
}

impl Cpu {
    pub fn new() -> Self {
        let mut cpu = Self::default();
        cpu.reset();

        let _ = instr::sequence_for_mode(instr::InstructionSequenceMode::FetchInstr);
        let _ = instr::sequence_for_op(instr::InstructionOp::Add);

        cpu
    }

    pub fn cycle_count_since_reset(&self) -> u128 {
        self.cycle_count_since_reset
    }

    pub fn instr_count_since_reset(&self) -> u128 {
        self.instr_count_since_reset
    }

    #[cfg(feature = "integration_test")]
    pub fn instr_ready(&self) -> bool {
        self.instr_ready
    }

    #[cfg(feature = "integration_test")]
    pub fn has_decoded(&self) -> bool {
        self.has_decoded
    }

    #[cfg(feature = "integration_test")]
    pub fn regs_as_log_line(&self) -> String {
        self.regs.as_log_line()
    }

    pub fn read_data_pins(&self) -> u8 {
        self.pins.data()
    }

    pub fn read_address_pins(&self) -> u16 {
        self.pins.address()
    }

    pub fn read_writing_to_memory_pin(&self) -> bool {
        self.pins.data_direction() == DataDirectionMode::Write
    }

    pub fn set_data_pins(&mut self, value: u8) {
        self.pins.set_data_input(value);
        let data_destination = self.data_destination.expect("no data destination set");
        self.regs.set_selected_register8(data_destination, value);
        if data_destination == SelectedRegister8::IR {
            self.instr_ready = true;
        }
    }

    pub fn set_nmi_pin(&mut self) {
        self.pins.set_nmi_input();
    }

    pub fn clear_nmi_pin(&mut self) {
        self.pins.clear_nmi_input();
    }

    pub fn set_irq_pin(&mut self) {
        self.pins.set_irq_input();
    }

    pub fn clear_irq_pin(&mut self) {
        self.pins.clear_irq_input();
    }

    pub fn reset(&mut self) {
        *self = Default::default();
        self.regs.reset();
        self.pins.reset();

        self.current_sequence = Some(instr::sequence_for_mode(
            instr::InstructionSequenceMode::Reset,
        ));

        assert!(
            self.current_sequence.is_some(),
            "There is no reset sequence"
        );

        self.cycle_count_since_reset = 0;
        self.instr_count_since_reset = 0;
    }

    fn is_waiting_interrupt(&self) -> Option<WaitingInterrupt> {
        if self.pins.is_nmi_set() {
            Some(WaitingInterrupt::NonMaskableInterrupt)
        } else if self.pins.is_irq_set()
            && (self.waiting_interrupt != Some(WaitingInterrupt::NonMaskableInterrupt))
            && (self
            .regs
            .status
            .are_any_flags_set(StatusRegFlags::IRQ_DISABLE)
            == false)
        {
            Some(WaitingInterrupt::Interrupt)
        } else {
            None
        }
    }

    pub fn run_op(&mut self) -> instr::ExecutionStatus {
        let mut run_status = instr::ExecutionStatus::Continue;

        if let Some(op) = &mut self.current_op {
            if let Some(&micro_instr) = op.next() {
                run_status = instr::execute(
                    micro_instr,
                    self.index_register,
                    &mut self.regs,
                    &mut self.pins,
                );
                #[cfg(feature = "logging")]
                {
                    trace!("{}", self.regs.as_log_line());
                }
            } else {
                self.running_op = false;
                self.current_op = None;
            }
        } else {
            self.running_op = false;
            self.current_op = None;
        }

        run_status
    }

    pub fn run_sequence(&mut self) -> instr::ExecutionStatus {
        let run_status;
        let sequence = &mut self.current_sequence.as_mut().unwrap();
        if let Some(&micro_instr) = sequence.next() {
            run_status = instr::execute(
                micro_instr,
                self.index_register,
                &mut self.regs,
                &mut self.pins,
            );
            #[cfg(feature = "logging")]
            {
                trace!("{}", self.regs.as_log_line());
            }
        } else {
            self.current_sequence = None;
            run_status = instr::ExecutionStatus::Continue;
        }

        run_status
    }

    pub fn run(&mut self) -> YieldStatus {
        #[cfg(feature = "integration_test")]
        {
            self.has_decoded = false;
        }

        if self.current_sequence.is_none() {
            if self.waiting_interrupt.is_some() {
                self.service_interrupt();
            } else if self.instr_ready && (self.clock_half == ClockHalf::BeforeMemory) {
                self.decode_instr();
            }
        }

        let mut run_status = instr::ExecutionStatus::Continue;

        while run_status == instr::ExecutionStatus::Continue {
            if self.running_op {
                run_status = self.run_op();
            } else if self.current_sequence.is_some() {
                run_status = self.run_sequence();
            } else {
                self.current_sequence = Some(instr::sequence_for_mode(
                    instr::InstructionSequenceMode::FetchInstr,
                ));
                run_status = self.run_sequence();
            }

            match run_status {
                instr::ExecutionStatus::YieldClock => {
                    self.waiting_interrupt = self.is_waiting_interrupt();
                    self.cycle_count_since_reset += 1;
                    self.clock_half = ClockHalf::BeforeMemory;
                    #[cfg(feature = "logging")]
                    {
                        trace!("--------------");
                    }
                }
                instr::ExecutionStatus::Continue => {}
                instr::ExecutionStatus::RunOp => {
                    self.running_op = true;
                    run_status = instr::ExecutionStatus::Continue;
                }
                instr::ExecutionStatus::WaitMemory { dst } => {
                    self.data_destination = dst;
                    self.clock_half = ClockHalf::AfterMemory;

                    return YieldStatus::WaitingMemory;
                }
                instr::ExecutionStatus::RunOpAndFinish => {
                    self.running_op = true;
                    self.current_sequence = Some(instr::finish_instr_sequence());
                }
                instr::ExecutionStatus::FinishInstruction => {
                    self.current_sequence = None;
                    self.current_op = None;
                    self.running_op = false;
                    self.cycle_count_since_reset += 1;
                    self.clock_half = ClockHalf::BeforeMemory;
                    #[cfg(feature = "logging")]
                    {
                        trace!("--------------");
                        trace!("--------------");
                    }
                }
                instr::ExecutionStatus::FinishInstructionBranch => {
                    self.current_sequence = None;
                    self.current_op = None;
                    self.running_op = false;
                    self.cycle_count_since_reset += 1;
                    self.clock_half = ClockHalf::BeforeMemory;
                    self.instr_ready = false;
                    #[cfg(feature = "logging")]
                    {
                        trace!("--------------");
                        trace!("--------------");
                    }
                }
            };
        }

        YieldStatus::ClockFinished
    }

    fn decode_instr(&mut self) {
        let mut _decode_start: time::Instant;

        let opcode = self.regs.ir.to_u8();
        let decoded_instr = instr::decode(opcode);

        self.current_sequence = Some(instr::sequence_for_mode(decoded_instr.sequence));

        self.current_op = Some(instr::sequence_for_op(decoded_instr.operation));

        self.index_register = decoded_instr.index;

        self.instr_ready = false;

        self.instr_count_since_reset += 1;

        #[cfg(feature = "integration_test")]
        {
            self.has_decoded = true;
        }

        #[cfg(feature = "logging")]
        {
            debug!("{:?}", decoded_instr);
        }
    }

    fn service_interrupt(&mut self) {
        let sequence_mode = match self.waiting_interrupt {
            None => {
                panic!("service_interrupt was called witout any waiting interrupt");
            }
            Some(WaitingInterrupt::NonMaskableInterrupt) => {
                instr::InstructionSequenceMode::StartNmi
            }
            Some(WaitingInterrupt::Interrupt) => instr::InstructionSequenceMode::StartIrq,
        };

        self.current_sequence = Some(instr::sequence_for_mode(sequence_mode));

        self.current_op = None;

        self.index_register = None;

        self.waiting_interrupt = None;
    }
}