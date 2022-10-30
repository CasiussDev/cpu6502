mod alu;
mod cpu;
mod instr;
mod pinout;
mod registers;

#[macro_use]
extern crate enum_primitive_derive;
extern crate enum_map;
extern crate lazy_static;
extern crate num_traits;

#[cfg(feature = "logging")]
extern crate log;

#[cfg(feature = "integration_test")]
pub use cpu::Cpu;
#[cfg(feature = "integration_test")]
pub use cpu::YieldStatus;
#[cfg(feature = "integration_test")]
pub use instr::get_sequence_for_mode;
#[cfg(feature = "integration_test")]
pub use instr::opcodes::decode;
#[cfg(feature = "integration_test")]
pub use instr::InstructionOp;
#[cfg(feature = "integration_test")]
pub use instr::MicroInstruction;
