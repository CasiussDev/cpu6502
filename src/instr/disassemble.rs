//! Disassembly utilities for the 6502 CPU instructions
//!
//! This module provides functionality to convert binary machine code into human-readable
//! assembly language. It's primarily used for debugging and logging CPU execution.
//!
//! The disassembler works by reading memory at a specified address and using the
//! `disasm6502` crate to convert raw bytes into assembly language instructions.
//! It also formats CPU state information for comprehensive execution logs.
//!
//! This module is only available when the `logging` feature is enabled.

use crate::MemorySpace;
use arrayvec::ArrayString;
use std::fmt::Write;

/// Disassembles and formats a 6502 instruction with CPU state information
///
/// This function reads memory at the specified address, disassembles the instruction
/// found there, and formats it along with the current CPU state into a log string.
///
/// # Arguments
/// * `addr` - The memory address to disassemble
/// * `memory` - A mutable reference to the system memory
/// * `cpu` - A mutable reference to the CPU for accessing register state
/// * `dst` - The destination string buffer where the formatted output will be written
///
/// # Returns
/// A `std::fmt::Result` indicating whether the formatting succeeded
///
/// # Format
/// The output format is: `ADDR BYTES MNEMONIC  REGISTERS CYC:N`
///
/// Example: `C000 4C F5 C5 JMP $C5F5  A:00 X:00 Y:00 P:24 SP:FD CYC:7`
///
/// # Panics
/// Panics if the instruction cannot be decoded or if the decoded instruction list is empty
pub fn disassemble(
    addr: u16,
    memory: &mut impl MemorySpace,
    cpu: &mut crate::cpu::Cpu,
    dst: &mut ArrayString<128>,
) -> std::fmt::Result {
    // Read up to 6 bytes from the current address in memory.
    let mut assembly = [0u8; 6];
    memory.read_array(addr, &mut assembly);

    // Disassemble up to 6 bytes from the current address.
    let instructions =
        disasm6502::from_addr_array(&assembly, addr).expect("could not decode instr");
    let decoded = instructions.first().expect("empty instr vector");

    // Format the instruction log line.
    let mut instr_log = ArrayString::<32>::new();
    write!(
        &mut instr_log,
        "{:04X} {} {}",
        addr,
        decoded.as_hex_str(),
        decoded.as_str()
    )?;

    let mut regs_log = ArrayString::<32>::new();
    cpu.regs_as_log_line(&mut regs_log)?;

    // Write the instruction and CPU state to the log file.
    write!(
        dst,
        "{:<25}{} CYC:{}",
        &instr_log,
        &regs_log,
        cpu.cycle_count_since_reset()
    )
}
