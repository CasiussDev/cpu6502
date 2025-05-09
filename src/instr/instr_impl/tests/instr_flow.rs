use crate::cpu::interrupt::{InterruptVector, InterruptVectorAddrBytePos};
use crate::instr::instr_impl::{execute, ClockEndStatus};
use crate::instr::{BranchOperation, Instruction};
use crate::memory::memory_space::new_basic_ram;
use crate::registers::{RegisterFile, StatusReg, StatusRegFlags};
use crate::MemorySpace;

#[test]
fn break_instr() {
    let mut regs = RegisterFile::default();
    let mut memory = new_basic_ram();

    // Set initial values
    regs.pc.set_u16(0x1000);
    regs.sp.set_u8(0xFF);
    regs.status.set_u8(0x00);

    // Execute break_instr
    let mut step = 0;
    while super::break_instr(step, &mut regs, &mut memory) == ClockEndStatus::Continue {
        step += 1;
    }

    let pushed_status_byte = memory[0x01FD];
    let pushed_status_register: StatusReg = pushed_status_byte.into();

    // Verify the results
    assert_eq!(regs.sp.to_u8(), 0xFC); // Stack pointer should be decremented 3 times
    assert_eq!(regs.status.to_u8(), StatusRegFlags::IRQ_DISABLE.bits()); // Status register should have IRQ_DISABLE flag set
    assert_eq!(pushed_status_register.to_u8(), StatusRegFlags::BREAK.bits()); // Status register in stack should have BREAK flag set
    assert_eq!(regs.pc.to_u16(), 0x0000); // Program counter should be set to the interrupt vector address
    assert_eq!(step, 5); // Ensure the step value is correct
}

#[test]
fn execute_start_irq() {
    let mut regs = RegisterFile::default();
    let mut memory = new_basic_ram();

    // Set initial values
    regs.pc.set_u16(0x1000);
    regs.sp.set_u8(0xFF);
    regs.status.set_u8(0x00);

    // Set interrupt vector values in memory
    memory[InterruptVector::Interrupt.addr(InterruptVectorAddrBytePos::Low) as usize] = 0x34;
    memory[InterruptVector::Interrupt.addr(InterruptVectorAddrBytePos::High) as usize] = 0x12;

    // Execute start_irq
    let mut step = 0;
    while execute(Instruction::StartIrq, step, &mut regs, &mut memory) == ClockEndStatus::Continue {
        step += 1;
    }

    // Verify the results
    assert_eq!(regs.pc.to_u16(), 0x1234);
    assert_eq!(regs.sp.to_u8(), 0xFC); // Stack pointer should be decremented 3 times
    assert!(regs.status.are_all_flags_set(StatusRegFlags::IRQ_DISABLE)); // IRQ_DISABLE flag should be set
    assert_eq!(step, 6);
}

#[test]
fn execute_start_nmi() {
    let mut regs = RegisterFile::default();
    let mut memory = new_basic_ram();

    // Set initial values
    regs.pc.set_u16(0x1000);
    regs.sp.set_u8(0xFF);
    regs.status.set_u8(0x00);

    // Set NMI vector values in memory
    let nmi_addr_low =
        InterruptVector::NonMaskableInterrupt.addr(InterruptVectorAddrBytePos::Low) as usize;
    let nmi_addr_high =
        InterruptVector::NonMaskableInterrupt.addr(InterruptVectorAddrBytePos::High) as usize;
    memory[nmi_addr_low] = 0x34;
    memory[nmi_addr_high] = 0x12;

    // Execute start_nmi
    let mut step = 0;
    while execute(Instruction::StartNmi, step, &mut regs, &mut memory) == ClockEndStatus::Continue {
        step += 1;
    }

    // Verify the results
    assert_eq!(regs.pc.to_u16(), 0x1234);
    assert_eq!(regs.sp.to_u8(), 0xFC); // Stack pointer should be decremented 3 times
    assert_eq!(step, 6);
}

#[test]
fn execute_return_interrupt() {
    let mut regs = RegisterFile::default();
    let mut memory = new_basic_ram();

    // Set initial values
    regs.pc.set_u16(0x1000);
    regs.sp.set_u8(0xFC);
    regs.status.set_u8(0x00);

    // Set stack values in memory
    memory[0x01FD] = StatusRegFlags::CARRY.bits(); // Status
    memory[0x01FE] = 0x34; // PCL
    memory[0x01FF] = 0x12; // PCH

    // Execute return_interrupt
    let mut step = 0;
    while execute(Instruction::ReturnInterrupt, step, &mut regs, &mut memory)
        == ClockEndStatus::Continue
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
    let mut memory = new_basic_ram();

    // Set initial values
    regs.pc.set_u16(0x1000);
    regs.sp.set_u8(0xFF);

    // Set subroutine address in memory
    memory[0x1000] = 0x34; // Low byte of subroutine address
    memory[0x1001] = 0x12; // High byte of subroutine address

    // Execute jump_subroutine
    let mut step = 0;
    while execute(Instruction::JumpSubroutine, step, &mut regs, &mut memory)
        == ClockEndStatus::Continue
    {
        step += 1;
    }

    // Verify the results
    assert_eq!(regs.pc.to_u16(), 0x1234);
    assert_eq!(regs.sp.to_u8(), 0xFD); // Stack pointer should be decremented 2 times
    assert_eq!(memory[0x01FF], 0x10); // High byte of return address should be on stack
    assert_eq!(memory[0x01FE], 0x01); // Low byte of return address should be on stack
    assert_eq!(step, 4);
}

#[test]
fn execute_return_subroutine() {
    let mut regs = RegisterFile::default();
    let mut memory = new_basic_ram();

    // Set initial values
    regs.pc.set_u16(0x1000);
    regs.sp.set_u8(0xFC);

    // Set stack values in memory
    memory[0x01FD] = 0x34; // PCL
    memory[0x01FE] = 0x12; // PCH

    // Execute return_subroutine
    let mut step = 0;
    while execute(Instruction::ReturnSubroutine, step, &mut regs, &mut memory)
        == ClockEndStatus::Continue
    {
        step += 1;
    }

    // Verify the results
    assert_eq!(regs.pc.to_u16(), 0x1235); // PC should be set to the return address + 1
    assert_eq!(regs.sp.to_u8(), 0xFE); // Stack pointer should be incremented 2 times
    assert_eq!(step, 4);
}

#[test]
fn execute_absolute_jump() {
    let mut regs = RegisterFile::default();
    let mut memory = new_basic_ram();

    // Set initial values
    regs.pc.set_u16(0x1000);
    memory[0x1000] = 0x34; // Low byte of jump address
    memory[0x1001] = 0x12; // High byte of jump address

    // Execute absolute_jump
    let mut step = 0;
    while execute(Instruction::AbsoluteJump, step, &mut regs, &mut memory)
        == ClockEndStatus::Continue
    {
        step += 1;
    }

    // Verify the results
    assert_eq!(regs.pc.to_u16(), 0x1234); // PC should be set to the jump address
    assert_eq!(step, 1); // Ensure the step value is correct
}

fn execute_relative_branch(
    mode: Instruction,
    regs: &mut RegisterFile,
    memory: &mut impl MemorySpace,
) -> (ClockEndStatus, u8) {
    let mut step = 0;
    let mut status;

    loop {
        status = execute(mode, step, regs, memory);
        if status == ClockEndStatus::Continue {
            step += 1;
        } else {
            break;
        }
    }

    (status, step)
}

#[test]
fn execute_relative_branch_taken() {
    let mut regs = RegisterFile::default();
    let mut memory = new_basic_ram();

    // Set initial values
    regs.pc.set_u16(0x1000);
    memory[0x1000] = 0x02; // Branch offset

    // Execute relative operation (BranchNotEqual)
    let (status, step) = execute_relative_branch(
        Instruction::Relative(BranchOperation::BranchNotEqual),
        &mut regs,
        &mut memory,
    );

    // Verify the results
    assert_eq!(regs.pc.to_u16(), 0x1003); // PC should be incremented by 2 (offset) + 1 (next instruction)
    assert_eq!(step, 1);
    assert_eq!(status, ClockEndStatus::EndInstruction);
}

#[test]
fn execute_relative_branch_not_taken() {
    let mut regs = RegisterFile::default();
    let mut memory = new_basic_ram();

    // Set initial values
    regs.pc.set_u16(0x1000);
    memory[0x1000] = 0x02; // Branch offset
    regs.status.set_flags(StatusRegFlags::ZERO); // Set ZERO flag to prevent branch

    // Execute relative operation (BranchNotEqual)
    let (status, step) = execute_relative_branch(
        Instruction::Relative(BranchOperation::BranchNotEqual),
        &mut regs,
        &mut memory,
    );

    // Verify the results
    assert_eq!(regs.pc.to_u16(), 0x1002); // PC should be incremented by 2 (next instruction already fetched)
    assert_eq!(step, 1);
    assert_eq!(
        status,
        ClockEndStatus::EndInstructionNextFetched {
            opcode_addr: 0x1001
        }
    ); // Status should indicate the next instruction was fetched
}

#[test]
fn execute_relative_branch_taken_page_boundary() {
    let mut regs = RegisterFile::default();
    let mut memory = new_basic_ram();

    // Set initial values
    regs.pc.set_u16(0x10F0);
    memory[0x10F0] = 0x10; // Branch offset

    // Execute relative operation (BranchNotEqual)
    let (status, step) = execute_relative_branch(
        Instruction::Relative(BranchOperation::BranchNotEqual),
        &mut regs,
        &mut memory,
    );

    // Verify the results
    assert_eq!(regs.pc.to_u16(), 0x1101); // PC should be incremented by 2 (offset) + 1 (next instruction)
    assert_eq!(step, 2); // Ensure the step value is correct for the page boundary crossing
    assert_eq!(status, ClockEndStatus::EndInstruction);
}

#[test]
fn execute_relative_branch_taken_negative_offset() {
    let mut regs = RegisterFile::default();
    let mut memory = new_basic_ram();

    // Set initial values
    regs.pc.set_u16(0x1001);
    memory[0x1001] = 0xFE; // Branch offset (-2 in two's complement)

    // Execute relative operation (BranchNotEqual)
    let (status, step) = execute_relative_branch(
        Instruction::Relative(BranchOperation::BranchNotEqual),
        &mut regs,
        &mut memory,
    );

    // Verify the results
    assert_eq!(regs.pc.to_u16(), 0x1000); // PC should be decremented by 1 [-2 (offset) + 1 (next instruction)]
    assert_eq!(step, 1);
    assert_eq!(status, ClockEndStatus::EndInstruction);
}

#[test]
fn execute_absolute_indirect_jump() {
    let mut regs = RegisterFile::default();
    let mut memory = new_basic_ram();

    // Set initial values
    regs.pc.set_u16(0x1000);
    memory[0x1000] = 0x34; // Low byte of indirect address
    memory[0x1001] = 0x12; // High byte of indirect address
    memory[0x1234] = 0x78; // Low byte of jump target
    memory[0x1235] = 0x56; // High byte of jump target

    // Execute absolute indirect jump
    let mut step = 0;
    while execute(
        Instruction::AbsoluteIndirectJump,
        step,
        &mut regs,
        &mut memory,
    ) == ClockEndStatus::Continue
    {
        step += 1;
    }

    // Verify the results
    assert_eq!(regs.pc.to_u16(), 0x5678); // PC should be set to the jump target
    assert_eq!(step, 3); // Ensure the step value is correct
}

#[test]
fn execute_absolute_indirect_jump_page_boundary_bug() {
    let mut regs = RegisterFile::default();
    let mut memory = new_basic_ram();

    // Set initial values
    regs.pc.set_u16(0x1000);
    memory[0x1000] = 0xFF; // Low byte of indirect address
    memory[0x1001] = 0x12; // High byte of indirect address
    memory[0x12FF] = 0x78; // Low byte of jump target
    memory[0x1200] = 0x56; // High byte of jump target (6502 bug: wraps to 0x1200 instead of 0x1300)

    // Execute absolute indirect jump
    let mut step = 0;
    while execute(
        Instruction::AbsoluteIndirectJump,
        step,
        &mut regs,
        &mut memory,
    ) == ClockEndStatus::Continue
    {
        step += 1;
    }

    // Verify the results
    assert_eq!(regs.pc.to_u16(), 0x5678); // PC should be set to the jump target
    assert_eq!(step, 3); // Ensure the step value is correct
}
