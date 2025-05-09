use crate::instr::instr_impl::{
    execute_branch_op, execute_implicit_op, execute_memory_modify_op, execute_op, ClockEndStatus,
};
use crate::instr::{
    BranchOperation, ImplicitOperation, MemoryModifyOperation, RegisterMemoryOperation,
};
use crate::memory::memory_space::new_basic_ram;
use crate::registers::{RegisterFile, StatusRegFlags};

#[test]
fn execute_increment_memory() {
    let mut regs = RegisterFile::default();
    regs.tmp.set_u8(0x01);
    execute_memory_modify_op(MemoryModifyOperation::IncrementMemory, &mut regs);
    assert_eq!(regs.tmp.to_u8(), 0x02);
}

#[test]
fn execute_increment_x() {
    let mut regs = RegisterFile::default();
    regs.x.set_u8(0x01);
    execute_implicit_op(ImplicitOperation::IncrementX, &mut regs);
    assert_eq!(regs.x.to_u8(), 0x02);
}

#[test]
fn execute_increment_y() {
    let mut regs = RegisterFile::default();
    regs.y.set_u8(0x01);
    execute_implicit_op(ImplicitOperation::IncrementY, &mut regs);
    assert_eq!(regs.y.to_u8(), 0x02);
}

#[test]
fn execute_decrement_memory() {
    let mut regs = RegisterFile::default();
    regs.tmp.set_u8(0x01);
    execute_memory_modify_op(MemoryModifyOperation::DecrementMemory, &mut regs);
    assert_eq!(regs.tmp.to_u8(), 0x00);
}

#[test]
fn execute_decrement_x() {
    let mut regs = RegisterFile::default();
    regs.x.set_u8(0x02);
    execute_implicit_op(ImplicitOperation::DecrementX, &mut regs);
    assert_eq!(regs.x.to_u8(), 0x01);
}

#[test]
fn execute_decrement_y() {
    let mut regs = RegisterFile::default();
    regs.y.set_u8(0x02);
    execute_implicit_op(ImplicitOperation::DecrementY, &mut regs);
    assert_eq!(regs.y.to_u8(), 0x01);
}

#[test]
fn execute_clear_carry() {
    let mut regs = RegisterFile::default();
    regs.status.set_flags(StatusRegFlags::CARRY);
    execute_implicit_op(ImplicitOperation::ClearCarry, &mut regs);
    assert!(!regs.status.are_all_flags_set(StatusRegFlags::CARRY));
}

#[test]
fn execute_set_carry() {
    let mut regs = RegisterFile::default();
    execute_implicit_op(ImplicitOperation::SetCarry, &mut regs);
    assert!(regs.status.are_all_flags_set(StatusRegFlags::CARRY));
}

#[test]
fn execute_clear_decimal() {
    let mut regs = RegisterFile::default();
    regs.status.set_flags(StatusRegFlags::DECIMAL);
    execute_implicit_op(ImplicitOperation::ClearDecimal, &mut regs);
    assert!(!regs.status.are_all_flags_set(StatusRegFlags::DECIMAL));
}

#[test]
fn execute_set_decimal() {
    let mut regs = RegisterFile::default();
    execute_implicit_op(ImplicitOperation::SetDecimal, &mut regs);
    assert!(regs.status.are_all_flags_set(StatusRegFlags::DECIMAL));
}

#[test]
fn execute_clear_interrupt_disable() {
    let mut regs = RegisterFile::default();
    regs.status.set_flags(StatusRegFlags::IRQ_DISABLE);
    execute_implicit_op(ImplicitOperation::ClearInterruptDisable, &mut regs);
    assert!(!regs.status.are_all_flags_set(StatusRegFlags::IRQ_DISABLE));
}

#[test]
fn execute_set_interrupt_disable() {
    let mut regs = RegisterFile::default();
    execute_implicit_op(ImplicitOperation::SetInterruptDisable, &mut regs);
    assert!(regs.status.are_all_flags_set(StatusRegFlags::IRQ_DISABLE));
}

#[test]
fn execute_clear_overflow() {
    let mut regs = RegisterFile::default();
    regs.status.set_flags(StatusRegFlags::OVERFLOW);
    execute_implicit_op(ImplicitOperation::ClearOverflow, &mut regs);
    assert!(!regs.status.are_all_flags_set(StatusRegFlags::OVERFLOW));
}

#[test]
fn execute_set_overflow() {
    let mut regs = RegisterFile::default();
    execute_implicit_op(ImplicitOperation::SetOverflow, &mut regs);
    assert!(regs.status.are_all_flags_set(StatusRegFlags::OVERFLOW));
}

#[test]
fn execute_transfer_accumulator_to_x() {
    let mut regs = RegisterFile::default();
    regs.a.set_u8(0x42);
    execute_implicit_op(ImplicitOperation::TransferAccumulatorToX, &mut regs);
    assert_eq!(regs.x.to_u8(), 0x42);
}

#[test]
fn execute_transfer_accumulator_to_y() {
    let mut regs = RegisterFile::default();
    regs.a.set_u8(0x42);
    execute_implicit_op(ImplicitOperation::TransferAccumulatorToY, &mut regs);
    assert_eq!(regs.y.to_u8(), 0x42);
}

#[test]
fn execute_transfer_stack_ptr_to_x() {
    let mut regs = RegisterFile::default();
    regs.sp.set_u8(0x42);
    execute_implicit_op(ImplicitOperation::TransferStackPtrToX, &mut regs);
    assert_eq!(regs.x.to_u8(), 0x42);
}

#[test]
fn execute_transfer_x_to_accumulator() {
    let mut regs = RegisterFile::default();
    regs.x.set_u8(0x42);
    execute_implicit_op(ImplicitOperation::TransferXToAccumulator, &mut regs);
    assert_eq!(regs.a.to_u8(), 0x42);
}

#[test]
fn execute_transfer_y_to_accumulator() {
    let mut regs = RegisterFile::default();
    regs.y.set_u8(0x42);
    execute_implicit_op(ImplicitOperation::TransferYToAccumulator, &mut regs);
    assert_eq!(regs.a.to_u8(), 0x42);
}

#[test]
fn execute_transfer_x_to_stack_ptr() {
    let mut regs = RegisterFile::default();
    regs.x.set_u8(0x42);
    execute_implicit_op(ImplicitOperation::TransferXToStackPtr, &mut regs);
    assert_eq!(regs.sp.to_u8(), 0x42);
}

#[test]
fn execute_or() {
    let mut regs = RegisterFile::default();
    let mut memory = new_basic_ram();
    memory[0x1000] = 0x01;
    regs.a.set_u8(0x02);
    regs.addr.set_u16(0x1000);
    execute_op(RegisterMemoryOperation::Or, &mut regs, &mut memory);
    assert_eq!(regs.a.to_u8(), 0x03);
}

#[test]
fn execute_and() {
    let mut regs = RegisterFile::default();
    let mut memory = new_basic_ram();
    memory[0x1000] = 0x01;
    regs.a.set_u8(0x03);
    regs.addr.set_u16(0x1000);
    execute_op(RegisterMemoryOperation::And, &mut regs, &mut memory);
    assert_eq!(regs.a.to_u8(), 0x01);
}

#[test]
fn execute_xor() {
    let mut regs = RegisterFile::default();
    let mut memory = new_basic_ram();
    memory[0x1000] = 0x01;
    regs.a.set_u8(0x03);
    regs.addr.set_u16(0x1000);
    execute_op(RegisterMemoryOperation::Xor, &mut regs, &mut memory);
    assert_eq!(regs.a.to_u8(), 0x02);
}

#[test]
fn execute_add() {
    let mut regs = RegisterFile::default();
    let mut memory = new_basic_ram();
    memory[0x1000] = 0x01;
    regs.a.set_u8(0x03);
    regs.addr.set_u16(0x1000);
    execute_op(RegisterMemoryOperation::Add, &mut regs, &mut memory);
    assert_eq!(regs.a.to_u8(), 0x04);
}

#[test]
fn execute_sub() {
    let mut regs = RegisterFile::default();
    let mut memory = new_basic_ram();
    memory[0x1000] = 0x01;
    regs.a.set_u8(0x03);
    regs.status.set_flags(StatusRegFlags::CARRY);
    regs.addr.set_u16(0x1000);
    execute_op(RegisterMemoryOperation::Sub, &mut regs, &mut memory);
    assert_eq!(regs.a.to_u8(), 0x02);
}

#[test]
fn execute_cmp() {
    let mut regs = RegisterFile::default();
    let mut memory = new_basic_ram();
    memory[0x1000] = 0x01;
    regs.a.set_u8(0x03);
    regs.addr.set_u16(0x1000);
    execute_op(RegisterMemoryOperation::Cmp, &mut regs, &mut memory);
    assert!(regs.status.are_all_flags_set(StatusRegFlags::CARRY));
}

#[test]
fn execute_cpx() {
    let mut regs = RegisterFile::default();
    let mut memory = new_basic_ram();
    memory[0x1000] = 0x01;
    regs.x.set_u8(0x03);
    regs.addr.set_u16(0x1000);
    execute_op(RegisterMemoryOperation::Cpx, &mut regs, &mut memory);
    assert!(regs.status.are_all_flags_set(StatusRegFlags::CARRY));
}

#[test]
fn execute_cpy() {
    let mut regs = RegisterFile::default();
    let mut memory = new_basic_ram();
    memory[0x1000] = 0x01;
    regs.y.set_u8(0x03);
    regs.addr.set_u16(0x1000);
    execute_op(RegisterMemoryOperation::Cpy, &mut regs, &mut memory);
    assert!(regs.status.are_all_flags_set(StatusRegFlags::CARRY));
}

#[test]
fn execute_shift_left_a() {
    let mut regs = RegisterFile::default();
    regs.a.set_u8(0x01);
    execute_implicit_op(ImplicitOperation::ShiftLeftA, &mut regs);
    assert_eq!(regs.a.to_u8(), 0x02);
}

#[test]
fn execute_shift_load_a() {
    let mut regs = RegisterFile::default();
    let mut memory = new_basic_ram();
    memory[0x1000] = 0x01;
    regs.addr.set_u16(0x1000);
    execute_op(RegisterMemoryOperation::LoadA, &mut regs, &mut memory);
    assert_eq!(regs.a.to_u8(), 0x01);
}

#[test]
fn execute_bit() {
    let mut regs = RegisterFile::default();
    let mut memory = new_basic_ram();
    memory[0x1000] = 0xC0; // 1100 0000
    regs.a.set_u8(0x80); // 1000 0000
    regs.addr.set_u16(0x1000);
    regs.status.clear_flags(StatusRegFlags::ZERO);
    execute_op(RegisterMemoryOperation::Bit, &mut regs, &mut memory);
    assert!(regs
        .status
        .are_all_flags_set(StatusRegFlags::NEGATIVE | StatusRegFlags::OVERFLOW));
    assert!(!regs.status.are_all_flags_set(StatusRegFlags::ZERO));
}

#[test]
fn execute_bit_immediate() {
    let mut regs = RegisterFile::default();
    let mut memory = new_basic_ram();
    regs.a.set_u8(0x80); // 1000 0000
    regs.tmp.set_u8(0xC0); // 1100 0000
    regs.status.clear_flags(StatusRegFlags::ZERO);
    execute_op(
        RegisterMemoryOperation::BitImmediate,
        &mut regs,
        &mut memory,
    );
    assert!(!regs.status.are_all_flags_set(StatusRegFlags::ZERO));
    assert!(!regs
        .status
        .are_any_flags_set(StatusRegFlags::NEGATIVE | StatusRegFlags::OVERFLOW));
}

#[test]
fn execute_shift_right_a() {
    let mut regs = RegisterFile::default();
    regs.a.set_u8(0x02);
    execute_implicit_op(ImplicitOperation::ShiftRightA, &mut regs);
    assert_eq!(regs.a.to_u8(), 0x01);
}

#[test]
fn execute_rotate_left_a() {
    let mut regs = RegisterFile::default();
    regs.a.set_u8(0x80);
    execute_implicit_op(ImplicitOperation::RotateLeftA, &mut regs);
    assert_eq!(regs.a.to_u8(), 0x00);
    assert!(regs.status.are_all_flags_set(StatusRegFlags::CARRY));
}

#[test]
fn execute_rotate_right_a() {
    let mut regs = RegisterFile::default();
    regs.a.set_u8(0x01);
    execute_implicit_op(ImplicitOperation::RotateRightA, &mut regs);
    assert_eq!(regs.a.to_u8(), 0x00);
    assert!(regs.status.are_all_flags_set(StatusRegFlags::CARRY));
}

#[test]
fn execute_shift_left_memory() {
    let mut regs = RegisterFile::default();
    regs.tmp.set_u8(0x01);
    execute_memory_modify_op(MemoryModifyOperation::ShiftLeftMemory, &mut regs);
    assert_eq!(regs.tmp.to_u8(), 0x02);
}

#[test]
fn execute_shift_right_memory() {
    let mut regs = RegisterFile::default();
    regs.tmp.set_u8(0x02);
    execute_memory_modify_op(MemoryModifyOperation::ShiftRightMemory, &mut regs);
    assert_eq!(regs.tmp.to_u8(), 0x01);
}

#[test]
fn execute_rotate_left_memory() {
    let mut regs = RegisterFile::default();
    regs.tmp.set_u8(0x80);
    execute_memory_modify_op(MemoryModifyOperation::RotateLeftMemory, &mut regs);
    assert_eq!(regs.tmp.to_u8(), 0x00);
    assert!(regs.status.are_all_flags_set(StatusRegFlags::CARRY));
}

#[test]
fn execute_rotate_right_memory() {
    let mut regs = RegisterFile::default();
    regs.tmp.set_u8(0x01);
    execute_memory_modify_op(MemoryModifyOperation::RotateRightMemory, &mut regs);
    assert_eq!(regs.tmp.to_u8(), 0x00);
    assert!(regs.status.are_all_flags_set(StatusRegFlags::CARRY));
}

#[test]
fn execute_store_a() {
    let mut regs = RegisterFile::default();
    let mut memory = new_basic_ram();
    regs.a.set_u8(0x42);
    regs.addr.set_u16(0x1000);
    execute_op(RegisterMemoryOperation::StoreA, &mut regs, &mut memory);
    assert_eq!(memory[0x1000], 0x42);
}

#[test]
fn execute_load_a() {
    let mut regs = RegisterFile::default();
    let mut memory = new_basic_ram();
    memory[0x1000] = 0x42;
    regs.addr.set_u16(0x1000);
    execute_op(RegisterMemoryOperation::LoadA, &mut regs, &mut memory);
    assert_eq!(regs.a.to_u8(), 0x42);
}

#[test]
fn execute_store_x() {
    let mut regs = RegisterFile::default();
    let mut memory = new_basic_ram();
    regs.x.set_u8(0x42);
    regs.addr.set_u16(0x1000);
    execute_op(RegisterMemoryOperation::StoreX, &mut regs, &mut memory);
    assert_eq!(memory[0x1000], 0x42);
}

#[test]
fn execute_load_x() {
    let mut regs = RegisterFile::default();
    let mut memory = new_basic_ram();
    memory[0x1000] = 0x42;
    regs.addr.set_u16(0x1000);
    execute_op(RegisterMemoryOperation::LoadX, &mut regs, &mut memory);
    assert_eq!(regs.x.to_u8(), 0x42);
}

#[test]
fn execute_store_y() {
    let mut regs = RegisterFile::default();
    let mut memory = new_basic_ram();
    regs.y.set_u8(0x42);
    regs.addr.set_u16(0x1000);
    execute_op(RegisterMemoryOperation::StoreY, &mut regs, &mut memory);
    assert_eq!(memory[0x1000], 0x42);
}

#[test]
fn execute_load_y() {
    let mut regs = RegisterFile::default();
    let mut memory = new_basic_ram();
    memory[0x1000] = 0x42;
    regs.addr.set_u16(0x1000);
    execute_op(RegisterMemoryOperation::LoadY, &mut regs, &mut memory);
    assert_eq!(regs.y.to_u8(), 0x42);
}

#[test]
fn execute_branch_plus() {
    let mut regs = RegisterFile::default();
    let mut memory = new_basic_ram();
    regs.status.clear_flags(StatusRegFlags::NEGATIVE);
    regs.pc.set_u16(0x1000);
    regs.tmp.set_u8(0x10); // Branch offset
    let status = execute_branch_op(BranchOperation::BranchPlus, &mut regs, &mut memory);
    assert_eq!(regs.pc.to_u16(), 0x1010);
    assert_eq!(status, ClockEndStatus::EndInstruction);
}

#[test]
fn execute_branch_plus_not_taken() {
    let mut regs = RegisterFile::default();
    let mut memory = new_basic_ram();
    regs.status.set_flags(StatusRegFlags::NEGATIVE);
    regs.pc.set_u16(0x1000);
    regs.tmp.set_u8(0x10); // Branch offset
    memory[0x1000] = 0xEA; // next opcode
    let status = execute_branch_op(BranchOperation::BranchPlus, &mut regs, &mut memory);
    assert_eq!(regs.pc.to_u16(), 0x1001);
    assert_eq!(regs.ir.to_u8(), 0xEA);
    assert_eq!(
        status,
        ClockEndStatus::EndInstructionNextFetched {
            opcode_addr: 0x1000
        }
    );
}

#[test]
fn execute_branch_minus() {
    let mut regs = RegisterFile::default();
    let mut memory = new_basic_ram();
    regs.status.set_flags(StatusRegFlags::NEGATIVE);
    regs.pc.set_u16(0x1000);
    regs.tmp.set_u8(0x10); // Branch offset
    let status = execute_branch_op(BranchOperation::BranchMinus, &mut regs, &mut memory);
    assert_eq!(regs.pc.to_u16(), 0x1010);
    assert_eq!(status, ClockEndStatus::EndInstruction);
}

#[test]
fn execute_branch_minus_not_taken() {
    let mut regs = RegisterFile::default();
    let mut memory = new_basic_ram();
    regs.status.clear_flags(StatusRegFlags::NEGATIVE);
    regs.pc.set_u16(0x1000);
    regs.tmp.set_u8(0x10); // Branch offset
    memory[0x1000] = 0xEA; // next opcode
    let status = execute_branch_op(BranchOperation::BranchMinus, &mut regs, &mut memory);
    assert_eq!(regs.pc.to_u16(), 0x1001);
    assert_eq!(regs.ir.to_u8(), 0xEA);
    assert_eq!(
        status,
        ClockEndStatus::EndInstructionNextFetched {
            opcode_addr: 0x1000
        }
    );
}

#[test]
fn execute_branch_overflow_clear() {
    let mut regs = RegisterFile::default();
    let mut memory = new_basic_ram();
    regs.status.clear_flags(StatusRegFlags::OVERFLOW);
    regs.pc.set_u16(0x1000);
    regs.tmp.set_u8(0x10); // Branch offset
    let status = execute_branch_op(BranchOperation::BranchOverflowClear, &mut regs, &mut memory);
    assert_eq!(regs.pc.to_u16(), 0x1010);
    assert_eq!(status, ClockEndStatus::EndInstruction);
}

#[test]
fn execute_branch_overflow_clear_not_taken() {
    let mut regs = RegisterFile::default();
    let mut memory = new_basic_ram();
    regs.status.set_flags(StatusRegFlags::OVERFLOW);
    regs.pc.set_u16(0x1000);
    regs.tmp.set_u8(0x10); // Branch offset
    memory[0x1000] = 0xEA; // next opcode
    let status = execute_branch_op(BranchOperation::BranchOverflowClear, &mut regs, &mut memory);
    assert_eq!(regs.pc.to_u16(), 0x1001);
    assert_eq!(regs.ir.to_u8(), 0xEA);
    assert_eq!(
        status,
        ClockEndStatus::EndInstructionNextFetched {
            opcode_addr: 0x1000
        }
    );
}

#[test]
fn execute_branch_overflow_set() {
    let mut regs = RegisterFile::default();
    let mut memory = new_basic_ram();
    regs.status.set_flags(StatusRegFlags::OVERFLOW);
    regs.pc.set_u16(0x1000);
    regs.tmp.set_u8(0x10); // Branch offset
    let status = execute_branch_op(BranchOperation::BranchOverflowSet, &mut regs, &mut memory);
    assert_eq!(regs.pc.to_u16(), 0x1010);
    assert_eq!(status, ClockEndStatus::EndInstruction);
}

#[test]
fn execute_branch_overflow_set_not_taken() {
    let mut regs = RegisterFile::default();
    let mut memory = new_basic_ram();
    regs.status.clear_flags(StatusRegFlags::OVERFLOW);
    regs.pc.set_u16(0x1000);
    regs.tmp.set_u8(0x10); // Branch offset
    memory[0x1000] = 0xEA; // next opcode
    let status = execute_branch_op(BranchOperation::BranchOverflowSet, &mut regs, &mut memory);
    assert_eq!(regs.pc.to_u16(), 0x1001);
    assert_eq!(regs.ir.to_u8(), 0xEA);
    assert_eq!(
        status,
        ClockEndStatus::EndInstructionNextFetched {
            opcode_addr: 0x1000
        }
    );
}

#[test]
fn execute_branch_carry_clear() {
    let mut regs = RegisterFile::default();
    let mut memory = new_basic_ram();
    regs.status.clear_flags(StatusRegFlags::CARRY);
    regs.pc.set_u16(0x1000);
    regs.tmp.set_u8(0x10); // Branch offset
    let status = execute_branch_op(BranchOperation::BranchCarryClear, &mut regs, &mut memory);
    assert_eq!(regs.pc.to_u16(), 0x1010);
    assert_eq!(status, ClockEndStatus::EndInstruction);
}

#[test]
fn execute_branch_carry_clear_not_taken() {
    let mut regs = RegisterFile::default();
    let mut memory = new_basic_ram();
    regs.status.set_flags(StatusRegFlags::CARRY);
    regs.pc.set_u16(0x1000);
    regs.tmp.set_u8(0x10); // Branch offset
    memory[0x1000] = 0xEA; // next opcode
    let status = execute_branch_op(BranchOperation::BranchCarryClear, &mut regs, &mut memory);
    assert_eq!(regs.pc.to_u16(), 0x1001);
    assert_eq!(regs.ir.to_u8(), 0xEA);
    assert_eq!(
        status,
        ClockEndStatus::EndInstructionNextFetched {
            opcode_addr: 0x1000
        }
    );
}

#[test]
fn execute_branch_carry_set() {
    let mut regs = RegisterFile::default();
    let mut memory = new_basic_ram();
    regs.status.set_flags(StatusRegFlags::CARRY);
    regs.pc.set_u16(0x1000);
    regs.tmp.set_u8(0x10); // Branch offset
    let status = execute_branch_op(BranchOperation::BranchCarrySet, &mut regs, &mut memory);
    assert_eq!(regs.pc.to_u16(), 0x1010);
    assert_eq!(status, ClockEndStatus::EndInstruction);
}

#[test]
fn execute_branch_carry_set_not_taken() {
    let mut regs = RegisterFile::default();
    let mut memory = new_basic_ram();
    regs.status.clear_flags(StatusRegFlags::CARRY);
    regs.pc.set_u16(0x1000);
    regs.tmp.set_u8(0x10); // Branch offset
    memory[0x1000] = 0xEA; // next opcode
    let status = execute_branch_op(BranchOperation::BranchCarrySet, &mut regs, &mut memory);
    assert_eq!(regs.pc.to_u16(), 0x1001);
    assert_eq!(regs.ir.to_u8(), 0xEA);
    assert_eq!(
        status,
        ClockEndStatus::EndInstructionNextFetched {
            opcode_addr: 0x1000
        }
    );
}

#[test]
fn execute_branch_not_equal() {
    let mut regs = RegisterFile::default();
    let mut memory = new_basic_ram();
    regs.status.clear_flags(StatusRegFlags::ZERO);
    regs.pc.set_u16(0x1000);
    regs.tmp.set_u8(0x10); // Branch offset
    let status = execute_branch_op(BranchOperation::BranchNotEqual, &mut regs, &mut memory);
    assert_eq!(regs.pc.to_u16(), 0x1010);
    assert_eq!(status, ClockEndStatus::EndInstruction);
}

#[test]
fn execute_branch_not_equal_not_taken() {
    let mut regs = RegisterFile::default();
    let mut memory = new_basic_ram();
    regs.status.set_flags(StatusRegFlags::ZERO);
    regs.pc.set_u16(0x1000);
    regs.tmp.set_u8(0x10); // Branch offset
    memory[0x1000] = 0xEA; // next opcode
    let status = execute_branch_op(BranchOperation::BranchNotEqual, &mut regs, &mut memory);
    assert_eq!(regs.pc.to_u16(), 0x1001);
    assert_eq!(regs.ir.to_u8(), 0xEA);
    assert_eq!(
        status,
        ClockEndStatus::EndInstructionNextFetched {
            opcode_addr: 0x1000
        }
    );
}

#[test]
fn execute_branch_equal() {
    let mut regs = RegisterFile::default();
    let mut memory = new_basic_ram();
    regs.status.set_flags(StatusRegFlags::ZERO);
    regs.pc.set_u16(0x1000);
    regs.tmp.set_u8(0x10); // Branch offset
    let status = execute_branch_op(BranchOperation::BranchEqual, &mut regs, &mut memory);
    assert_eq!(regs.pc.to_u16(), 0x1010);
    assert_eq!(status, ClockEndStatus::EndInstruction);
}

#[test]
fn execute_branch_equal_not_taken() {
    let mut regs = RegisterFile::default();
    let mut memory = new_basic_ram();
    regs.status.clear_flags(StatusRegFlags::ZERO);
    regs.pc.set_u16(0x1000);
    regs.tmp.set_u8(0x10); // Branch offset
    memory[0x1000] = 0xEA; // next opcode
    let status = execute_branch_op(BranchOperation::BranchEqual, &mut regs, &mut memory);
    assert_eq!(regs.pc.to_u16(), 0x1001);
    assert_eq!(regs.ir.to_u8(), 0xEA);
    assert_eq!(
        status,
        ClockEndStatus::EndInstructionNextFetched {
            opcode_addr: 0x1000
        }
    );
}
