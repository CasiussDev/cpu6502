use super::execute;
use crate::cpu::interrupt::{InterruptVector, InterruptVectorAddrBytePos};
use crate::instr::instr_impl::ClockEndStatus;
use crate::instr::Instruction;
use crate::memory::memory_space::new_basic_ram;
use crate::registers::RegisterFile;

#[test]
fn fetch_instr() {
    let mut regs = RegisterFile::default();
    let mut memory = new_basic_ram();

    // Set initial values
    regs.pc.set_u16(0x1000);
    memory[0x1000] = 0xA9; // Example instruction

    // Execute fetch_instr
    let result = super::fetch_instr(0, &mut regs, &mut memory);

    // Verify the results
    assert_eq!(regs.ir.to_u8(), 0xA9); // Instruction register should be updated
    assert_eq!(
        result,
        ClockEndStatus::EndInstructionNextFetched {
            opcode_addr: 0x1000
        }
    ); // Check the result
}

#[test]
fn reset() {
    let mut regs = RegisterFile::default();
    let mut memory = new_basic_ram();

    // Set initial values
    regs.pc.set_u16(0x1234);
    regs.sp.set_u8(0xFF);
    regs.status.set_u8(0x00);

    // Set interrupt vector values in memory
    memory[InterruptVector::ProgramStart.addr(InterruptVectorAddrBytePos::Low) as usize] =
        0x78;
    memory[InterruptVector::ProgramStart.addr(InterruptVectorAddrBytePos::High) as usize] =
        0x56;

    // Execute reset
    let mut step = 0;
    while execute(Instruction::Reset, step, &mut regs, &mut memory) == ClockEndStatus::Continue {
        step += 1;
    }

    // Verify the results
    assert_eq!(regs.pc.to_u16(), 0x5678);
    assert_eq!(regs.sp.to_u8(), 0xFC); // Stack pointer should be decremented 3 times
    assert_eq!(regs.status.to_u8(), 0x00); // Status register should remain unchanged
    assert_eq!(step, 5);
}
