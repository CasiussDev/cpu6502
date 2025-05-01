#[cfg(feature = "disassembly")]
use cpu6502::instr::destruct_instruction;

use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct OpcodeInfo {
    opcode: u8,
    mnemonics: Vec<String>,
    #[serde(rename = "addressingMode")]
    addressing_mode: Option<String>,
    cycles: Option<u8>,
    #[serde(rename = "pageBoundaryCycle")]
    page_boundary_cycle: Option<bool>,
    illegal: bool,
}
