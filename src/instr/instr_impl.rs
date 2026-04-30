//! Implementation of all 6502 CPU instruction execution logic.
//!
//! This module contains the cycle-accurate implementation of each 6502 instruction type.
//! The 6502 uses a multi-cycle execution model where each instruction requires several
//! clock cycles to complete. Each cycle performs a specific operation such as:
//! - Reading from memory
//! - Writing to memory
//! - Performing internal calculations
//! - Updating registers and flags
//!
//! # Addressing Modes
//!
//! The module implements all standard 6502 addressing modes:
//! - Implied/Implicit
//! - Immediate
//! - Absolute
//! - Zero Page
//! - Indexed (with X or Y registers)
//! - Indirect
//! - Relative (for branch instructions)
//!
//! # Timing
//!
//! Each instruction maintains cycle-accurate timing matching the original 6502.
//! The execution functions return a `ClockEndStatus` indicating whether more cycles
//! are needed or the instruction has completed.
//!
//! # Page Boundary Crossing
//!
//! The module properly handles the special timing cases when indexed addressing
//! crosses page boundaries, adding extra cycles as needed per the original hardware.

#[cfg(test)]
mod tests;

use super::{
    BranchOperation, ImplicitOperation, Instruction, MemoryModifyOperation, PullStackOperation,
    PushStackOperation, RegisterMemoryOperation,
};
use crate::cpu::interrupt::{InterruptVector, InterruptVectorAddrBytePos};
use crate::memory::STACK_PAGE;
use crate::registers::{IndexRegister, RegisterFile, StatusRegFlags};
use crate::{alu, MemorySpace};
#[cfg(feature = "logging")]
use log::trace;

/// Represents the status of the CPU clock at the end of an instruction step.
///
/// In the 6502's multi-cycle execution model, this enum communicates whether
/// the current instruction requires more cycles to complete or if it has finished
/// execution. For completed instructions, it also indicates whether the next opcode
/// has already been fetched as an optimization.
///
/// This is returned by instruction execution functions to control the CPU's
/// instruction sequencing logic.
#[derive(Debug, PartialEq, Eq)]
pub enum ClockEndStatus {
    /// The current instruction needs more cycles to complete.
    /// The CPU should advance to the next step of the current instruction.
    Continue,

    /// The current instruction has completed and the CPU should fetch the next instruction.
    EndInstruction,

    /// The current instruction has completed, and the next opcode has already been fetched.
    /// This is an optimization used by branch instructions when the branch is not taken.
    /// The CPU can skip the fetch cycle and use the pre-fetched opcode from the specified address.
    EndInstructionNextFetched {
        /// The memory address from which the next opcode was pre-fetched
        opcode_addr: u16,
    },
}

/// Indicates whether an address required adjustment when adding an index value.
///
/// When performing indexed addressing operations on the 6502, adding an index register
/// to an address may cause a carry from the low byte to the high byte (crossing a page boundary).
/// This enum communicates whether such an adjustment was necessary.
///
/// This is used to determine whether extra CPU cycles are needed when crossing page boundaries
/// in indexed addressing modes, which is a key timing characteristic of the 6502 processor.
#[derive(Debug, PartialEq, Eq)]
pub enum FixAddressResult {
    /// The high byte of the address was incremented due to a page boundary crossing.
    Fixed,

    /// No page boundary was crossed, so the high byte remained unchanged.
    Untouched,
}

/// Retrieves the value from the specified index register (X or Y).
///
/// This utility function extracts the current value from either the X or Y index register
/// as specified by the `idx` parameter. It's commonly used in indexed addressing modes
/// to calculate effective memory addresses.
///
/// # Arguments
///
/// * `regs` - A mutable reference to the CPU's register file
/// * `idx` - The index register (X or Y) to retrieve the value from
///
/// # Returns
///
/// * The 8-bit value contained in the specified index register
fn get_index_value(regs: &mut RegisterFile, idx: IndexRegister) -> u8 {
    regs.index_register_u8(idx)
}

/// Sets the address register to point to the current stack location.
///
/// In the 6502, the stack is located in page 1 of memory (0x0100-0x01FF), with the stack pointer
/// providing the low byte of the address. This function composes the full 16-bit stack address
/// by setting the high byte to the fixed stack page (0x01) and the low byte to the current stack
/// pointer value.
///
/// This is commonly used during stack operations (push/pull) to prepare the address register
/// for memory access to the stack.
///
/// # Arguments
///
/// * `regs` - A mutable reference to the CPU's register file
fn set_stack_address(regs: &mut RegisterFile) {
    let addr_high = STACK_PAGE;
    regs.addr.set_high_u8(addr_high);

    let addr_low = regs.sp.to_u8();
    regs.addr.set_low_u8(addr_low);
}

/// Adds the value of an index register to the current address.
///
/// This function is used in indexed addressing modes to calculate effective addresses.
/// It adds the value of the specified index register (X or Y) to the low byte of the
/// address register, with wrapping behavior if the addition overflows.
///
/// Note that this function only modifies the low byte of the address. Page boundary
/// crossing detection and high-byte adjustment must be handled separately if needed,
/// typically with the `fix_addr` function.
///
/// # Arguments
///
/// * `regs` - A mutable reference to the CPU's register file
/// * `idx` - The index register (X or Y) whose value should be added to the address
fn add_index_to_address(regs: &mut RegisterFile, idx: IndexRegister) {
    let index_value = get_index_value(regs, idx);

    let addr_low = regs.addr.low_u8();
    let addr_low = addr_low.wrapping_add(index_value);
    regs.addr.set_low_u8(addr_low);
}

/// Adjusts the high byte of the address register if indexing crosses a page boundary.
///
/// When adding an index value to a memory address, if the addition of the index to the
/// low byte causes a carry (i.e., the index value is greater than the low byte), the
/// high byte needs to be incremented to correctly represent the target address.
///
/// # Arguments
///
/// * `regs` - A mutable reference to the CPU's register file containing the address to fix
/// * `index_value` - The index value that was added to the low byte of the address
///
/// # Returns
///
/// * `FixAddressResult::Fixed` - If the high byte needed to be incremented (page boundary crossed)
/// * `FixAddressResult::Untouched` - If no page boundary was crossed
fn fix_addr(regs: &mut RegisterFile, index_value: u8) -> FixAddressResult {
    let addr_low = regs.addr.low_u8();
    if index_value > addr_low {
        let addr_high = regs.addr.high_u8().wrapping_add(1);
        regs.addr.set_high_u8(addr_high);
        FixAddressResult::Fixed
    } else {
        FixAddressResult::Untouched
    }
}

/// Handles page crossing in indexed addressing modes and completes the instruction if possible.
///
/// This function is used during indexed memory operations to determine whether:
/// 1. A page boundary was crossed, requiring an additional cycle, or
/// 2. The operation can complete immediately because no page boundary was crossed
///
/// If no page boundary was crossed, the operation is executed immediately and the instruction ends.
/// If a page boundary was crossed, a dummy read is performed and the clock cycle continues.
///
/// # Arguments
///
/// * `op` - The register-memory operation to perform
/// * `regs` - A mutable reference to the CPU's register file
/// * `memory` - A mutable reference to the memory space for reading/writing
/// * `idx` - The index register (X or Y) used in the address calculation
///
/// # Returns
///
/// * `ClockEndStatus::EndInstruction` - If no page boundary was crossed, and the operation completed
/// * `ClockEndStatus::Continue` - If a page boundary was crossed and another cycle is needed
#[must_use]
fn fix_addr_or_run_op_finish(
    op: RegisterMemoryOperation,
    regs: &mut RegisterFile,
    memory: &mut impl MemorySpace,
    idx: IndexRegister,
) -> ClockEndStatus {
    let index_value = get_index_value(regs, idx);

    if fix_addr(regs, index_value) == FixAddressResult::Untouched {
        execute_op(op, regs, memory);
        ClockEndStatus::EndInstruction
    } else {
        let _ = memory.read(regs.addr.to_u16());
        ClockEndStatus::Continue
    }
}

/// Executes a conditional branch instruction based on the provided condition.
///
/// This function implements the core branching logic used by all branch instructions (BEQ, BNE, BCS, etc.).
/// It performs the following:
/// - If the condition is true:
///   - Calculates a new program counter by adding the signed branch offset (from tmp register) to the current PC
///   - If this branch crosses a page boundary, an extra cycle is needed (returns Continue)
///   - Otherwise completes the branch (returns EndInstruction)
/// - If the condition is false:
///   - Increments the PC to skip the branch offset
///   - Pre-fetches the next opcode for optimization
///   - Completes the instruction
///
/// # Arguments
///
/// * `condition` - The branch condition to evaluate (true or false)
/// * `regs` - A mutable reference to the CPU's register file
/// * `memory` - A mutable reference to the memory space for reading the next opcode
///
/// # Returns
///
/// * `ClockEndStatus::Continue` - If the branch was taken and crossed a page boundary (requiring an extra cycle)
/// * `ClockEndStatus::EndInstruction` - If the branch was taken but didn't cross a page boundary
/// * `ClockEndStatus::EndInstructionNextFetched` - If the branch wasn't taken (with the next opcode pre-fetched)
fn branch_if(
    condition: bool,
    regs: &mut RegisterFile,
    memory: &mut impl MemorySpace,
) -> ClockEndStatus {
    let next_opcode = memory.read(regs.pc.to_u16());

    if condition {
        let old_pch = regs.pc.high_u8();
        let new_pc = regs.pc.to_u16().wrapping_add(regs.tmp.to_i8() as u16);
        regs.pc.set_u16(new_pc);
        if old_pch != regs.pc.high_u8() {
            // Crossed a page boundary
            ClockEndStatus::Continue
        } else {
            ClockEndStatus::EndInstruction
        }
    } else {
        let status = ClockEndStatus::EndInstructionNextFetched {
            opcode_addr: regs.pc.to_u16(),
        };
        regs.pc.inc();
        regs.ir.set_u8(next_opcode);
        status
    }
}

/// Executes the appropriate branch operation based on the provided branch type.
///
/// This function selects the correct condition to evaluate based on the branch operation (BEQ, BNE, BCS, etc.)
/// and delegates to `branch_if()` to perform the actual branching logic. It serves as a higher-level
/// dispatcher for all branch instructions.
///
/// The supported branch operations are:
/// - `BranchPlus` (BPL): Branch if the result is positive (N flag clear)
/// - `BranchMinus` (BMI): Branch if the result is negative (N flag set)
/// - `BranchEqual` (BEQ): Branch if the result is zero (Z flag set)
/// - `BranchNotEqual` (BNE): Branch if the result is not zero (Z flag clear)
/// - `BranchCarryClear` (BCC): Branch if the carry flag is clear
/// - `BranchCarrySet` (BCS): Branch if the carry flag is set
/// - `BranchOverflowClear` (BVC): Branch if the overflow flag is clear
/// - `BranchOverflowSet` (BVS): Branch if the overflow flag is set
///
/// # Arguments
///
/// * `op` - The specific branch operation to execute
/// * `regs` - A mutable reference to the CPU's register file
/// * `memory` - A mutable reference to the memory space for reading
///
/// # Returns
///
/// * `ClockEndStatus` - The clock end status returned by the `branch_if()` function, indicating
///   whether more cycles are needed or if the instruction is complete
fn execute_branch_op(
    op: BranchOperation,
    regs: &mut RegisterFile,
    memory: &mut impl MemorySpace,
) -> ClockEndStatus {
    match op {
        BranchOperation::BranchPlus => branch_if(!regs.status.negative(), regs, memory),
        BranchOperation::BranchMinus => branch_if(regs.status.negative(), regs, memory),
        BranchOperation::BranchEqual => branch_if(regs.status.zero(), regs, memory),
        BranchOperation::BranchNotEqual => branch_if(!regs.status.zero(), regs, memory),
        BranchOperation::BranchCarryClear => branch_if(!regs.status.carry(), regs, memory),
        BranchOperation::BranchCarrySet => branch_if(regs.status.carry(), regs, memory),
        BranchOperation::BranchOverflowClear => branch_if(!regs.status.overflow(), regs, memory),
        BranchOperation::BranchOverflowSet => branch_if(regs.status.overflow(), regs, memory),
    }
}

/// Executes an implicit mode operation that doesn't involve memory access.
///
/// Implicit (or implied) mode operations work only with registers and status flags.
/// These operations include register transfers (TAX, TXA, etc.), flag manipulations
/// (CLC, SEC, CLI, etc.), and register-only operations (INX, DEY, ASL A, etc.).
///
/// This function doesn't read from or write to memory, as all data is already
/// present in CPU registers. It updates registers and/or processor status flags
/// according to the operation performed.
///
/// # Arguments
///
/// * `op` - The specific implicit operation to execute
/// * `regs` - A mutable reference to the CPU's register file containing all CPU registers and flags
fn execute_implicit_op(op: ImplicitOperation, regs: &mut RegisterFile) {
    match op {
        ImplicitOperation::Nop => (),
        ImplicitOperation::ShiftLeftA => alu::shift_left(&mut regs.a, &mut regs.status),
        ImplicitOperation::ShiftRightA => alu::shift_right(&mut regs.a, &mut regs.status),
        ImplicitOperation::RotateLeftA => alu::rotate_left(&mut regs.a, &mut regs.status),
        ImplicitOperation::RotateRightA => alu::rotate_right(&mut regs.a, &mut regs.status),
        ImplicitOperation::IncrementX => alu::inc(&mut regs.x, &mut regs.status),
        ImplicitOperation::IncrementY => alu::inc(&mut regs.y, &mut regs.status),
        ImplicitOperation::DecrementX => alu::dec(&mut regs.x, &mut regs.status),
        ImplicitOperation::DecrementY => alu::dec(&mut regs.y, &mut regs.status),
        ImplicitOperation::ClearCarry => regs.status.clear_flags(StatusRegFlags::CARRY),
        ImplicitOperation::SetCarry => regs.status.set_flags(StatusRegFlags::CARRY),
        ImplicitOperation::ClearDecimal => regs.status.clear_flags(StatusRegFlags::DECIMAL),
        ImplicitOperation::SetDecimal => regs.status.set_flags(StatusRegFlags::DECIMAL),
        ImplicitOperation::ClearInterruptDisable => {
            regs.status.clear_flags(StatusRegFlags::IRQ_DISABLE)
        }
        ImplicitOperation::SetInterruptDisable => {
            regs.status.set_flags(StatusRegFlags::IRQ_DISABLE)
        }
        ImplicitOperation::ClearOverflow => regs.status.clear_flags(StatusRegFlags::OVERFLOW),
        ImplicitOperation::SetOverflow => regs.status.set_flags(StatusRegFlags::OVERFLOW),
        ImplicitOperation::TransferAccumulatorToX => {
            regs.x = regs.a;
            alu::update_status_nz(regs.x.to_i8(), &mut regs.status)
        }
        ImplicitOperation::TransferAccumulatorToY => {
            regs.y = regs.a;
            alu::update_status_nz(regs.y.to_i8(), &mut regs.status);
        }
        ImplicitOperation::TransferXToAccumulator => {
            regs.a = regs.x;
            alu::update_status_nz(regs.a.to_i8(), &mut regs.status);
        }
        ImplicitOperation::TransferYToAccumulator => {
            regs.a = regs.y;
            alu::update_status_nz(regs.a.to_i8(), &mut regs.status);
        }
        ImplicitOperation::TransferStackPtrToX => {
            regs.x = regs.sp;
            alu::update_status_nz(regs.sp.to_i8(), &mut regs.status);
        }
        ImplicitOperation::TransferXToStackPtr => {
            regs.sp = regs.x;
        }
    }
}

/// Executes a register-memory operation, which typically involves a CPU register and memory access.
///
/// This function dispatches to the appropriate ALU operation based on the `op` parameter. It handles
/// all operations that involve interaction between a register (typically the accumulator, X, or Y)
/// and a memory location. These include logical operations (OR, AND, XOR), arithmetic operations
/// (ADD, SUB), comparisons (CMP, CPX, CPY), loads (LDA, LDX, LDY), stores (STA, STX, STY), and bit
/// manipulation operations.
///
/// The memory is accessed at the address already set in the address register of the CPU.
/// For most operations, the memory value is first read into a temporary register, then
/// the actual operation is performed using that temporary value and a CPU register.
///
/// # Arguments
///
/// * `op` - The specific register-memory operation to execute
/// * `regs` - A mutable reference to the CPU's register file
/// * `memory` - A mutable reference to the memory space for reading/writing
///
/// # Arguments
///
/// * `op` - The specific register-memory operation to execute
/// * `regs` - A mutable reference to the CPU's register file
/// * `memory` - A mutable reference to the memory space for reading/writing
fn execute_op(op: RegisterMemoryOperation, regs: &mut RegisterFile, memory: &mut impl MemorySpace) {
    match op {
        RegisterMemoryOperation::Or => {
            regs.tmp.set_u8(memory.read(regs.addr.to_u16()));
            alu::or(&mut regs.a, &regs.tmp, &mut regs.status)
        }
        RegisterMemoryOperation::And => {
            regs.tmp.set_u8(memory.read(regs.addr.to_u16()));
            alu::and(&mut regs.a, &regs.tmp, &mut regs.status)
        }
        RegisterMemoryOperation::Xor => {
            regs.tmp.set_u8(memory.read(regs.addr.to_u16()));
            alu::xor(&mut regs.a, &regs.tmp, &mut regs.status)
        }
        RegisterMemoryOperation::Add => {
            regs.tmp.set_u8(memory.read(regs.addr.to_u16()));
            alu::add(&mut regs.a, &regs.tmp, &mut regs.status)
        }
        RegisterMemoryOperation::Sub => {
            regs.tmp.set_u8(memory.read(regs.addr.to_u16()));
            alu::sub(&mut regs.a, &regs.tmp, &mut regs.status)
        }
        RegisterMemoryOperation::Cmp => {
            regs.tmp.set_u8(memory.read(regs.addr.to_u16()));
            alu::cmp(&regs.a, &regs.tmp, &mut regs.status)
        }
        RegisterMemoryOperation::Cpx => {
            regs.tmp.set_u8(memory.read(regs.addr.to_u16()));
            alu::cmp(&regs.x, &regs.tmp, &mut regs.status)
        }
        RegisterMemoryOperation::Cpy => {
            regs.tmp.set_u8(memory.read(regs.addr.to_u16()));
            alu::cmp(&regs.y, &regs.tmp, &mut regs.status)
        }
        RegisterMemoryOperation::LoadA => {
            regs.a.set_u8(memory.read(regs.addr.to_u16()));
            alu::update_status_nz(regs.a.to_i8(), &mut regs.status);
        }
        RegisterMemoryOperation::LoadX => {
            regs.x.set_u8(memory.read(regs.addr.to_u16()));
            alu::update_status_nz(regs.x.to_i8(), &mut regs.status);
        }
        RegisterMemoryOperation::LoadY => {
            regs.y.set_u8(memory.read(regs.addr.to_u16()));
            alu::update_status_nz(regs.y.to_i8(), &mut regs.status);
        }
        RegisterMemoryOperation::StoreA => {
            memory.write(regs.a.to_u8(), regs.addr.to_u16());
        }
        RegisterMemoryOperation::StoreX => {
            memory.write(regs.x.to_u8(), regs.addr.to_u16());
        }
        RegisterMemoryOperation::StoreY => {
            memory.write(regs.y.to_u8(), regs.addr.to_u16());
        }
        RegisterMemoryOperation::Bit => {
            regs.tmp.set_u8(memory.read(regs.addr.to_u16()));
            alu::bit_compare(regs.a, regs.tmp, &mut regs.status);
        }
        RegisterMemoryOperation::BitImmediate => {
            alu::bit_compare_imm(regs.a, regs.tmp, &mut regs.status);
        }
    }
}

/// Executes a read-modify-write (RMW) operation on a memory value.
///
/// Read-modify-write operations first read a value from memory (previously stored in the
/// temporary register), then modify it according to the specified operation, and finally
/// write the result back to memory. These operations include increment (INC), decrement (DEC),
/// and various shifts and rotates (ASL, LSR, ROL, ROR).
///
/// The input value is expected to be in the temporary register (regs.tmp) before calling
/// this function, and the modified result will also be placed in the temporary register
/// for subsequent writing back to memory by the caller.
///
/// # Arguments
///
/// * `op` - The specific memory modification operation to execute
/// * `regs` - A mutable reference to the CPU's register file, where regs.tmp contains the value to modify
fn execute_memory_modify_op(op: MemoryModifyOperation, regs: &mut RegisterFile) {
    match op {
        MemoryModifyOperation::IncrementMemory => alu::inc(&mut regs.tmp, &mut regs.status),
        MemoryModifyOperation::DecrementMemory => alu::dec(&mut regs.tmp, &mut regs.status),
        MemoryModifyOperation::ShiftLeftMemory => alu::shift_left(&mut regs.tmp, &mut regs.status),
        MemoryModifyOperation::ShiftRightMemory => {
            alu::shift_right(&mut regs.tmp, &mut regs.status)
        }
        MemoryModifyOperation::RotateLeftMemory => {
            alu::rotate_left(&mut regs.tmp, &mut regs.status)
        }
        MemoryModifyOperation::RotateRightMemory => {
            alu::rotate_right(&mut regs.tmp, &mut regs.status)
        }
    }
}

/// Reads a byte from memory at the current program counter address without incrementing PC.
///
/// This function performs a simple memory read operation at the address pointed to by the
/// program counter (PC). It does not advance the PC, making it useful in situations where
/// the CPU needs to examine the current instruction or operand without moving to the next
/// memory location.
///
/// # Arguments
///
/// * `regs` - A mutable reference to the CPU's register file containing the program counter
/// * `memory` - A mutable reference to the memory space to read from
///
/// # Returns
///
/// * The byte value at the memory location pointed to by the program counter
fn read_pc(regs: &mut RegisterFile, memory: &mut impl MemorySpace) -> u8 {
    memory.read(regs.pc.to_u16())
}

/// Reads a byte from memory at the current program counter address and then increments PC.
///
/// This function is typically used during instruction fetch and execution to read an opcode
/// or operand byte and then advance the program counter to the next memory location.
///
/// # Arguments
///
/// * `regs` - A mutable reference to the CPU's register file containing the program counter
/// * `memory` - A mutable reference to the memory space to read from
///
/// # Returns
///
/// * The byte value at the memory location pointed to by the program counter before incrementing
fn read_pc_inc(regs: &mut RegisterFile, memory: &mut impl MemorySpace) -> u8 {
    let pc = memory.read(regs.pc.to_u16());
    regs.pc.inc();
    pc
}

/// Writes a byte to the stack and decrements the stack pointer.
///
/// The 6502 stack operates in a "top-down" fashion, with the stack pointer initially pointing
/// to the current top of the stack. This function performs three key operations:
/// 1. Sets the address register to point to the current stack location (0x01xx)
/// 2. Writes the provided byte value to that location in memory
/// 3. Decrements the stack pointer to prepare for the next stack write
///
/// This function is used in push operations, interrupt handling, and subroutine calls
/// to store processor state or return information on the stack.
///
/// # Arguments
///
/// * `regs` - A mutable reference to the CPU's register file
/// * `memory` - A mutable reference to the memory space for writing
/// * `value` - The byte value to write to the stack
fn write_stack(regs: &mut RegisterFile, memory: &mut impl MemorySpace, value: u8) {
    set_stack_address(regs);
    memory.write(value, regs.addr.to_u16());
    regs.sp.dec();
}

/// Reads a byte from an interrupt vector address and loads it into the program counter.
///
/// The 6502 uses fixed memory locations (vectors) to determine where to jump when handling
/// interrupts or reset conditions. Each vector is a 16-bit address stored in two consecutive
/// memory locations (low byte first, then high byte). This function:
/// 1. Calculates the correct address for the specified interrupt vector and byte position
/// 2. Reads the byte from that address in memory
/// 3. Loads the byte into either the low or high byte of the program counter
///
/// This is used during interrupt handling (IRQ, NMI), reset sequences, and the BRK instruction
/// to set the program counter to the appropriate interrupt handler address.
///
/// # Arguments
///
/// * `regs` - A mutable reference to the CPU's register file
/// * `memory` - A mutable reference to the memory space to read from
/// * `vector` - The specific interrupt vector to read (NMI, IRQ/BRK, or Reset)
/// * `pos` - Whether to read the low or high byte of the vector address
fn read_interrupt_vector(
    regs: &mut RegisterFile,
    memory: &mut impl MemorySpace,
    vector: InterruptVector,
    pos: InterruptVectorAddrBytePos,
) {
    regs.addr.set_u16(vector.addr(pos));
    let value = memory.read(regs.addr.to_u16());

    match pos {
        InterruptVectorAddrBytePos::Low => regs.pc.set_low_u8(value),
        InterruptVectorAddrBytePos::High => regs.pc.set_high_u8(value),
    }
}

/// Executes one step of an interrupt handling sequence (IRQ or NMI).
///
/// This function handles the multi-cycle process of responding to either an IRQ or NMI interrupt.
/// The process involves:
/// - Step 0-1: Perform dummy reads from PC (internal operation cycles)
/// - Step 2: Push high byte of PC onto stack
/// - Step 3: Push low byte of PC onto stack
/// - Step 4: Push processor status (P) onto stack with the appropriate B flag state, then set the I flag
/// - Step 5: Load the low byte of the appropriate interrupt vector into PC
/// - Step 6: Load the high byte of the interrupt vector into PC
///
/// # Arguments
///
/// * `step` - The current step number (0-6) within the multi-cycle sequence
/// * `regs` - A mutable reference to the CPU's register file
/// * `memory` - A mutable reference to the memory space for reading/writing
/// * `vector` - The specific interrupt vector to use (IRQ/BRK or NMI)
/// * `set_break_flag` - Whether to set the break flag in the pushed status byte (true for BRK, false for hardware interrupts)
///
/// # Returns
///
/// * `ClockEndStatus::Continue` - After steps 0 through 5, indicating more cycles are needed
/// * `ClockEndStatus::EndInstruction` - After step 6, indicating the interrupt handling sequence is complete
fn handle_interrupt(
    step: u8,
    regs: &mut RegisterFile,
    memory: &mut impl MemorySpace,
    vector: InterruptVector,
    set_break_flag: bool,
) -> ClockEndStatus {
    match step {
        0 => {
            let _ = read_pc(regs, memory);
        }
        1 => {
            let _ = read_pc(regs, memory);
        }
        2 => write_stack(regs, memory, regs.pc.high_u8()),
        3 => write_stack(regs, memory, regs.pc.low_u8()),
        4 => {
            let mut status = regs.status;
            if set_break_flag {
                status.set_flags(StatusRegFlags::BREAK);
            }
            write_stack(regs, memory, status.to_u8());
            regs.status.set_flags(StatusRegFlags::IRQ_DISABLE);
        }
        5 => read_interrupt_vector(regs, memory, vector, InterruptVectorAddrBytePos::Low),
        6 => {
            read_interrupt_vector(regs, memory, vector, InterruptVectorAddrBytePos::High);
            return ClockEndStatus::EndInstruction;
        }
        _ => unreachable!(),
    }
    ClockEndStatus::Continue
}

/// Executes the instruction fetch cycle of the CPU.
///
/// This function performs the initial fetch of an instruction's opcode from memory.
/// It's the first step in the execution of any instruction and is responsible for:
/// 1. Reading the opcode byte from memory at the current program counter (PC) address
/// 2. Storing this opcode in the instruction register (IR)
/// 3. Incrementing the program counter to prepare for operand fetch
///
/// This function is called at the start of each new instruction cycle and sets up
/// the CPU state for instruction decoding and subsequent execution.
///
/// # Arguments
///
/// * `step` - The current step (always 0 for this function as it's a single-cycle operation)
/// * `regs` - A mutable reference to the CPU's register file
/// * `memory` - A mutable reference to the memory space to read the opcode from
///
/// # Returns
///
/// * `ClockEndStatus::EndInstructionNextFetched` - Always returns this status with the current PC value,
///   indicating that an opcode has been fetched and the CPU should decode and execute it
fn fetch_instr(step: u8, regs: &mut RegisterFile, memory: &mut impl MemorySpace) -> ClockEndStatus {
    match step {
        0 => {
            let instruction = read_pc(regs, memory);
            regs.ir.set_u8(instruction);
            let result = ClockEndStatus::EndInstructionNextFetched {
                opcode_addr: regs.pc.to_u16(),
            };
            regs.pc.inc();
            result
        }
        _ => unreachable!(),
    }
}

/// Executes one step of the BRK (Break) instruction sequence.
///
/// The BRK instruction initiates a software interrupt. This multi-cycle process typically involves:
/// (Cycle 1: Fetch BRK opcode - handled externally)
/// Step 0 (Cycle 2): Read and discard the byte after BRK (padding), increment PC.
/// Step 1 (Cycle 3): Push high byte of PC onto stack.
/// Step 2 (Cycle 4): Push low byte of PC onto stack.
/// Step 3 (Cycle 5): Push the processor status register (P) onto the stack, with the B (Break) flag set. Set the I (Interrupt Disable) flag.
/// Step 4 (Cycle 6): Load the low byte of the interrupt vector address ($FFFE) into PCL.
/// Step 5 (Cycle 7): Load the high byte of the interrupt vector address ($FFFF) into PCH.
///
/// This function performs a single step of this sequence based on the `step` parameter.
///
/// # Arguments
///
/// * `step` - The current step number (0-5) within the multi-cycle sequence, corresponding to cycles 2-7.
/// * `regs` - A mutable reference to the CPU's register file.
/// * `memory` - A mutable reference to the memory space for reading/writing.
///
/// # Returns
///
/// * `ClockEndStatus` - Indicates whether the sequence should continue (`Continue`) or if the
///   interrupt handling sequence is complete (`EndInstruction`).
fn break_instr(step: u8, regs: &mut RegisterFile, memory: &mut impl MemorySpace) -> ClockEndStatus {
    match step {
        0 => {
            let _ = read_pc_inc(regs, memory);
        }
        1 => write_stack(regs, memory, regs.pc.high_u8()),
        2 => write_stack(regs, memory, regs.pc.low_u8()),
        3 => {
            let mut status = regs.status;
            status.set_flags(StatusRegFlags::BREAK);
            write_stack(regs, memory, status.to_u8());
            regs.status.set_flags(StatusRegFlags::IRQ_DISABLE);
        }
        4 => read_interrupt_vector(
            regs,
            memory,
            InterruptVector::Interrupt,
            InterruptVectorAddrBytePos::Low,
        ),
        5 => {
            read_interrupt_vector(
                regs,
                memory,
                InterruptVector::Interrupt,
                InterruptVectorAddrBytePos::High,
            );
            return ClockEndStatus::EndInstruction;
        }
        _ => unreachable!(),
    }

    ClockEndStatus::Continue
}

/// Executes one step of the NMI (Non-Maskable Interrupt) sequence.
///
/// An NMI is triggered externally and cannot be disabled by the I flag. This multi-cycle process typically involves:
/// Step 0 (Cycle 1): Perform dummy read (e.g., re-read last instruction byte).
/// Step 1 (Cycle 2): Perform dummy read (e.g., re-read last instruction byte).
/// Step 2 (Cycle 3): Push high byte of the current PC onto stack.
/// Step 3 (Cycle 4): Push low byte of the current PC onto stack.
/// Step 4 (Cycle 5): Push the processor status register (P) onto the stack, with the B (Break) flag clear. Set the I (Interrupt Disable) flag.
/// Step 5 (Cycle 6): Load the low byte of the NMI vector address ($FFFA) into PCL.
/// Step 6 (Cycle 7): Load the high byte of the NMI vector address ($FFFB) into PCH.
///
/// This function performs a single step of this sequence based on the `step` parameter.
///
/// # Arguments
///
/// * `step` - The current step number (0-6) within the multi-cycle sequence, corresponding to cycles 1-7.
/// * `regs` - A mutable reference to the CPU's register file.
/// * `memory` - A mutable reference to the memory space for reading/writing.
///
/// # Returns
///
/// * `ClockEndStatus` - Indicates whether the sequence should continue (`Continue`) or if the
///   interrupt handling sequence is complete (`EndInstruction`).
fn start_irq(step: u8, regs: &mut RegisterFile, memory: &mut impl MemorySpace) -> ClockEndStatus {
    handle_interrupt(step, regs, memory, InterruptVector::Interrupt, false)
}

/// Executes one step of the NMI (Non-Maskable Interrupt) sequence.
///
/// An NMI is triggered externally and cannot be disabled by the I flag. This multi-cycle process typically involves:
/// Step 0 (Cycle 1): Perform dummy read (e.g., re-read last instruction byte).
/// Step 1 (Cycle 2): Perform dummy read (e.g., re-read last instruction byte).
/// Step 2 (Cycle 3): Push high byte of the current PC onto stack.
/// Step 3 (Cycle 4): Push low byte of the current PC onto stack.
/// Step 4 (Cycle 5): Push the processor status register (P) onto the stack, with the B (Break) flag clear. Set the I (Interrupt Disable) flag.
/// Step 5 (Cycle 6): Load the low byte of the NMI vector address ($FFFA) into PCL.
/// Step 6 (Cycle 7): Load the high byte of the NMI vector address ($FFFB) into PCH.
///
/// This function performs a single step of this sequence based on the `step` parameter.
///
/// # Arguments
///
/// * `step` - The current step number (0-6) within the multi-cycle sequence, corresponding to cycles 1-7.
/// * `regs` - A mutable reference to the CPU's register file.
/// * `memory` - A mutable reference to the memory space for reading/writing.
///
/// # Returns
///
/// * `ClockEndStatus` - Indicates whether the sequence should continue (`Continue`) or if the
///   interrupt handling sequence is complete (`EndInstruction`).
fn start_nmi(step: u8, regs: &mut RegisterFile, memory: &mut impl MemorySpace) -> ClockEndStatus {
    handle_interrupt(
        step,
        regs,
        memory,
        InterruptVector::NonMaskableInterrupt,
        false,
    )
}

/// Executes one step of the push operation (PHA or PHP instruction) sequence.
///
/// The push operation stores a value from either the Accumulator (A) or Status register (P) onto the stack.
/// This multi-cycle process involves:
/// (Cycle 1: Fetch opcode - handled externally)
/// Step 0 (Cycle 2): Dummy read from the PC address.
/// Step 1 (Cycle 3): Push the value (A or P) onto the stack at the current stack pointer address, then decrement SP.
///
/// # Arguments
///
/// * `step` - The current step number (0-1) within the multi-cycle sequence.
/// * `regs` - A mutable reference to the CPU's register file.
/// * `memory` - A mutable reference to the memory space for reading/writing.
/// * `op` - The specific push operation (PushA for accumulator, PushStatus for processor status).
///
/// # Returns
///
/// * `ClockEndStatus` - Indicates whether the sequence should continue (`Continue`) or if the
///   instruction is complete (`EndInstruction`).
fn push(
    step: u8,
    regs: &mut RegisterFile,
    memory: &mut impl MemorySpace,
    op: PushStackOperation,
) -> ClockEndStatus {
    match step {
        0 => {
            let _ = memory.read(regs.pc.to_u16());
        }
        1 => {
            set_stack_address(regs);

            match op {
                PushStackOperation::PushA => {
                    let data = regs.a.to_u8();
                    memory.write(data, regs.addr.to_u16());
                }
                PushStackOperation::PushStatus => {
                    let mut status = regs.status.to_u8();
                    status |= (StatusRegFlags::BREAK | StatusRegFlags::UNUSED).bits();
                    memory.write(status, regs.addr.to_u16());
                }
            }

            regs.sp.dec();

            return ClockEndStatus::EndInstruction;
        }
        _ => unreachable!(),
    }

    ClockEndStatus::Continue
}

/// Executes one step of the pull operation (PLA or PLP instruction) sequence.
///
/// The pull operation retrieves a value from the stack and loads it into either the Accumulator (A)
/// or Status register (P). This multi-cycle process involves:
/// (Cycle 1: Fetch opcode - handled externally)
/// Step 0 (Cycle 2): Dummy read from the PC address.
/// Step 1 (Cycle 3): Increment the stack pointer.
/// Step 2 (Cycle 4): Read the value from the stack at the current stack pointer address and load it
///                    into either A or P. For PLA, update the N and Z status flags based on the value.
///
/// # Arguments
///
/// * `step` - The current step number (0-2) within the multi-cycle sequence.
/// * `regs` - A mutable reference to the CPU's register file.
/// * `memory` - A mutable reference to the memory space for reading/writing.
/// * `op` - The specific pull operation (PullA for accumulator, PullStatus for processor status).
///
/// # Returns
///
/// * `ClockEndStatus` - Indicates whether the sequence should continue (`Continue`) or if the
///   instruction is complete (`EndInstruction`).
fn pull(
    step: u8,
    regs: &mut RegisterFile,
    memory: &mut impl MemorySpace,
    op: PullStackOperation,
) -> ClockEndStatus {
    match step {
        0 => {
            let _ = read_pc(regs, memory);
        }
        1 => regs.sp.inc(),
        2 => {
            set_stack_address(regs);
            let data = memory.read(regs.addr.to_u16());

            match op {
                PullStackOperation::PullA => {
                    regs.a.set_u8(data);
                    alu::update_status_nz(data as i8, &mut regs.status);
                }
                PullStackOperation::PullStatus => {
                    let ignored_bits = (StatusRegFlags::BREAK | StatusRegFlags::UNUSED).bits();
                    let status = regs.status.to_u8();
                    let new_status = (status & ignored_bits) | (data & !ignored_bits);
                    regs.status.set_u8(new_status);
                }
            }

            return ClockEndStatus::EndInstruction;
        }
        _ => unreachable!(),
    }

    ClockEndStatus::Continue
}

/// Executes one step of the Reset sequence.
///
/// The Reset sequence initializes the CPU. This multi-cycle process typically involves:
/// Step 0 (Cycle 1): Dummy read of the current PC.
/// Step 1 (Cycle 2): Dummy read from stack pointer address, decrement SP.
/// Step 2 (Cycle 3): Dummy read from stack pointer address, decrement SP.
/// Step 3 (Cycle 4): Dummy write to stack pointer address (status register is pushed), decrement SP.
/// Step 4 (Cycle 5): Read the low byte of the program start vector address ($FFFC) into PCL.
/// Step 5 (Cycle 6): Read the high byte of the program start vector address ($FFFD) into PCH.
///
/// This function performs a single step of this sequence based on the `step` parameter.
///
/// # Arguments
///
/// * `step` - The current step number (0-5) within the multi-cycle sequence.
/// * `regs` - A mutable reference to the CPU's register file.
/// * `memory` - A mutable reference to the memory space for reading/writing.
///
/// # Returns
///
/// * `ClockEndStatus` - Indicates whether the sequence should continue (`Continue`) or if the
///   reset sequence is complete and the CPU should start fetching the first instruction (`EndInstruction`).
fn reset(step: u8, regs: &mut RegisterFile, memory: &mut impl MemorySpace) -> ClockEndStatus {
    match step {
        0 => {
            let _ = read_pc(regs, memory);
        }
        1 => {
            set_stack_address(regs);
            let _ = memory.read(regs.addr.to_u16());
            regs.sp.dec();
        }
        2 => {
            set_stack_address(regs);
            let _ = memory.read(regs.addr.to_u16());
            regs.sp.dec();
        }
        3 => {
            set_stack_address(regs);
            write_stack(regs, memory, regs.status.to_u8());
        }
        4 => {
            read_interrupt_vector(
                regs,
                memory,
                InterruptVector::ProgramStart,
                InterruptVectorAddrBytePos::Low,
            );
        }
        5 => {
            read_interrupt_vector(
                regs,
                memory,
                InterruptVector::ProgramStart,
                InterruptVectorAddrBytePos::High,
            );
            return ClockEndStatus::EndInstruction;
        }
        _ => unreachable!(),
    }

    ClockEndStatus::Continue
}

/// Executes one step of the RTI (Return from Interrupt) instruction sequence.
///
/// The RTI instruction is used to return from an interrupt service routine. This multi-cycle process involves:
/// (Cycle 1: Fetch RTI opcode - handled externally)
/// Step 0 (Cycle 2): Dummy read of the byte after the opcode.
/// Step 1 (Cycle 3): Increment stack pointer (points to the stored status register).
/// Step 2 (Cycle 4): Pull the status register (P) from the stack and update the CPU's status register. Increment stack pointer.
/// Step 3 (Cycle 5): Pull the low byte of the program counter (PCL) from the stack and update PCL. Increment stack pointer.
/// Step 4 (Cycle 6): Pull the high byte of the program counter (PCH) from the stack and update PCH.
///
/// This function performs a single step of this sequence based on the `step` parameter.
///
/// # Arguments
///
/// * `step` - The current step number (0-4) within the multi-cycle sequence, corresponding to cycles 2-6.
/// * `regs` - A mutable reference to the CPU's register file.
/// * `memory` - A mutable reference to the memory space for reading.
///
/// # Returns
///
/// * `ClockEndStatus` - Indicates whether the sequence should continue (`Continue`) or if the
///   instruction is complete (`EndInstruction`).
fn return_interrupt(
    step: u8,
    regs: &mut RegisterFile,
    memory: &mut impl MemorySpace,
) -> ClockEndStatus {
    match step {
        0 => {
            let _ = read_pc(regs, memory);
        }
        1 => regs.sp.inc(),
        2 => {
            set_stack_address(regs);
            let status = memory.read(regs.addr.to_u16());
            let ignored_bits = (StatusRegFlags::BREAK | StatusRegFlags::UNUSED).bits();
            let current_status = regs.status.to_u8();
            let new_status = (current_status & ignored_bits) | (status & !ignored_bits);
            regs.status.set_u8(new_status);
            regs.sp.inc();
        }
        3 => {
            set_stack_address(regs);
            let pcl = memory.read(regs.addr.to_u16());
            regs.pc.set_low_u8(pcl);
            regs.sp.inc();
        }
        4 => {
            set_stack_address(regs);
            let pch = memory.read(regs.addr.to_u16());
            regs.pc.set_high_u8(pch);
            return ClockEndStatus::EndInstruction;
        }
        _ => unreachable!(),
    }

    ClockEndStatus::Continue
}

/// Executes one step of the JSR (Jump to Subroutine) instruction sequence.
///
/// The JSR instruction saves the return address and transfers control to a subroutine at a specified address.
/// This multi-cycle process involves:
/// (Cycle 1: Fetch JSR opcode - handled externally)
/// Step 0 (Cycle 2): Read the low byte of the subroutine address from the next byte after the opcode into temp register.
/// Step 1 (Cycle 3): Perform an internal operation (setting up stack address, performing a dummy read).
/// Step 2 (Cycle 4): Push the high byte of the return address (PC+1) onto the stack.
/// Step 3 (Cycle 5): Push the low byte of the return address onto the stack.
/// Step 4 (Cycle 6): Read the high byte of the subroutine address and combine with the previously read low byte to set the program counter.
///
/// # Arguments
///
/// * `step` - The current step number (0-4) within the multi-cycle sequence, corresponding to cycles 2-6.
/// * `regs` - A mutable reference to the CPU's register file.
/// * `memory` - A mutable reference to the memory space for reading/writing.
///
/// # Returns
///
/// * `ClockEndStatus` - Indicates whether the sequence should continue (`Continue`) or if the
///   instruction is complete (`EndInstruction`).
fn jump_subroutine(
    step: u8,
    regs: &mut RegisterFile,
    memory: &mut impl MemorySpace,
) -> ClockEndStatus {
    match step {
        0 => {
            let data = read_pc(regs, memory);
            regs.tmp.set_u8(data);
            regs.pc.inc();
        }
        1 => {
            set_stack_address(regs);
            let _ = memory.read(regs.addr.to_u16());
        }
        2 => write_stack(regs, memory, regs.pc.high_u8()),
        3 => write_stack(regs, memory, regs.pc.low_u8()),
        4 => {
            let data = read_pc(regs, memory);
            regs.pc.set_high_u8(data);
            regs.pc.set_low_u8(regs.tmp.to_u8());
            return ClockEndStatus::EndInstruction;
        }
        _ => unreachable!(),
    }

    ClockEndStatus::Continue
}

/// Executes one step of the RTS (Return from Subroutine) instruction sequence.
///
/// The RTS instruction returns control from a subroutine to the calling program. This multi-cycle process involves:
/// (Cycle 1: Fetch RTS opcode - handled externally)
/// Step 0 (Cycle 2): Dummy read of the byte after the opcode.
/// Step 1 (Cycle 3): Increment stack pointer.
/// Step 2 (Cycle 4): Pull the low byte of the return address from the stack into a temporary register. Increment stack pointer.
/// Step 3 (Cycle 5): Pull the high byte of the return address from the stack and update the program counter.
/// Step 4 (Cycle 6): Increment the program counter to skip past the JSR instruction's operand.
///
/// # Arguments
///
/// * `step` - The current step number (0-4) within the multi-cycle sequence, corresponding to cycles 2-6.
/// * `regs` - A mutable reference to the CPU's register file.
/// * `memory` - A mutable reference to the memory space for reading.
///
/// # Returns
///
/// * `ClockEndStatus` - Indicates whether the sequence should continue (`Continue`) or if the
///   instruction is complete (`EndInstruction`).
fn return_subroutine(
    step: u8,
    regs: &mut RegisterFile,
    memory: &mut impl MemorySpace,
) -> ClockEndStatus {
    match step {
        0 => {
            let _ = read_pc(regs, memory);
        }
        1 => regs.sp.inc(),
        2 => {
            set_stack_address(regs);
            let data = memory.read(regs.addr.to_u16());
            regs.tmp.set_u8(data);
            regs.sp.inc();
        }
        3 => {
            set_stack_address(regs);
            let data = memory.read(regs.addr.to_u16());
            regs.pc.set_high_u8(data);
            regs.pc.set_low_u8(regs.tmp.to_u8());
        }
        4 => {
            regs.pc.inc();
            return ClockEndStatus::EndInstruction;
        }
        _ => unreachable!(),
    }

    ClockEndStatus::Continue
}

/// Executes an implied mode instruction operation.
///
/// Implied mode instructions have no operands and operate on registers or flags directly.
/// These instructions are typically single-cycle operations (after the opcode fetch) and
/// include operations like NOP, register transfers (TAX, TXA, etc.), and flag manipulations (CLC, SEC, etc.).
///
/// # Arguments
///
/// * `_step` - The step counter (unused in implied operations as they only take one cycle).
/// * `regs` - A mutable reference to the CPU's register file.
/// * `_memory` - A mutable reference to the memory space (unused in implied operations).
/// * `op` - The specific implicit operation to execute.
///
/// # Returns
///
/// * `ClockEndStatus::EndInstruction` - Always returns this as implied instructions complete in one cycle.
fn implied(
    _step: u8,
    regs: &mut RegisterFile,
    _memory: &mut impl MemorySpace,
    op: ImplicitOperation,
) -> ClockEndStatus {
    execute_implicit_op(op, regs);

    ClockEndStatus::EndInstruction
}

/// Executes an immediate mode instruction operation.
///
/// Immediate mode instructions use the byte immediately following the opcode as the operand,
/// rather than reading a value from memory. This mode is typically used for loading immediate
/// values into registers or for immediate arithmetic/logical operations.
///
/// The process involves:
/// (Cycle 1: Fetch opcode - handled externally)
/// (Single step): Read the immediate byte from the program counter address, increment PC,
/// and perform the operation using this immediate value.
///
/// # Arguments
///
/// * `_step` - The step counter (unused in immediate operations as they only take one cycle).
/// * `regs` - A mutable reference to the CPU's register file.
/// * `memory` - A mutable reference to the memory space for reading the immediate value.
/// * `op` - The specific register-memory operation to execute with the immediate value.
///
/// # Returns
///
/// * `ClockEndStatus::EndInstruction` - Always returns this as immediate instructions complete in one cycle.
fn immediate(
    _step: u8,
    regs: &mut RegisterFile,
    memory: &mut impl MemorySpace,
    op: RegisterMemoryOperation,
) -> ClockEndStatus {
    regs.addr = regs.pc;
    regs.pc.inc();

    execute_op(op, regs, memory);

    ClockEndStatus::EndInstruction
}

/// Executes an absolute jump instruction operation (JMP).
///
/// Absolute jump instructions change the program counter to a new 16-bit address
/// specified by the two bytes following the opcode. This is a direct jump to
/// the target address without any conditions.
///
/// The process involves:
/// (Cycle 1: Fetch opcode - handled externally)
/// Cycle 2 (step 0): Read the low byte of the target address from memory at PC,
///                   store it in a temporary register, and increment PC.
/// Cycle 3 (step 1): Read the high byte of the target address from memory at PC
///                   and set the PC to the complete 16-bit address.
///
/// # Arguments
///
/// * `step` - The current step in the execution process (0 or 1).
/// * `regs` - A mutable reference to the CPU's register file.
/// * `memory` - A mutable reference to the memory space for reading address bytes.
///
/// # Returns
///
/// * `ClockEndStatus::Continue` - After step 0, indicating more cycles are needed.
/// * `ClockEndStatus::EndInstruction` - After step 1, indicating the instruction is complete.
fn absolute_jump(
    step: u8,
    regs: &mut RegisterFile,
    memory: &mut impl MemorySpace,
) -> ClockEndStatus {
    match step {
        0 => {
            let addr_low = read_pc(regs, memory);
            regs.tmp.set_u8(addr_low);
            regs.pc.inc();
        }
        1 => {
            let addr_high = read_pc(regs, memory);
            regs.pc.set_high_u8(addr_high);
            regs.pc.set_low_u8(regs.tmp.to_u8());
            return ClockEndStatus::EndInstruction;
        }
        _ => unreachable!(),
    }

    ClockEndStatus::Continue
}

/// Executes an absolute mode instruction operation.
///
/// Absolute mode instructions operate on a memory location specified by a full 16-bit address
/// that follows the opcode. This addressing mode allows access to any memory location in the
/// entire address space.
///
/// The process involves:
/// (Cycle 1: Fetch opcode - handled externally)
/// Cycle 2 (step 0): Read the low byte of the target address from memory at PC,
///                   store it in the address register, and increment PC.
/// Cycle 3 (step 1): Read the high byte of the target address from memory at PC,
///                   complete the 16-bit address, and increment PC.
/// Cycle 4 (step 2): Execute the operation on the memory location specified by the address.
///
/// # Arguments
///
/// * `step` - The current step in the execution process (0, 1, or 2).
/// * `regs` - A mutable reference to the CPU's register file.
/// * `memory` - A mutable reference to the memory space for reading address bytes and accessing memory.
/// * `op` - The specific register-memory operation to execute at the absolute address.
///
/// # Returns
///
/// * `ClockEndStatus::Continue` - After steps 0 and 1, indicating more cycles are needed.
/// * `ClockEndStatus::EndInstruction` - After step 2, indicating the instruction is complete.
fn absolute(
    step: u8,
    regs: &mut RegisterFile,
    memory: &mut impl MemorySpace,
    op: RegisterMemoryOperation,
) -> ClockEndStatus {
    match step {
        0 => {
            let addr_low = read_pc(regs, memory);
            regs.addr.set_low_u8(addr_low);
            regs.pc.inc();
        }
        1 => {
            let addr_high = read_pc(regs, memory);
            regs.addr.set_high_u8(addr_high);
            regs.pc.inc();
        }
        2 => {
            execute_op(op, regs, memory);
            return ClockEndStatus::EndInstruction;
        }
        _ => unreachable!(),
    }

    ClockEndStatus::Continue
}

/// Executes an absolute mode read-modify-write (RMW) instruction operation.
///
/// Absolute RMW instructions operate on a memory location specified by a full 16-bit address
/// that follows the opcode. These instructions read a value from memory, modify it according to
/// the operation, and write the result back to the same memory location. Common examples include
/// increment (INC), decrement (DEC), and shift/rotate operations.
///
/// The process involves:
/// (Cycle 1: Fetch opcode - handled externally)
/// Cycle 2 (step 0): Read the low byte of the target address from memory at PC,
///                   store it in the address register, and increment PC.
/// Cycle 3 (step 1): Read the high byte of the target address from memory at PC,
///                   complete the 16-bit address, and increment PC.
/// Cycle 4 (step 2): Read the operand from the specified memory address and store it in a temporary register.
/// Cycle 5 (step 3): Write the unchanged value back to memory (dummy write), then perform the modification operation.
/// Cycle 6 (step 4): Write the modified value back to memory.
///
/// # Arguments
///
/// * `step` - The current step in the execution process (0 through 4).
/// * `regs` - A mutable reference to the CPU's register file.
/// * `memory` - A mutable reference to the memory space for reading/writing.
/// * `op` - The specific memory modification operation to execute.
///
/// # Returns
///
/// * `ClockEndStatus::Continue` - After steps 0 through 3, indicating more cycles are needed.
/// * `ClockEndStatus::EndInstruction` - After step 4, indicating the instruction is complete.
fn absolute_rmw(
    step: u8,
    regs: &mut RegisterFile,
    memory: &mut impl MemorySpace,
    op: MemoryModifyOperation,
) -> ClockEndStatus {
    match step {
        0 => {
            let addr_low = read_pc(regs, memory);
            regs.addr.set_low_u8(addr_low);
            regs.pc.inc();
        }
        1 => {
            let addr_high = read_pc(regs, memory);
            regs.addr.set_high_u8(addr_high);
            regs.pc.inc();
        }
        2 => {
            let operand = memory.read(regs.addr.to_u16());
            regs.tmp.set_u8(operand);
        }
        3 => {
            memory.write(regs.tmp.to_u8(), regs.addr.to_u16());
            execute_memory_modify_op(op, regs);
        }
        4 => {
            memory.write(regs.tmp.to_u8(), regs.addr.to_u16());
            return ClockEndStatus::EndInstruction;
        }
        _ => unreachable!(),
    }

    ClockEndStatus::Continue
}

/// Executes a zero-page mode instruction operation.
///
/// Zero-page mode instructions operate on a memory location in the first 256 bytes of memory (the "zero-page").
/// This is specified by a single byte address that follows the opcode. This addressing mode is more efficient
/// in terms of both memory usage and execution time, as it requires only a single byte for the address and
/// eliminates the need for carrying from low byte to high byte during address calculations.
///
/// The process involves:
/// (Cycle 1: Fetch opcode - handled externally)
/// Cycle 2 (step 0): Read the zero-page address from memory at PC, store it in the address register,
///                   set the high byte of the address to 0, and increment PC.
/// Cycle 3 (step 1): Execute the operation on the memory location specified by the zero-page address.
///
/// # Arguments
///
/// * `step` - The current step in the execution process (0 or 1).
/// * `regs` - A mutable reference to the CPU's register file.
/// * `memory` - A mutable reference to the memory space for reading/writing.
/// * `op` - The specific register-memory operation to execute at the zero-page address.
///
/// # Returns
///
/// * `ClockEndStatus::Continue` - After step 0, indicating more cycles are needed.
/// * `ClockEndStatus::EndInstruction` - After step 1, indicating the instruction is complete.
fn zero_page(
    step: u8,
    regs: &mut RegisterFile,
    memory: &mut impl MemorySpace,
    op: RegisterMemoryOperation,
) -> ClockEndStatus {
    match step {
        0 => {
            let addr_low = read_pc(regs, memory);
            regs.addr.set_low_u8(addr_low);
            regs.pc.inc();
            regs.addr.set_high_u8(0);
        }
        1 => {
            execute_op(op, regs, memory);
            return ClockEndStatus::EndInstruction;
        }
        _ => unreachable!(),
    }

    ClockEndStatus::Continue
}

/// Executes a zero-page read-modify-write (RMW) instruction operation.
///
/// Zero-page RMW instructions operate on a memory location in the first 256 bytes of memory (the "zero-page").
/// These instructions read a value from memory, modify it according to the operation, and write
/// the result back to the same memory location. Common examples include increment (INC),
/// decrement (DEC), and shift/rotate operations on zero-page memory locations.
///
/// The process involves:
/// (Cycle 1: Fetch opcode - handled externally)
/// Cycle 2 (step 0): Read the zero-page address from memory at PC, store it in the address register,
///                   set the high byte of the address to 0, and increment PC.
/// Cycle 3 (step 1): Read the operand from the specified zero-page memory address and store it in a temporary register.
/// Cycle 4 (step 2): Write the unchanged value back to memory (dummy write), then perform the modification operation.
/// Cycle 5 (step 3): Write the modified value back to memory.
///
/// # Arguments
///
/// * `step` - The current step in the execution process (0 through 3).
/// * `regs` - A mutable reference to the CPU's register file.
/// * `memory` - A mutable reference to the memory space for reading/writing.
/// * `op` - The specific memory modification operation to execute.
///
/// # Returns
///
/// * `ClockEndStatus::Continue` - After steps 0 through 2, indicating more cycles are needed.
/// * `ClockEndStatus::EndInstruction` - After step 3, indicating the instruction is complete.
fn zero_page_rmw(
    step: u8,
    regs: &mut RegisterFile,
    memory: &mut impl MemorySpace,
    op: MemoryModifyOperation,
) -> ClockEndStatus {
    match step {
        0 => {
            let addr_low = read_pc(regs, memory);
            regs.addr.set_low_u8(addr_low);
            regs.pc.inc();
            regs.addr.set_high_u8(0);
        }
        1 => {
            let operand = memory.read(regs.addr.to_u16());
            regs.tmp.set_u8(operand);
        }
        2 => {
            memory.write(regs.tmp.to_u8(), regs.addr.to_u16());
            execute_memory_modify_op(op, regs);
        }
        3 => {
            memory.write(regs.tmp.to_u8(), regs.addr.to_u16());
            return ClockEndStatus::EndInstruction;
        }
        _ => unreachable!(),
    }

    ClockEndStatus::Continue
}

/// Executes a zero-page indexed mode instruction operation.
///
/// Zero-page indexed mode instructions operate on a memory location in the first 256 bytes of memory (the "zero-page")
/// with an offset from an index register (X or Y). The zero-page address is provided as a single byte after the opcode,
/// and the final address is computed by adding the index register value to the zero-page address (with wrap-around
/// within the zero-page).
///
/// The process involves:
/// (Cycle 1: Fetch opcode - handled externally)
/// Cycle 2 (step 0): Read the zero-page address from memory at PC, store it in the address register,
///                   set the high byte of the address to 0, and increment PC.
/// Cycle 3 (step 1): Read from the unindexed zero-page address (dummy read).
///                   Add the index register value to the low byte of the address (with wrap-around in zero-page).
/// Cycle 4 (step 2): Execute the operation on the memory location specified by the indexed zero-page address.
///
/// # Arguments
///
/// * `step` - The current step in the execution process (0, 1, or 2).
/// * `regs` - A mutable reference to the CPU's register file.
/// * `memory` - A mutable reference to the memory space for reading/writing.
/// * `op` - The specific register-memory operation to execute at the indexed zero-page address.
/// * `idx` - The index register (X or Y) to use for the address calculation.
///
/// # Returns
///
/// * `ClockEndStatus::Continue` - After steps 0 and 1, indicating more cycles are needed.
/// * `ClockEndStatus::EndInstruction` - After step 2, indicating the instruction is complete.
fn zero_page_indexed(
    step: u8,
    regs: &mut RegisterFile,
    memory: &mut impl MemorySpace,
    op: RegisterMemoryOperation,
    idx: IndexRegister,
) -> ClockEndStatus {
    match step {
        0 => {
            let addr_low = memory.read(regs.pc.to_u16());
            regs.addr.set_low_u8(addr_low);
            regs.pc.inc();
            regs.addr.set_high_u8(0);
        }
        1 => {
            let _ = memory.read(regs.addr.to_u16());

            add_index_to_address(regs, idx);
        }
        2 => {
            execute_op(op, regs, memory);
            return ClockEndStatus::EndInstruction;
        }
        _ => unreachable!(),
    }

    ClockEndStatus::Continue
}

/// Executes a zero-page indexed read-modify-write (RMW) instruction operation.
///
/// Zero-page indexed RMW instructions operate on a memory location in the first 256 bytes of memory (the "zero-page")
/// with an offset from an index register (X or Y). These instructions read a value from memory, modify it according to
/// the operation, and write the result back to the same memory location. The zero-page address is provided as a single
/// byte after the opcode, and the final address is computed by adding the index register value to the zero-page address
/// (with wrap-around within the zero-page).
///
/// The process involves:
/// (Cycle 1: Fetch opcode - handled externally)
/// Cycle 2 (step 0): Read the zero-page address from memory at PC, store it in the address register,
///                   set the high byte of the address to 0, and increment PC.
/// Cycle 3 (step 1): Read from the unindexed zero-page address (dummy read).
///                   Add the index register value to the low byte of the address (with wrap-around in zero-page).
/// Cycle 4 (step 2): Read the operand from the indexed zero-page memory address and store it in a temporary register.
/// Cycle 5 (step 3): Write the unchanged value back to memory (dummy write), then perform the modification operation.
/// Cycle 6 (step 4): Write the modified value back to memory.
///
/// # Arguments
///
/// * `step` - The current step in the execution process (0 through 4).
/// * `regs` - A mutable reference to the CPU's register file.
/// * `memory` - A mutable reference to the memory space for reading/writing.
/// * `op` - The specific memory modification operation to execute.
/// * `idx` - The index register (X or Y) to use for the address calculation.
///
/// # Returns
///
/// * `ClockEndStatus::Continue` - After steps 0 through 3, indicating more cycles are needed.
/// * `ClockEndStatus::EndInstruction` - After step 4, indicating the instruction is complete.
fn zero_page_indexed_rmw(
    step: u8,
    regs: &mut RegisterFile,
    memory: &mut impl MemorySpace,
    op: MemoryModifyOperation,
    idx: IndexRegister,
) -> ClockEndStatus {
    match step {
        0 => {
            let addr_low = memory.read(regs.pc.to_u16());
            regs.addr.set_low_u8(addr_low);
            regs.pc.inc();
            regs.addr.set_high_u8(0);
        }
        1 => {
            let _ = memory.read(regs.addr.to_u16());
            add_index_to_address(regs, idx);
        }
        2 => {
            let data = memory.read(regs.addr.to_u16());
            regs.tmp.set_u8(data);
        }
        3 => {
            memory.write(regs.tmp.to_u8(), regs.addr.to_u16());
            execute_memory_modify_op(op, regs);
        }
        4 => {
            memory.write(regs.tmp.to_u8(), regs.addr.to_u16());
            return ClockEndStatus::EndInstruction;
        }
        _ => unreachable!(),
    }

    ClockEndStatus::Continue
}

/// Executes an absolute indexed read instruction operation.
///
/// Absolute indexed read instructions operate on a memory location specified by a 16-bit address plus
/// an index register offset (X or Y). These are commonly used for accessing array elements or table entries.
/// If the addition of the index causes a page boundary crossing, an extra cycle is needed.
///
/// The process involves:
/// (Cycle 1: Fetch opcode - handled externally)
/// Cycle 2 (step 0): Read the low byte of the base address from memory at PC, store it in the address register, and increment PC.
/// Cycle 3 (step 1): Read the high byte of the base address from memory at PC, complete the 16-bit address, and increment PC.
/// Cycle 4 (step 2): Add the index register value to the low byte of the address.
///                   If this addition causes a carry to the high byte (page crossing):
///                   - Perform a dummy read from the incorrectly computed address (low byte + index, high byte unchanged)
///                   - Fix the high byte of the address and continue to step 3
///                   If no page boundary is crossed:
///                   - Immediately execute the operation and end the instruction
/// Cycle 5 (step 3): Execute the operation on the memory location specified by the indexed address (only reached if page crossing occurred).
///
/// # Arguments
///
/// * `step` - The current step in the execution process (0 through 3).
/// * `regs` - A mutable reference to the CPU's register file.
/// * `memory` - A mutable reference to the memory space for reading.
/// * `op` - The specific register-memory operation to execute at the indexed address.
/// * `idx` - The index register (X or Y) to use for the address calculation.
///
/// # Returns
///
/// * `ClockEndStatus::Continue` - After steps 0, 1, and possibly 2 (with page crossing), indicating more cycles are needed.
/// * `ClockEndStatus::EndInstruction` - After step 2 (without page crossing) or step 3, indicating the instruction is complete.
fn absolute_indexed_read(
    step: u8,
    regs: &mut RegisterFile,
    memory: &mut impl MemorySpace,
    op: RegisterMemoryOperation,
    idx: IndexRegister,
) -> ClockEndStatus {
    match step {
        0 => {
            let addr_low = memory.read(regs.pc.to_u16());
            regs.addr.set_low_u8(addr_low);
            regs.pc.inc();
        }
        1 => {
            let addr_high = memory.read(regs.pc.to_u16());
            regs.addr.set_high_u8(addr_high);
            regs.pc.inc();
        }
        2 => {
            add_index_to_address(regs, idx);
            //let data = memory.read(regs.addr.to_u16());
            // regs.tmp.set_u8(data);

            return fix_addr_or_run_op_finish(op, regs, memory, idx);
        }
        3 => {
            execute_op(op, regs, memory);
            return ClockEndStatus::EndInstruction;
        }
        _ => unreachable!(),
    }

    ClockEndStatus::Continue
}

/// Executes an absolute indexed read-modify-write (RMW) instruction operation.
///
/// Absolute indexed RMW instructions operate on a memory location specified by a 16-bit address plus
/// an index register offset (X or Y). These instructions read a value from the indexed memory location,
/// modify it according to the operation, and write the result back to the same memory location.
/// Examples include increment/decrement and shift/rotate operations on indexed memory locations.
///
/// The process involves:
/// (Cycle 1: Fetch opcode - handled externally)
/// Cycle 2 (step 0): Read the low byte of the base address from memory at PC, store it in the address register, and increment PC.
/// Cycle 3 (step 1): Read the high byte of the base address from memory at PC, complete the 16-bit address, and increment PC.
/// Cycle 4 (step 2): Add the index register value to the low byte of the address.
///                   Perform a dummy read from the incorrectly computed address (low byte + index, high byte unchanged).
///                   Fix the high byte of the address if page crossing occurred.
/// Cycle 5 (step 3): Read the operand from the correctly computed memory location and store it in a temporary register.
/// Cycle 6 (step 4): Write the unchanged value back to memory (dummy write), then perform the modification operation.
/// Cycle 7 (step 5): Write the modified value back to memory.
///
/// # Arguments
///
/// * `step` - The current step in the execution process (0 through 5).
/// * `regs` - A mutable reference to the CPU's register file.
/// * `memory` - A mutable reference to the memory space for reading/writing.
/// * `op` - The specific memory modification operation to execute.
/// * `idx` - The index register (X or Y) to use for the address calculation.
///
/// # Returns
///
/// * `ClockEndStatus::Continue` - After steps 0 through 4, indicating more cycles are needed.
/// * `ClockEndStatus::EndInstruction` - After step 5, indicating the instruction is complete.
fn absolute_indexed_rmw(
    step: u8,
    regs: &mut RegisterFile,
    memory: &mut impl MemorySpace,
    op: MemoryModifyOperation,
    idx: IndexRegister,
) -> ClockEndStatus {
    match step {
        0 => {
            let addr_low = memory.read(regs.pc.to_u16());
            regs.addr.set_low_u8(addr_low);
            regs.pc.inc();
        }
        1 => {
            let addr_high = memory.read(regs.pc.to_u16());
            regs.addr.set_high_u8(addr_high);
            regs.pc.inc();
        }
        2 => {
            add_index_to_address(regs, idx);
            let _ = memory.read(regs.addr.to_u16());

            let index_value = get_index_value(regs, idx);
            fix_addr(regs, index_value);
        }
        3 => {
            let data = memory.read(regs.addr.to_u16());
            regs.tmp.set_u8(data);
        }
        4 => {
            memory.write(regs.tmp.to_u8(), regs.addr.to_u16());
            execute_memory_modify_op(op, regs);
        }
        5 => {
            memory.write(regs.tmp.to_u8(), regs.addr.to_u16());
            return ClockEndStatus::EndInstruction;
        }
        _ => unreachable!(),
    }

    ClockEndStatus::Continue
}

/// Executes an absolute indexed write instruction operation.
///
/// Absolute indexed write instructions store a value from a register (A, X, or Y) to a memory location
/// specified by a 16-bit address plus an index register offset (X or Y). Unlike read operations,
/// write operations always take an extra cycle to fix the address in case of page crossing.
///
/// The process involves:
/// (Cycle 1: Fetch opcode - handled externally)
/// Cycle 2 (step 0): Read the low byte of the base address from memory at PC, store it in the address register, and increment PC.
/// Cycle 3 (step 1): Read the high byte of the base address from memory at PC, complete the 16-bit address, and increment PC.
/// Cycle 4 (step 2): Add the index register value to the low byte of the address.
///                   Perform a dummy read from the incorrectly computed address (low byte + index, high byte unchanged).
///                   Fix the high byte of the address if page crossing occurred.
/// Cycle 5 (step 3): Execute the write operation to the correctly computed memory location.
///
/// # Arguments
///
/// * `step` - The current step in the execution process (0 through 3).
/// * `regs` - A mutable reference to the CPU's register file.
/// * `memory` - A mutable reference to the memory space for reading/writing.
/// * `op` - The specific register-memory write operation to execute.
/// * `idx` - The index register (X or Y) to use for the address calculation.
///
/// # Returns
///
/// * `ClockEndStatus::Continue` - After steps 0 through 2, indicating more cycles are needed.
/// * `ClockEndStatus::EndInstruction` - After step 3, indicating the instruction is complete.
fn absolute_indexed_write(
    step: u8,
    regs: &mut RegisterFile,
    memory: &mut impl MemorySpace,
    op: RegisterMemoryOperation,
    idx: IndexRegister,
) -> ClockEndStatus {
    match step {
        0 => {
            let addr_low = memory.read(regs.pc.to_u16());
            regs.addr.set_low_u8(addr_low);
            regs.pc.inc();
        }
        1 => {
            let addr_high = memory.read(regs.pc.to_u16());
            regs.addr.set_high_u8(addr_high);
            regs.pc.inc();
        }
        2 => {
            add_index_to_address(regs, idx);
            let _ = memory.read(regs.addr.to_u16());

            let index_value = get_index_value(regs, idx);
            fix_addr(regs, index_value);
        }
        3 => {
            execute_op(op, regs, memory);
            return ClockEndStatus::EndInstruction;
        }
        _ => unreachable!(),
    }

    ClockEndStatus::Continue
}

/// Executes a relative mode branch instruction operation.
///
/// Relative mode is used exclusively for branch instructions (BEQ, BNE, BCC, etc.) in the 6502.
/// These instructions conditionally alter program flow by adding a signed offset to the program counter.
/// The offset is provided as a single byte after the opcode, representing a signed 8-bit value
/// (-128 to +127 bytes from the instruction following the branch).
///
/// The process involves:
/// (Cycle 1: Fetch opcode - handled externally)
/// Cycle 2 (step 0): Read the branch offset from memory at PC, store it in the temporary register,
///                   and increment PC.
/// Cycle 3 (step 1): Evaluate the branch condition and, if taken, calculate the new PC value
///                   by adding the signed offset to PC. If the branch is not taken, skip to the next instruction.
/// Cycle 4 (step 2): Only executed if the branch was taken and crossed a page boundary.
///                   This extra cycle is needed for the CPU to fix the high byte of the PC.
///
/// # Arguments
///
/// * `step` - The current step in the execution process (0, 1, or 2).
/// * `regs` - A mutable reference to the CPU's register file.
/// * `memory` - A mutable reference to the memory space for reading the branch offset.
/// * `op` - The specific branch operation that determines the condition to test.
///
/// # Returns
///
/// * `ClockEndStatus::Continue` - After step 0, or after step 1 if branch taken and page crossed.
/// * `ClockEndStatus::EndInstruction` - After branch processing is complete.
fn relative(
    step: u8,
    regs: &mut RegisterFile,
    memory: &mut impl MemorySpace,
    op: BranchOperation,
) -> ClockEndStatus {
    match step {
        0 => {
            let operand = memory.read(regs.pc.to_u16());
            regs.tmp.set_u8(operand);
            regs.pc.inc();
        }
        1 => {
            return execute_branch_op(op, regs, memory);
        }
        2 => {
            return ClockEndStatus::EndInstruction;
        }

        _ => unreachable!(),
    }

    ClockEndStatus::Continue
}

/// Executes a zero-page indexed indirect (pre-indexed) instruction operation.
///
/// This addressing mode (sometimes called "(Indirect,X)") is used primarily for table lookups and
/// array access. It works in multiple steps:
/// 1. Take a zero-page address from the byte following the opcode
/// 2. Add the X index register to this address (with zero-page wrap-around)
/// 3. Use the resulting zero-page location and the one following it as a pointer to the effective address
/// 4. Execute the operation using this effective address
///
/// The process involves:
/// (Cycle 1: Fetch opcode - handled externally)
/// Cycle 2 (step 0): Read the zero-page base address from memory at PC, store it in the address register,
///                   set the high byte of the address to 0, and increment PC.
/// Cycle 3 (step 1): Perform dummy read from the unmodified zero-page address, then add index register to it.
/// Cycle 4 (step 2): Read the low byte of the effective address from the indexed zero-page location.
/// Cycle 5 (step 3): Read the high byte of the effective address from the next zero-page location,
///                   and combine with the low byte to form the complete target address.
/// Cycle 6 (step 4): Execute the operation on the memory location specified by the effective address.
///
/// # Arguments
///
/// * `step` - The current step in the execution process (0 through 4).
/// * `regs` - A mutable reference to the CPU's register file.
/// * `memory` - A mutable reference to the memory space for reading/writing.
/// * `op` - The specific register-memory operation to execute at the effective address.
/// * `idx` - The index register (typically X) to use for the address calculation.
///
/// # Returns
///
/// * `ClockEndStatus::Continue` - After steps 0 through 3, indicating more cycles are needed.
/// * `ClockEndStatus::EndInstruction` - After step 4, indicating the instruction is complete.
fn zero_page_indexed_indirect(
    step: u8,
    regs: &mut RegisterFile,
    memory: &mut impl MemorySpace,
    op: RegisterMemoryOperation,
    idx: IndexRegister,
) -> ClockEndStatus {
    match step {
        0 => {
            regs.addr.set_high_u8(0);

            let addr_low = memory.read(regs.pc.to_u16());
            regs.addr.set_low_u8(addr_low);
            regs.pc.inc();
        }
        1 => {
            let _ = memory.read(regs.pc.to_u16());

            add_index_to_address(regs, idx);
        }
        2 => {
            let data = memory.read(regs.addr.to_u16());
            regs.tmp.set_u8(data);
        }
        3 => {
            let addr_low = regs.addr.low_u8().wrapping_add(1);
            regs.addr.set_low_u8(addr_low);

            let addr_high = memory.read(regs.addr.to_u16());
            regs.addr.set_high_u8(addr_high);

            regs.addr.set_low_u8(regs.tmp.to_u8());
        }
        4 => {
            execute_op(op, regs, memory);
            return ClockEndStatus::EndInstruction;
        }
        _ => unreachable!(),
    }

    ClockEndStatus::Continue
}

/// Executes a zero-page indexed indirect (pre-indexed) read-modify-write instruction operation.
///
/// This addressing mode combines the zero-page indexed indirect addressing with read-modify-write operations.
/// It follows the same addressing steps as regular zero-page indexed indirect instructions, but then
/// performs the read-modify-write sequence on the memory at the calculated effective address.
///
/// The process involves:
/// (Cycle 1: Fetch opcode - handled externally)
/// Cycle 2 (step 0): Read the zero-page base address from memory at PC, store it in the address register,
///                   set the high byte of the address to 0, and increment PC.
/// Cycle 3 (step 1): Perform dummy read from the unmodified zero-page address, then add index register to it.
/// Cycle 4 (step 2): Read the low byte of the effective address from the indexed zero-page location.
/// Cycle 5 (step 3): Read the high byte of the effective address from the next zero-page location,
///                   and combine with the low byte to form the complete target address.
/// Cycle 6 (step 4): Read the operand from the effective address and store it in a temporary register.
/// Cycle 7 (step 5): Write the unchanged value back to memory (dummy write), then perform the modification operation.
/// Cycle 8 (step 6): Write the modified value back to memory.
///
/// # Arguments
///
/// * `step` - The current step in the execution process (0 through 6).
/// * `regs` - A mutable reference to the CPU's register file.
/// * `memory` - A mutable reference to the memory space for reading/writing.
/// * `op` - The specific memory modification operation to execute.
/// * `idx` - The index register (typically X) to use for the address calculation.
///
/// # Returns
///
/// * `ClockEndStatus::Continue` - After steps 0 through 5, indicating more cycles are needed.
/// * `ClockEndStatus::EndInstruction` - After step 6, indicating the instruction is complete.
fn zero_page_indexed_indirect_rmw(
    step: u8,
    regs: &mut RegisterFile,
    memory: &mut impl MemorySpace,
    op: MemoryModifyOperation,
    idx: IndexRegister,
) -> ClockEndStatus {
    match step {
        0 => {
            regs.addr.set_high_u8(0);

            let addr_low = memory.read(regs.pc.to_u16());
            regs.addr.set_low_u8(addr_low);
            regs.pc.inc();
        }
        1 => {
            let _ = memory.read(regs.pc.to_u16());

            add_index_to_address(regs, idx);
        }
        2 => {
            let data = memory.read(regs.addr.to_u16());
            regs.tmp.set_u8(data);
        }
        3 => {
            let addr_low = regs.addr.low_u8().wrapping_add(1);
            regs.addr.set_low_u8(addr_low);

            let addr_high = memory.read(regs.addr.to_u16());
            regs.addr.set_high_u8(addr_high);

            regs.addr.set_low_u8(regs.tmp.to_u8());
        }
        4 => {
            let operand = memory.read(regs.addr.to_u16());
            regs.tmp.set_u8(operand);
        }
        5 => {
            memory.write(regs.tmp.to_u8(), regs.addr.to_u16());
            execute_memory_modify_op(op, regs);
        }
        6 => {
            memory.write(regs.tmp.to_u8(), regs.addr.to_u16());
            return ClockEndStatus::EndInstruction;
        }
        _ => unreachable!(),
    }

    ClockEndStatus::Continue
}

/// Executes a zero-page indirect indexed (post-indexed) read instruction operation.
///
/// This addressing mode (sometimes called "(Indirect),Y") is used primarily for accessing arrays or tables
/// of data. It works in multiple steps:
/// 1. Take a zero-page address from the byte following the opcode
/// 2. Use this zero-page location and the one following it to form a 16-bit base address
/// 3. Add the Y index register to this base address to get the effective address
/// 4. Execute the operation using this effective address
///
/// The key difference from zero-page indexed indirect is that the indexing occurs after the indirection,
/// making it useful for working with arrays of data rather than arrays of pointers.
///
/// The process involves:
/// (Cycle 1: Fetch opcode - handled externally)
/// Cycle 2 (step 0): Read the zero-page pointer address from memory at PC, store it in the address register,
///                   set the high byte of the address to 0, and increment PC.
/// Cycle 3 (step 1): Read the low byte of the base address from the zero-page pointer location.
/// Cycle 4 (step 2): Read the high byte of the base address from the next zero-page location,
///                   combine with the low byte to form the complete base address, then add the index register value.
/// Cycle 5 (step 3): If a page boundary was crossed by adding the index:
///                   - Perform a dummy read from the incorrectly computed address
///                   - Fix the high byte of the address and continue to step 4
///                   If no page boundary was crossed:
///                   - Immediately execute the operation and end the instruction
/// Cycle 6 (step 4): Execute the operation on the memory location specified by the indexed address
///                   (only reached if page crossing occurred).
///
/// # Arguments
///
/// * `step` - The current step in the execution process (0 through 4).
/// * `regs` - A mutable reference to the CPU's register file.
/// * `memory` - A mutable reference to the memory space for reading.
/// * `op` - The specific register-memory operation to execute at the indexed address.
/// * `idx` - The index register (typically Y) to use for the address calculation.
///
/// # Returns
///
/// * `ClockEndStatus::Continue` - After steps 0, 1, 2, and possibly 3 (with page crossing), indicating more cycles are needed.
/// * `ClockEndStatus::EndInstruction` - After step 3 (without page crossing) or step 4, indicating the instruction is complete.
fn zero_page_indirect_indexed_read(
    step: u8,
    regs: &mut RegisterFile,
    memory: &mut impl MemorySpace,
    op: RegisterMemoryOperation,
    idx: IndexRegister,
) -> ClockEndStatus {
    match step {
        0 => {
            regs.addr.set_high_u8(0);
            let addr_low = memory.read(regs.pc.to_u16());
            regs.addr.set_low_u8(addr_low);
            regs.pc.inc();
        }
        1 => {
            let addr_low = memory.read(regs.addr.to_u16());
            regs.tmp.set_u8(addr_low);
        }
        2 => {
            let addr_low = regs.addr.low_u8().wrapping_add(1);
            regs.addr.set_low_u8(addr_low);

            let addr_high = memory.read(regs.addr.to_u16());
            regs.addr.set_high_u8(addr_high);
            regs.addr.set_low_u8(regs.tmp.to_u8());

            add_index_to_address(regs, idx);
        }
        3 => {
            return fix_addr_or_run_op_finish(op, regs, memory, idx);
        }
        4 => {
            execute_op(op, regs, memory);
            return ClockEndStatus::EndInstruction;
        }
        _ => unreachable!(),
    }

    ClockEndStatus::Continue
}

/// Executes a zero-page indirect indexed (post-indexed) read-modify-write instruction operation.
///
/// This addressing mode combines the zero-page indirect indexed addressing with read-modify-write operations.
/// It follows the same addressing steps as regular zero-page indirect indexed instructions, but then
/// performs the read-modify-write sequence on the memory at the calculated effective address.
///
/// The process involves:
/// (Cycle 1: Fetch opcode - handled externally)
/// Cycle 2 (step 0): Read the zero-page pointer address from memory at PC, store it in the address register,
///                   set the high byte of the address to 0, and increment PC.
/// Cycle 3 (step 1): Read the low byte of the base address from the zero-page pointer location.
/// Cycle 4 (step 2): Read the high byte of the base address from the next zero-page location,
///                   combine with the low byte to form the complete base address, then add the index register value.
/// Cycle 5 (step 3): Perform a dummy read from the incorrectly computed address (low byte + index, high byte unchanged).
///                   Fix the high byte of the address if page crossing occurred.
/// Cycle 6 (step 4): Read the operand from the correctly computed memory location and store it in a temporary register.
/// Cycle 7 (step 5): Write the unchanged value back to memory (dummy write), then perform the modification operation.
/// Cycle 8 (step 6): Write the modified value back to memory.
///
/// # Arguments
///
/// * `step` - The current step in the execution process (0 through 6).
/// * `regs` - A mutable reference to the CPU's register file.
/// * `memory` - A mutable reference to the memory space for reading/writing.
/// * `op` - The specific memory modification operation to execute.
/// * `idx` - The index register (typically Y) to use for the address calculation.
///
/// # Returns
///
/// * `ClockEndStatus::Continue` - After steps 0 through 5, indicating more cycles are needed.
/// * `ClockEndStatus::EndInstruction` - After step 6, indicating the instruction is complete.
fn zero_page_indirect_indexed_rmw(
    step: u8,
    regs: &mut RegisterFile,
    memory: &mut impl MemorySpace,
    op: MemoryModifyOperation,
    idx: IndexRegister,
) -> ClockEndStatus {
    match step {
        0 => {
            regs.addr.set_high_u8(0);
            let addr_low = memory.read(regs.pc.to_u16());
            regs.addr.set_low_u8(addr_low);
            regs.pc.inc();
        }
        1 => {
            let addr_low = memory.read(regs.addr.to_u16());
            regs.tmp.set_u8(addr_low);
        }
        2 => {
            let addr_low = regs.addr.low_u8().wrapping_add(1);
            regs.addr.set_low_u8(addr_low);

            let addr_high = memory.read(regs.addr.to_u16());
            regs.addr.set_high_u8(addr_high);
            regs.addr.set_low_u8(regs.tmp.to_u8());

            add_index_to_address(regs, idx);
        }
        3 => {
            let _ = memory.read(regs.addr.to_u16());
            let index_value = get_index_value(regs, idx);
            fix_addr(regs, index_value);
        }
        4 => {
            let operand = memory.read(regs.addr.to_u16());
            regs.tmp.set_u8(operand);
        }
        5 => {
            memory.write(regs.tmp.to_u8(), regs.addr.to_u16());
            execute_memory_modify_op(op, regs);
        }
        6 => {
            memory.write(regs.tmp.to_u8(), regs.addr.to_u16());
            return ClockEndStatus::EndInstruction;
        }
        _ => unreachable!(),
    }

    ClockEndStatus::Continue
}

/// Executes a zero-page indirect indexed (post-indexed) write instruction operation.
///
/// This addressing mode is similar to zero-page indirect indexed read, but used for store operations
/// (STA, STX, STY). Unlike read operations, write operations always take an extra cycle to fix the address
/// in case of page crossing, regardless of whether a page boundary was actually crossed.
///
/// The process involves:
/// (Cycle 1: Fetch opcode - handled externally)
/// Cycle 2 (step 0): Read the zero-page pointer address from memory at PC, store it in the address register,
///                   set the high byte of the address to 0, and increment PC.
/// Cycle 3 (step 1): Read the low byte of the base address from the zero-page pointer location.
/// Cycle 4 (step 2): Read the high byte of the base address from the next zero-page location,
///                   combine with the low byte to form the complete base address, then add the index register value.
/// Cycle 5 (step 3): Perform a dummy read from the incorrectly computed address (low byte + index, high byte unchanged).
///                   Fix the high byte of the address if page crossing occurred.
/// Cycle 6 (step 4): Execute the write operation to the correctly computed memory location.
///
/// # Arguments
///
/// * `step` - The current step in the execution process (0 through 4).
/// * `regs` - A mutable reference to the CPU's register file.
/// * `memory` - A mutable reference to the memory space for reading/writing.
/// * `op` - The specific register-memory write operation to execute.
/// * `idx` - The index register (typically Y) to use for the address calculation.
///
/// # Returns
///
/// * `ClockEndStatus::Continue` - After steps 0 through 3, indicating more cycles are needed.
/// * `ClockEndStatus::EndInstruction` - After step 4, indicating the instruction is complete.
fn zero_page_indirect_indexed_write(
    step: u8,
    regs: &mut RegisterFile,
    memory: &mut impl MemorySpace,
    op: RegisterMemoryOperation,
    idx: IndexRegister,
) -> ClockEndStatus {
    match step {
        0 => {
            regs.addr.set_high_u8(0);
            let addr_low = memory.read(regs.pc.to_u16());
            regs.addr.set_low_u8(addr_low);
            regs.pc.inc();
        }
        1 => {
            let addr_low = memory.read(regs.addr.to_u16());
            regs.tmp.set_u8(addr_low);
        }
        2 => {
            let addr_low = regs.addr.low_u8().wrapping_add(1);
            regs.addr.set_low_u8(addr_low);

            let addr_high = memory.read(regs.addr.to_u16());
            regs.addr.set_high_u8(addr_high);
            regs.addr.set_low_u8(regs.tmp.to_u8());

            add_index_to_address(regs, idx);
        }
        3 => {
            let _ = memory.read(regs.addr.to_u16());
            let index_value = get_index_value(regs, idx);
            fix_addr(regs, index_value);
        }
        4 => {
            execute_op(op, regs, memory);
            return ClockEndStatus::EndInstruction;
        }
        _ => unreachable!(),
    }

    ClockEndStatus::Continue
}

/// Executes an absolute indirect jump instruction (JMP indirect).
///
/// This addressing mode is unique to the JMP instruction. It allows the program to jump
/// to a location stored at a given memory address. This is equivalent to the concept of
/// function pointers or jump tables in higher level languages.
///
/// The process involves:
/// (Cycle 1: Fetch opcode - handled externally)
/// Cycle 2 (step 0): Read the low byte of the pointer address from memory at PC, store it temporarily,
///                   and increment PC.
/// Cycle 3 (step 1): Read the high byte of the pointer address from memory at PC, store it in the address register,
///                   combine with the low byte to form the complete pointer address, and increment PC.
/// Cycle 4 (step 2): Read the low byte of the target address from the pointer address in memory,
///                   and store it temporarily.
/// Cycle 5 (step 3): Read the high byte of the target address from the pointer address + 1 in memory,
///                   combine with the low byte to form the complete target address,
///                   then set PC to this address.
///
/// Note: The 6502 has a hardware bug where if the pointer address crosses a page boundary
/// (e.g., $xxFF where xx is any byte), the high byte is fetched from the same page rather than
/// the next page.
///
/// # Arguments
///
/// * `step` - The current step in the execution process (0 through 3).
/// * `regs` - A mutable reference to the CPU's register file.
/// * `memory` - A mutable reference to the memory space for reading.
///
/// # Returns
///
/// * `ClockEndStatus::Continue` - After steps 0 through 2, indicating more cycles are needed.
/// * `ClockEndStatus::EndInstruction` - After step 3, indicating the instruction is complete.
fn absolute_indirect_jump(
    step: u8,
    regs: &mut RegisterFile,
    memory: &mut impl MemorySpace,
) -> ClockEndStatus {
    match step {
        0 => {
            let addr_low = memory.read(regs.pc.to_u16());
            regs.tmp.set_u8(addr_low);
            regs.pc.inc();
        }
        1 => {
            let addr_high = memory.read(regs.pc.to_u16());
            regs.addr.set_high_u8(addr_high);
            regs.pc.inc();
            regs.addr.set_low_u8(regs.tmp.to_u8());
        }
        2 => {
            let pc_low = memory.read(regs.addr.to_u16());
            regs.tmp.set_u8(pc_low);
        }
        3 => {
            let addr_low = regs.addr.low_u8().wrapping_add(1);
            regs.addr.set_low_u8(addr_low);
            let pc_high = memory.read(regs.addr.to_u16());
            regs.pc.set_high_u8(pc_high);
            regs.pc.set_low_u8(regs.tmp.to_u8());
            return ClockEndStatus::EndInstruction;
        }
        _ => unreachable!(),
    }

    ClockEndStatus::Continue
}

/// Executes a single step of a 6502 CPU instruction.
///
/// This is the core instruction execution function that processes a single cycle of a 6502
/// instruction. The 6502 uses a multi-cycle execution model where each instruction requires
/// several clock cycles to complete. Each cycle performs a specific part of the instruction,
/// such as memory reads, writes, or internal operations.
///
/// The function dispatches to the appropriate handler based on the instruction type and maintains
/// the cycle-accurate timing of the 6502 processor.
///
/// # Arguments
///
/// * `instr` - The instruction to execute
/// * `step` - The current step (cycle) within the instruction's execution sequence
/// * `regs` - A mutable reference to the CPU's register file
/// * `memory` - A mutable reference to the memory space for reading/writing
///
/// # Returns
///
/// * `ClockEndStatus::Continue` - If more cycles are needed to complete the instruction
/// * `ClockEndStatus::EndInstruction` - If the instruction has completed and the CPU should fetch the next instruction
/// * `ClockEndStatus::EndInstructionNextFetched` - If the instruction has completed and the next opcode has already been fetched
pub fn execute(
    instr: Instruction,
    step: u8,
    regs: &mut RegisterFile,
    memory: &mut impl MemorySpace,
) -> ClockEndStatus {
    #[cfg(feature = "logging")]
    {
        trace!("\t{:?}", instr);
    }

    match instr {
        Instruction::FetchInstr => fetch_instr(step, regs, memory),
        Instruction::Break => break_instr(step, regs, memory),
        Instruction::StartIrq => start_irq(step, regs, memory),
        Instruction::StartNmi => start_nmi(step, regs, memory),
        Instruction::Reset => reset(step, regs, memory),
        Instruction::ReturnInterrupt => return_interrupt(step, regs, memory),
        Instruction::JumpSubroutine => jump_subroutine(step, regs, memory),
        Instruction::ReturnSubroutine => return_subroutine(step, regs, memory),
        Instruction::Push(op) => push(step, regs, memory, op),
        Instruction::Pull(op) => pull(step, regs, memory, op),
        Instruction::Implied(op) => implied(step, regs, memory, op),
        Instruction::Immediate(op) => immediate(step, regs, memory, op),
        Instruction::AbsoluteJump => absolute_jump(step, regs, memory),
        Instruction::Absolute(op) => absolute(step, regs, memory, op),
        Instruction::AbsoluteReadModifyWrite(op) => absolute_rmw(step, regs, memory, op),
        Instruction::ZeroPage(op) => zero_page(step, regs, memory, op),
        Instruction::ZeroPageReadModifyWrite(op) => zero_page_rmw(step, regs, memory, op),
        Instruction::ZeroPageIdx(op, idx) => zero_page_indexed(step, regs, memory, op, idx),
        Instruction::ZeroPageIdxReadModifyWrite(op, idx) => {
            zero_page_indexed_rmw(step, regs, memory, op, idx)
        }
        Instruction::AbsoluteIdxRead(op, idx) => absolute_indexed_read(step, regs, memory, op, idx),
        Instruction::AbsoluteIdxReadModifyWrite(op, idx) => {
            absolute_indexed_rmw(step, regs, memory, op, idx)
        }
        Instruction::AbsoluteIdxWrite(op, idx) => {
            absolute_indexed_write(step, regs, memory, op, idx)
        }
        Instruction::Relative(op) => relative(step, regs, memory, op),
        Instruction::ZeroPageIdxIndirect(op, idx) => {
            zero_page_indexed_indirect(step, regs, memory, op, idx)
        }
        Instruction::ZeroPageIdxIndirectReadModifyWrite(op, idx) => {
            zero_page_indexed_indirect_rmw(step, regs, memory, op, idx)
        }
        Instruction::ZeroPageIndirectIdxRead(op, idx) => {
            zero_page_indirect_indexed_read(step, regs, memory, op, idx)
        }
        Instruction::ZeroPageIndirectIdxReadModifyWrite(op, idx) => {
            zero_page_indirect_indexed_rmw(step, regs, memory, op, idx)
        }
        Instruction::ZeroPageIndirectIdxWrite(op, idx) => {
            zero_page_indirect_indexed_write(step, regs, memory, op, idx)
        }
        Instruction::AbsoluteIndirectJump => absolute_indirect_jump(step, regs, memory),
    }
}
