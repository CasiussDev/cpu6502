mod alu;
mod cpu;
pub mod instr;
mod interrupts;
mod memory;
mod registers;

#[cfg(feature = "gen_write_cycle_query")]
pub use crate::instr::instr_impl::execute;
#[cfg(feature = "gen_write_cycle_query")]
pub use crate::registers::RegisterFile;

#[cfg(not(feature = "decode_switch"))]
#[macro_use]
extern crate enum_primitive_derive;

extern crate num_traits;

pub use cpu::Cpu;
pub use memory::MemorySpace;

#[cfg(feature = "logging")]
extern crate log;

#[cfg(feature = "disassembly")]
pub use instr::opcodes::decode;
#[cfg(feature = "disassembly")]
pub use instr::sequence_for_mode;
#[cfg(feature = "disassembly")]
pub use instr::FetchedInstr;
#[cfg(feature = "disassembly")]
pub use instr::InstructionOp;
#[cfg(feature = "disassembly")]
pub use instr::MicroInstruction;
