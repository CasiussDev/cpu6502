//! Module responsible for decoding 6502 CPU opcodes into instruction representations.
//!
//! This module provides functionality to decode raw 6502 instruction opcodes into their
//! corresponding instruction types and addressing modes. It supports two different
//! decoding implementations that can be selected via the "decode_logic" feature flag:
//!
//! - When "decode_logic" is enabled, it uses the decode_logic implementation
//! - When disabled, it falls back to the decode_switch implementation
//!
//! The module exposes a single `decode` function that handles converting an 8-bit opcode
//! into the appropriate instruction representation.

#[cfg(not(feature = "decode_logic"))]
mod decode_switch;

#[cfg(feature = "decode_logic")]
mod decode_logic;

#[cfg(not(feature = "decode_logic"))]
pub use decode_switch::decode;

#[cfg(feature = "decode_logic")]
pub use decode_logic::decode;

#[cfg(test)]
mod tests {
    use crate::instr;
    use crate::instr::opcodes::*;
    use crate::instr::InstructionSequenceMode;

    #[test]
    fn g1_print() {
        println!("\nGroup 1\n");
        for i in 0_u8..=0b_0011_1111 {
            let opcode = (i << 2) + 1;
            let decoded = decode(opcode);

            println!("\t{:#04X}\t{:?}", opcode, decoded);
        }
    }

    #[test]
    fn g2_print() {
        println!("\nGroup 2\n");
        for i in 0_u8..=0b_0011_1111 {
            let opcode = (i << 2) + 2;
            let decoded = decode(opcode);

            println!("\t{:#04X}\t{:?}", opcode, decoded);
        }
    }

    #[test]
    fn g3_print() {
        println!("\nGroup 3\n");
        for i in 0_u8..=0b_0011_1111 {
            let opcode = i << 2;
            let decoded = decode(opcode);
            let (sequence, operation, _) = decoded.into();
            if sequence != InstructionSequenceMode::Relative
                && (operation != instr::InstructionOp::Nop)
                || matches!(
                    sequence,
                    InstructionSequenceMode::AbsoluteJump
                        | InstructionSequenceMode::AbsoluteIndirectJump
                )
            {
                println!("\t{:#04X}\t{:?}", opcode, decoded);
            }
        }
    }

    #[test]
    fn cond_branch_print() {
        println!("\nConditional Branches\n");
        for i in 0_u8..=0b_0011_1111 {
            let opcode = i << 2;
            let decoded = decode(opcode);
            let sequence: InstructionSequenceMode = decoded.into();
            if sequence == InstructionSequenceMode::Relative {
                println!("\t{:#04X}\t{:?}", opcode, decoded);
            }
        }
    }

    #[test]
    fn subroutine_print() {
        println!("\nSubroutines and Interrupts\n");
        for i in 0_u8..=0b_0011_1111 {
            let opcode = i << 2;
            let decoded = decode(opcode);
            let sequence: InstructionSequenceMode = decoded.into();
            if matches!(
                sequence,
                InstructionSequenceMode::Break
                    | InstructionSequenceMode::ReturnSubroutine
                    | InstructionSequenceMode::ReturnInterrupt
                    | InstructionSequenceMode::JumpSubroutine
            ) {
                println!("\t{:#04X}\t{:?}", opcode, decoded);
            }
        }
    }

    #[test]
    fn allopcodes_decoding_neverpanic() {
        for opcode in 0..=u8::MAX {
            decode(opcode);
        }
    }
}
