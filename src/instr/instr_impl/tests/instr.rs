use crate::instr::instr_impl::tests::MockMemory;
use crate::instr::instr_impl::ClockEndStatus;
use crate::instr::InstructionSequenceMode;
use crate::registers::{RegisterFile, SelectedRegister16, StatusRegFlags};

#[test]
fn fetch_instr() {
    let mut regs = RegisterFile::default();
    let mut memory = MockMemory { data: [0; 65536] };

    // Set initial values
    regs.pc.set_u16(0x1000);
    memory.data[0x1000] = 0xA9; // Example instruction

    // Execute fetch_instr
    let result = super::fetch_instr(None, 0, &mut regs, &mut memory);

    // Verify the results
    assert_eq!(regs.ir.to_u8(), 0xA9); // Instruction register should be updated
    assert_eq!(result, ClockEndStatus::EndInstructionNextFetched);
}

#[test]
fn break_instr() {
    let mut regs = RegisterFile::default();
    let mut memory = MockMemory { data: [0; 65536] };

    // Set initial values
    regs.pc.set_u16(0x1000);
    regs.sp.set_u8(0xFF);
    regs.status.set_u8(0x00);

    // Execute break_instr
    let mut step = 0;
    while super::break_instr(None, step, &mut regs, &mut memory) == ClockEndStatus::Continue {
        step += 1;
    }

    // Verify the results
    assert_eq!(regs.sp.to_u8(), 0xFC); // Stack pointer should be decremented 3 times
    assert_eq!(
        regs.status.to_u8(),
        StatusRegFlags::IRQ_DISABLE.bits() | StatusRegFlags::BREAK.bits()
    ); // Status register should have IRQ_DISABLE and BREAK flags set
    assert_eq!(regs.pc.to_u16(), 0x0000); // Program counter should be set to the interrupt vector address
    assert_eq!(step, 5); // Ensure the step value is correct
}

#[test]
fn execute_start_irq() {
    let mut regs = RegisterFile::default();
    let mut memory = MockMemory { data: [0; 65536] };

    // Set initial values
    regs.pc.set_u16(0x1000);
    regs.sp.set_u8(0xFF);
    regs.status.set_u8(0x00);

    // Set interrupt vector values in memory
    memory.data[SelectedRegister16::InterruptAddrLow as usize] = 0x34;
    memory.data[SelectedRegister16::InterruptAddrHigh as usize] = 0x12;

    // Execute start_irq
    let mut step = 0;
    while super::execute(
        None,
        step,
        &mut regs,
        &mut memory,
        None,
        InstructionSequenceMode::StartIrq,
    ) == ClockEndStatus::Continue
    {
        step += 1;
    }

    // Verify the results
    assert_eq!(regs.pc.to_u16(), 0x1234);
    assert_eq!(regs.sp.to_u8(), 0xFC); // Stack pointer should be decremented 3 times
    assert!(regs.status.are_all_flags_set(StatusRegFlags::IRQ_DISABLE)); // IRQ_DISABLE flag should be set
    assert_eq!(step, 5);
}

#[test]
fn execute_start_nmi() {
    let mut regs = RegisterFile::default();
    let mut memory = MockMemory { data: [0; 65536] };

    // Set initial values
    regs.pc.set_u16(0x1000);
    regs.sp.set_u8(0xFF);
    regs.status.set_u8(0x00);

    // Set NMI vector values in memory
    memory.data[SelectedRegister16::NMInterruptAddrLow as usize] = 0x34;
    memory.data[SelectedRegister16::NMInterruptAddHigh as usize] = 0x12;

    // Execute start_nmi
    let mut step = 0;
    while super::execute(
        None,
        step,
        &mut regs,
        &mut memory,
        None,
        InstructionSequenceMode::StartNmi,
    ) == ClockEndStatus::Continue
    {
        step += 1;
    }

    // Verify the results
    assert_eq!(regs.pc.to_u16(), 0x1234);
    assert_eq!(regs.sp.to_u8(), 0xFC); // Stack pointer should be decremented 3 times
    assert_eq!(step, 5);
}

#[test]
fn reset() {
    let mut regs = RegisterFile::default();
    let mut memory = MockMemory { data: [0; 65536] };

    // Set initial values
    regs.pc.set_u16(0x1234);
    regs.sp.set_u8(0xFF);
    regs.status.set_u8(0x00);

    // Set interrupt vector values in memory
    memory.data[SelectedRegister16::ProgramStartAddrLow as usize] = 0x78;
    memory.data[SelectedRegister16::ProgramStartAddrHigh as usize] = 0x56;

    // Execute reset
    let mut step = 0;
    while super::execute(
        None,
        step,
        &mut regs,
        &mut memory,
        None,
        InstructionSequenceMode::Reset,
    ) == ClockEndStatus::Continue
    {
        step += 1;
    }

    // Verify the results
    assert_eq!(regs.pc.to_u16(), 0x5678);
    assert_eq!(regs.sp.to_u8(), 0xFC); // Stack pointer should be decremented 3 times
    assert_eq!(regs.status.to_u8(), 0x00); // Status register should remain unchanged
    assert_eq!(step, 5);
}

#[test]
fn execute_return_interrupt() {
    let mut regs = RegisterFile::default();
    let mut memory = MockMemory { data: [0; 65536] };

    // Set initial values
    regs.pc.set_u16(0x1000);
    regs.sp.set_u8(0xFC);
    regs.status.set_u8(0x00);

    // Set stack values in memory
    memory.data[0x01FD] = StatusRegFlags::CARRY.bits(); // Status
    memory.data[0x01FE] = 0x34; // PCL
    memory.data[0x01FF] = 0x12; // PCH

    // Execute return_interrupt
    let mut step = 0;
    while super::execute(
        None,
        step,
        &mut regs,
        &mut memory,
        None,
        InstructionSequenceMode::ReturnInterrupt,
    ) == ClockEndStatus::Continue
    {
        step += 1;
    }

    // Verify the results
    assert_eq!(regs.pc.to_u16(), 0x1234);
    assert_eq!(regs.sp.to_u8(), 0xFF); // Stack pointer should be incremented 3 times
    assert!(regs.status.are_all_flags_set(StatusRegFlags::CARRY)); // CARRY flag should be set
    assert_eq!(step, 4);
}

#[test]
fn execute_jump_subroutine() {
    let mut regs = RegisterFile::default();
    let mut memory = MockMemory { data: [0; 65536] };

    // Set initial values
    regs.pc.set_u16(0x1000);
    regs.sp.set_u8(0xFF);

    // Set subroutine address in memory
    memory.data[0x1000] = 0x34; // Low byte of subroutine address
    memory.data[0x1001] = 0x12; // High byte of subroutine address

    // Execute jump_subroutine
    let mut step = 0;
    while super::execute(
        None,
        step,
        &mut regs,
        &mut memory,
        None,
        InstructionSequenceMode::JumpSubroutine,
    ) == ClockEndStatus::Continue
    {
        step += 1;
    }

    // Verify the results
    assert_eq!(regs.pc.to_u16(), 0x1234);
    assert_eq!(regs.sp.to_u8(), 0xFD); // Stack pointer should be decremented 2 times
    assert_eq!(memory.data[0x01FF], 0x10); // High byte of return address should be on stack
    assert_eq!(memory.data[0x01FE], 0x02); // Low byte of return address should be on stack
    assert_eq!(step, 4);
}

#[test]
fn execute_return_subroutine() {
    let mut regs = RegisterFile::default();
    let mut memory = MockMemory { data: [0; 65536] };

    // Set initial values
    regs.pc.set_u16(0x1000);
    regs.sp.set_u8(0xFC);

    // Set stack values in memory
    memory.data[0x01FD] = 0x34; // PCL
    memory.data[0x01FE] = 0x12; // PCH

    // Execute return_subroutine
    let mut step = 0;
    while super::execute(
        None,
        step,
        &mut regs,
        &mut memory,
        None,
        InstructionSequenceMode::ReturnSubroutine,
    ) == ClockEndStatus::Continue
    {
        step += 1;
    }

    // Verify the results
    assert_eq!(regs.pc.to_u16(), 0x1235); // PC should be set to the return address + 1
    assert_eq!(regs.sp.to_u8(), 0xFE); // Stack pointer should be incremented 2 times
    assert_eq!(step, 4);
}

#[test]
fn execute_push_a() {
    let mut regs = RegisterFile::default();
    let mut memory = MockMemory { data: [0; 65536] };

    // Set initial values
    regs.a.set_u8(0x42);
    regs.sp.set_u8(0xFF);

    // Execute push_a
    let mut step = 0;
    while super::execute(
        Some(crate::InstructionOp::PushA),
        step,
        &mut regs,
        &mut memory,
        None,
        InstructionSequenceMode::Push,
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
    while super::execute(
        Some(crate::InstructionOp::PullA),
        step,
        &mut regs,
        &mut memory,
        None,
        InstructionSequenceMode::Pull,
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
fn execute_implied() {
    let mut regs = RegisterFile::default();
    let mut memory = MockMemory { data: [0; 65536] };

    // Set initial values
    regs.a.set_u8(0x01);

    // Execute implied operation (NOP)
    let result = super::execute(
        Some(crate::InstructionOp::Nop),
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
fn execute_immediate() {
    let mut regs = RegisterFile::default();
    let mut memory = MockMemory { data: [0; 65536] };

    // Set initial values
    regs.pc.set_u16(0x1000);
    memory.data[0x1000] = 0x42; // Immediate value

    // Execute immediate operation (LoadA)
    let result = super::execute(
        Some(crate::InstructionOp::LoadA),
        0,
        &mut regs,
        &mut memory,
        None,
        InstructionSequenceMode::Immediate,
    );

    // Verify the results
    assert_eq!(result, ClockEndStatus::EndInstruction);
    assert_eq!(regs.a.to_u8(), 0x42); // A register should be loaded with immediate value
    assert_eq!(regs.pc.to_u16(), 0x1001); // Program counter should be incremented
}

#[test]
fn execute_absolute_jump() {
    let mut regs = RegisterFile::default();
    let mut memory = MockMemory { data: [0; 65536] };

    // Set initial values
    regs.pc.set_u16(0x1000);
    memory.data[0x1000] = 0x34; // Low byte of jump address
    memory.data[0x1001] = 0x12; // High byte of jump address

    // Execute absolute_jump
    let mut step = 0;
    while super::execute(
        None,
        step,
        &mut regs,
        &mut memory,
        None,
        InstructionSequenceMode::AbsoluteJump,
    ) == ClockEndStatus::Continue
    {
        step += 1;
    }

    // Verify the results
    assert_eq!(regs.pc.to_u16(), 0x1234); // PC should be set to the jump address
    assert_eq!(step, 1);
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
    while super::execute(
        Some(crate::InstructionOp::LoadA),
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
    while super::execute(
        Some(crate::InstructionOp::IncrementMemory),
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

