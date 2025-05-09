#[cfg(test)]
mod tests;

use super::{
    BranchOperation, ImplicitOperation, Instruction, MemoryModifyOperation, PullStackOperation,
    PushStackOperation, RegisterMemoryOperation,
};
use crate::cpu::interrupt::{InterruptVector, InterruptVectorAddrBytePos};
use crate::registers::{IndexRegister, RegisterFile, SelectedRegister8, StatusRegFlags};
use crate::{alu, MemorySpace};
#[cfg(feature = "logging")]
use log::trace;

#[derive(Debug, PartialEq, Eq)]
pub enum ClockEndStatus {
    Continue,
    EndInstruction,
    EndInstructionNextFetched { opcode_addr: u16 },
}

#[derive(Debug, PartialEq, Eq)]
pub enum FixAddressResult {
    Fixed,
    Untouched,
}

fn get_index_value(regs: &mut RegisterFile, idx: IndexRegister) -> u8 {
    let index_reg = regs.copy_selected_register8(idx.into());
    index_reg.to_u8()
}

fn set_stack_address(regs: &mut RegisterFile) {
    let addr_high = SelectedRegister8::StackPage as u8;
    regs.addr.set_high_u8(addr_high);

    let addr_low = regs.sp.to_u8();
    regs.addr.set_low_u8(addr_low);
}

fn add_index_to_address(regs: &mut RegisterFile, idx: IndexRegister) {
    let index_value = get_index_value(regs, idx);

    let addr_low = regs.addr.low_u8();
    let addr_low = addr_low.wrapping_add(index_value);
    regs.addr.set_low_u8(addr_low);
}

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
            regs.status
                .update_flags(StatusRegFlags::OVERFLOW, (regs.tmp.to_u8() & 0x40) != 0);
            regs.status
                .update_flags(StatusRegFlags::NEGATIVE, (regs.tmp.to_u8() & 0x80) != 0);
        }
        RegisterMemoryOperation::BitImmediate => {
            alu::bit_compare(regs.a, regs.tmp, &mut regs.status);
        }
    }
}

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

fn read_pc(regs: &mut RegisterFile, memory: &mut impl MemorySpace) -> u8 {
    memory.read(regs.pc.to_u16())
}

fn read_pc_inc(regs: &mut RegisterFile, memory: &mut impl MemorySpace) -> u8 {
    let pc = memory.read(regs.pc.to_u16());
    regs.pc.inc();
    pc
}

fn write_stack(regs: &mut RegisterFile, memory: &mut impl MemorySpace, value: u8) {
    set_stack_address(regs);
    memory.write(value, regs.addr.to_u16());
    regs.sp.dec();
}

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

fn implied(
    _step: u8,
    regs: &mut RegisterFile,
    _memory: &mut impl MemorySpace,
    op: ImplicitOperation,
) -> ClockEndStatus {
    execute_implicit_op(op, regs);

    ClockEndStatus::EndInstruction
}

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
            //let _ = memory.read(regs.pc.to_u16());
            return execute_branch_op(op, regs, memory);
        }
        2 => {
            return ClockEndStatus::EndInstruction;
        }

        _ => unreachable!(),
    }

    ClockEndStatus::Continue
}

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

#[allow(dead_code)]
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
