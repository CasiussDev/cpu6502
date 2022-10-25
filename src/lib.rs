mod alu;
mod cpu;
mod instr;
mod pinout;
mod registers;

#[macro_use]
extern crate enum_primitive_derive;
extern crate lazy_static;
extern crate num_traits;

//#[cfg(test)]
pub use cpu::Cpu;
//#[cfg(test)]
pub use cpu::YieldStatus;
//#[cfg(test)]
pub use instr::get_sequences_map;
//#[cfg(test)]
pub use instr::opcodes::decode;
//#[cfg(test)]
pub use instr::InstructionOp;
//#[cfg(test)]
pub use instr::MicroInstruction;
