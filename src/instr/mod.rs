pub mod instr_impl;
pub mod instr_operation;
pub mod instr_sequences;
pub mod m_instr;
pub mod opcodes;

#[cfg(feature = "gen_write_cycle_query")]
pub use instr_impl::*;

pub use instr_operation::*;
pub use instr_sequences::*;
pub use m_instr::*;
pub use opcodes::*;
