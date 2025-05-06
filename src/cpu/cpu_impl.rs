use crate::cpu::interrupt::{waiting_interrupt, InterruptType, Interrupts};
use crate::instr::{instr_impl, Instruction};
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

        self.waiting_interrupt = waiting_interrupt(
            self.waiting_interrupt,
            &mut self.pins,
            self.regs.status.irq_disable(),
        );

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu::interrupt::InterruptType::{Interrupt, NonMaskableInterrupt};
    use crate::instr::MemoryModifyOperation;
    use crate::registers::StatusRegFlags;

    struct MockMemory {
        mem: Vec<u8>,
    }
    impl MemorySpace for MockMemory {
        fn read(&mut self, addr: u16) -> u8 {
            self.mem[addr as usize]
        }
        fn write(&mut self, value: u8, addr: u16) {
            self.mem[addr as usize] = value;
        }
    }

    fn setup_cpu() -> (Cpu, MockMemory) {
        let mut cpu = Cpu::new();

        let regs = &mut cpu.regs;
        let mut memory = MockMemory {
            mem: vec![0; u16::MAX as usize + 1],
        };

        // Define the IRQ vector ($FFFE/$FFFF) pointing to a known address
        memory.mem[0xFFFE] = 0xEF;
        memory.mem[0xFFFF] = 0xBE;

        // Set INC zeropage as the first and second instructions
        let initial_pc = 0x2000;
        regs.pc.set_u16(initial_pc);
        memory.mem[0x2000] = 0xE6;
        memory.mem[0x2001] = 0x01;
        memory.mem[0x2002] = 0xE6;
        memory.mem[0x2003] = 0x01;

        // Set known initial values for SP
        let initial_sp = 0xFD;
        regs.sp.set_u8(initial_sp);

        regs.status.clear_flags(StatusRegFlags::IRQ_DISABLE); // Ensure I flag is clear

        // Set cpu to fetch, to avoid executing reset sequence
        cpu.current_instruction = Instruction::FetchInstr;
        cpu.current_instruction_step = 0;

        (cpu, memory)
    }

    #[test]
    fn nmi_start() {
        let (mut cpu, mut memory) = setup_cpu();

        // Initial Fetch
        cpu.run(&mut memory);

        // Set NMI Pin
        cpu.set_nmi_pin();

        // Execute INC zeropage
        for _ in 0..=3 {
            cpu.run(&mut memory);
        }

        // Check instruction finished execution and waiting_interrupt is set
        assert_eq!(cpu.current_instruction, Instruction::FetchInstr);
        assert_eq!(cpu.current_instruction_step, 0);
        assert_eq!(
            cpu.waiting_interrupt,
            Some(NonMaskableInterrupt),
            "Waiting interrupt should be set to NMI"
        );
        assert_eq!(
            memory.mem[0x0001], 0x01,
            "INC should have incremented memory"
        );

        // Run first cycle of servicing interrupt request
        cpu.run(&mut memory);
        assert_eq!(cpu.current_instruction, Instruction::StartNmi);
        assert_eq!(cpu.current_instruction_step, 1);
        assert_eq!(
            cpu.waiting_interrupt, None,
            "Waiting interrupt should be cleared after starting NMI"
        );
    }

    #[test]
    fn nmi_irq_disabled() {
        let (mut cpu, mut memory) = setup_cpu();
        cpu.regs.status.set_flags(StatusRegFlags::IRQ_DISABLE);

        // Initial Fetch
        cpu.run(&mut memory);

        // Set NMI Pin
        cpu.set_nmi_pin();

        // Execute INC zeropage
        for _ in 0..=3 {
            cpu.run(&mut memory);
        }

        // Check instruction finished execution and waiting_interrupt is set
        assert_eq!(cpu.current_instruction, Instruction::FetchInstr);
        assert_eq!(cpu.current_instruction_step, 0);
        assert_eq!(
            cpu.waiting_interrupt,
            Some(NonMaskableInterrupt),
            "Waiting interrupt should be set to NMI"
        );
        assert_eq!(
            memory.mem[0x0001], 0x01,
            "INC should have incremented memory"
        );

        // Run first cycle of servicing interrupt request
        cpu.run(&mut memory);
        assert_eq!(cpu.current_instruction, Instruction::StartNmi);
        assert_eq!(cpu.current_instruction_step, 1);
        assert_eq!(
            cpu.waiting_interrupt, None,
            "Waiting interrupt should be cleared after starting NMI"
        );
    }

    #[test]
    fn nmi_edge_sensitive() {
        let (mut cpu, mut memory) = setup_cpu();

        // Initial Fetch
        cpu.run(&mut memory);

        // Set NMI Pin
        cpu.set_nmi_pin();

        // Execute first cycle of INC zeropage
        cpu.run(&mut memory);
        cpu.clear_nmi_pin();

        // Execute INC zeropage
        for _ in 1..=3 {
            cpu.run(&mut memory);
        }

        // Check instruction finished execution and waiting_interrupt is set
        assert_eq!(cpu.current_instruction, Instruction::FetchInstr);
        assert_eq!(cpu.current_instruction_step, 0);
        assert_eq!(
            cpu.waiting_interrupt,
            Some(NonMaskableInterrupt),
            "Waiting interrupt should be set to NMI"
        );
        assert_eq!(
            memory.mem[0x0001], 0x01,
            "INC should have incremented memory"
        );

        // Run first cycle of servicing interrupt request
        cpu.run(&mut memory);
        assert_eq!(cpu.current_instruction, Instruction::StartNmi);
        assert_eq!(cpu.current_instruction_step, 1);
        assert_eq!(
            cpu.waiting_interrupt, None,
            "Waiting interrupt should be cleared after starting NMI"
        );
    }

    #[test]
    fn irq_start() {
        let (mut cpu, mut memory) = setup_cpu();

        // Initial Fetch
        cpu.run(&mut memory);

        // Set IRQ Pin
        cpu.set_irq_pin();

        // Execute INC zeropage
        for _ in 0..=3 {
            cpu.run(&mut memory);
        }

        // Check instruction finished execution and waiting_interrupt is set
        assert_eq!(cpu.current_instruction, Instruction::FetchInstr);
        assert_eq!(cpu.current_instruction_step, 0);
        assert_eq!(
            cpu.waiting_interrupt,
            Some(Interrupt),
            "Waiting interrupt should be set to IRQ"
        );
        assert_eq!(
            memory.mem[0x0001], 0x01,
            "INC should have incremented memory"
        );

        // Run first cycle of servicing interrupt request
        cpu.run(&mut memory);
        assert_eq!(cpu.current_instruction, Instruction::StartIrq);
        assert_eq!(cpu.current_instruction_step, 1);
        assert_eq!(
            cpu.waiting_interrupt, None,
            "Waiting interrupt should be cleared after starting IRQ"
        );
    }

    #[test]
    fn irq_disabled() {
        let (mut cpu, mut memory) = setup_cpu();
        cpu.regs.status.set_flags(StatusRegFlags::IRQ_DISABLE);

        // Initial Fetch
        cpu.run(&mut memory);

        // Set IRQ Pin
        cpu.set_irq_pin();

        // Execute INC zeropage
        for _ in 0..=3 {
            cpu.run(&mut memory);
        }

        // Check instruction finished execution and waiting_interrupt is not set
        assert_eq!(cpu.current_instruction, Instruction::FetchInstr);
        assert_eq!(cpu.current_instruction_step, 0);
        assert_eq!(
            cpu.waiting_interrupt, None,
            "Waiting interrupt should be set to None"
        );
        assert_eq!(
            memory.mem[0x0001], 0x01,
            "INC should have incremented memory"
        );

        // Run next fetch
        cpu.run(&mut memory);
        assert_eq!(
            cpu.current_instruction,
            Instruction::ZeroPageReadModifyWrite(MemoryModifyOperation::IncrementMemory)
        );
        assert_eq!(cpu.current_instruction_step, 0);
        assert_eq!(
            cpu.waiting_interrupt, None,
            "Waiting interrupt should be set to None"
        );
    }

    #[test]
    fn irq_level_sensitive() {
        let (mut cpu, mut memory) = setup_cpu();

        // Initial Fetch
        cpu.run(&mut memory);

        // Set IRQ Pin
        cpu.set_irq_pin();

        // Execute INC zeropage
        for _ in 0..=3 {
            cpu.run(&mut memory);
        }

        cpu.clear_irq_pin();

        // Check instruction finished execution and waiting_interrupt is not set
        assert_eq!(cpu.current_instruction, Instruction::FetchInstr);
        assert_eq!(cpu.current_instruction_step, 0);
        assert_eq!(
            cpu.waiting_interrupt,
            Some(Interrupt),
            "Waiting interrupt should be set to IRQ"
        );
        assert_eq!(
            memory.mem[0x0001], 0x01,
            "INC should have incremented memory"
        );

        // Run next fetch
        cpu.run(&mut memory);
        assert_eq!(
            cpu.current_instruction,
            Instruction::ZeroPageReadModifyWrite(MemoryModifyOperation::IncrementMemory)
        );
        assert_eq!(cpu.current_instruction_step, 0);
        assert_eq!(
            cpu.waiting_interrupt, None,
            "Waiting interrupt should be set to None since irq was cleared before running fetch"
        );
    }

    #[test]
    fn interrupt_priority() {
        let (mut cpu, mut memory) = setup_cpu();

        // Initial Fetch
        cpu.run(&mut memory);

        // Set NMI Pin
        cpu.set_nmi_pin();
        cpu.set_irq_pin();

        // Execute INC zeropage
        for _ in 0..=3 {
            cpu.run(&mut memory);
        }

        // Check instruction finished execution and waiting_interrupt is set
        assert_eq!(cpu.current_instruction, Instruction::FetchInstr);
        assert_eq!(cpu.current_instruction_step, 0);
        assert_eq!(
            cpu.waiting_interrupt,
            Some(NonMaskableInterrupt),
            "Waiting interrupt should be set to NMI"
        );
        assert_eq!(
            memory.mem[0x0001], 0x01,
            "INC should have incremented memory"
        );

        // Run first cycle of servicing interrupt request
        cpu.run(&mut memory);
        assert_eq!(cpu.current_instruction, Instruction::StartNmi);
        assert_eq!(cpu.current_instruction_step, 1);
        assert_eq!(
            cpu.waiting_interrupt, None,
            "Waiting interrupt should be cleared after starting NMI"
        );
    }
}
