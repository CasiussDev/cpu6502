use crate::interrupts::Interrupts;
use crate::memory::MemorySpace;
use crate::registers::{IndexRegister, RegisterFile, SelectedRegister8, StatusRegFlags};
use crate::{instr, MicroInstruction};
use std::{slice, time};

use crate::instr::FetchedInstr;

#[cfg(feature = "logging")]
use crate::cpu::logging_memory::LoggingMemory;
#[cfg(feature = "logging")]
use log::{debug, trace};

#[derive(PartialEq, Eq, Debug)]
enum WaitingInterrupt {
    NonMaskableInterrupt,
    Interrupt,
}

#[derive(Default, Debug)]
pub struct Cpu {
    regs: RegisterFile,
    pins: Interrupts,
    current_sequence: Option<slice::Iter<'static, instr::MicroInstruction>>,
    current_op: Option<slice::Iter<'static, instr::MicroInstruction>>,
    index_register: Option<IndexRegister>,
    waiting_interrupt: Option<WaitingInterrupt>,
    cycle_count_since_reset: u128,
    instr_count_since_reset: u128,
    fetched_instr: FetchedInstr,
    running_op: bool,
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
    pub fn fetched_instr(&self) -> FetchedInstr {
        self.fetched_instr
    }

    #[cfg(feature = "integration_test")]
    pub fn regs_as_log_line(&self) -> String {
        self.regs.as_log_line()
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

    pub fn run_op(&mut self, memory: &mut impl MemorySpace) -> instr::ExecutionStatus {
        let mut run_status = instr::ExecutionStatus::Continue;

        if let Some(op) = &mut self.current_op {
            if let Some(&micro_instr) = op.next() {
                let fetched_instr;
                (run_status, fetched_instr) =
                    instr::execute(micro_instr, self.index_register, &mut self.regs, memory);
                #[cfg(feature = "logging")]
                {
                    trace!("{}", self.regs.as_log_line());
                }
                if fetched_instr == FetchedInstr::Invalidate {
                    self.fetched_instr = FetchedInstr::None;
                    if self.current_sequence.is_some() {
                        self.current_sequence = None;
                        self.instr_count_since_reset -= 1;
                    }
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

    pub fn run_sequence(&mut self, memory: &mut impl MemorySpace) -> instr::ExecutionStatus {
        let run_status;
        let sequence = &mut self.current_sequence.as_mut().unwrap();
        if let Some(&micro_instr) = sequence.next() {
            let fetched_instr;
            (run_status, fetched_instr) =
                instr::execute(micro_instr, self.index_register, &mut self.regs, memory);
            #[cfg(feature = "logging")]
            {
                trace!("{}", self.regs.as_log_line());
            }
            if micro_instr == MicroInstruction::Fetch {
                self.fetched_instr = fetched_instr;
            } else if let MicroInstruction::ReadPC { dst, .. } = micro_instr {
                if dst == SelectedRegister8::IR {
                    self.fetched_instr = fetched_instr;
                }
            } else if fetched_instr == FetchedInstr::Invalidate {
                self.fetched_instr = FetchedInstr::None;
                if self.current_sequence.is_some() {
                    self.current_sequence = None;
                    self.instr_count_since_reset -= 1;
                }
            }
        } else {
            self.current_sequence = None;
            run_status = instr::ExecutionStatus::Continue;
        }

        run_status
    }

    #[cfg(feature = "logging")]
    pub fn run(&mut self, memory: &mut impl MemorySpace) {
        let mut memory = LoggingMemory::new(memory);
        self.run_inner(&mut memory);
    }

    #[cfg(not(feature = "logging"))]
    pub fn run(&mut self, memory: &mut impl MemorySpace) {
        self.run_inner(memory);
    }

    fn run_inner(&mut self, memory: &mut impl MemorySpace) {
        if self.current_sequence.is_none() {
            if self.waiting_interrupt.is_some() {
                self.service_interrupt();
            } else if self.fetched_instr.is_some() {
                self.decode_instr();
                self.fetched_instr = FetchedInstr::None;
            }
        }

        let mut run_status = instr::ExecutionStatus::Continue;

        while run_status == instr::ExecutionStatus::Continue {
            if self.running_op {
                run_status = self.run_op(memory);
            } else if self.current_sequence.is_some() {
                run_status = self.run_sequence(memory);
            } else {
                self.current_sequence = Some(instr::sequence_for_mode(
                    instr::InstructionSequenceMode::FetchInstr,
                ));
                run_status = self.run_sequence(memory);
            }

            match run_status {
                instr::ExecutionStatus::YieldClock => {
                    self.waiting_interrupt = self.is_waiting_interrupt();
                    self.cycle_count_since_reset += 1;
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
                instr::ExecutionStatus::RunOpAndFinish => {
                    self.running_op = true;
                    self.current_sequence = Some(instr::finish_instr_sequence());
                }
                instr::ExecutionStatus::FinishInstruction => {
                    self.current_sequence = None;
                    self.current_op = None;
                    self.running_op = false;
                    self.cycle_count_since_reset += 1;
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
                    #[cfg(feature = "logging")]
                    {
                        trace!("--------------");
                        trace!("--------------");
                    }
                }
            };
        }
    }

    fn decode_instr(&mut self) {
        let mut _decode_start: time::Instant;

        let opcode = self.regs.ir.to_u8();
        let decoded_instr = instr::decode(opcode);

        self.current_sequence = Some(instr::sequence_for_mode(decoded_instr.sequence));

        self.current_op = Some(instr::sequence_for_op(decoded_instr.operation));

        self.index_register = decoded_instr.index;

        self.fetched_instr = FetchedInstr::None;

        self.instr_count_since_reset += 1;

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
