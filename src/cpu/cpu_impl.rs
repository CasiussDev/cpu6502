use crate::instr::{instr_impl, Instruction};
use crate::interrupts::InterruptType;
use crate::interrupts::Interrupts;
use crate::registers::RegisterFile;
use crate::{instr, MemorySpace};

#[cfg(feature = "disassembly")]
extern crate disasm6502;

#[cfg(feature = "logging")]
use crate::cpu::logging_memory::LoggingMemory;

#[cfg(not(feature = "gen_write_cycle_query"))]
use crate::cpu::write_cycle_query::write_cycle_query;

#[derive(Debug, Default)]
pub struct Cpu {
    regs: RegisterFile,
    pins: Interrupts,
    waiting_interrupt: Option<InterruptType>,

    current_instruction: Instruction,
    current_instruction_step: u8,

    cycle_count_since_reset: u128,
    instr_count_since_reset: u128,

    #[cfg(feature = "disassembly")]
    fetched_instr_addr: Option<u16>,
}

fn waiting_interrupt(
    prev_waiting_interrupt: Option<InterruptType>,
    pins: &mut Interrupts,
) -> Option<InterruptType> {
    if pins.waiting_nmi() || (prev_waiting_interrupt == Some(InterruptType::NonMaskableInterrupt)) {
        Some(InterruptType::NonMaskableInterrupt)
    } else if pins.is_irq_set() || (prev_waiting_interrupt == Some(InterruptType::Interrupt)) {
        Some(InterruptType::Interrupt)
    } else {
        None
    }
}

impl Cpu {
    pub fn new() -> Self {
        let mut cpu = Self::default();

        cpu.reset();
        cpu
    }

    pub fn reset(&mut self) {
        self.regs.reset();
        self.pins.reset();

        self.waiting_interrupt = None;
        self.current_instruction = Instruction::Reset;
        self.current_instruction_step = 0;

        self.cycle_count_since_reset = 0;
        self.instr_count_since_reset = 0;

        #[cfg(feature = "disassembly")]
        {
            self.fetched_instr_addr = None;
        }
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

    pub fn cycle_count_since_reset(&self) -> u128 {
        self.cycle_count_since_reset
    }

    pub fn instr_count_since_reset(&self) -> u128 {
        self.instr_count_since_reset
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

    #[cfg(not(feature = "gen_write_cycle_query"))]
    pub fn is_current_cycle_write(&self) -> bool {
        write_cycle_query(
            self.current_instruction.into(),
            self.current_instruction_step,
        )
    }

    #[cfg(feature = "disassembly")]
    pub fn fetched_instr_addr(&self) -> Option<u16> {
        self.fetched_instr_addr
    }

    #[cfg(any(feature = "disassembly", feature = "logging"))]
    pub fn regs_as_log_line(&self) -> String {
        self.regs.as_log_line()
    }

    fn run_inner(&mut self, memory: &mut impl MemorySpace) {
        #[cfg(feature = "disassembly")]
        {
            self.fetched_instr_addr = None;
        }

        self.waiting_interrupt = waiting_interrupt(self.waiting_interrupt, &mut self.pins);

        if let Some(interrupt) = self.waiting_interrupt {
            if self.current_instruction == Instruction::FetchInstr {
                if interrupt == InterruptType::NonMaskableInterrupt {
                    self.current_instruction = Instruction::StartNmi;
                } else {
                    self.current_instruction = Instruction::StartIrq;
                }
                self.current_instruction_step = 0;
                self.waiting_interrupt = None;
            }
        }

        let status = instr_impl::execute(
            self.current_instruction,
            self.current_instruction_step,
            &mut self.regs,
            memory,
        );

        match status {
            instr_impl::ClockEndStatus::Continue => {
                self.current_instruction_step += 1;
            }
            instr_impl::ClockEndStatus::EndInstructionNextFetched {
                opcode_addr: _opcode_addr,
            } => {
                #[cfg(feature = "disassembly")]
                {
                    self.fetched_instr_addr = Some(_opcode_addr);
                }

                let opcode = self.regs.ir.to_u8();
                self.current_instruction = instr::decode(opcode);
                self.current_instruction_step = 0;

                self.instr_count_since_reset += 1;
            }
            instr_impl::ClockEndStatus::EndInstruction => {
                self.current_instruction = Instruction::FetchInstr;
                self.current_instruction_step = 0;

                self.instr_count_since_reset += 1;
            }
        }

        self.cycle_count_since_reset += 1;
    }
}
