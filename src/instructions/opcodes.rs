use crate::instructions::InstructionOp;
use crate::instructions::InstructionSequenceMode;
use crate::registers::SelectedRegister;
use num_traits::FromPrimitive;

const OPCODE_GROUP_MASK: u8 = 0b_0000_0011;
const OPCODE_G1_OP_MASK: u8 = 0b_1110_0011;
const OPCODE_G1_ADDR_MASK: u8 = 0b_0001_1100;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Primitive)]
pub enum OpsG01 {
    ORA = 0x01,
    AND = 0x21,
    EOR = 0x41,
    ADC = 0x61,
    STA = 0x81,
    LDA = 0xA1,
    CMP = 0xC1,
    SBC = 0xE1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Primitive)]
pub enum AddrModeG01 {
    ZeroPageIndxIndirect = 0x00,
    ZeroPage = 0x04,
    Immediate = 0x08,
    Absolute = 0x0C,
    ZeroPageIndirectIdx = 0x10,
    ZeroPageIdx = 0x14,
    AbsoluteIdxY = 0x18,
    AbsoluteIdxX = 0x1C,
}

#[derive(Clone, Copy, Default, Debug, Eq, PartialEq)]
pub struct DecodedOpcode {
    pub sequence: InstructionSequenceMode,
    pub operation: InstructionOp,
    pub index: Option<SelectedRegister>,
}

impl DecodedOpcode {
    pub fn new(
        sequence: InstructionSequenceMode,
        operation: InstructionOp,
        index: Option<SelectedRegister>,
    ) -> Self {
        Self {
            sequence,
            operation,
            index,
        }
    }
}

fn instr_op_g1(op: OpsG01) -> InstructionOp {
    match op {
        OpsG01::ORA => InstructionOp::Or,
        OpsG01::AND => InstructionOp::And,
        OpsG01::EOR => InstructionOp::Xor,
        OpsG01::ADC => InstructionOp::Add,
        OpsG01::STA => InstructionOp::StoreA,
        OpsG01::LDA => InstructionOp::LoadA,
        OpsG01::CMP => InstructionOp::Cmp,
        OpsG01::SBC => InstructionOp::Sub,
    }
}

fn sequence_mode_g1(op: OpsG01, addr_mode: AddrModeG01) -> InstructionSequenceMode {
    match addr_mode {
        AddrModeG01::ZeroPage => InstructionSequenceMode::ZeroPage,
        AddrModeG01::Immediate => InstructionSequenceMode::Immediate,
        AddrModeG01::Absolute => InstructionSequenceMode::Absolute,
        AddrModeG01::ZeroPageIndirectIdx => InstructionSequenceMode::ZeroPageIndirectIdx,
        AddrModeG01::ZeroPageIdx => InstructionSequenceMode::ZeroPageIndx,
        AddrModeG01::AbsoluteIdxY | AddrModeG01::AbsoluteIdxX => match op {
            OpsG01::STA => InstructionSequenceMode::AbsoluteIdxWrite,
            OpsG01::ORA
            | OpsG01::AND
            | OpsG01::EOR
            | OpsG01::ADC
            | OpsG01::LDA
            | OpsG01::CMP
            | OpsG01::SBC => InstructionSequenceMode::AbsoluteIdxRead,
        },
        AddrModeG01::ZeroPageIndxIndirect => match op {
            OpsG01::STA => InstructionSequenceMode::ZeroPageIdxIndirectWrite,
            OpsG01::ORA
            | OpsG01::AND
            | OpsG01::EOR
            | OpsG01::ADC
            | OpsG01::LDA
            | OpsG01::CMP
            | OpsG01::SBC => InstructionSequenceMode::ZeroPageIdxIndirectRead,
        },
    }
}

fn index_reg_g1(addr_mode: AddrModeG01) -> Option<SelectedRegister> {
    match addr_mode {
        AddrModeG01::ZeroPageIdx
        | AddrModeG01::ZeroPageIndxIndirect
        | AddrModeG01::AbsoluteIdxX => Some(SelectedRegister::X),
        AddrModeG01::ZeroPageIndirectIdx | AddrModeG01::AbsoluteIdxY => Some(SelectedRegister::Y),
        _ => None,
    }
}

#[allow(dead_code)]
pub fn decode(opcode: u8) -> DecodedOpcode {
    match opcode & OPCODE_GROUP_MASK {
        1 => {
            let op = OpsG01::from_u8(opcode & OPCODE_G1_OP_MASK).unwrap();
            let addr_mode = AddrModeG01::from_u8(opcode & OPCODE_G1_ADDR_MASK).unwrap();
            let operation = instr_op_g1(op);
            let sequence = sequence_mode_g1(op, addr_mode);
            let index = index_reg_g1(addr_mode);
            return DecodedOpcode::new( sequence, operation, index );
        }
        _ => (),
    };

    //if let Some(op) = OpsG01::try_from(opcode & OPCODE_GROUP_MASK) {
    //    let addr_mode = AddrModeG01::
    //}

    DecodedOpcode::default()
}

#[cfg(test)]
mod tests {
    #[test]
    fn g1_print() {
        for i in 0_u8..=0b_0011_1111 {
            let opcode = (i<<2) + 1;
            let decoded = super::decode(opcode);

            println!("{:#04X}\t{:?}", opcode, decoded);
        }
    }
}
