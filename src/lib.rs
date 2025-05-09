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

pub use cpu::Cpu;
pub use memory::MemorySpace;

#[cfg(feature = "logging")]
extern crate log;

#[cfg(feature = "disassembly")]
pub use instr::opcodes::decode;
#[cfg(feature = "disassembly")]
pub use instr::InstructionOp;
