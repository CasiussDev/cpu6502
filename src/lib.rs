//! # cpu6502 Emulator Library
//!
//! This crate provides an emulator for the 6502 CPU, including CPU core, ALU,
//! instruction set, memory, and register file implementations.
//! It is designed for use in educational, experimental, and retrocomputing projects.
//!
//! ## Quick Start
//!
//! ```
//! use cpu6502::{Cpu, new_basic_ram};
//!
//! let mut cpu = Cpu::new();
//! let mut memory = new_basic_ram();
//!
//! // Init memory here (including code to execute and interrupt/reset vectors)
//!
//! // Run the CPU for 1000 cycles
//! for _ in 0..1000 {
//!     cpu.run(&mut memory);
//! }
//!
//! // Check CPU state
//! let cycles = cpu.cycle_count_since_reset();
//! let instructions = cpu.instr_count_since_reset();
//! println!("Executed {} cycles across {} instructions", cycles, instructions);
//! ```
//!
//! ## Main Modules
//! - `cpu`: The main CPU emulation logic.
//! - `alu`: Arithmetic and logic unit operations.
//! - `instr`: Instruction set and decoding.
//! - `memory`: Memory space abstractions.
//! - `registers`: CPU register file and register types.
//!
//! ## Feature Flags
//! - `logging`: Enables instruction disassembly and execution logging.
//! - `decimal`: Enables decimal mode arithmetic *(not yet implemented)*.
//! - `undoc_opcodes`: Enables support for undocumented opcodes *(not yet implemented)*.
//! - `decode_logic`: Provides the canonical instruction decoder implementation.
//! - `gen_write_cycle_query`: Enables generation of write cycle detection logic.

#![no_std]
#![warn(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(any(feature = "std", test))]
#[macro_use]
extern crate std;

mod alu;
mod cpu;
pub mod instr;
mod memory;
mod registers;

pub use alu::*;
pub use cpu::{interrupt::InterruptType, Cpu};
pub use instr::{instr_operation::*, instr_sequences::Instruction, opcodes::decode, InstructionOp};
pub use memory::{memory_space::new_basic_ram, MemorySpace};
pub use registers::{IndexRegister, RegisterFile, StatusReg, StatusRegFlags};
