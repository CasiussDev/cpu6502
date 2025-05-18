//! Instruction processing and execution for the 6502 CPU
//!
//! This module contains the complete implementation of the 6502 instruction set,
//! including:
//!
//! - Instruction operation definitions (what each instruction does)
//! - Instruction execution sequences (how instructions are executed over multiple cycles)
//! - Opcode mapping and decoding (translating binary opcodes to executable instructions)
//! - Instruction implementation (the cycle-by-cycle execution logic)
//!
//! ## Module Structure
//!
//! - `instr_operation`: Defines the operations performed by instructions (ADD, AND, etc.)
//! - `instr_sequences`: Defines the instruction execution sequences for different addressing modes
//! - `opcodes`: Maps binary opcodes to instruction operations and addressing modes
//! - `instr_impl`: Implements the cycle-by-cycle execution of all instructions
//!
//! The design separates the operation (what is done) from the addressing mode (how operands
//! are accessed), which matches how the actual 6502 architecture works.

pub mod instr_impl;
pub mod instr_operation;
pub mod instr_sequences;
pub mod opcodes;

#[cfg(feature = "gen_write_cycle_query")]
pub use instr_impl::*;

pub use instr_operation::*;
pub use instr_sequences::*;
pub use opcodes::*;
