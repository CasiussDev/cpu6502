use crate::instr::instr_impl::tests::MockMemory;
use crate::instr::instr_impl::{
    add_index_to_address, fix_addr_or_run_op_finish, ClockEndStatus, FixAddressResult,
};
use crate::instr::RegisterMemoryOperation;
use crate::registers::{IndexRegister, RegisterFile};

#[test]
fn get_index_value() {
    let mut regs = RegisterFile::default();
    regs.x.set_u8(0x12);
    regs.y.set_u8(0xFF);
    let idx = IndexRegister::X;
    assert_eq!(super::get_index_value(idx, &mut regs), 0x12);
}

#[test]
fn get_index_value_y() {
    let mut regs = RegisterFile::default();
    regs.x.set_u8(0x12);
    regs.y.set_u8(0xFF);
    let idx = IndexRegister::Y;
    assert_eq!(super::get_index_value(idx, &mut regs), 0xFF);
}

#[test]
fn set_stack_address() {
    let mut regs = RegisterFile::default();
    regs.sp.set_u8(0x34);
    super::set_stack_address(&mut regs);
    assert_eq!(regs.addr.to_u16(), 0x0134);
}

#[test]
fn add_index_to_address_x() {
    let mut regs = RegisterFile::default();
    regs.addr.set_u16(0x1000);
    regs.x.set_u8(0x10);
    regs.y.set_u8(0xFF);
    add_index_to_address(&mut regs, IndexRegister::X);
    assert_eq!(regs.addr.to_u16(), 0x1010);
}

#[test]
fn add_index_to_address_y() {
    let mut regs = RegisterFile::default();
    regs.addr.set_u16(0x1000);
    regs.x.set_u8(0xFF);
    regs.y.set_u8(0x10);
    add_index_to_address(&mut regs, IndexRegister::Y);
    assert_eq!(regs.addr.to_u16(), 0x1010);
}

#[test]
fn fix_addr() {
    let mut regs = RegisterFile::default();
    regs.addr.set_u16(0x0FF);
    let result = super::fix_addr(&mut regs, 0x01);
    assert_eq!(result, FixAddressResult::Untouched);
    assert_eq!(regs.addr.to_u16(), 0x00FF);
}

#[test]
fn fix_addr_fixed() {
    let mut regs = RegisterFile::default();
    regs.addr.set_u16(0x0FA);
    let result = super::fix_addr(&mut regs, 0xFF);
    assert_eq!(result, FixAddressResult::Fixed);
    assert_eq!(regs.addr.to_u16(), 0x01FA);
}

#[test]
fn fix_addr_or_run_op_finish_x() {
    let mut regs = RegisterFile::default();
    regs.a.set_u8(0x0F);
    regs.x.set_u8(0x01);
    regs.addr.set_u16(0x1010);
    let mut memory = MockMemory { data: [0; 65536] };
    memory.data[0x1010] = 0x01;

    let result = fix_addr_or_run_op_finish(
        RegisterMemoryOperation::Add,
        &mut regs,
        &mut memory,
        IndexRegister::X,
    );
    assert_eq!(regs.a.to_u8(), 0x10);
    assert_eq!(result, ClockEndStatus::EndInstruction);
}

#[test]
fn fix_addr_or_run_op_finish_x_fixed() {
    let mut regs = RegisterFile::default();
    regs.a.set_u8(0x0F);
    regs.x.set_u8(0x11);
    regs.addr.set_u16(0x1010);
    let mut memory = MockMemory { data: [0; 65536] };
    memory.data[0x1010] = 0x01;

    let result = fix_addr_or_run_op_finish(
        RegisterMemoryOperation::Add,
        &mut regs,
        &mut memory,
        IndexRegister::X,
    );
    assert_eq!(regs.a.to_u8(), 0x0F);
    assert_eq!(regs.addr.to_u16(), 0x1110);
    assert_eq!(result, ClockEndStatus::Continue);
}

#[test]
fn fix_addr_or_run_op_finish_y() {
    let mut regs = RegisterFile::default();
    regs.a.set_u8(0x0F);
    regs.y.set_u8(0x01);
    regs.addr.set_u16(0x1010);
    let mut memory = MockMemory { data: [0; 65536] };
    memory.data[0x1010] = 0x01;

    let result = fix_addr_or_run_op_finish(
        RegisterMemoryOperation::Add,
        &mut regs,
        &mut memory,
        IndexRegister::Y,
    );
    assert_eq!(regs.a.to_u8(), 0x10);
    assert_eq!(result, ClockEndStatus::EndInstruction);
}

#[test]
fn fix_addr_or_run_op_finish_y_fixed() {
    let mut regs = RegisterFile::default();
    regs.a.set_u8(0x0F);
    regs.y.set_u8(0x11);
    regs.addr.set_u16(0x1010);
    let mut memory = MockMemory { data: [0; 65536] };
    memory.data[0x1010] = 0x01;

    let result = fix_addr_or_run_op_finish(
        RegisterMemoryOperation::Add,
        &mut regs,
        &mut memory,
        IndexRegister::Y,
    );
    assert_eq!(regs.a.to_u8(), 0x0F);
    assert_eq!(regs.addr.to_u16(), 0x1110);
    assert_eq!(result, ClockEndStatus::Continue);
}
