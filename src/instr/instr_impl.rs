#[cfg(test)]
mod tests;

use super::InstructionOp;
use super::InstructionSequenceMode;
use crate::registers::{
    IndexRegister, RegisterFile, SelectedRegister16, SelectedRegister8, StatusRegFlags,
};
use crate::{alu, MemorySpace};
#[cfg(feature = "logging")]
use log::trace;

#[derive(Debug, PartialEq, Eq)]
pub enum ClockEndStatus {
    Continue,
    EndInstruction,
    EndInstructionNextFetched,
}

#[derive(Debug, PartialEq, Eq)]
pub enum FixAddressResult {
    Fixed,
    Untouched,
}

fn get_index_value(idx: Option<IndexRegister>, regs: &mut RegisterFile) -> u8 {
    debug_assert!(idx.is_some());
    let idx = unsafe { idx.unwrap_unchecked() };

    let index_reg = regs.copy_selected_register8(idx.into());
    index_reg.to_u8()
}

fn set_stack_address(regs: &mut RegisterFile) {
    let addr_high = SelectedRegister8::StackPage as u8;
    regs.addr.set_high_u8(addr_high);

    let addr_low = regs.sp.to_u8();
    regs.addr.set_low_u8(addr_low);
}

fn add_index_to_address(regs: &mut RegisterFile, idx: Option<IndexRegister>) {
    let index_value = get_index_value(idx, regs);

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
    op: Option<InstructionOp>,
    regs: &mut RegisterFile,
    memory: &mut impl MemorySpace,
    idx: Option<IndexRegister>,
) -> ClockEndStatus {
    let index_value = get_index_value(idx, regs);

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
            return ClockEndStatus::Continue;
        } else {
            return ClockEndStatus::EndInstruction;
        }
    }

    regs.pc.inc();
    regs.ir.set_u8(next_opcode);
    ClockEndStatus::EndInstructionNextFetched
}

fn execute_branch_op(
    op: Option<InstructionOp>,
    regs: &mut RegisterFile,
    memory: &mut impl MemorySpace,
) -> ClockEndStatus {
    match op {
        Some(InstructionOp::BranchPlus) => branch_if(!regs.status.negative(), regs, memory),
        Some(InstructionOp::BranchMinus) => branch_if(regs.status.negative(), regs, memory),
        Some(InstructionOp::BranchEqual) => branch_if(regs.status.zero(), regs, memory),
        Some(InstructionOp::BranchNotEqual) => branch_if(!regs.status.zero(), regs, memory),
        Some(InstructionOp::BranchCarryClear) => branch_if(!regs.status.carry(), regs, memory),
        Some(InstructionOp::BranchCarrySet) => branch_if(regs.status.carry(), regs, memory),
        Some(InstructionOp::BranchOverflowClear) => {
            branch_if(!regs.status.overflow(), regs, memory)
        }
        Some(InstructionOp::BranchOverflowSet) => branch_if(regs.status.overflow(), regs, memory),
        _ => unreachable!(),
    }
}

fn execute_implicit_op(op: Option<InstructionOp>, regs: &mut RegisterFile) {
    match op {
        Some(InstructionOp::Nop) => (),
        Some(InstructionOp::ShiftLeftA) => alu::shift_left(&mut regs.a, &mut regs.status),
        Some(InstructionOp::ShiftRightA) => alu::shift_right(&mut regs.a, &mut regs.status),
        Some(InstructionOp::RotateLeftA) => alu::rotate_left(&mut regs.a, &mut regs.status),
        Some(InstructionOp::RotateRightA) => alu::rotate_right(&mut regs.a, &mut regs.status),
        Some(InstructionOp::IncrementX) => alu::inc(&mut regs.x, &mut regs.status),
        Some(InstructionOp::IncrementY) => alu::inc(&mut regs.y, &mut regs.status),
        Some(InstructionOp::DecrementX) => alu::dec(&mut regs.x, &mut regs.status),
        Some(InstructionOp::DecrementY) => alu::dec(&mut regs.y, &mut regs.status),
        Some(InstructionOp::ClearCarry) => regs.status.clear_flags(StatusRegFlags::CARRY),
        Some(InstructionOp::SetCarry) => regs.status.set_flags(StatusRegFlags::CARRY),
        Some(InstructionOp::ClearDecimal) => regs.status.clear_flags(StatusRegFlags::DECIMAL),
        Some(InstructionOp::SetDecimal) => regs.status.set_flags(StatusRegFlags::DECIMAL),
        Some(InstructionOp::ClearInterruptDisable) => {
            regs.status.clear_flags(StatusRegFlags::IRQ_DISABLE)
        }
        Some(InstructionOp::SetInterruptDisable) => {
            regs.status.set_flags(StatusRegFlags::IRQ_DISABLE)
        }
        Some(InstructionOp::ClearOverflow) => regs.status.clear_flags(StatusRegFlags::OVERFLOW),
        Some(InstructionOp::SetOverflow) => regs.status.set_flags(StatusRegFlags::OVERFLOW),
        Some(InstructionOp::TransferAccumulatorToX) => {
            regs.x = regs.a;
            alu::update_status_nz(regs.x.to_i8(), &mut regs.status)
        }
        Some(InstructionOp::TransferAccumulatorToY) => {
            regs.y = regs.a;
            alu::update_status_nz(regs.y.to_i8(), &mut regs.status);
        }
        Some(InstructionOp::TransferXToAccumulator) => {
            regs.a = regs.x;
            alu::update_status_nz(regs.a.to_i8(), &mut regs.status);
        }
        Some(InstructionOp::TransferYToAccumulator) => {
            regs.a = regs.y;
            alu::update_status_nz(regs.a.to_i8(), &mut regs.status);
        }
        Some(InstructionOp::TransferStackPtrToX) => {
            regs.x = regs.sp;
            alu::update_status_nz(regs.sp.to_i8(), &mut regs.status);
        }
        Some(InstructionOp::TransferXToStackPtr) => {
            regs.sp = regs.x;
        }
        _ => unreachable!(),
    }
}

fn execute_op(op: Option<InstructionOp>, regs: &mut RegisterFile, memory: &mut impl MemorySpace) {
    match op {
        Some(InstructionOp::Or) => {
            regs.tmp.set_u8(memory.read(regs.addr.to_u16()));
            alu::or(&mut regs.a, &regs.tmp, &mut regs.status)
        }
        Some(InstructionOp::And) => {
            regs.tmp.set_u8(memory.read(regs.addr.to_u16()));
            alu::and(&mut regs.a, &regs.tmp, &mut regs.status)
        }
        Some(InstructionOp::Xor) => {
            regs.tmp.set_u8(memory.read(regs.addr.to_u16()));
            alu::xor(&mut regs.a, &regs.tmp, &mut regs.status)
        }
        Some(InstructionOp::Add) => {
            regs.tmp.set_u8(memory.read(regs.addr.to_u16()));
            alu::add(&mut regs.a, &regs.tmp, &mut regs.status)
        }
        Some(InstructionOp::Sub) => {
            regs.tmp.set_u8(memory.read(regs.addr.to_u16()));
            alu::sub(&mut regs.a, &regs.tmp, &mut regs.status)
        }
        Some(InstructionOp::Cmp) => {
            regs.tmp.set_u8(memory.read(regs.addr.to_u16()));
            alu::cmp(&mut regs.a, &regs.tmp, &mut regs.status)
        }
        Some(InstructionOp::Cpx) => {
            regs.tmp.set_u8(memory.read(regs.addr.to_u16()));
            alu::cmp(&mut regs.x, &regs.tmp, &mut regs.status)
        }
        Some(InstructionOp::Cpy) => {
            regs.tmp.set_u8(memory.read(regs.addr.to_u16()));
            alu::cmp(&mut regs.y, &regs.tmp, &mut regs.status)
        }
        Some(InstructionOp::LoadA) => {
            regs.a.set_u8(memory.read(regs.addr.to_u16()));
            alu::update_status_nz(regs.a.to_i8(), &mut regs.status);
        }
        Some(InstructionOp::LoadX) => {
            regs.x.set_u8(memory.read(regs.addr.to_u16()));
            alu::update_status_nz(regs.x.to_i8(), &mut regs.status);
        }
        Some(InstructionOp::LoadY) => {
            regs.y.set_u8(memory.read(regs.addr.to_u16()));
            alu::update_status_nz(regs.y.to_i8(), &mut regs.status);
        }
        Some(InstructionOp::StoreA) => {
            memory.write(regs.a.to_u8(), regs.addr.to_u16());
        }
        Some(InstructionOp::StoreX) => {
            memory.write(regs.x.to_u8(), regs.addr.to_u16());
        }
        Some(InstructionOp::StoreY) => {
            memory.write(regs.y.to_u8(), regs.addr.to_u16());
        }
        Some(InstructionOp::Bit) => {
            regs.tmp.set_u8(memory.read(regs.addr.to_u16()));
            alu::bit_compare(regs.a, regs.tmp, &mut regs.status);
            regs.status
                .update_flags(StatusRegFlags::OVERFLOW, (regs.tmp.to_u8() & 0x40) != 0);
            regs.status
                .update_flags(StatusRegFlags::NEGATIVE, (regs.tmp.to_u8() & 0x80) != 0);
        }
        Some(InstructionOp::BitImmediate) => {
            alu::bit_compare(regs.a, regs.tmp, &mut regs.status);
        }
        _ => unreachable!(),
    }
}

fn execute_memory_modify_op(op: Option<InstructionOp>, regs: &mut RegisterFile) {
    match op {
        Some(InstructionOp::IncrementMemory) => alu::inc(&mut regs.tmp, &mut regs.status),
        Some(InstructionOp::DecrementMemory) => alu::dec(&mut regs.tmp, &mut regs.status),
        Some(InstructionOp::ShiftLeftMemory) => alu::shift_left(&mut regs.tmp, &mut regs.status),
        Some(InstructionOp::ShiftRightMemory) => alu::shift_right(&mut regs.tmp, &mut regs.status),
        Some(InstructionOp::RotateLeftMemory) => alu::rotate_left(&mut regs.tmp, &mut regs.status),
        Some(InstructionOp::RotateRightMemory) => {
            alu::rotate_right(&mut regs.tmp, &mut regs.status)
        }
        _ => unreachable!(),
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
    vector: SelectedRegister16,
) {
    regs.addr.set_u16(vector as u16);
    let value = memory.read(regs.addr.to_u16());

    match vector {
        SelectedRegister16::InterruptAddrLow
        | SelectedRegister16::ProgramStartAddrLow
        | SelectedRegister16::NMInterruptAddrLow => regs.pc.set_low_u8(value),
        SelectedRegister16::InterruptAddrHigh
        | SelectedRegister16::ProgramStartAddrHigh
        | SelectedRegister16::NMInterruptAddHigh => regs.pc.set_high_u8(value),
        _ => unreachable!(),
    }
}

fn fetch_instr(
    op: Option<InstructionOp>,
    step: u8,
    regs: &mut RegisterFile,
    memory: &mut impl MemorySpace,
) -> ClockEndStatus {
    debug_assert!(op.is_none());

    match step {
        0 => {
            let instruction = read_pc(regs, memory);
            regs.ir.set_u8(instruction);
            ClockEndStatus::EndInstructionNextFetched
        }
        _ => unreachable!(),
    }
}

fn break_instr(
    op: Option<InstructionOp>,
    step: u8,
    regs: &mut RegisterFile,
    memory: &mut impl MemorySpace,
) -> ClockEndStatus {
    debug_assert!(op.is_none());

    match step {
        0 => {
            let _ = read_pc_inc(regs, memory);
        }
        1 => write_stack(regs, memory, regs.pc.high_u8()),
        2 => write_stack(regs, memory, regs.pc.low_u8()),
        3 => {
            regs.status
                .set_flags(StatusRegFlags::IRQ_DISABLE | StatusRegFlags::BREAK);
            let status = regs.status.to_u8();
            write_stack(regs, memory, status);
        }
        4 => read_interrupt_vector(regs, memory, SelectedRegister16::InterruptAddrLow),
        5 => {
            read_interrupt_vector(regs, memory, SelectedRegister16::InterruptAddrHigh);
            return ClockEndStatus::EndInstruction;
        }
        _ => unreachable!(),
    }

    ClockEndStatus::Continue
}

fn start_irq(
    op: Option<InstructionOp>,
    step: u8,
    regs: &mut RegisterFile,
    memory: &mut impl MemorySpace,
) -> ClockEndStatus {
    debug_assert!(op.is_none());

    match step {
        0 => {
            let _ = read_pc(regs, memory);
        }
        1 => write_stack(regs, memory, regs.pc.high_u8()),
        2 => write_stack(regs, memory, regs.pc.low_u8()),
        3 => {
            regs.status.set_flags(StatusRegFlags::IRQ_DISABLE);
            write_stack(regs, memory, regs.status.to_u8());
        }
        4 => read_interrupt_vector(regs, memory, SelectedRegister16::InterruptAddrLow),
        5 => {
            read_interrupt_vector(regs, memory, SelectedRegister16::InterruptAddrHigh);
            return ClockEndStatus::EndInstruction;
        }
        _ => unreachable!(),
    }

    ClockEndStatus::Continue
}

fn start_nmi(
    op: Option<InstructionOp>,
    step: u8,
    regs: &mut RegisterFile,
    memory: &mut impl MemorySpace,
) -> ClockEndStatus {
    debug_assert!(op.is_none());

    match step {
        0 => {
            let _ = read_pc(regs, memory);
        }
        1 => write_stack(regs, memory, regs.pc.high_u8()),
        2 => write_stack(regs, memory, regs.pc.low_u8()),
        3 => write_stack(regs, memory, regs.status.to_u8()),
        4 => read_interrupt_vector(regs, memory, SelectedRegister16::NMInterruptAddrLow),
        5 => {
            read_interrupt_vector(regs, memory, SelectedRegister16::NMInterruptAddHigh);
            return ClockEndStatus::EndInstruction;
        }
        _ => unreachable!(),
    }

    ClockEndStatus::Continue
}

fn push(
    op: Option<InstructionOp>,
    step: u8,
    regs: &mut RegisterFile,
    memory: &mut impl MemorySpace,
) -> ClockEndStatus {
    match step {
        0 => {
            let _ = memory.read(regs.pc.to_u16());
        }
        1 => {
            set_stack_address(regs);

            match op {
                Some(InstructionOp::PushA) => {
                    let data = regs.a.to_u8();
                    memory.write(data, regs.addr.to_u16());
                }
                Some(InstructionOp::PushStatus) => {
                    let mut status = regs.status.to_u8();
                    status |= (StatusRegFlags::BREAK | StatusRegFlags::UNUSED).bits();
                    memory.write(status, regs.addr.to_u16());
                }
                _ => unreachable!(),
            }

            regs.sp.dec();

            return ClockEndStatus::EndInstruction;
        }
        _ => unreachable!(),
    }

    ClockEndStatus::Continue
}

fn pull(
    op: Option<InstructionOp>,
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

            match op {
                Some(InstructionOp::PullA) => {
                    regs.a.set_u8(data);
                    alu::update_status_nz(data as i8, &mut regs.status);
                }
                Some(InstructionOp::PullStatus) => {
                    let ignored_bits = (StatusRegFlags::BREAK | StatusRegFlags::UNUSED).bits();
                    let status = regs.status.to_u8();
                    let new_status = (status & ignored_bits) | (data & !ignored_bits);
                    regs.status.set_u8(new_status);
                }
                _ => unreachable!(),
            }

            return ClockEndStatus::EndInstruction;
        }
        _ => unreachable!(),
    }

    ClockEndStatus::Continue
}

fn reset(
    op: Option<InstructionOp>,
    step: u8,
    regs: &mut RegisterFile,
    memory: &mut impl MemorySpace,
) -> ClockEndStatus {
    debug_assert!(op.is_none());

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
            read_interrupt_vector(regs, memory, SelectedRegister16::ProgramStartAddrLow);
        }
        5 => {
            read_interrupt_vector(regs, memory, SelectedRegister16::ProgramStartAddrHigh);
            return ClockEndStatus::EndInstruction;
        }
        _ => unreachable!(),
    }

    ClockEndStatus::Continue
}

fn return_interrupt(
    op: Option<InstructionOp>,
    step: u8,
    regs: &mut RegisterFile,
    memory: &mut impl MemorySpace,
) -> ClockEndStatus {
    debug_assert!(op.is_none());

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
    op: Option<InstructionOp>,
    step: u8,
    regs: &mut RegisterFile,
    memory: &mut impl MemorySpace,
) -> ClockEndStatus {
    debug_assert!(op.is_none());

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
        3 => write_stack(regs, memory, regs.pc.low_u8() + 1), // Increment PC to point to the next instruction
        4 => {
            let data = read_pc(regs, memory);
            regs.pc.set_high_u8(data);
            regs.pc.inc();
            regs.pc.set_low_u8(regs.tmp.to_u8());
            return ClockEndStatus::EndInstruction;
        }
        _ => unreachable!(),
    }

    ClockEndStatus::Continue
}

fn return_subroutine(
    op: Option<InstructionOp>,
    step: u8,
    regs: &mut RegisterFile,
    memory: &mut impl MemorySpace,
) -> ClockEndStatus {
    debug_assert!(op.is_none());

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
    op: Option<InstructionOp>,
    _step: u8,
    regs: &mut RegisterFile,
    _memory: &mut impl MemorySpace,
) -> ClockEndStatus {
    execute_implicit_op(op, regs);

    ClockEndStatus::EndInstruction
}

fn immediate(
    op: Option<InstructionOp>,
    _step: u8,
    regs: &mut RegisterFile,
    memory: &mut impl MemorySpace,
) -> ClockEndStatus {
    regs.addr = regs.pc;
    regs.pc.inc();

    execute_op(op, regs, memory);

    ClockEndStatus::EndInstruction
}

fn absolute_jump(
    op: Option<InstructionOp>,
    step: u8,
    regs: &mut RegisterFile,
    memory: &mut impl MemorySpace,
) -> ClockEndStatus {
    debug_assert!(op.is_none());

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
    op: Option<InstructionOp>,
    step: u8,
    regs: &mut RegisterFile,
    memory: &mut impl MemorySpace,
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
    op: Option<InstructionOp>,
    step: u8,
    regs: &mut RegisterFile,
    memory: &mut impl MemorySpace,
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
    op: Option<InstructionOp>,
    step: u8,
    regs: &mut RegisterFile,
    memory: &mut impl MemorySpace,
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
    op: Option<InstructionOp>,
    step: u8,
    regs: &mut RegisterFile,
    memory: &mut impl MemorySpace,
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
    op: Option<InstructionOp>,
    step: u8,
    regs: &mut RegisterFile,
    memory: &mut impl MemorySpace,
    idx: Option<IndexRegister>,
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
    op: Option<InstructionOp>,
    step: u8,
    regs: &mut RegisterFile,
    memory: &mut impl MemorySpace,
    idx: Option<IndexRegister>,
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
    op: Option<InstructionOp>,
    step: u8,
    regs: &mut RegisterFile,
    memory: &mut impl MemorySpace,
    idx: Option<IndexRegister>,
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
    op: Option<InstructionOp>,
    step: u8,
    regs: &mut RegisterFile,
    memory: &mut impl MemorySpace,
    idx: Option<IndexRegister>,
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

            let index_value = get_index_value(idx, regs);
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
    op: Option<InstructionOp>,
    step: u8,
    regs: &mut RegisterFile,
    memory: &mut impl MemorySpace,
    idx: Option<IndexRegister>,
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

            let index_value = get_index_value(idx, regs);
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
    op: Option<InstructionOp>,
    step: u8,
    regs: &mut RegisterFile,
    memory: &mut impl MemorySpace,
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
    op: Option<InstructionOp>,
    step: u8,
    regs: &mut RegisterFile,
    memory: &mut impl MemorySpace,
    idx: Option<IndexRegister>,
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
    op: Option<InstructionOp>,
    step: u8,
    regs: &mut RegisterFile,
    memory: &mut impl MemorySpace,
    idx: Option<IndexRegister>,
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
    op: Option<InstructionOp>,
    step: u8,
    regs: &mut RegisterFile,
    memory: &mut impl MemorySpace,
    idx: Option<IndexRegister>,
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
    op: Option<InstructionOp>,
    step: u8,
    regs: &mut RegisterFile,
    memory: &mut impl MemorySpace,
    idx: Option<IndexRegister>,
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
            let index_value = get_index_value(idx, regs);
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
    op: Option<InstructionOp>,
    step: u8,
    regs: &mut RegisterFile,
    memory: &mut impl MemorySpace,
    idx: Option<IndexRegister>,
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
            let index_value = get_index_value(idx, regs);
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
    op: Option<InstructionOp>,
    step: u8,
    regs: &mut RegisterFile,
    memory: &mut impl MemorySpace,
) -> ClockEndStatus {
    debug_assert!(op.is_none());

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
    op: Option<InstructionOp>,
    step: u8,
    regs: &mut RegisterFile,
    memory: &mut impl MemorySpace,
    idx: Option<IndexRegister>,
    mode: InstructionSequenceMode,
) -> ClockEndStatus {
    #[cfg(feature = "logging")]
    {
        trace!("\t{:?}", mode);
    }

    match mode {
        InstructionSequenceMode::FetchInstr => fetch_instr(op, step, regs, memory),
        InstructionSequenceMode::Break => break_instr(op, step, regs, memory),
        InstructionSequenceMode::StartIrq => start_irq(op, step, regs, memory),
        InstructionSequenceMode::StartNmi => start_nmi(op, step, regs, memory),
        InstructionSequenceMode::Reset => reset(op, step, regs, memory),
        InstructionSequenceMode::ReturnInterrupt => return_interrupt(op, step, regs, memory),
        InstructionSequenceMode::JumpSubroutine => jump_subroutine(op, step, regs, memory),
        InstructionSequenceMode::ReturnSubroutine => return_subroutine(op, step, regs, memory),
        InstructionSequenceMode::Push => push(op, step, regs, memory),
        InstructionSequenceMode::Pull => pull(op, step, regs, memory),
        InstructionSequenceMode::Implied => implied(op, step, regs, memory),
        InstructionSequenceMode::Immediate => immediate(op, step, regs, memory),
        InstructionSequenceMode::AbsoluteJump => absolute_jump(op, step, regs, memory),
        InstructionSequenceMode::Absolute => absolute(op, step, regs, memory),
        InstructionSequenceMode::AbsoluteReadModifyWrite => absolute_rmw(op, step, regs, memory),
        InstructionSequenceMode::ZeroPage => zero_page(op, step, regs, memory),
        InstructionSequenceMode::ZeroPageReadModifyWrite => zero_page_rmw(op, step, regs, memory),
        InstructionSequenceMode::ZeroPageIdx => zero_page_indexed(op, step, regs, memory, idx),
        InstructionSequenceMode::ZeroPageIdxReadModifyWrite => {
            zero_page_indexed_rmw(op, step, regs, memory, idx)
        }
        InstructionSequenceMode::AbsoluteIdxRead => {
            absolute_indexed_read(op, step, regs, memory, idx)
        }
        InstructionSequenceMode::AbsoluteIdxReadModifyWrite => {
            absolute_indexed_rmw(op, step, regs, memory, idx)
        }
        InstructionSequenceMode::AbsoluteIdxWrite => {
            absolute_indexed_write(op, step, regs, memory, idx)
        }
        InstructionSequenceMode::Relative => relative(op, step, regs, memory),
        InstructionSequenceMode::ZeroPageIdxIndirect => {
            zero_page_indexed_indirect(op, step, regs, memory, idx)
        }
        InstructionSequenceMode::ZeroPageIdxIndirectReadModifyWrite => {
            zero_page_indexed_indirect_rmw(op, step, regs, memory, idx)
        }
        InstructionSequenceMode::ZeroPageIndirectIdxRead => {
            zero_page_indirect_indexed_read(op, step, regs, memory, idx)
        }
        InstructionSequenceMode::ZeroPageIndirectIdxReadModifyWrite => {
            zero_page_indirect_indexed_rmw(op, step, regs, memory, idx)
        }
        InstructionSequenceMode::ZeroPageIndirectIdxWrite => {
            zero_page_indirect_indexed_write(op, step, regs, memory, idx)
        }
        InstructionSequenceMode::AbsoluteIndirectJump => {
            absolute_indirect_jump(op, step, regs, memory)
        }
    }
}
