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
//! - `gen_write_cycle_query`: Controls the implementation of write cycle detection
//!
//! # Example
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

mod cpu_impl;

#[cfg_attr(docsrs, doc(cfg(feature = "logging")))]
#[cfg(feature = "logging")]
mod logging_memory;

pub(crate) mod interrupt;
#[cfg_attr(docsrs, doc(cfg(not(feature = "gen_write_cycle_query"))))]
#[cfg(not(feature = "gen_write_cycle_query"))]
mod write_cycle_query;

pub use cpu_impl::*;
