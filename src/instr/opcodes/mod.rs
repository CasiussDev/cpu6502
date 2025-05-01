#[cfg(feature = "decode_switch")]
mod decode_switch;

#[cfg(not(feature = "decode_switch"))]
mod decode_logic;

#[cfg(feature = "decode_switch")]
pub use decode_switch::decode;

#[cfg(not(feature = "decode_switch"))]
pub use decode_logic::decode;

#[cfg(test)]
mod tests {
    use crate::instr;
    use crate::instr::opcodes::*;
    use crate::instr::InstructionSequenceMode;
    use crate::instr::destruct_instruction;

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
            let (sequence, operation, _) = destruct_instruction(decoded);
            if sequence != instr::InstructionSequenceMode::Relative
                && (operation != instr::InstructionOp::Nop)
                || matches!(
                    sequence,
                    instr::InstructionSequenceMode::AbsoluteJump
                        | instr::InstructionSequenceMode::AbsoluteIndirectJump
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
                instr::InstructionSequenceMode::Break
                    | instr::InstructionSequenceMode::ReturnSubroutine
                    | instr::InstructionSequenceMode::ReturnInterrupt
                    | instr::InstructionSequenceMode::JumpSubroutine
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
