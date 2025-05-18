//! Core 6502 CPU implementation module
//!
//! This module contains the implementation of the 6502 CPU core, including:
//!
//! - The main CPU structure that manages processor state and execution
//! - Interrupt handling logic for IRQ and NMI signals
//! - Optional memory space wrapper adding logging capabilities
//! - Write cycle detection
//!
//! The CPU emulation is cycle-accurate, meaning that it correctly models
//! the timing and behavior of the original 6502 processor, including its
//! multi-cycle instruction execution pattern.
//!
//! # Features
//!
//! The module supports several compile-time features:
//!
//! - `logging`: Enables memory access logging for debugging
//! - `disassembly`: Adds support for disassembling instructions during execution
//! - `gen_write_cycle_query`: Controls the implementation of write cycle detection
//!
//! # Example
//!
//! ```
//! use cpu6502::{Cpu, MemorySpace};
//!
//! // Create a CPU instance
//! let mut cpu = Cpu::new();
//!
//! // Set up memory and execute one CPU cycle.
//! // let mut memory = ...
//! // cpu.run(&mut memory);
//! ```

mod cpu_impl;

#[cfg(feature = "logging")]
mod logging_memory;

pub(crate) mod interrupt;
#[cfg(not(feature = "gen_write_cycle_query"))]
mod write_cycle_query;

pub use cpu_impl::*;
