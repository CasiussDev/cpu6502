use crate::instr;
use crate::instr::{InstructionOp, InstructionSequenceMode, InstructionSequenceMode2};
use crate::registers::IndexRegister;

#[derive(Clone, Copy, Default, Debug, Eq, PartialEq)]
pub struct DecodedOpcode {
    pub sequence: instr::InstructionSequenceMode2,
    pub operation: instr::InstructionOp,
    pub index: Option<IndexRegister>,
}

impl DecodedOpcode {
    #[cfg(feature = "disassembly")]
    pub fn new(
        sequence: instr::InstructionSequenceMode2,
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

pub fn destruct_sequence(sequence: InstructionSequenceMode2)
                     -> (InstructionSequenceMode, InstructionOp, IndexRegister) {
    match sequence {
        InstructionSequenceMode2::Reset => (
            InstructionSequenceMode::Reset,
            InstructionOp::default(),
            IndexRegister::default(),
        ),
        InstructionSequenceMode2::FetchInstr => (
            InstructionSequenceMode::FetchInstr,
            InstructionOp::default(),
            IndexRegister::default(),
        ),
        InstructionSequenceMode2::StartNmi => (
            InstructionSequenceMode::StartNmi,
            InstructionOp::default(),
            IndexRegister::default(),
        ),
        InstructionSequenceMode2::StartIrq => (
            InstructionSequenceMode::StartIrq,
            InstructionOp::default(),
            IndexRegister::default(),
        ),
        InstructionSequenceMode2::Break => (
            InstructionSequenceMode::Break,
            InstructionOp::default(),
            IndexRegister::default(),
        ),
        InstructionSequenceMode2::ReturnInterrupt => (
            InstructionSequenceMode::ReturnInterrupt,
            InstructionOp::default(),
            IndexRegister::default(),
        ),
        InstructionSequenceMode2::JumpSubroutine => (
            InstructionSequenceMode::JumpSubroutine,
            InstructionOp::default(),
            IndexRegister::default(),
        ),
        InstructionSequenceMode2::ReturnSubroutine => (
            InstructionSequenceMode::ReturnSubroutine,
            InstructionOp::default(),
            IndexRegister::default(),
        ),
        InstructionSequenceMode2::Push(op) => (
            InstructionSequenceMode::Push,
            InstructionOp::from(op),
            IndexRegister::default(),
        ),
        InstructionSequenceMode2::Pull(op) => (
            InstructionSequenceMode::Pull,
            InstructionOp::from(op),
            IndexRegister::default(),
        ),
        InstructionSequenceMode2::Implied(op) => (
            InstructionSequenceMode::Implied,
            InstructionOp::from(op),
            IndexRegister::default(),
        ),
        InstructionSequenceMode2::Immediate(op) => (
            InstructionSequenceMode::Immediate,
            InstructionOp::from(op),
            IndexRegister::default(),
        ),
        InstructionSequenceMode2::AbsoluteJump => (
            InstructionSequenceMode::AbsoluteJump,
            InstructionOp::default(),
            IndexRegister::default(),
        ),
        InstructionSequenceMode2::Absolute(op) => (
            InstructionSequenceMode::Absolute,
            InstructionOp::from(op),
            IndexRegister::default(),
        ),
        InstructionSequenceMode2::AbsoluteReadModifyWrite(op) => (
            InstructionSequenceMode::AbsoluteReadModifyWrite,
            InstructionOp::from(op),
            IndexRegister::default(),
        ),
        InstructionSequenceMode2::ZeroPage(op) => (
            InstructionSequenceMode::ZeroPage,
            InstructionOp::from(op),
            IndexRegister::default(),
        ),
        InstructionSequenceMode2::ZeroPageReadModifyWrite(op) => (
            InstructionSequenceMode::ZeroPageReadModifyWrite,
            InstructionOp::from(op),
            IndexRegister::default(),
        ),
        InstructionSequenceMode2::ZeroPageIdx(op, idx) => (
            InstructionSequenceMode::ZeroPageIdx,
            InstructionOp::from(op),
            idx,
        ),
        InstructionSequenceMode2::ZeroPageIdxReadModifyWrite(op, idx) => (
            InstructionSequenceMode::ZeroPageIdxReadModifyWrite,
            InstructionOp::from(op),
            idx,
        ),
        InstructionSequenceMode2::AbsoluteIdxRead(op, idx) => (
            InstructionSequenceMode::AbsoluteIdxRead,
            InstructionOp::from(op),
            idx,
        ),
        InstructionSequenceMode2::AbsoluteIdxReadModifyWrite(op, idx) => (
            InstructionSequenceMode::AbsoluteIdxReadModifyWrite,
            InstructionOp::from(op),
            idx,
        ),
        InstructionSequenceMode2::AbsoluteIdxWrite(op, idx) => (
            InstructionSequenceMode::AbsoluteIdxWrite,
            InstructionOp::from(op),
            idx,
        ),
        InstructionSequenceMode2::Relative(op) => (
            InstructionSequenceMode::Relative,
            InstructionOp::from(op),
            IndexRegister::default(),
        ),
        InstructionSequenceMode2::ZeroPageIdxIndirect(op, idx) => (
            InstructionSequenceMode::ZeroPageIdxIndirect,
            InstructionOp::from(op),
            idx,
        ),
        InstructionSequenceMode2::ZeroPageIdxIndirectReadModifyWrite(op, idx) => (
            InstructionSequenceMode::ZeroPageIdxIndirectReadModifyWrite,
            InstructionOp::from(op),
            idx,
        ),
        InstructionSequenceMode2::ZeroPageIndirectIdxRead(op, idx) => (
            InstructionSequenceMode::ZeroPageIndirectIdxRead,
            InstructionOp::from(op),
            idx,
        ),
        InstructionSequenceMode2::ZeroPageIndirectIdxReadModifyWrite(op, idx) => (
            InstructionSequenceMode::ZeroPageIndirectIdxReadModifyWrite,
            InstructionOp::from(op),
            idx,
        ),
        InstructionSequenceMode2::ZeroPageIndirectIdxWrite(op, idx) => (
            InstructionSequenceMode::ZeroPageIndirectIdxWrite,
            InstructionOp::from(op),
            idx,
        ),
        InstructionSequenceMode2::AbsoluteIndirectJump => (
            InstructionSequenceMode::AbsoluteIndirectJump,
            InstructionOp::default(),
            IndexRegister::default(),
        ),
    }
}

#[cfg(test)]
mod tests {
    use crate::instr;
    use crate::instr::InstructionSequenceMode;
    use crate::instr::opcodes::*;
    use crate::instr::opcodes::decoded_opcode::destruct_sequence;

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
            let (sequence, operation, _) = destruct_sequence(decoded);
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
