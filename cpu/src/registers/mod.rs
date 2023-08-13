mod reg16;
mod reg8;

pub mod register_file;
pub mod status_register;

pub use reg16::Reg16;
pub use reg8::Reg8;
pub use register_file::*;
pub use status_register::*;
