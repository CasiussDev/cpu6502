mod opcodes;

#[cfg(feature = "decode_switch")]
mod decode_switch;

#[cfg(not(feature = "decode_switch"))]
mod decode_logic;

#[cfg(feature = "decode_switch")]
pub use decode_switch::decode;

#[cfg(not(feature = "decode_switch"))]
pub use decode_logic::decode;

use opcodes::*;