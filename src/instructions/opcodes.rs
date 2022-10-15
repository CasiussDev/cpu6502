use crate::instructions::InstructionOp;
use crate::instructions::InstructionSequenceMode;
use crate::registers::IndexRegister;
use num_traits::FromPrimitive;

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

#[derive(Clone, Copy, Default, Debug, Eq, PartialEq)]
pub struct DecodedOpcode {
    pub sequence: InstructionSequenceMode,
    pub operation: InstructionOp,
    pub index: Option<IndexRegister>,
}

impl DecodedOpcode {
    pub fn new(
        sequence: InstructionSequenceMode,
        operation: InstructionOp,
        index: Option<IndexRegister>,
    ) -> Self {
        Self {
            sequence,
            operation,
            index,
        }
    }
}

fn instr_op_g1(op: OpsG1) -> InstructionOp {
    match op {
        OpsG1::ORA => InstructionOp::Or,
        OpsG1::AND => InstructionOp::And,
        OpsG1::EOR => InstructionOp::Xor,
        OpsG1::ADC => InstructionOp::Add,
        OpsG1::STA => InstructionOp::StoreA,
        OpsG1::LDA => InstructionOp::LoadA,
        OpsG1::CMP => InstructionOp::Cmp,
        OpsG1::SBC => InstructionOp::Sub,
    }
}

fn instr_op_g2(op: OpsG2, addr_mode: AddrModeG2) -> InstructionOp {
    match op {
        OpsG2::ASL => match addr_mode {
            AddrModeG2::Accumulator => InstructionOp::ShiftLeftA,
            _ => InstructionOp::ShiftLeftMemory,
        },
        OpsG2::ROL => match addr_mode {
            AddrModeG2::Accumulator => InstructionOp::RotateLeftA,
            _ => InstructionOp::RotateLeftMemory,
        },
        OpsG2::LSR => match addr_mode {
            AddrModeG2::Accumulator => InstructionOp::ShiftRightA,
            _ => InstructionOp::ShiftRightMemory,
        },
        OpsG2::ROR => match addr_mode {
            AddrModeG2::Accumulator => InstructionOp::RotateRightA,
            _ => InstructionOp::RotateRightMemory,
        },
        OpsG2::STX => InstructionOp::StoreX,
        OpsG2::LDX => InstructionOp::LoadX,
        OpsG2::DEC => InstructionOp::DecrementMemory,
        OpsG2::INC => InstructionOp::IncrementMemory,
    }
}

fn instr_op_g3(op: OpsG3, addr_mode: AddrModeG3) -> InstructionOp {
    match op {
        OpsG3::Unused => InstructionOp::Nop,
        OpsG3::BIT => {
            if addr_mode == AddrModeG3::Immediate {
                InstructionOp::BitImmediate
            } else {
                InstructionOp::Bit
            }
        }
        OpsG3::JMP => InstructionOp::Nop,
        OpsG3::JMPInd => InstructionOp::Nop,
        OpsG3::STY => InstructionOp::StoreY,
        OpsG3::LDY => InstructionOp::LoadY,
        OpsG3::CPY => InstructionOp::Cpy,
        OpsG3::CPX => InstructionOp::Cpx,
    }
}

fn instr_op_cond_branch(op: OpsCompBranch) -> InstructionOp {
    match op {
        OpsCompBranch::BPL => InstructionOp::BranchPlus,
        OpsCompBranch::BMI => InstructionOp::BranchMinus,
        OpsCompBranch::BVC => InstructionOp::BranchOverflowClear,
        OpsCompBranch::BVS => InstructionOp::BranchOverflowSet,
        OpsCompBranch::BCC => InstructionOp::BranchCarryClear,
        OpsCompBranch::BCS => InstructionOp::BranchCarrySet,
        OpsCompBranch::BNE => InstructionOp::BranchNotEqual,
        OpsCompBranch::BEQ => InstructionOp::BranchEqual,
    }
}

fn instr_op_single_byte0(op: OpsSingleByte0) -> InstructionOp {
    match op {
        OpsSingleByte0::PHP => InstructionOp::PushStatus,
        OpsSingleByte0::PLP => InstructionOp::PullStatus,
        OpsSingleByte0::PHA => InstructionOp::PushA,
        OpsSingleByte0::PLA => InstructionOp::PullA,
        OpsSingleByte0::DEY => InstructionOp::DecrementY,
        OpsSingleByte0::TAY => InstructionOp::TransferAccumulatorToY,
        OpsSingleByte0::INY => InstructionOp::IncrementY,
        OpsSingleByte0::INX => InstructionOp::IncrementX,
        OpsSingleByte0::CLC => InstructionOp::ClearCarry,
        OpsSingleByte0::SEC => InstructionOp::SetCarry,
        OpsSingleByte0::CLI => InstructionOp::ClearInterruptDisable,
        OpsSingleByte0::SEI => InstructionOp::SetInterruptDisable,
        OpsSingleByte0::TYA => InstructionOp::TransferYToAccumulator,
        OpsSingleByte0::CLV => InstructionOp::ClearOverflow,
        OpsSingleByte0::CLD => InstructionOp::ClearDecimal,
        OpsSingleByte0::SED => InstructionOp::SetDecimal,
    }
}
fn instr_op_single_byte2(op: OpsSingleByte2) -> InstructionOp {
    match op {
        OpsSingleByte2::TXA => InstructionOp::TransferXToAccumulator,
        OpsSingleByte2::TXS => InstructionOp::TransferXToStackPtr,
        OpsSingleByte2::TAX => InstructionOp::TransferAccumulatorToX,
        OpsSingleByte2::TSX => InstructionOp::TransferStackPtrToX,
        OpsSingleByte2::DEX => InstructionOp::DecrementX,
        OpsSingleByte2::NOP => InstructionOp::Nop,
    }
}

fn sequence_mode_g1(op: OpsG1, addr_mode: AddrModeG1) -> InstructionSequenceMode {
    match addr_mode {
        AddrModeG1::ZeroPage => InstructionSequenceMode::ZeroPage,
        AddrModeG1::Immediate => InstructionSequenceMode::Immediate,
        AddrModeG1::Absolute => InstructionSequenceMode::Absolute,
        AddrModeG1::ZeroPageIndxIndirect => InstructionSequenceMode::ZeroPageIdxIndirect,
        AddrModeG1::ZeroPageIdx => InstructionSequenceMode::ZeroPageIndx,
        AddrModeG1::AbsoluteIdxY | AddrModeG1::AbsoluteIdxX => match op {
            OpsG1::STA => InstructionSequenceMode::AbsoluteIdxWrite,
            OpsG1::ORA
            | OpsG1::AND
            | OpsG1::EOR
            | OpsG1::ADC
            | OpsG1::LDA
            | OpsG1::CMP
            | OpsG1::SBC => InstructionSequenceMode::AbsoluteIdxRead,
        },
        AddrModeG1::ZeroPageIndirectIdx => match op {
            OpsG1::STA => InstructionSequenceMode::ZeroPageIndirectIdxWrite,
            OpsG1::ORA
            | OpsG1::AND
            | OpsG1::EOR
            | OpsG1::ADC
            | OpsG1::LDA
            | OpsG1::CMP
            | OpsG1::SBC => InstructionSequenceMode::ZeroPageIndirectIdxRead,
        },
    }
}

fn sequence_mode_g2(op: OpsG2, addr_mode: AddrModeG2) -> InstructionSequenceMode {
    match op {
        OpsG2::ASL | OpsG2::ROL | OpsG2::LSR | OpsG2::ROR | OpsG2::DEC | OpsG2::INC => {
            match addr_mode {
                AddrModeG2::ZeroPage => InstructionSequenceMode::ZeroPageReadModifyWrite,
                AddrModeG2::Immediate => InstructionSequenceMode::Immediate, // Illegal
                AddrModeG2::Absolute => InstructionSequenceMode::AbsoluteReadModifyWrite,
                AddrModeG2::ZeroPageIdx => InstructionSequenceMode::ZeroPageIdxReadModifyWrite,
                AddrModeG2::AbsoluteIdxX => InstructionSequenceMode::AbsoluteIdxReadModifyWrite,
                AddrModeG2::Accumulator => InstructionSequenceMode::Implied, // Illegal for Inc and Dec
                AddrModeG2::Unused1 | AddrModeG2::Unused2 => InstructionSequenceMode::default(), // Illegal
            }
        }
        OpsG2::STX | OpsG2::LDX => {
            match addr_mode {
                AddrModeG2::Immediate => InstructionSequenceMode::Immediate, // Illegal for STX
                AddrModeG2::ZeroPage => InstructionSequenceMode::ZeroPage,
                AddrModeG2::Accumulator => InstructionSequenceMode::Implied, // Illegal
                AddrModeG2::Absolute => InstructionSequenceMode::Absolute,
                AddrModeG2::ZeroPageIdx => InstructionSequenceMode::ZeroPageIdxIndirect,
                AddrModeG2::AbsoluteIdxX => {
                    if op == OpsG2::STX {
                        InstructionSequenceMode::AbsoluteIdxWrite // Illegal
                    } else {
                        InstructionSequenceMode::AbsoluteIdxRead
                    }
                }
                AddrModeG2::Unused1 | AddrModeG2::Unused2 => InstructionSequenceMode::default(), // Illegal
            }
        }
    }
}

fn sequence_mode_g3(op: OpsG3, addr_mode: AddrModeG3) -> InstructionSequenceMode {
    match op {
        OpsG3::JMP => InstructionSequenceMode::AbsoluteJump,
        OpsG3::JMPInd => InstructionSequenceMode::AbsoluteIndirectJump,
        _ => match addr_mode {
            AddrModeG3::Immediate => InstructionSequenceMode::Immediate, // Illegal for BIT, Jumps and STY
            AddrModeG3::ZeroPage => InstructionSequenceMode::ZeroPage,   // Illegal for Jumps
            AddrModeG3::Unused1 | AddrModeG3::Unused2 | AddrModeG3::Unused3 => {
                InstructionSequenceMode::default()
            } // Illegal
            AddrModeG3::Absolute => InstructionSequenceMode::Absolute,
            AddrModeG3::ZeroPageIdx => InstructionSequenceMode::ZeroPageIndx, // Illegal for BIT, Jumps and Cp
            AddrModeG3::AbsoluteIdxX => InstructionSequenceMode::AbsoluteIdxRead, // Only legal for LDY
        },
    }
}

fn sequence_mode_subroutine(op: OpsSubroutine) -> InstructionSequenceMode {
    match op {
        OpsSubroutine::BRK => InstructionSequenceMode::Break,
        OpsSubroutine::JSR => InstructionSequenceMode::JumpSubroutine,
        OpsSubroutine::RTI => InstructionSequenceMode::ReturnInterrupt,
        OpsSubroutine::RTS => InstructionSequenceMode::ReturnSubroutine,
    }
}

fn sequence_mode_single_byte0(op: OpsSingleByte0) -> InstructionSequenceMode {
    match op {
        OpsSingleByte0::PHP | OpsSingleByte0::PHA => InstructionSequenceMode::Push,
        OpsSingleByte0::PLP | OpsSingleByte0::PLA => InstructionSequenceMode::Pull,
        _ => InstructionSequenceMode::Implied,
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
                    DecodedOpcode::new(InstructionSequenceMode::Implied, operation, None);
            } else if cfg!(feature = "undoc_opcodes") {
                todo!();
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
                    DecodedOpcode::new(InstructionSequenceMode::Relative, operation, None);
            } else if let Some(op) = OpsSubroutine::from_u8(opcode) {
                let sequence = sequence_mode_subroutine(op);
                decoded_opcode = DecodedOpcode::new(sequence, InstructionOp::Nop, None);
            } else if let Some(op) = OpsSingleByte0::from_u8(opcode) {
                let operation = instr_op_single_byte0(op);
                let sequence = sequence_mode_single_byte0(op);
                decoded_opcode = DecodedOpcode::new(sequence, operation, None);
            } else if cfg!(feature = "undoc_opcodes") {
                todo!();
            }
        }
        _ => (),
    };

    decoded_opcode
}

#[cfg(test)]
mod tests {
    use crate::instructions::opcodes::{OpsSingleByte0, OpsSingleByte2};
    use crate::instructions::{InstructionOp, InstructionSequenceMode};
    use crate::num_traits::FromPrimitive;

    #[test]
    fn g1_print() {
        println!("\nGroup 1\n");
        for i in 0_u8..=0b_0011_1111 {
            let opcode = (i << 2) + 1;
            let decoded = super::decode(opcode);

            println!("\t{:#04X}\t{:?}", opcode, decoded);
        }
    }

    #[test]
    fn g2_print() {
        println!("\nGroup 2\n");
        for i in 0_u8..=0b_0011_1111 {
            let opcode = (i << 2) + 2;
            let decoded = super::decode(opcode);

            println!("\t{:#04X}\t{:?}", opcode, decoded);
        }
    }

    #[test]
    fn g3_print() {
        println!("\nGroup 3\n");
        for i in 0_u8..=0b_0011_1111 {
            let opcode = i << 2;
            let decoded = super::decode(opcode);

            if decoded.sequence != InstructionSequenceMode::Relative
                && (decoded.operation != InstructionOp::Nop)
                || matches!(
                    decoded.sequence,
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
            let decoded = super::decode(opcode);

            if decoded.sequence == InstructionSequenceMode::Relative {
                println!("\t{:#04X}\t{:?}", opcode, decoded);
            }
        }
    }

    #[test]
    fn subroutine_print() {
        println!("\nSubroutines and Interrupts\n");
        for i in 0_u8..=0b_0011_1111 {
            let opcode = i << 2;
            let decoded = super::decode(opcode);

            if matches!(
                decoded.sequence,
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
    fn single_byte_print() {
        println!("\nSingle Bytes\n");
        for i in 0_u8..=0b_0011_1111 {
            let opcode = i << 2;

            if let Some(_) = OpsSingleByte0::from_u8(opcode) {
                let decoded = super::decode(opcode);
                println!("\t{:#04X}\t{:?}", opcode, decoded);
            }
        }

        for i in 0_u8..=0b_0011_1111 {
            let opcode = (i << 2) + 2;

            if let Some(_) = OpsSingleByte2::from_u8(opcode) {
                let decoded = super::decode(opcode);
                println!("\t{:#04X}\t{:?}", opcode, decoded);
            }
        }
    }

    #[test]
    fn allopcodes_decoding_neverpanic() {
        for opcode in 0..=u8::MAX {
            super::decode(opcode);
        }
    }
}
