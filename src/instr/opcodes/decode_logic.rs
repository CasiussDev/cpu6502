use super::*;
use crate::instr;
use crate::num_traits::FromPrimitive;
use crate::registers::IndexRegister;

const OPCODE_GROUP_MASK: u8 = 0b_0000_0011;

const OPCODE_G123_OP_MASK: u8 = 0b_1110_0011;
const OPCODE_G123_ADDR_MASK: u8 = 0b_0001_1100;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Primitive)]
pub enum OpsG1 {
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
pub enum AddrModeG1 {
    ZeroPageIndxIndirect = 0x00,
    ZeroPage = 0x04,
    Immediate = 0x08,
    Absolute = 0x0C,
    ZeroPageIndirectIdx = 0x10,
    ZeroPageIdx = 0x14,
    AbsoluteIdxY = 0x18,
    AbsoluteIdxX = 0x1C,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Primitive)]
pub enum OpsG2 {
    ASL = 0x02,
    ROL = 0x22,
    LSR = 0x42,
    ROR = 0x62,
    STX = 0x82,
    LDX = 0xA2,
    DEC = 0xC2,
    INC = 0xE2,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Primitive)]
pub enum AddrModeG2 {
    Immediate = 0x00,
    ZeroPage = 0x04,
    Accumulator = 0x08,
    Absolute = 0x0C,
    Unused1 = 0x10,
    ZeroPageIdx = 0x14,
    Unused2 = 0x18,
    AbsoluteIdxX = 0x1C,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Primitive)]
pub enum OpsG3 {
    Unused = 0x00,
    BIT = 0x20,
    JMP = 0x40,
    JMPInd = 0x60,
    STY = 0x80,
    LDY = 0xA0,
    CPY = 0xC0,
    CPX = 0xE0,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Primitive)]
pub enum AddrModeG3 {
    Immediate = 0x00,
    ZeroPage = 0x04,
    Unused1 = 0x08,
    Absolute = 0x0C,
    Unused2 = 0x10,
    ZeroPageIdx = 0x14,
    Unused3 = 0x18,
    AbsoluteIdxX = 0x1C,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Primitive)]
pub enum OpsCompBranch {
    BPL = 0x10,
    BMI = 0x30,
    BVC = 0x50,
    BVS = 0x70,
    BCC = 0x90,
    BCS = 0xB0,
    BNE = 0xD0,
    BEQ = 0xF0,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Primitive)]
pub enum OpsSubroutine {
    BRK = 0x00,
    JSR = 0x20,
    RTI = 0x40,
    RTS = 0x60,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Primitive)]
pub enum OpsSingleByte0 {
    PHP = 0x08,
    PLP = 0x28,
    PHA = 0x48,
    PLA = 0x68,
    DEY = 0x88,
    TAY = 0xA8,
    INY = 0xC8,
    INX = 0xE8,
    CLC = 0x18,
    SEC = 0x38,
    CLI = 0x58,
    SEI = 0x78,
    TYA = 0x98,
    CLV = 0xB8,
    CLD = 0xD8,
    SED = 0xF8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Primitive)]
pub enum OpsSingleByte2 {
    TXA = 0x8A,
    TXS = 0x9A,
    TAX = 0xAA,
    TSX = 0xBA,
    DEX = 0xCA,
    NOP = 0xEA,
}

fn instr_op_g1(op: OpsG1) -> instr::InstructionOp {
    match op {
        OpsG1::ORA => instr::InstructionOp::Or,
        OpsG1::AND => instr::InstructionOp::And,
        OpsG1::EOR => instr::InstructionOp::Xor,
        OpsG1::ADC => instr::InstructionOp::Add,
        OpsG1::STA => instr::InstructionOp::StoreA,
        OpsG1::LDA => instr::InstructionOp::LoadA,
        OpsG1::CMP => instr::InstructionOp::Cmp,
        OpsG1::SBC => instr::InstructionOp::Sub,
    }
}

fn instr_op_g2(op: OpsG2, addr_mode: AddrModeG2) -> instr::InstructionOp {
    match op {
        OpsG2::ASL => match addr_mode {
            AddrModeG2::Accumulator => instr::InstructionOp::ShiftLeftA,
            _ => instr::InstructionOp::ShiftLeftMemory,
        },
        OpsG2::ROL => match addr_mode {
            AddrModeG2::Accumulator => instr::InstructionOp::RotateLeftA,
            _ => instr::InstructionOp::RotateLeftMemory,
        },
        OpsG2::LSR => match addr_mode {
            AddrModeG2::Accumulator => instr::InstructionOp::ShiftRightA,
            _ => instr::InstructionOp::ShiftRightMemory,
        },
        OpsG2::ROR => match addr_mode {
            AddrModeG2::Accumulator => instr::InstructionOp::RotateRightA,
            _ => instr::InstructionOp::RotateRightMemory,
        },
        OpsG2::STX => instr::InstructionOp::StoreX,
        OpsG2::LDX => instr::InstructionOp::LoadX,
        OpsG2::DEC => instr::InstructionOp::DecrementMemory,
        OpsG2::INC => instr::InstructionOp::IncrementMemory,
    }
}

fn instr_op_g3(op: OpsG3, addr_mode: AddrModeG3) -> instr::InstructionOp {
    match op {
        OpsG3::Unused => instr::InstructionOp::Nop,
        OpsG3::BIT => {
            if addr_mode == AddrModeG3::Immediate {
                instr::InstructionOp::BitImmediate
            } else {
                instr::InstructionOp::Bit
            }
        }
        OpsG3::JMP => instr::InstructionOp::Nop,
        OpsG3::JMPInd => instr::InstructionOp::Nop,
        OpsG3::STY => instr::InstructionOp::StoreY,
        OpsG3::LDY => instr::InstructionOp::LoadY,
        OpsG3::CPY => instr::InstructionOp::Cpy,
        OpsG3::CPX => instr::InstructionOp::Cpx,
    }
}

fn instr_op_cond_branch(op: OpsCompBranch) -> instr::InstructionOp {
    match op {
        OpsCompBranch::BPL => instr::InstructionOp::BranchPlus,
        OpsCompBranch::BMI => instr::InstructionOp::BranchMinus,
        OpsCompBranch::BVC => instr::InstructionOp::BranchOverflowClear,
        OpsCompBranch::BVS => instr::InstructionOp::BranchOverflowSet,
        OpsCompBranch::BCC => instr::InstructionOp::BranchCarryClear,
        OpsCompBranch::BCS => instr::InstructionOp::BranchCarrySet,
        OpsCompBranch::BNE => instr::InstructionOp::BranchNotEqual,
        OpsCompBranch::BEQ => instr::InstructionOp::BranchEqual,
    }
}

fn instr_op_single_byte0(op: OpsSingleByte0) -> instr::InstructionOp {
    match op {
        OpsSingleByte0::PHP => instr::InstructionOp::PushStatus,
        OpsSingleByte0::PLP => instr::InstructionOp::PullStatus,
        OpsSingleByte0::PHA => instr::InstructionOp::PushA,
        OpsSingleByte0::PLA => instr::InstructionOp::PullA,
        OpsSingleByte0::DEY => instr::InstructionOp::DecrementY,
        OpsSingleByte0::TAY => instr::InstructionOp::TransferAccumulatorToY,
        OpsSingleByte0::INY => instr::InstructionOp::IncrementY,
        OpsSingleByte0::INX => instr::InstructionOp::IncrementX,
        OpsSingleByte0::CLC => instr::InstructionOp::ClearCarry,
        OpsSingleByte0::SEC => instr::InstructionOp::SetCarry,
        OpsSingleByte0::CLI => instr::InstructionOp::ClearInterruptDisable,
        OpsSingleByte0::SEI => instr::InstructionOp::SetInterruptDisable,
        OpsSingleByte0::TYA => instr::InstructionOp::TransferYToAccumulator,
        OpsSingleByte0::CLV => instr::InstructionOp::ClearOverflow,
        OpsSingleByte0::CLD => instr::InstructionOp::ClearDecimal,
        OpsSingleByte0::SED => instr::InstructionOp::SetDecimal,
    }
}

fn instr_op_single_byte2(op: OpsSingleByte2) -> instr::InstructionOp {
    match op {
        OpsSingleByte2::TXA => instr::InstructionOp::TransferXToAccumulator,
        OpsSingleByte2::TXS => instr::InstructionOp::TransferXToStackPtr,
        OpsSingleByte2::TAX => instr::InstructionOp::TransferAccumulatorToX,
        OpsSingleByte2::TSX => instr::InstructionOp::TransferStackPtrToX,
        OpsSingleByte2::DEX => instr::InstructionOp::DecrementX,
        OpsSingleByte2::NOP => instr::InstructionOp::Nop,
    }
}

fn sequence_mode_g1(op: OpsG1, addr_mode: AddrModeG1) -> instr::InstructionSequenceMode {
    match addr_mode {
        AddrModeG1::ZeroPage => instr::InstructionSequenceMode::ZeroPage,
        AddrModeG1::Immediate => instr::InstructionSequenceMode::Immediate,
        AddrModeG1::Absolute => instr::InstructionSequenceMode::Absolute,
        AddrModeG1::ZeroPageIndxIndirect => instr::InstructionSequenceMode::ZeroPageIdxIndirect,
        AddrModeG1::ZeroPageIdx => instr::InstructionSequenceMode::ZeroPageIdx,
        AddrModeG1::AbsoluteIdxY | AddrModeG1::AbsoluteIdxX => match op {
            OpsG1::STA => instr::InstructionSequenceMode::AbsoluteIdxWrite,
            OpsG1::ORA
            | OpsG1::AND
            | OpsG1::EOR
            | OpsG1::ADC
            | OpsG1::LDA
            | OpsG1::CMP
            | OpsG1::SBC => instr::InstructionSequenceMode::AbsoluteIdxRead,
        },
        AddrModeG1::ZeroPageIndirectIdx => match op {
            OpsG1::STA => instr::InstructionSequenceMode::ZeroPageIndirectIdxWrite,
            OpsG1::ORA
            | OpsG1::AND
            | OpsG1::EOR
            | OpsG1::ADC
            | OpsG1::LDA
            | OpsG1::CMP
            | OpsG1::SBC => instr::InstructionSequenceMode::ZeroPageIndirectIdxRead,
        },
    }
}

fn sequence_mode_g2(op: OpsG2, addr_mode: AddrModeG2) -> instr::InstructionSequenceMode {
    match op {
        OpsG2::ASL | OpsG2::ROL | OpsG2::LSR | OpsG2::ROR | OpsG2::DEC | OpsG2::INC => {
            match addr_mode {
                AddrModeG2::ZeroPage => instr::InstructionSequenceMode::ZeroPageReadModifyWrite,
                AddrModeG2::Immediate => instr::InstructionSequenceMode::Immediate, // Illegal
                AddrModeG2::Absolute => instr::InstructionSequenceMode::AbsoluteReadModifyWrite,
                AddrModeG2::ZeroPageIdx => {
                    instr::InstructionSequenceMode::ZeroPageIdxReadModifyWrite
                }
                AddrModeG2::AbsoluteIdxX => {
                    instr::InstructionSequenceMode::AbsoluteIdxReadModifyWrite
                }
                AddrModeG2::Accumulator => instr::InstructionSequenceMode::Implied, // Illegal for Inc and Dec
                AddrModeG2::Unused1 | AddrModeG2::Unused2 => {
                    instr::InstructionSequenceMode::default()
                } // Illegal
            }
        }
        OpsG2::STX | OpsG2::LDX => {
            match addr_mode {
                AddrModeG2::Immediate => instr::InstructionSequenceMode::Immediate, // Illegal for STX
                AddrModeG2::ZeroPage => instr::InstructionSequenceMode::ZeroPage,
                AddrModeG2::Accumulator => instr::InstructionSequenceMode::Implied, // Illegal
                AddrModeG2::Absolute => instr::InstructionSequenceMode::Absolute,
                AddrModeG2::ZeroPageIdx => instr::InstructionSequenceMode::ZeroPageIdx,
                AddrModeG2::AbsoluteIdxX => {
                    if op == OpsG2::STX {
                        instr::InstructionSequenceMode::AbsoluteIdxWrite // Illegal
                    } else {
                        instr::InstructionSequenceMode::AbsoluteIdxRead
                    }
                }
                AddrModeG2::Unused1 | AddrModeG2::Unused2 => {
                    instr::InstructionSequenceMode::default()
                } // Illegal
            }
        }
    }
}

fn sequence_mode_g3(op: OpsG3, addr_mode: AddrModeG3) -> instr::InstructionSequenceMode {
    match op {
        OpsG3::JMP => instr::InstructionSequenceMode::AbsoluteJump,
        OpsG3::JMPInd => instr::InstructionSequenceMode::AbsoluteIndirectJump,
        _ => match addr_mode {
            AddrModeG3::Immediate => instr::InstructionSequenceMode::Immediate, // Illegal for BIT, Jumps and STY
            AddrModeG3::ZeroPage => instr::InstructionSequenceMode::ZeroPage,   // Illegal for Jumps
            AddrModeG3::Unused1 | AddrModeG3::Unused2 | AddrModeG3::Unused3 => {
                instr::InstructionSequenceMode::default()
            } // Illegal
            AddrModeG3::Absolute => instr::InstructionSequenceMode::Absolute,
            AddrModeG3::ZeroPageIdx => instr::InstructionSequenceMode::ZeroPageIdx, // Illegal for BIT, Jumps and Cp
            AddrModeG3::AbsoluteIdxX => instr::InstructionSequenceMode::AbsoluteIdxRead, // Only legal for LDY
        },
    }
}

fn sequence_mode_subroutine(op: OpsSubroutine) -> instr::InstructionSequenceMode {
    match op {
        OpsSubroutine::BRK => instr::InstructionSequenceMode::Break,
        OpsSubroutine::JSR => instr::InstructionSequenceMode::JumpSubroutine,
        OpsSubroutine::RTI => instr::InstructionSequenceMode::ReturnInterrupt,
        OpsSubroutine::RTS => instr::InstructionSequenceMode::ReturnSubroutine,
    }
}

fn sequence_mode_single_byte0(op: OpsSingleByte0) -> instr::InstructionSequenceMode {
    match op {
        OpsSingleByte0::PHP | OpsSingleByte0::PHA => instr::InstructionSequenceMode::Push,
        OpsSingleByte0::PLP | OpsSingleByte0::PLA => instr::InstructionSequenceMode::Pull,
        _ => instr::InstructionSequenceMode::Implied,
    }
}

fn index_reg_g1(addr_mode: AddrModeG1) -> Option<IndexRegister> {
    match addr_mode {
        AddrModeG1::ZeroPageIdx | AddrModeG1::ZeroPageIndxIndirect | AddrModeG1::AbsoluteIdxX => {
            Some(IndexRegister::X)
        }
        AddrModeG1::ZeroPageIndirectIdx | AddrModeG1::AbsoluteIdxY => Some(IndexRegister::Y),
        _ => None,
    }
}

fn index_reg_g2(op: OpsG2, addr_mode: AddrModeG2) -> Option<IndexRegister> {
    match addr_mode {
        AddrModeG2::ZeroPageIdx => {
            if matches!(op, OpsG2::STX | OpsG2::LDX) {
                Some(IndexRegister::Y)
            } else {
                Some(IndexRegister::X)
            }
        }
        AddrModeG2::AbsoluteIdxX => {
            if op == OpsG2::LDX {
                Some(IndexRegister::Y)
            } else {
                Some(IndexRegister::X)
            }
        }
        _ => None,
    }
}

fn index_reg_g3(addr_mode: AddrModeG3) -> Option<IndexRegister> {
    if matches!(
        addr_mode,
        AddrModeG3::ZeroPageIdx | AddrModeG3::AbsoluteIdxX
    ) {
        Some(IndexRegister::X)
    } else {
        None
    }
}

fn illegal_instruction_g1(op: OpsG1, addr_mode: AddrModeG1) -> bool {
    (op == OpsG1::STA) && (addr_mode == AddrModeG1::Immediate)
}

fn illegal_instruction_g2(op: OpsG2, addr_mode: AddrModeG2) -> bool {
    ((op == OpsG2::STX) && (addr_mode == AddrModeG2::AbsoluteIdxX))
        || ((op != OpsG2::LDX) && (addr_mode == AddrModeG2::Immediate))
        || (matches!(op, OpsG2::STX | OpsG2::LDX | OpsG2::DEC | OpsG2::INC)
            && (addr_mode == AddrModeG2::Accumulator))
        || matches!(addr_mode, AddrModeG2::Unused1 | AddrModeG2::Unused2)
}

fn illegal_instruction_g3(op: OpsG3, addr_mode: AddrModeG3) -> bool {
    (op == OpsG3::Unused)
        || matches!(
            addr_mode,
            AddrModeG3::Unused1 | AddrModeG3::Unused2 | AddrModeG3::Unused3
        )
        || ((addr_mode == AddrModeG3::AbsoluteIdxX) && (op != OpsG3::LDY))
        || ((addr_mode == AddrModeG3::Immediate)
            && (matches!(op, OpsG3::BIT | OpsG3::JMP | OpsG3::JMPInd | OpsG3::STY)))
        || ((addr_mode == AddrModeG3::ZeroPage) && matches!(op, OpsG3::JMP | OpsG3::JMPInd))
        || ((addr_mode == AddrModeG3::ZeroPageIdx) && !matches!(op, OpsG3::STY | OpsG3::LDY))
}

pub fn decode(opcode: u8) -> DecodedOpcode {
    let mut decoded_opcode = DecodedOpcode::default();
    match opcode & OPCODE_GROUP_MASK {
        1 => {
            let op = OpsG1::from_u8(opcode & OPCODE_G123_OP_MASK).unwrap();
            let addr_mode = AddrModeG1::from_u8(opcode & OPCODE_G123_ADDR_MASK).unwrap();
            if illegal_instruction_g1(op, addr_mode) == false {
                let operation = instr_op_g1(op);
                let sequence = sequence_mode_g1(op, addr_mode);
                let index = index_reg_g1(addr_mode);
                decoded_opcode = DecodedOpcode::new(sequence, operation, index);
            } else if cfg!(feature = "undoc_opcodes") {
                todo!();
            } else {
                unimplemented!();
            }
        }
        2 => {
            let op = OpsG2::from_u8(opcode & OPCODE_G123_OP_MASK).unwrap();
            let addr_mode = AddrModeG2::from_u8(opcode & OPCODE_G123_ADDR_MASK).unwrap();
            if illegal_instruction_g2(op, addr_mode) == false {
                let operation = instr_op_g2(op, addr_mode);
                let sequence = sequence_mode_g2(op, addr_mode);
                let index = index_reg_g2(op, addr_mode);
                decoded_opcode = DecodedOpcode::new(sequence, operation, index);
            } else if let Some(op) = OpsSingleByte2::from_u8(opcode) {
                let operation = instr_op_single_byte2(op);
                decoded_opcode =
                    DecodedOpcode::new(instr::InstructionSequenceMode::Implied, operation, None);
            } else if cfg!(feature = "undoc_opcodes") {
                todo!();
            } else {
                unimplemented!()
            }
        }
        0 => {
            let op = OpsG3::from_u8(opcode & OPCODE_G123_OP_MASK).unwrap();
            let addr_mode = AddrModeG3::from_u8(opcode & OPCODE_G123_ADDR_MASK).unwrap();
            if illegal_instruction_g3(op, addr_mode) == false {
                let operation = instr_op_g3(op, addr_mode);
                let sequence = sequence_mode_g3(op, addr_mode);
                let index = index_reg_g3(addr_mode);
                decoded_opcode = DecodedOpcode::new(sequence, operation, index);
            } else if let Some(op) = OpsCompBranch::from_u8(opcode) {
                let operation = instr_op_cond_branch(op);
                decoded_opcode =
                    DecodedOpcode::new(instr::InstructionSequenceMode::Relative, operation, None);
            } else if let Some(op) = OpsSubroutine::from_u8(opcode) {
                let sequence = sequence_mode_subroutine(op);
                decoded_opcode = DecodedOpcode::new(sequence, instr::InstructionOp::Nop, None);
            } else if let Some(op) = OpsSingleByte0::from_u8(opcode) {
                let operation = instr_op_single_byte0(op);
                let sequence = sequence_mode_single_byte0(op);
                decoded_opcode = DecodedOpcode::new(sequence, operation, None);
            } else if cfg!(feature = "undoc_opcodes") {
                todo!();
            } else {
                unimplemented!()
            }
        }
        _ => (),
    };

    decoded_opcode
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn single_byte_print() {
        println!("\nSingle Bytes\n");
        for i in 0_u8..=0b_0011_1111 {
            let opcode = i << 2;

            if OpsSingleByte0::from_u8(opcode).is_some() {
                let decoded = super::decode(opcode);
                println!("\t{:#04X}\t{:?}", opcode, decoded);
            }
        }

        for i in 0_u8..=0b_0011_1111 {
            let opcode = (i << 2) + 2;

            if OpsSingleByte2::from_u8(opcode).is_some() {
                let decoded = super::decode(opcode);
                println!("\t{:#04X}\t{:?}", opcode, decoded);
            }
        }
    }
}
