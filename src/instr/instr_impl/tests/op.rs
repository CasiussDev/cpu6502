use crate::instr::instr_impl::tests::MockMemory;
use crate::instr::instr_impl::{
    execute_branch_op, execute_implicit_op, execute_memory_modify_op, execute_op,
};
use crate::registers::{RegisterFile, StatusRegFlags};
use crate::InstructionOp;

#[test]
fn execute_increment_memory() {
    let mut regs = RegisterFile::default();
    regs.tmp.set_u8(0x01);
    execute_memory_modify_op(Some(InstructionOp::IncrementMemory), &mut regs);
    assert_eq!(regs.tmp.to_u8(), 0x02);
}

#[test]
fn execute_increment_x() {
    let mut regs = RegisterFile::default();
    regs.x.set_u8(0x01);
    execute_implicit_op(Some(InstructionOp::IncrementX), &mut regs);
    assert_eq!(regs.x.to_u8(), 0x02);
}

#[test]
fn execute_increment_y() {
    let mut regs = RegisterFile::default();
    regs.y.set_u8(0x01);
    execute_implicit_op(Some(InstructionOp::IncrementY), &mut regs);
    assert_eq!(regs.y.to_u8(), 0x02);
}

#[test]
fn execute_decrement_memory() {
    let mut regs = RegisterFile::default();
    regs.tmp.set_u8(0x01);
    execute_memory_modify_op(Some(InstructionOp::DecrementMemory), &mut regs);
    assert_eq!(regs.tmp.to_u8(), 0x00);
}

#[test]
fn execute_decrement_x() {
    let mut regs = RegisterFile::default();
    regs.x.set_u8(0x02);
    execute_implicit_op(Some(InstructionOp::DecrementX), &mut regs);
    assert_eq!(regs.x.to_u8(), 0x01);
}

#[test]
fn execute_decrement_y() {
    let mut regs = RegisterFile::default();
    regs.y.set_u8(0x02);
    execute_implicit_op(Some(InstructionOp::DecrementY), &mut regs);
    assert_eq!(regs.y.to_u8(), 0x01);
}

#[test]
fn execute_clear_carry() {
    let mut regs = RegisterFile::default();
    regs.status.set_flags(StatusRegFlags::CARRY);
    execute_implicit_op(Some(InstructionOp::ClearCarry), &mut regs);
    assert!(!regs.status.are_all_flags_set(StatusRegFlags::CARRY));
}

#[test]
fn execute_set_carry() {
    let mut regs = RegisterFile::default();
    execute_implicit_op(Some(InstructionOp::SetCarry), &mut regs);
    assert!(regs.status.are_all_flags_set(StatusRegFlags::CARRY));
}

#[test]
fn execute_clear_decimal() {
    let mut regs = RegisterFile::default();
    regs.status.set_flags(StatusRegFlags::DECIMAL);
    execute_implicit_op(Some(InstructionOp::ClearDecimal), &mut regs);
    assert!(!regs.status.are_all_flags_set(StatusRegFlags::DECIMAL));
}

#[test]
fn execute_set_decimal() {
    let mut regs = RegisterFile::default();
    execute_implicit_op(Some(InstructionOp::SetDecimal), &mut regs);
    assert!(regs.status.are_all_flags_set(StatusRegFlags::DECIMAL));
}

#[test]
fn execute_clear_interrupt_disable() {
    let mut regs = RegisterFile::default();
    regs.status.set_flags(StatusRegFlags::IRQ_DISABLE);
    execute_implicit_op(Some(InstructionOp::ClearInterruptDisable), &mut regs);
    assert!(!regs.status.are_all_flags_set(StatusRegFlags::IRQ_DISABLE));
}

#[test]
fn execute_set_interrupt_disable() {
    let mut regs = RegisterFile::default();
    execute_implicit_op(Some(InstructionOp::SetInterruptDisable), &mut regs);
    assert!(regs.status.are_all_flags_set(StatusRegFlags::IRQ_DISABLE));
}

#[test]
fn execute_clear_overflow() {
    let mut regs = RegisterFile::default();
    regs.status.set_flags(StatusRegFlags::OVERFLOW);
    execute_implicit_op(Some(InstructionOp::ClearOverflow), &mut regs);
    assert!(!regs.status.are_all_flags_set(StatusRegFlags::OVERFLOW));
}

#[test]
fn execute_set_overflow() {
    let mut regs = RegisterFile::default();
    execute_implicit_op(Some(InstructionOp::SetOverflow), &mut regs);
    assert!(regs.status.are_all_flags_set(StatusRegFlags::OVERFLOW));
}

#[test]
fn execute_transfer_accumulator_to_x() {
    let mut regs = RegisterFile::default();
    regs.a.set_u8(0x42);
    execute_implicit_op(Some(InstructionOp::TransferAccumulatorToX), &mut regs);
    assert_eq!(regs.x.to_u8(), 0x42);
}

#[test]
fn execute_transfer_accumulator_to_y() {
    let mut regs = RegisterFile::default();
    regs.a.set_u8(0x42);
    execute_implicit_op(Some(InstructionOp::TransferAccumulatorToY), &mut regs);
    assert_eq!(regs.y.to_u8(), 0x42);
}

#[test]
fn execute_transfer_stack_ptr_to_x() {
    let mut regs = RegisterFile::default();
    regs.sp.set_u8(0x42);
    execute_implicit_op(Some(InstructionOp::TransferStackPtrToX), &mut regs);
    assert_eq!(regs.x.to_u8(), 0x42);
}

#[test]
fn execute_transfer_x_to_accumulator() {
    let mut regs = RegisterFile::default();
    regs.x.set_u8(0x42);
    execute_implicit_op(Some(InstructionOp::TransferXToAccumulator), &mut regs);
    assert_eq!(regs.a.to_u8(), 0x42);
}

#[test]
fn execute_transfer_y_to_accumulator() {
    let mut regs = RegisterFile::default();
    regs.y.set_u8(0x42);
    execute_implicit_op(Some(InstructionOp::TransferYToAccumulator), &mut regs);
    assert_eq!(regs.a.to_u8(), 0x42);
}

#[test]
fn execute_transfer_x_to_stack_ptr() {
    let mut regs = RegisterFile::default();
    regs.x.set_u8(0x42);
    execute_implicit_op(Some(InstructionOp::TransferXToStackPtr), &mut regs);
    assert_eq!(regs.sp.to_u8(), 0x42);
}

#[test]
fn execute_or() {
    let mut regs = RegisterFile::default();
    let mut memory = MockMemory { data: [0; 65536] };
    memory.data[0x1000] = 0x01;
    regs.a.set_u8(0x02);
    regs.addr.set_u16(0x1000);
    execute_op(Some(InstructionOp::Or), &mut regs, &mut memory);
    assert_eq!(regs.a.to_u8(), 0x03);
}

#[test]
fn execute_and() {
    let mut regs = RegisterFile::default();
    let mut memory = MockMemory { data: [0; 65536] };
    memory.data[0x1000] = 0x01;
    regs.a.set_u8(0x03);
    regs.addr.set_u16(0x1000);
    execute_op(Some(InstructionOp::And), &mut regs, &mut memory);
    assert_eq!(regs.a.to_u8(), 0x01);
}

#[test]
fn execute_xor() {
    let mut regs = RegisterFile::default();
    let mut memory = MockMemory { data: [0; 65536] };
    memory.data[0x1000] = 0x01;
    regs.a.set_u8(0x03);
    regs.addr.set_u16(0x1000);
    execute_op(Some(InstructionOp::Xor), &mut regs, &mut memory);
    assert_eq!(regs.a.to_u8(), 0x02);
}

#[test]
fn execute_add() {
    let mut regs = RegisterFile::default();
    let mut memory = MockMemory { data: [0; 65536] };
    memory.data[0x1000] = 0x01;
    regs.a.set_u8(0x03);
    regs.addr.set_u16(0x1000);
    execute_op(Some(InstructionOp::Add), &mut regs, &mut memory);
    assert_eq!(regs.a.to_u8(), 0x04);
}

#[test]
fn execute_sub() {
    let mut regs = RegisterFile::default();
    let mut memory = MockMemory { data: [0; 65536] };
    memory.data[0x1000] = 0x01;
    regs.a.set_u8(0x03);
    regs.status.set_flags(StatusRegFlags::CARRY);
    regs.addr.set_u16(0x1000);
    execute_op(Some(InstructionOp::Sub), &mut regs, &mut memory);
    assert_eq!(regs.a.to_u8(), 0x02);
}

#[test]
fn execute_cmp() {
    let mut regs = RegisterFile::default();
    let mut memory = MockMemory { data: [0; 65536] };
    memory.data[0x1000] = 0x01;
    regs.a.set_u8(0x03);
    regs.addr.set_u16(0x1000);
    execute_op(Some(InstructionOp::Cmp), &mut regs, &mut memory);
    assert!(regs.status.are_all_flags_set(StatusRegFlags::CARRY));
}

#[test]
fn execute_cpx() {
    let mut regs = RegisterFile::default();
    let mut memory = MockMemory { data: [0; 65536] };
    memory.data[0x1000] = 0x01;
    regs.x.set_u8(0x03);
    regs.addr.set_u16(0x1000);
    execute_op(Some(InstructionOp::Cpx), &mut regs, &mut memory);
    assert!(regs.status.are_all_flags_set(StatusRegFlags::CARRY));
}

#[test]
fn execute_cpy() {
    let mut regs = RegisterFile::default();
    let mut memory = MockMemory { data: [0; 65536] };
    memory.data[0x1000] = 0x01;
    regs.y.set_u8(0x03);
    regs.addr.set_u16(0x1000);
    execute_op(Some(InstructionOp::Cpy), &mut regs, &mut memory);
    assert!(regs.status.are_all_flags_set(StatusRegFlags::CARRY));
}

#[test]
fn execute_shift_left_a() {
    let mut regs = RegisterFile::default();
    regs.a.set_u8(0x01);
    execute_implicit_op(Some(InstructionOp::ShiftLeftA), &mut regs);
    assert_eq!(regs.a.to_u8(), 0x02);
}

#[test]
fn execute_shift_load_a() {
    let mut regs = RegisterFile::default();
    let mut memory = MockMemory { data: [0; 65536] };
    memory.data[0x1000] = 0x01;
    regs.addr.set_u16(0x1000);
    execute_op(Some(InstructionOp::LoadA), &mut regs, &mut memory);
    assert_eq!(regs.a.to_u8(), 0x01);
}

#[test]
fn execute_bit() {
    let mut regs = RegisterFile::default();
    let mut memory = MockMemory { data: [0; 65536] };
    memory.data[0x1000] = 0xC0; // 1100 0000
    regs.a.set_u8(0x80); // 1000 0000
    regs.addr.set_u16(0x1000);
    regs.status.set_flags(StatusRegFlags::ZERO);
    execute_op(Some(InstructionOp::Bit), &mut regs, &mut memory);
    assert!(regs
        .status
        .are_all_flags_set(StatusRegFlags::NEGATIVE | StatusRegFlags::OVERFLOW));
    assert!(!regs.status.are_all_flags_set(StatusRegFlags::ZERO));
}

#[test]
fn execute_bit_immediate() {
    let mut regs = RegisterFile::default();
    regs.a.set_u8(0x80); // 1000 0000
    regs.tmp.set_u8(0xC0); // 1100 0000
    regs.status.set_flags(StatusRegFlags::ZERO);
    execute_op(
        Some(InstructionOp::BitImmediate),
        &mut regs,
        &mut MockMemory { data: [0; 65536] },
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
    execute_implicit_op(Some(InstructionOp::ShiftRightA), &mut regs);
    assert_eq!(regs.a.to_u8(), 0x01);
}

#[test]
fn execute_rotate_left_a() {
    let mut regs = RegisterFile::default();
    regs.a.set_u8(0x80);
    execute_implicit_op(Some(InstructionOp::RotateLeftA), &mut regs);
    assert_eq!(regs.a.to_u8(), 0x00);
    assert!(regs.status.are_all_flags_set(StatusRegFlags::CARRY));
}

#[test]
fn execute_rotate_right_a() {
    let mut regs = RegisterFile::default();
    regs.a.set_u8(0x01);
    execute_implicit_op(Some(InstructionOp::RotateRightA), &mut regs);
    assert_eq!(regs.a.to_u8(), 0x00);
    assert!(regs.status.are_all_flags_set(StatusRegFlags::CARRY));
}

#[test]
fn execute_shift_left_memory() {
    let mut regs = RegisterFile::default();
    regs.tmp.set_u8(0x01);
    execute_memory_modify_op(Some(InstructionOp::ShiftLeftMemory), &mut regs);
    assert_eq!(regs.tmp.to_u8(), 0x02);
}

#[test]
fn execute_shift_right_memory() {
    let mut regs = RegisterFile::default();
    regs.tmp.set_u8(0x02);
    execute_memory_modify_op(Some(InstructionOp::ShiftRightMemory), &mut regs);
    assert_eq!(regs.tmp.to_u8(), 0x01);
}

#[test]
fn execute_rotate_left_memory() {
    let mut regs = RegisterFile::default();
    regs.tmp.set_u8(0x80);
    execute_memory_modify_op(Some(InstructionOp::RotateLeftMemory), &mut regs);
    assert_eq!(regs.tmp.to_u8(), 0x00);
    assert!(regs.status.are_all_flags_set(StatusRegFlags::CARRY));
}

#[test]
fn execute_rotate_right_memory() {
    let mut regs = RegisterFile::default();
    regs.tmp.set_u8(0x01);
    execute_memory_modify_op(Some(InstructionOp::RotateRightMemory), &mut regs);
    assert_eq!(regs.tmp.to_u8(), 0x00);
    assert!(regs.status.are_all_flags_set(StatusRegFlags::CARRY));
}

#[test]
fn execute_store_a() {
    let mut regs = RegisterFile::default();
    let mut memory = MockMemory { data: [0; 65536] };
    regs.a.set_u8(0x42);
    regs.addr.set_u16(0x1000);
    execute_op(Some(InstructionOp::StoreA), &mut regs, &mut memory);
    assert_eq!(memory.data[0x1000], 0x42);
}

#[test]
fn execute_load_a() {
    let mut regs = RegisterFile::default();
    let mut memory = MockMemory { data: [0; 65536] };
    memory.data[0x1000] = 0x42;
    regs.addr.set_u16(0x1000);
    execute_op(Some(InstructionOp::LoadA), &mut regs, &mut memory);
    assert_eq!(regs.a.to_u8(), 0x42);
}

#[test]
fn execute_store_x() {
    let mut regs = RegisterFile::default();
    let mut memory = MockMemory { data: [0; 65536] };
    regs.x.set_u8(0x42);
    regs.addr.set_u16(0x1000);
    execute_op(Some(InstructionOp::StoreX), &mut regs, &mut memory);
    assert_eq!(memory.data[0x1000], 0x42);
}

#[test]
fn execute_load_x() {
    let mut regs = RegisterFile::default();
    let mut memory = MockMemory { data: [0; 65536] };
    memory.data[0x1000] = 0x42;
    regs.addr.set_u16(0x1000);
    execute_op(Some(InstructionOp::LoadX), &mut regs, &mut memory);
    assert_eq!(regs.x.to_u8(), 0x42);
}

#[test]
fn execute_store_y() {
    let mut regs = RegisterFile::default();
    let mut memory = MockMemory { data: [0; 65536] };
    regs.y.set_u8(0x42);
    regs.addr.set_u16(0x1000);
    execute_op(Some(InstructionOp::StoreY), &mut regs, &mut memory);
    assert_eq!(memory.data[0x1000], 0x42);
}

#[test]
fn execute_load_y() {
    let mut regs = RegisterFile::default();
    let mut memory = MockMemory { data: [0; 65536] };
    memory.data[0x1000] = 0x42;
    regs.addr.set_u16(0x1000);
    execute_op(Some(InstructionOp::LoadY), &mut regs, &mut memory);
    assert_eq!(regs.y.to_u8(), 0x42);
}

#[test]
fn execute_branch_plus() {
    let mut regs = RegisterFile::default();
    let mut memory = MockMemory { data: [0; 65536] };
    regs.status.clear_flags(StatusRegFlags::NEGATIVE);
    regs.pc.set_u16(0x1000);
    memory.data[0x1000] = 0x01; // Branch offset
    execute_branch_op(Some(InstructionOp::BranchPlus), &mut regs, &mut memory);
    assert_eq!(regs.pc.to_u16(), 0x1002);
}

#[test]
fn execute_branch_plus_not_taken() {
    let mut regs = RegisterFile::default();
    let mut memory = MockMemory { data: [0; 65536] };
    regs.status.set_flags(StatusRegFlags::NEGATIVE);
    regs.pc.set_u16(0x1000);
    memory.data[0x1000] = 0x01; // Branch offset
    execute_branch_op(Some(InstructionOp::BranchPlus), &mut regs, &mut memory);
    assert_eq!(regs.pc.to_u16(), 0x1001);
}

#[test]
fn execute_branch_minus() {
    let mut regs = RegisterFile::default();
    let mut memory = MockMemory { data: [0; 65536] };
    regs.status.set_flags(StatusRegFlags::NEGATIVE);
    regs.pc.set_u16(0x1000);
    memory.data[0x1000] = 0x01; // Branch offset
    execute_branch_op(Some(InstructionOp::BranchMinus), &mut regs, &mut memory);
    assert_eq!(regs.pc.to_u16(), 0x1002);
}

#[test]
fn execute_branch_minus_not_taken() {
    let mut regs = RegisterFile::default();
    let mut memory = MockMemory { data: [0; 65536] };
    regs.status.clear_flags(StatusRegFlags::NEGATIVE);
    regs.pc.set_u16(0x1000);
    memory.data[0x1000] = 0x01; // Branch offset
    execute_branch_op(Some(InstructionOp::BranchMinus), &mut regs, &mut memory);
    assert_eq!(regs.pc.to_u16(), 0x1001);
}

#[test]
fn execute_branch_overflow_clear() {
    let mut regs = RegisterFile::default();
    let mut memory = MockMemory { data: [0; 65536] };
    regs.status.clear_flags(StatusRegFlags::OVERFLOW);
    regs.pc.set_u16(0x1000);
    memory.data[0x1000] = 0x01; // Branch offset
    execute_branch_op(
        Some(InstructionOp::BranchOverflowClear),
        &mut regs,
        &mut memory,
    );
    assert_eq!(regs.pc.to_u16(), 0x1002);
}

#[test]
fn execute_branch_overflow_clear_not_taken() {
    let mut regs = RegisterFile::default();
    let mut memory = MockMemory { data: [0; 65536] };
    regs.status.set_flags(StatusRegFlags::OVERFLOW);
    regs.pc.set_u16(0x1000);
    memory.data[0x1000] = 0x01; // Branch offset
    execute_branch_op(
        Some(InstructionOp::BranchOverflowClear),
        &mut regs,
        &mut memory,
    );
    assert_eq!(regs.pc.to_u16(), 0x1001);
}

#[test]
fn execute_branch_overflow_set() {
    let mut regs = RegisterFile::default();
    let mut memory = MockMemory { data: [0; 65536] };
    regs.status.set_flags(StatusRegFlags::OVERFLOW);
    regs.pc.set_u16(0x1000);
    memory.data[0x1000] = 0x01; // Branch offset
    execute_branch_op(
        Some(InstructionOp::BranchOverflowSet),
        &mut regs,
        &mut memory,
    );
    assert_eq!(regs.pc.to_u16(), 0x1002);
}

#[test]
fn execute_branch_overflow_set_not_taken() {
    let mut regs = RegisterFile::default();
    let mut memory = MockMemory { data: [0; 65536] };
    regs.status.clear_flags(StatusRegFlags::OVERFLOW);
    regs.pc.set_u16(0x1000);
    memory.data[0x1000] = 0x01; // Branch offset
    execute_branch_op(
        Some(InstructionOp::BranchOverflowSet),
        &mut regs,
        &mut memory,
    );
    assert_eq!(regs.pc.to_u16(), 0x1001);
}

#[test]
fn execute_branch_carry_clear() {
    let mut regs = RegisterFile::default();
    let mut memory = MockMemory { data: [0; 65536] };
    regs.status.clear_flags(StatusRegFlags::CARRY);
    regs.pc.set_u16(0x1000);
    memory.data[0x1000] = 0x01; // Branch offset
    execute_branch_op(
        Some(InstructionOp::BranchCarryClear),
        &mut regs,
        &mut memory,
    );
    assert_eq!(regs.pc.to_u16(), 0x1002);
}

#[test]
fn execute_branch_carry_clear_not_taken() {
    let mut regs = RegisterFile::default();
    let mut memory = MockMemory { data: [0; 65536] };
    regs.status.set_flags(StatusRegFlags::CARRY);
    regs.pc.set_u16(0x1000);
    memory.data[0x1000] = 0x01; // Branch offset
    execute_branch_op(
        Some(InstructionOp::BranchCarryClear),
        &mut regs,
        &mut memory,
    );
    assert_eq!(regs.pc.to_u16(), 0x1001);
}

#[test]
fn execute_branch_carry_set() {
    let mut regs = RegisterFile::default();
    let mut memory = MockMemory { data: [0; 65536] };
    regs.status.set_flags(StatusRegFlags::CARRY);
    regs.pc.set_u16(0x1000);
    memory.data[0x1000] = 0x01; // Branch offset
    execute_branch_op(Some(InstructionOp::BranchCarrySet), &mut regs, &mut memory);
    assert_eq!(regs.pc.to_u16(), 0x1002);
}

#[test]
fn execute_branch_carry_set_not_taken() {
    let mut regs = RegisterFile::default();
    let mut memory = MockMemory { data: [0; 65536] };
    regs.status.clear_flags(StatusRegFlags::CARRY);
    regs.pc.set_u16(0x1000);
    memory.data[0x1000] = 0x01; // Branch offset
    execute_branch_op(Some(InstructionOp::BranchCarrySet), &mut regs, &mut memory);
    assert_eq!(regs.pc.to_u16(), 0x1001);
}

#[test]
fn execute_branch_not_equal() {
    let mut regs = RegisterFile::default();
    let mut memory = MockMemory { data: [0; 65536] };
    regs.status.clear_flags(StatusRegFlags::ZERO);
    regs.pc.set_u16(0x1000);
    memory.data[0x1000] = 0x01; // Branch offset
    execute_branch_op(Some(InstructionOp::BranchNotEqual), &mut regs, &mut memory);
    assert_eq!(regs.pc.to_u16(), 0x1002);
}

#[test]
fn execute_branch_not_equal_not_taken() {
    let mut regs = RegisterFile::default();
    let mut memory = MockMemory { data: [0; 65536] };
    regs.status.set_flags(StatusRegFlags::ZERO);
    regs.pc.set_u16(0x1000);
    memory.data[0x1000] = 0x01; // Branch offset
    execute_branch_op(Some(InstructionOp::BranchNotEqual), &mut regs, &mut memory);
    assert_eq!(regs.pc.to_u16(), 0x1001);
}

#[test]
fn execute_branch_equal() {
    let mut regs = RegisterFile::default();
    let mut memory = MockMemory { data: [0; 65536] };
    regs.status.set_flags(StatusRegFlags::ZERO);
    regs.pc.set_u16(0x1000);
    memory.data[0x1000] = 0x01; // Branch offset
    execute_branch_op(Some(InstructionOp::BranchEqual), &mut regs, &mut memory);
    assert_eq!(regs.pc.to_u16(), 0x1002);
}

#[test]
fn execute_branch_equal_not_taken() {
    let mut regs = RegisterFile::default();
    let mut memory = MockMemory { data: [0; 65536] };
    regs.status.clear_flags(StatusRegFlags::ZERO);
    regs.pc.set_u16(0x1000);
    memory.data[0x1000] = 0x01; // Branch offset
    execute_branch_op(Some(InstructionOp::BranchEqual), &mut regs, &mut memory);
    assert_eq!(regs.pc.to_u16(), 0x1001);
}
