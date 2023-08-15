mod alu;
mod cpu;
mod instr;
mod pinout;
mod registers;
mod memory;

#[cfg(not(feature = "decode_switch"))]
#[macro_use]
extern crate enum_primitive_derive;

extern crate enum_map;
extern crate lazy_static;
extern crate num_traits;

pub use cpu::Cpu;
pub use memory::MemorySpace;

#[cfg(feature = "logging")]
extern crate log;

#[cfg(feature = "integration_test")]
pub use instr::opcodes::decode;
#[cfg(feature = "integration_test")]
pub use instr::sequence_for_mode;
#[cfg(feature = "integration_test")]
pub use instr::FetchedInstr;
#[cfg(feature = "integration_test")]
pub use instr::InstructionOp;
#[cfg(feature = "integration_test")]
pub use instr::MicroInstruction;
