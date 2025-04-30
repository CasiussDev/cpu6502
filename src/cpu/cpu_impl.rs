use crate::instr;
use crate::instr::{destruct_sequence, MicroInstruction};
use crate::interrupts::Interrupts;
use crate::memory::MemorySpace;
use crate::registers::{IndexRegister, RegisterFile, SelectedRegister8, StatusRegFlags};
use std::slice;

use crate::instr::FetchedInstr;

#[cfg(feature = "logging")]
use crate::cpu::logging_memory::LoggingMemory;
#[cfg(feature = "logging")]
use log::{debug, trace};
#[cfg(feature = "logging")]
use std::fs;

#[cfg(feature = "disassembly")]
extern crate disasm6502;

#[derive(PartialEq, Eq, Debug)]
enum WaitingInterrupt {
    NonMaskableInterrupt,
    Interrupt,
}

#[derive(Default, Debug)]
pub struct Cpu {
    regs: RegisterFile,
    pins: Interrupts,
    current_sequence: Option<slice::Iter<'static, MicroInstruction>>,
    current_op: Option<slice::Iter<'static, MicroInstruction>>,
    index_register: Option<IndexRegister>,
    waiting_interrupt: Option<WaitingInterrupt>,
    cycle_count_since_reset: u128,
    instr_count_since_reset: u128,
    fetched_instr: FetchedInstr,
    running_op: bool,

    #[cfg(feature = "logging")]
    logging_inited: bool,
}

impl Cpu {
    pub fn new() -> Self {
        let mut cpu = Self::default();
        cpu.reset();

        let _ = instr::sequence_for_mode(instr::InstructionSequenceMode::FetchInstr);
        let _ = instr::sequence_for_op(instr::InstructionOp::Add);

        cpu
    }

    #[cfg(feature = "logging")]
    pub fn init_logging_trace(&mut self) {
        self.init_logging(log::LevelFilter::Trace);
    }

    #[cfg(feature = "logging")]
    pub fn init_logging_debug(&mut self) {
        self.init_logging(log::LevelFilter::Debug);
    }

    #[cfg(feature = "logging")]
    fn init_logging(&mut self, level: log::LevelFilter) {
        if !self.logging_inited {
            let log_config = simplelog::ConfigBuilder::new()
                .set_max_level(log::LevelFilter::Off)
                .set_time_level(log::LevelFilter::Off)
                .set_thread_level(log::LevelFilter::Off)
                .set_target_level(log::LevelFilter::Off)
                .set_location_level(log::LevelFilter::Off)
                .build();

            let trace_file = fs::File::create("trace.log.txt").expect("cannot open trace file");
            let _ = simplelog::WriteLogger::init(level, log_config, trace_file);

            self.logging_inited = true;
        }
    }

    pub fn cycle_count_since_reset(&self) -> u128 {
        self.cycle_count_since_reset
    }

    pub fn instr_count_since_reset(&self) -> u128 {
        self.instr_count_since_reset
    }

    #[cfg(feature = "disassembly")]
    pub fn fetched_instr_addr(&self) -> Option<u16> {
        if let FetchedInstr::Some(addr) = self.fetched_instr {
            return Some(addr);
        }
        None
    }

    #[cfg(any(feature = "disassembly", feature = "logging"))]
    pub fn regs_as_log_line(&self) -> String {
        self.regs.as_log_line()
    }

    pub fn set_nmi_pin(&mut self) {
        self.pins.set_nmi_input();
    }

    pub fn set_nmi_pin_value(&mut self, value: bool) {
        self.pins.set_nmi_input_value(value);
    }

    pub fn clear_nmi_pin(&mut self) {
        self.pins.clear_nmi_input();
    }

    pub fn set_irq_pin(&mut self) {
        self.pins.set_irq_input();
    }

    pub fn set_irq_pin_value(&mut self, value: bool) {
        self.pins.set_irq_input_value(value);
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

    fn waiting_interrupt(&mut self) -> Option<WaitingInterrupt> {
        if (self.waiting_interrupt == Some(WaitingInterrupt::NonMaskableInterrupt))
            || self.pins.waiting_nmi()
        {
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
                } else if let FetchedInstr::Some(instr) = fetched_instr {
                    self.fetched_instr = FetchedInstr::Some(instr);
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

    pub fn is_current_cycle_write(&self) -> bool {
        let empty: [MicroInstruction; 0] = [];
        let iter = &mut empty.iter();
        self.current_sequence
            .clone()
            .as_mut()
            .unwrap_or(iter)
            .any(|&m_instr| matches!(m_instr, MicroInstruction::WriteAddress { .. }))
    }

    fn run_inner(&mut self, memory: &mut impl MemorySpace) {
        if self.current_sequence.is_none() {
            if self.fetched_instr.is_some() {
                self.decode_instr(memory);
                self.fetched_instr = FetchedInstr::None;
            } else if self.waiting_interrupt.is_some() {
                self.service_interrupt();
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
                    self.waiting_interrupt = self.waiting_interrupt();
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
                    self.waiting_interrupt = self.waiting_interrupt();
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

    fn decode_instr(&mut self, _memory: &mut impl MemorySpace) {
        #[cfg(feature = "disassembly")]
        #[cfg(feature = "logging")]
        {
            if let FetchedInstr::Some(addr) = self.fetched_instr {
                let mut code = [0u8; 6];
                _memory.read_array(addr, code.as_mut_slice());
                let instructions = disasm6502::from_addr_array(code.as_slice(), addr)
                    .expect("could not decode instr");
                let decoded = instructions.first().expect("empty instr vector");

                debug!(
                    "{:04X} {} {} \t{} CYC:{}",
                    addr,
                    decoded.as_hex_str(),
                    decoded.as_str(),
                    self.regs_as_log_line(),
                    self.cycle_count_since_reset()
                );
            }
        }

        let opcode = self.regs.ir.to_u8();
        let decoded_instr = instr::decode(opcode);
        let (sequence, op, idx) = destruct_sequence(decoded_instr);

        self.current_sequence = Some(instr::sequence_for_mode(sequence));

        self.current_op = Some(instr::sequence_for_op(op));

        self.index_register = Some(idx);

        self.fetched_instr = FetchedInstr::None;

        self.instr_count_since_reset += 1;
    }

    fn service_interrupt(&mut self) {
        let sequence_mode = match self.waiting_interrupt {
            None => {
                panic!("service_interrupt was called witout any waiting interrupt");
            }
            Some(WaitingInterrupt::NonMaskableInterrupt) => {
                #[cfg(feature = "logging")]
                {
                    debug!("--------------------------\n  Non Maskable Interrupt\n--------------------------");
                }
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
