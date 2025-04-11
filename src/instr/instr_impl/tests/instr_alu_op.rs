use crate::instr::instr_impl::tests::MockMemory;
use crate::instr::instr_impl::{execute, ClockEndStatus};
use crate::instr::InstructionSequenceMode;
use crate::registers::{IndexRegister, RegisterFile};
use crate::InstructionOp;

#[test]
fn execute_implied() {
    let mut regs = RegisterFile::default();
    let mut memory = MockMemory { data: [0; 65536] };

    // Set initial values
    regs.a.set_u8(0x01);

    // Execute implied operation (NOP)
    let result = execute(
        Some(InstructionOp::Nop),
        0,
        &mut regs,
        &mut memory,
        None,
        InstructionSequenceMode::Implied,
    );

    // Verify the results
    assert_eq!(result, ClockEndStatus::EndInstruction);
    assert_eq!(regs.a.to_u8(), 0x01); // A register should remain unchanged
}

#[test]
#[should_panic(expected = "unreachable")]
fn execute_implied_unsupported_op() {
    let mut regs = RegisterFile::default();
    let mut memory = MockMemory { data: [0; 65536] };

    // Execute implied operation with an unsupported operation
    execute(
        Some(InstructionOp::StoreA), // StoreA is not valid for implied mode
        0,
        &mut regs,
        &mut memory,
        None,
        InstructionSequenceMode::Implied,
    );
}

#[test]
fn execute_absolute() {
    let mut regs = RegisterFile::default();
    let mut memory = MockMemory { data: [0; 65536] };

    // Set initial values
    regs.pc.set_u16(0x1000);
    memory.data[0x1000] = 0x34; // Low byte of address
    memory.data[0x1001] = 0x12; // High byte of address
    memory.data[0x1234] = 0x42; // Value to load into A

    // Execute absolute operation (LoadA)
    let mut step = 0;
    while execute(
        Some(InstructionOp::LoadA),
        step,
        &mut regs,
        &mut memory,
        None,
        InstructionSequenceMode::Absolute,
    ) == ClockEndStatus::Continue
    {
        step += 1;
    }

    // Verify the results
    assert_eq!(regs.a.to_u8(), 0x42); // A register should be loaded with value from memory
    assert_eq!(regs.pc.to_u16(), 0x1002); // Program counter should be incremented
    assert_eq!(step, 2);
}

#[test]
fn execute_absolute_read_modify_write() {
    let mut regs = RegisterFile::default();
    let mut memory = MockMemory { data: [0; 65536] };

    // Set initial values
    regs.pc.set_u16(0x1000);
    memory.data[0x1000] = 0x34; // Low byte of address
    memory.data[0x1001] = 0x12; // High byte of address
    memory.data[0x1234] = 0x01; // Initial value in memory

    // Execute absolute read-modify-write operation (IncrementMemory)
    let mut step = 0;
    while execute(
        Some(InstructionOp::IncrementMemory),
        step,
        &mut regs,
        &mut memory,
        None,
        InstructionSequenceMode::AbsoluteReadModifyWrite,
    ) == ClockEndStatus::Continue
    {
        step += 1;
    }

    // Verify the results
    assert_eq!(memory.data[0x1234], 0x02); // Value in memory should be incremented
    assert_eq!(regs.pc.to_u16(), 0x1002); // Program counter should be incremented
    assert_eq!(step, 4);
}

#[test]
fn execute_zero_page() {
    let mut regs = RegisterFile::default();
    let mut memory = MockMemory { data: [0; 65536] };

    // Set initial values
    regs.pc.set_u16(0x1000);
    memory.data[0x1000] = 0x42; // Zero page address
    memory.data[0x0042] = 0x84; // Value to load into A

    // Execute zero page operation (LoadA)
    let mut step = 0;
    while execute(
        Some(InstructionOp::LoadA),
        step,
        &mut regs,
        &mut memory,
        None,
        InstructionSequenceMode::ZeroPage,
    ) == ClockEndStatus::Continue
    {
        step += 1;
    }

    // Verify the results
    assert_eq!(regs.a.to_u8(), 0x84); // A register should be loaded with value from zero page
    assert_eq!(regs.pc.to_u16(), 0x1001); // Program counter should be incremented
    assert_eq!(step, 1);
}

#[test]
fn execute_zero_page_read_modify_write() {
    let mut regs = RegisterFile::default();
    let mut memory = MockMemory { data: [0; 65536] };

    // Set initial values
    regs.pc.set_u16(0x1000);
    memory.data[0x1000] = 0x42; // Zero page address
    memory.data[0x0042] = 0x01; // Initial value in zero page

    // Execute zero page read-modify-write operation (IncrementMemory)
    let mut step = 0;
    while execute(
        Some(InstructionOp::IncrementMemory),
        step,
        &mut regs,
        &mut memory,
        None,
        InstructionSequenceMode::ZeroPageReadModifyWrite,
    ) == ClockEndStatus::Continue
    {
        step += 1;
    }

    // Verify the results
    assert_eq!(memory.data[0x0042], 0x02); // Value in zero page should be incremented
    assert_eq!(regs.pc.to_u16(), 0x1001); // Program counter should be incremented
    assert_eq!(step, 3);
}

#[test]
fn execute_zero_page_indexed() {
    let mut regs = RegisterFile::default();
    let mut memory = MockMemory { data: [0; 65536] };

    // Set initial values
    regs.pc.set_u16(0x1000);
    regs.x.set_u8(0x10); // Index register X
    memory.data[0x1000] = 0x42; // Zero page base address
    memory.data[0x0052] = 0x84; // Value to load into A (0x42 + 0x10)

    // Execute zero page indexed operation (LoadA)
    let mut step = 0;
    while execute(
        Some(InstructionOp::LoadA),
        step,
        &mut regs,
        &mut memory,
        Some(IndexRegister::X),
        InstructionSequenceMode::ZeroPageIdx,
    ) == ClockEndStatus::Continue
    {
        step += 1;
    }

    // Verify the results
    assert_eq!(regs.a.to_u8(), 0x84); // A register should be loaded with value from zero page indexed
    assert_eq!(regs.pc.to_u16(), 0x1001); // Program counter should be incremented
    assert_eq!(step, 2);
}

#[test]
fn execute_zero_page_indexed_read_modify_write() {
    let mut regs = RegisterFile::default();
    let mut memory = MockMemory { data: [0; 65536] };

    // Set initial values
    regs.pc.set_u16(0x1000);
    regs.x.set_u8(0x10); // Index register X
    memory.data[0x1000] = 0x42; // Zero page base address
    memory.data[0x0052] = 0x01; // Initial value in zero page indexed (0x42 + 0x10)

    // Execute zero page indexed read-modify-write operation (IncrementMemory)
    let mut step = 0;
    while execute(
        Some(InstructionOp::IncrementMemory),
        step,
        &mut regs,
        &mut memory,
        Some(IndexRegister::X),
        InstructionSequenceMode::ZeroPageIdxReadModifyWrite,
    ) == ClockEndStatus::Continue
    {
        step += 1;
    }

    // Verify the results
    assert_eq!(memory.data[0x0052], 0x02); // Value in zero page indexed should be incremented
    assert_eq!(regs.pc.to_u16(), 0x1001); // Program counter should be incremented
    assert_eq!(step, 4);
}

#[test]
fn execute_absolute_indexed_read() {
    let mut regs = RegisterFile::default();
    let mut memory = MockMemory { data: [0; 65536] };

    // Set initial values
    regs.pc.set_u16(0x1000);
    regs.x.set_u8(0x10); // Index register X
    memory.data[0x1000] = 0x34; // Low byte of address
    memory.data[0x1001] = 0x12; // High byte of address
    memory.data[0x1244] = 0x42; // Value to load into A (0x1234 + 0x10)

    // Execute absolute indexed read operation (LoadA)
    let mut step = 0;
    while execute(
        Some(InstructionOp::LoadA),
        step,
        &mut regs,
        &mut memory,
        Some(IndexRegister::X),
        InstructionSequenceMode::AbsoluteIdxRead,
    ) == ClockEndStatus::Continue
    {
        step += 1;
    }

    // Verify the results
    assert_eq!(regs.a.to_u8(), 0x42); // A register should be loaded with value from memory
    assert_eq!(regs.pc.to_u16(), 0x1002); // Program counter should be incremented
    assert_eq!(step, 2);
}

#[test]
fn execute_absolute_indexed_read_extra_cycle() {
    let mut regs = RegisterFile::default();
    let mut memory = MockMemory { data: [0; 65536] };

    // Set initial values
    regs.pc.set_u16(0x1000);
    regs.x.set_u8(0xFF); // Index register X
    memory.data[0x1000] = 0x34; // Low byte of address
    memory.data[0x1001] = 0x12; // High byte of address
    memory.data[0x1333] = 0x42; // Value to load into A (0x1234 + 0xFF)

    // Execute absolute indexed read operation (LoadA)
    let mut step = 0;
    while execute(
        Some(InstructionOp::LoadA),
        step,
        &mut regs,
        &mut memory,
        Some(IndexRegister::X),
        InstructionSequenceMode::AbsoluteIdxRead,
    ) == ClockEndStatus::Continue
    {
        step += 1;
    }

    // Verify the results
    assert_eq!(regs.a.to_u8(), 0x42); // A register should be loaded with value from memory
    assert_eq!(regs.pc.to_u16(), 0x1002); // Program counter should be incremented
    assert_eq!(step, 3); // Ensure the step value is correct for the extra cycle
}

#[test]
fn execute_absolute_indexed_read_modify_write() {
    let mut regs = RegisterFile::default();
    let mut memory = MockMemory { data: [0; 65536] };

    // Set initial values
    regs.pc.set_u16(0x1000);
    regs.x.set_u8(0x10); // Index register X
    memory.data[0x1000] = 0x34; // Low byte of address
    memory.data[0x1001] = 0x12; // High byte of address
    memory.data[0x1244] = 0x01; // Initial value in memory (0x1234 + 0x10)

    // Execute absolute indexed read-modify-write operation (IncrementMemory)
    let mut step = 0;
    while execute(
        Some(InstructionOp::IncrementMemory),
        step,
        &mut regs,
        &mut memory,
        Some(IndexRegister::X),
        InstructionSequenceMode::AbsoluteIdxReadModifyWrite,
    ) == ClockEndStatus::Continue
    {
        step += 1;
    }

    // Verify the results
    assert_eq!(memory.data[0x1244], 0x02); // Value in memory should be incremented
    assert_eq!(regs.pc.to_u16(), 0x1002); // Program counter should be incremented
    assert_eq!(step, 5);
}

#[test]
fn execute_absolute_indexed_write() {
    let mut regs = RegisterFile::default();
    let mut memory = MockMemory { data: [0; 65536] };

    // Set initial values
    regs.pc.set_u16(0x1000);
    regs.x.set_u8(0x10); // Index register X
    regs.a.set_u8(0x42); // Value to store
    memory.data[0x1000] = 0x34; // Low byte of address
    memory.data[0x1001] = 0x12; // High byte of address

    // Execute absolute indexed write operation (StoreA)
    let mut step = 0;
    while execute(
        Some(InstructionOp::StoreA),
        step,
        &mut regs,
        &mut memory,
        Some(IndexRegister::X),
        InstructionSequenceMode::AbsoluteIdxWrite,
    ) == ClockEndStatus::Continue
    {
        step += 1;
    }

    // Verify the results
    assert_eq!(memory.data[0x1244], 0x42); // Value in memory should be updated
    assert_eq!(regs.pc.to_u16(), 0x1002); // Program counter should be incremented
    assert_eq!(step, 3);
}

#[test]
fn execute_zero_page_indexed_indirect() {
    let mut regs = RegisterFile::default();
    let mut memory = MockMemory { data: [0; 65536] };

    // Set initial values
    regs.pc.set_u16(0x1000);
    regs.x.set_u8(0x10); // Index register X
    memory.data[0x1000] = 0x42; // Zero page base address
    memory.data[0x0052] = 0x34; // Low byte of effective address (0x42 + 0x10)
    memory.data[0x0053] = 0x12; // High byte of effective address
    memory.data[0x1234] = 0x84; // Value to load into A

    // Execute zero page indexed indirect operation (LoadA)
    let mut step = 0;
    while execute(
        Some(InstructionOp::LoadA),
        step,
        &mut regs,
        &mut memory,
        Some(IndexRegister::X),
        InstructionSequenceMode::ZeroPageIdxIndirect,
    ) == ClockEndStatus::Continue
    {
        step += 1;
    }

    // Verify the results
    assert_eq!(regs.a.to_u8(), 0x84); // A register should be loaded with value from effective address
    assert_eq!(regs.pc.to_u16(), 0x1001); // Program counter should be incremented
    assert_eq!(step, 4);
}

#[test]
fn execute_zero_page_indexed_indirect_read_modify_write() {
    let mut regs = RegisterFile::default();
    let mut memory = MockMemory { data: [0; 65536] };

    // Set initial values
    regs.pc.set_u16(0x1000);
    regs.x.set_u8(0x10); // Index register X
    memory.data[0x1000] = 0x42; // Zero page base address
    memory.data[0x0052] = 0x34; // Low byte of effective address (0x42 + 0x10)
    memory.data[0x0053] = 0x12; // High byte of effective address
    memory.data[0x1234] = 0x01; // Initial value in memory

    // Execute zero page indexed indirect read-modify-write operation (IncrementMemory)
    let mut step = 0;
    while execute(
        Some(InstructionOp::IncrementMemory),
        step,
        &mut regs,
        &mut memory,
        Some(IndexRegister::X),
        InstructionSequenceMode::ZeroPageIdxIndirectReadModifyWrite,
    ) == ClockEndStatus::Continue
    {
        step += 1;
    }

    // Verify the results
    assert_eq!(memory.data[0x1234], 0x02); // Value in memory should be incremented
    assert_eq!(regs.pc.to_u16(), 0x1001); // Program counter should be incremented
    assert_eq!(step, 6);
}

#[test]
fn execute_zero_page_indirect_idx_read() {
    let mut regs = RegisterFile::default();
    let mut memory = MockMemory { data: [0; 65536] };

    // Set initial values
    regs.pc.set_u16(0x1000);
    regs.y.set_u8(0x10); // Index register Y
    memory.data[0x1000] = 0x42; // Zero page base address
    memory.data[0x0042] = 0x34; // Low byte of effective address
    memory.data[0x0043] = 0x12; // High byte of effective address
    memory.data[0x1244] = 0x84; // Value to load into A (0x1234 + 0x10)

    // Execute zero page indirect indexed read operation (LoadA)
    let mut step = 0;
    while execute(
        Some(InstructionOp::LoadA),
        step,
        &mut regs,
        &mut memory,
        Some(IndexRegister::Y),
        InstructionSequenceMode::ZeroPageIndirectIdxRead,
    ) == ClockEndStatus::Continue
    {
        step += 1;
    }

    // Verify the results
    assert_eq!(regs.a.to_u8(), 0x84); // A register should be loaded with value from effective address
    assert_eq!(regs.pc.to_u16(), 0x1001); // Program counter should be incremented
    assert_eq!(step, 3);
}

#[test]
fn execute_zero_page_indirect_idx_read_modify_write() {
    let mut regs = RegisterFile::default();
    let mut memory = MockMemory { data: [0; 65536] };

    // Set initial values
    regs.pc.set_u16(0x1000);
    regs.y.set_u8(0x10); // Index register Y
    memory.data[0x1000] = 0x42; // Zero page base address
    memory.data[0x0042] = 0x34; // Low byte of effective address
    memory.data[0x0043] = 0x12; // High byte of effective address
    memory.data[0x1244] = 0x01; // Initial value in memory (0x1234 + 0x10)

    // Execute zero page indirect indexed read-modify-write operation (IncrementMemory)
    let mut step = 0;
    while execute(
        Some(InstructionOp::IncrementMemory),
        step,
        &mut regs,
        &mut memory,
        Some(IndexRegister::Y),
        InstructionSequenceMode::ZeroPageIndirectIdxReadModifyWrite,
    ) == ClockEndStatus::Continue
    {
        step += 1;
    }

    // Verify the results
    assert_eq!(memory.data[0x1244], 0x02); // Value in memory should be incremented
    assert_eq!(regs.pc.to_u16(), 0x1001); // Program counter should be incremented
    assert_eq!(step, 6);
}

#[test]
fn execute_zero_page_indirect_idx_write() {
    let mut regs = RegisterFile::default();
    let mut memory = MockMemory { data: [0; 65536] };

    // Set initial values
    regs.pc.set_u16(0x1000);
    regs.y.set_u8(0x10); // Index register Y
    regs.a.set_u8(0x42); // Value to store
    memory.data[0x1000] = 0x42; // Zero page base address
    memory.data[0x0042] = 0x34; // Low byte of effective address
    memory.data[0x0043] = 0x12; // High byte of effective address

    // Execute zero page indirect indexed write operation (StoreA)
    let mut step = 0;
    while execute(
        Some(InstructionOp::StoreA),
        step,
        &mut regs,
        &mut memory,
        Some(IndexRegister::Y),
        InstructionSequenceMode::ZeroPageIndirectIdxWrite,
    ) == ClockEndStatus::Continue
    {
        step += 1;
    }

    // Verify the results
    assert_eq!(memory.data[0x1244], 0x42); // Value in memory should be updated
    assert_eq!(regs.pc.to_u16(), 0x1001); // Program counter should be incremented
    assert_eq!(step, 4);
}
