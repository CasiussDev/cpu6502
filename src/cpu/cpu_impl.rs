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

/// A 6502 CPU implementation
///
/// This structure contains the entire state of the CPU including:
/// - All internal registers
/// - Current interrupt status
/// - Execution state for the current instruction
/// - Performance counters
///
/// # Example
/// ```
/// use cpu6502::Cpu;
///
/// let mut cpu = Cpu::new();
/// // Now you can use the CPU with memory and run programs
/// ```
#[derive(Debug, Default)]
pub struct Cpu {
    /// CPU registers (A, X, Y, PC, SP, status)
    regs: RegisterFile,
    /// Hardware interrupt pins state (IRQ, NMI)
    pins: Interrupts,
    /// Currently pending interrupt (if any) that will be serviced at the end of the current instruction
    waiting_interrupt: Option<InterruptType>,

    /// The instruction currently being executed
    current_instruction: Instruction,
    /// The current step in the instruction's execution (each instruction takes multiple cycles)
    current_instruction_step: u8,

    /// Number of clock cycles executed since the CPU was last reset
    cycle_count_since_reset: u128,
    /// Number of instructions executed since the CPU was last reset
    instr_count_since_reset: u128,

    /// Address of the last fetched instruction, only when disassembly feature is enabled
    #[cfg(feature = "disassembly")]
    fetched_instr_addr: Option<u16>,
}

impl Cpu {
    /// Creates a new CPU instance with default values and resets it.
    ///
    /// Returns a new CPU ready to be used.
    ///
    /// # Example
    /// ```
    /// use cpu6502::Cpu;
    ///
    /// let mut cpu = Cpu::new();
    /// ```
    pub fn new() -> Self {
        let mut cpu = Self::default();

        cpu.reset();
        cpu
    }

    /// Resets the CPU to its initial state.
    ///
    /// This resets all registers, clears interrupt pins, and resets performance counters.
    /// The CPU will start executing from the address specified in the reset vector.
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

    /// Executes a single CPU cycle with memory logging enabled.
    ///
    /// This version is only available when the "logging" feature is enabled.
    ///
    /// # Parameters
    /// * `memory` - Memory space implementation that the CPU will interact with
    #[cfg(feature = "logging")]
    pub fn run(&mut self, memory: &mut impl MemorySpace) {
        let mut memory = LoggingMemory::new(memory);
        self.run_inner(&mut memory);
    }

    /// Executes a single CPU cycle.
    ///
    /// This function advances the CPU's state by one clock cycle, which may involve
    /// reading from or writing to memory, while executing part of an instruction.
    ///
    /// # Parameters
    /// * `memory` - Memory space implementation that the CPU will interact with
    #[cfg(not(feature = "logging"))]
    pub fn run(&mut self, memory: &mut impl MemorySpace) {
        self.run_inner(memory);
    }

    /// Returns the number of clock cycles executed since the last reset.
    ///
    /// # Returns
    /// The cycle count as a 128-bit unsigned integer
    pub fn cycle_count_since_reset(&self) -> u128 {
        self.cycle_count_since_reset
    }

    /// Returns the number of instructions executed since the last reset.
    ///
    /// # Returns
    /// The instruction count as a 128-bit unsigned integer
    pub fn instr_count_since_reset(&self) -> u128 {
        self.instr_count_since_reset
    }

    /// Sets the Non-Maskable Interrupt (NMI) pin.
    ///
    /// This will trigger an NMI interrupt at the end of the current instruction.
    pub fn set_nmi_pin(&mut self) {
        self.pins.set_nmi_input();
    }

    /// Sets the Non-Maskable Interrupt (NMI) pin to a specific value.
    ///
    /// # Parameters
    /// * `value` - The value to set the NMI pin to
    pub fn set_nmi_pin_value(&mut self, value: bool) {
        self.pins.set_nmi_input_value(value);
    }

    /// Clears the Non-Maskable Interrupt (NMI) pin.
    ///
    /// This sets the NMI pin to low.
    pub fn clear_nmi_pin(&mut self) {
        self.pins.clear_nmi_input();
    }

    /// Sets the Interrupt Request (IRQ) pin.
    ///
    /// This will trigger an IRQ interrupt at the end of the current instruction
    /// if the interrupt disable flag is not set.
    pub fn set_irq_pin(&mut self) {
        self.pins.set_irq_input();
    }

    /// Sets the Interrupt Request (IRQ) pin to a specific value.
    ///
    /// # Parameters
    /// * `value` - The value to set the IRQ pin to
    pub fn set_irq_pin_value(&mut self, value: bool) {
        self.pins.set_irq_input_value(value);
    }

    /// Clears the Interrupt Request (IRQ) pin.
    ///
    /// This sets the IRQ pin to low.
    pub fn clear_irq_pin(&mut self) {
        self.pins.clear_irq_input();
    }

    /// Determines if the current CPU cycle is a write cycle.
    ///
    /// This method is only available when the "gen_write_cycle_query" feature is not enabled.
    ///
    /// # Returns
    /// `true` if the current cycle is a write cycle, `false` otherwise
    #[cfg(not(feature = "gen_write_cycle_query"))]
    pub fn is_current_cycle_write(&self) -> bool {
        write_cycle_query(
            self.current_instruction.into(),
            self.current_instruction_step,
        )
    }

    /// Returns the address of the last fetched instruction.
    ///
    /// This method is only available when the "disassembly" feature is enabled.
    ///
    /// # Returns
    /// The address of the last fetched instruction, or None if no instruction has been fetched
    #[cfg(feature = "disassembly")]
    pub fn fetched_instr_addr(&self) -> Option<u16> {
        self.fetched_instr_addr
    }

    /// Returns a formatted string representation of the CPU registers.
    ///
    /// This method is only available when either the "disassembly" or "logging" feature is enabled.
    ///
    /// # Returns
    /// A string representing the current state of all CPU registers
    #[cfg(any(feature = "disassembly", feature = "logging"))]
    pub fn regs_as_log_line(&self) -> String {
        self.regs.as_log_line()
    }

    /// Internal implementation of the CPU cycle execution.
    ///
    /// This method contains the core logic for executing a single CPU cycle,
    /// handling interrupts, and processing instructions.
    ///
    /// # Parameters
    /// * `memory` - Memory space implementation that the CPU will interact with
    fn run_inner(&mut self, memory: &mut impl MemorySpace) {
        // Reset the instruction address tracking when disassembly is enabled
        #[cfg(feature = "disassembly")]
        {
            self.fetched_instr_addr = None;
        }

        // Check for and update any pending interrupts (NMI or IRQ)
        // This function evaluates pin states and the CPU's interrupt disable flag
        self.waiting_interrupt = waiting_interrupt(
            self.waiting_interrupt,
            &mut self.pins,
            self.regs.status.irq_disable(),
        );

        // Handle interrupt request if we have one and we're at an instruction boundary
        if let Some(interrupt) = self.waiting_interrupt {
            if self.current_instruction == Instruction::FetchInstr {
                // Start the appropriate interrupt sequence based on type
                if interrupt == InterruptType::NonMaskableInterrupt {
                    self.current_instruction = Instruction::StartNmi; // NMI has priority
                } else {
                    self.current_instruction = Instruction::StartIrq; // Regular IRQ
                }
                self.current_instruction_step = 0; // Reset to first step of the interrupt sequence
                self.waiting_interrupt = None; // Clear the interrupt now that we're handling it
            }
        }

        // Execute one step of the current instruction and get its status
        let status = instr_impl::execute(
            self.current_instruction,
            self.current_instruction_step,
            &mut self.regs,
            memory,
        );

        // Process the result of the instruction execution
        match status {
            instr_impl::ClockEndStatus::Continue => {
                // Instruction needs more cycles, advance to next step
                self.current_instruction_step += 1;
            }
            instr_impl::ClockEndStatus::EndInstructionNextFetched {
                opcode_addr: _opcode_addr,
            } => {
                // Instruction is complete and the next opcode has already been fetched
                #[cfg(feature = "disassembly")]
                {
                    // Store the address of the fetched instruction for disassembly
                    self.fetched_instr_addr = Some(_opcode_addr);
                }

                // Decode the opcode from the instruction register
                let opcode = self.regs.ir.to_u8();
                self.current_instruction = instr::decode(opcode);
                self.current_instruction_step = 0; // Reset to first step of the new instruction

                // Increment instruction counter since we've completed one
                self.instr_count_since_reset += 1;
            }
            instr_impl::ClockEndStatus::EndInstruction => {
                // Instruction is complete, prepare to fetch the next instruction
                self.current_instruction = Instruction::FetchInstr;
                self.current_instruction_step = 0; // Reset to first step of the fetch sequence

                // Increment instruction counter since we've completed one
                self.instr_count_since_reset += 1;
            }
        }

        // Always increment the cycle counter after each CPU cycle
        self.cycle_count_since_reset += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu::interrupt::InterruptType::{Interrupt, NonMaskableInterrupt};
    use crate::instr::MemoryModifyOperation;
    use crate::memory::memory_space::{BasicRam, MEMORY_64K};
    use crate::registers::StatusRegFlags;

    fn setup_cpu(memory: &mut BasicRam) -> Cpu {
        let mut cpu = Cpu::new();

        let regs = &mut cpu.regs;

        // Define the IRQ vector ($FFFE/$FFFF) pointing to a known address
        memory[0xFFFE] = 0xEF;
        memory[0xFFFF] = 0xBE;

        // Set INC zeropage as the first and second instructions
        let initial_pc = 0x2000;
        regs.pc.set_u16(initial_pc);
        memory[0x2000] = 0xE6;
        memory[0x2001] = 0x01;
        memory[0x2002] = 0xE6;
        memory[0x2003] = 0x01;

        // Set known initial values for SP
        let initial_sp = 0xFD;
        regs.sp.set_u8(initial_sp);

        regs.status.clear_flags(StatusRegFlags::IRQ_DISABLE); // Ensure I flag is clear

        // Set cpu to fetch, to avoid executing reset sequence
        cpu.current_instruction = Instruction::FetchInstr;
        cpu.current_instruction_step = 0;

        cpu
    }

    #[test]
    fn nmi_start() {
        let mut memory = [0; MEMORY_64K];
        let mut cpu = setup_cpu(&mut memory);
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
        assert_eq!(memory[0x0001], 0x01, "INC should have incremented memory");

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
        let mut memory = [0; MEMORY_64K];
        let mut cpu = setup_cpu(&mut memory);
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
        assert_eq!(memory[0x0001], 0x01, "INC should have incremented memory");

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
        let mut memory = [0; MEMORY_64K];
        let mut cpu = setup_cpu(&mut memory);

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
        assert_eq!(memory[0x0001], 0x01, "INC should have incremented memory");

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
        let mut memory = [0; MEMORY_64K];
        let mut cpu = setup_cpu(&mut memory);

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
        assert_eq!(memory[0x0001], 0x01, "INC should have incremented memory");

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
        let mut memory = [0; MEMORY_64K];
        let mut cpu = setup_cpu(&mut memory);
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
        assert_eq!(memory[0x0001], 0x01, "INC should have incremented memory");

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
        let mut memory = [0; MEMORY_64K];
        let mut cpu = setup_cpu(&mut memory);

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
        assert_eq!(memory[0x0001], 0x01, "INC should have incremented memory");

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
        let mut memory = [0; MEMORY_64K];
        let mut cpu = setup_cpu(&mut memory);

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
        assert_eq!(memory[0x0001], 0x01, "INC should have incremented memory");

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
    fn reset() {
        let mut memory = [0; MEMORY_64K];
        let mut cpu = setup_cpu(&mut memory);

        // Initial Fetch
        cpu.run(&mut memory);

        // First cycle of INC zeropage
        cpu.run(&mut memory);

        // Reset CPU
        cpu.reset();

        // Check CPU state
        assert_eq!(cpu.current_instruction, Instruction::Reset);
        assert_eq!(cpu.current_instruction_step, 0);
        assert_eq!(cpu.cycle_count_since_reset, 0);
        assert_eq!(cpu.instr_count_since_reset, 0);
        assert_eq!(cpu.regs.status.to_u8(), 0x24);
    }
}
