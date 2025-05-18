//! CPU Register implementation for the 6502 processor
//!
//! This module provides register abstractions needed to emulate the 6502 CPU, including:
//!
//! - General-purpose 8-bit registers (A, X, Y)
//! - 16-bit registers (PC, SP)
//! - The processor status register (P) with individual flag management
//! - A complete register file that organizes all CPU registers
//!
//! The 6502 has a relatively simple register set:
//! - Accumulator (A): Main register for arithmetic and logical operations
//! - Index registers (X, Y): Used for indexed addressing and loops
//! - Program Counter (PC): Points to the next instruction to execute
//! - Stack Pointer (SP): Points to the top of the stack (located in page 1)
//! - Processor Status (P): Contains status flags (C, Z, I, D, B, V, N)
//!
//! This module abstracts all register operations including setting, getting,
//! incrementing, decrementing, and managing register state during emulation.

mod reg16;
mod reg8;

pub mod register_file;
pub mod status_register;

pub use reg16::Reg16;
pub use reg8::Reg8;
pub use register_file::*;
pub use status_register::*;
