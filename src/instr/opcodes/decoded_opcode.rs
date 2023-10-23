use crate::instr;
use crate::registers::IndexRegister;

#[derive(Clone, Copy, Default, Debug, Eq, PartialEq)]
pub struct DecodedOpcode {
    pub sequence: instr::InstructionSequenceMode,
    pub operation: instr::InstructionOp,
    pub index: Option<IndexRegister>,
}

impl DecodedOpcode {
    #[cfg(feature = "disassembly")]
    pub fn new(
        sequence: instr::InstructionSequenceMode,
        operation: instr::InstructionOp,
        index: Option<IndexRegister>,
    ) -> Self {
        Self {
            sequence,
            operation,
            index,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::instr;
    use crate::instr::opcodes::*;

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

            if decoded.sequence != instr::InstructionSequenceMode::Relative
                && (decoded.operation != instr::InstructionOp::Nop)
                || matches!(
                    decoded.sequence,
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

            if decoded.sequence == instr::InstructionSequenceMode::Relative {
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

            if matches!(
                decoded.sequence,
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
