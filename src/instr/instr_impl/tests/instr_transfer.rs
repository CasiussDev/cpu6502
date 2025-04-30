use crate::instr::instr_impl::tests::MockMemory;
use crate::instr::instr_impl::{execute, ClockEndStatus};
use crate::instr::InstructionSequenceMode2;
use crate::registers::RegisterFile;

#[test]
fn execute_push_a() {
    let mut regs = RegisterFile::default();
    let mut memory = MockMemory { data: [0; 65536] };

    // Set initial values
    regs.a.set_u8(0x42);
    regs.sp.set_u8(0xFF);

    // Execute push_a
    let mut step = 0;
    while execute(
        InstructionSequenceMode2::Push(crate::instr::PushStackOperation::PushA),
        step,
        &mut regs,
        &mut memory,
    ) == ClockEndStatus::Continue
    {
        step += 1;
    }

    // Verify the results
    assert_eq!(regs.sp.to_u8(), 0xFE); // Stack pointer should be decremented
    assert_eq!(memory.data[0x01FF], 0x42); // Value of A should be on stack
    assert_eq!(step, 1);
}

#[test]
fn execute_pull_a() {
    let mut regs = RegisterFile::default();
    let mut memory = MockMemory { data: [0; 65536] };

    // Set initial values
    regs.sp.set_u8(0xFE);
    memory.data[0x01FF] = 0x42; // Value to pull into A

    // Execute pull_a
    let mut step = 0;
    while execute(
        InstructionSequenceMode2::Pull(crate::instr::PullStackOperation::PullA),
        step,
        &mut regs,
        &mut memory,
    ) == ClockEndStatus::Continue
    {
        step += 1;
    }

    // Verify the results
    assert_eq!(regs.sp.to_u8(), 0xFF); // Stack pointer should be incremented
    assert_eq!(regs.a.to_u8(), 0x42); // Value of A should be updated
    assert_eq!(step, 2);
}

#[test]
fn execute_immediate() {
    let mut regs = RegisterFile::default();
    let mut memory = MockMemory { data: [0; 65536] };

    // Set initial values
    regs.pc.set_u16(0x1000);
    memory.data[0x1000] = 0x42; // Immediate value

    // Execute immediate operation (LoadA)
    let result = execute(
        InstructionSequenceMode2::Immediate(crate::instr::RegisterMemoryOperation::LoadA),
        0,
        &mut regs,
        &mut memory,
    );

    // Verify the results
    assert_eq!(result, ClockEndStatus::EndInstruction);
    assert_eq!(regs.a.to_u8(), 0x42); // A register should be loaded with immediate value
    assert_eq!(regs.pc.to_u16(), 0x1001); // Program counter should be incremented
}
