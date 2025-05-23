//! # cpu6502 Emulator Library
//!
//! This crate provides an emulator for the 6502 CPU, including CPU core, ALU,
//! instruction set, memory, and register file implementations.
//! It is designed for use in educational, experimental, and retrocomputing projects.
//!
//! ## Main Modules
//! - `cpu`: The main CPU emulation logic.
//! - `alu`: Arithmetic and logic unit operations.
//! - `instr`: Instruction set and decoding.
//! - `memory`: Memory space abstractions.
//! - `registers`: CPU register file and register types.

#![warn(missing_docs)]

mod alu;
mod cpu;
pub mod instr;
mod memory;
mod registers;

#[cfg(feature = "gen_write_cycle_query")]
pub use crate::instr::instr_impl::execute;
#[cfg(feature = "gen_write_cycle_query")]
pub use crate::registers::RegisterFile;

#[cfg(feature = "decode_logic")]
#[macro_use]
extern crate enum_primitive_derive;

#[cfg(feature = "decode_logic")]
extern crate num_traits;

pub use cpu::Cpu;
pub use memory::memory_space::new_basic_ram;
pub use memory::MemorySpace;

#[cfg(feature = "logging")]
extern crate log;

pub use instr::opcodes::decode;
#[cfg(feature = "disassembly")]
pub use instr::InstructionOp;
