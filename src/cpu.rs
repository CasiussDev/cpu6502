use crate::instr;
use crate::instr::InstructionSequenceMode;
use crate::pinout::Pinout;
use crate::registers::{IndexRegister, RegisterFile, SelectedRegister8, StatusRegFlags};
use std::slice::Iter;

#[derive(PartialEq, Debug)]
enum WaitingInterrupt {
    NonMaskableInterrupt,
    Interrupt,
}

#[derive(Default, Debug)]
pub struct Cpu {
    regs: RegisterFile,
    pins: Pinout,
    current_sequence: Option<Iter<'static, instr::MicroInstruction>>,
    current_op: Option<Iter<'static, instr::MicroInstruction>>,
    data_destination: Option<SelectedRegister8>,
    index_register: Option<IndexRegister>,
    waiting_interrupt: Option<WaitingInterrupt>,
    cycle_count_since_reset: u64,
    instr_count_since_reset: u64,
    instr_ready: bool,
    running_op: bool,
}

#[derive(PartialEq, Eq, Debug)]
pub enum YieldStatus {
    ClockFinished,
    WaitingMemory,
}

impl Cpu {
    pub fn new() -> Self {
        let mut cpu = Self::default();
        cpu.reset();
        cpu
    }

    pub fn get_cycle_count_since_reset(&self) -> u64 {
        self.cycle_count_since_reset
    }

    pub fn get_instr_count_since_reset(&self) -> u64 {
        self.instr_count_since_reset
    }

    pub fn read_data_pins(&self) -> u8 {
        self.pins.get_data()
    }

    pub fn read_address_pins(&self) -> u16 {
        self.pins.get_address()
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

        self.current_sequence = instr::get_sequences_map()
            .get(&InstructionSequenceMode::Reset)
            .map(|v| v.iter());

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
        } else {
            self.current_sequence = None;
            run_status = instr::ExecutionStatus::Continue;
        }

        run_status
    }

    pub fn run(&mut self) -> YieldStatus {
        if self.current_sequence.is_none() && self.waiting_interrupt.is_some() {
            self.service_interrupt();
        } else if self.instr_ready {
            self.decode_instr();
        }

        let mut run_status = instr::ExecutionStatus::Continue;

        while run_status == instr::ExecutionStatus::Continue {
            if self.running_op {
                run_status = self.run_op();
            } else if self.current_sequence.is_some() {
                run_status = self.run_sequence();
            } else {
                run_status = instr::execute(
                    instr::MicroInstruction::Fetch,
                    None,
                    &mut self.regs,
                    &mut self.pins,
                );
            }

            match run_status {
                instr::ExecutionStatus::YieldClock => {
                    self.waiting_interrupt = self.is_waiting_interrupt();
                    self.cycle_count_since_reset += 1;
                }
                instr::ExecutionStatus::Continue => {}
                instr::ExecutionStatus::RunOp => {
                    self.running_op = true;
                    run_status = instr::ExecutionStatus::Continue;
                }
                instr::ExecutionStatus::WaitMemory { dst } => {
                    self.data_destination = dst;
                    return YieldStatus::WaitingMemory;
                }
                instr::ExecutionStatus::RunOpAndFinish => {}
                instr::ExecutionStatus::FinishInstruction => {
                    self.current_sequence = None;
                    self.current_op = None;
                    self.cycle_count_since_reset += 1;
                    self.instr_count_since_reset += 1;
                }
            };
        }

        YieldStatus::ClockFinished
    }

    fn decode_instr(&mut self) {
        let opcode = self.regs.ir.get_u8();
        let decoded_intr = instr::decode(opcode);

        self.current_sequence = instr::get_sequences_map()
            .get(&decoded_intr.sequence)
            .map(|v| v.iter());

        self.current_op = instr::get_ops_map()
            .get(&decoded_intr.operation)
            .map(|v| v.iter());

        self.index_register = decoded_intr.index;

        self.instr_ready = false;
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

        self.current_sequence = instr::get_sequences_map()
            .get(&sequence_mode)
            .map(|v| v.iter());

        self.current_op = None;

        self.index_register = None;

        self.waiting_interrupt = None;
    }
}
