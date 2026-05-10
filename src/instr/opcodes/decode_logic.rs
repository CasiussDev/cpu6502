//! # 6502 CPU Instruction Decoder
//!
//! This module implements the instruction decoding logic for the 6502 CPU.
//! It handles converting raw opcode bytes into decoded instructions by:
//! 1. Identifying the instruction group (Groups 1-3 or special instructions)
//! 2. Extracting the operation and addressing mode
//! 3. Determining the instruction sequence mode and index register usage
//!
//! The 6502's opcodes are organized into several groups:
//! - Group 1: Basic operations (ORA, AND, EOR, ADC, STA, LDA, CMP, SBC)
//! - Group 2: Shift/rotate and increment/decrement operations
//! - Group 3: Bit operations, jumps, and load/store operations
//! - Special instructions: Branches, subroutines, and single-byte operations

use crate::instr::{Instruction, InstructionOp, InstructionSequenceMode};
use crate::registers::IndexRegister;
use enum_primitive_derive::Primitive;
use num_traits::FromPrimitive;

/// Mask to extract the opcode group (bits 0-1)
const OPCODE_GROUP_MASK: u8 = 0b_0000_0011;

/// Mask to extract the operation type for groups 1-3 (bits 0-1 and 5-7)
const OPCODE_G123_OP_MASK: u8 = 0b_1110_0011;
/// Mask to extract the addressing mode for groups 1-3 (bits 2-4)
const OPCODE_G123_ADDR_MASK: u8 = 0b_0001_1100;

/// Group 1 operations - Basic arithmetic and logic operations
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

/// Addressing modes for Group 1 instructions
///
/// These modes determine how the operand address is calculated:
/// - Indexed indirect: Address is fetched from zero page location indexed by X
/// - Zero page: Direct zero-page address
/// - Immediate: Value is directly in instruction
/// - Absolute: Full 16-bit address
/// - Indirect indexed: Zero-page address contains base, Y register adds offset
/// - Zero page indexed: Zero-page address offset by index register
/// - Absolute indexed: Full address offset by X or Y register
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

/// Group 2 operations - Shifts, rotates, and memory operations
///
/// These instructions handle:
/// - Arithmetic/logical shifts (ASL, LSR)
/// - Rotates through carry (ROL, ROR)
/// - X register operations (STX, LDX)
/// - Memory increment/decrement (INC, DEC)
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

/// Addressing modes for Group 2 instructions
///
/// Similar to Group 1 modes but with some differences:
/// - Includes accumulator mode for shift/rotate operations
/// - Some modes are unused/illegal
/// - Indexed operations use different registers based on instruction
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

/// Group 3 operations - Tests, jumps and Y register operations
///
/// These instructions include:
/// - Bit test operations (BIT)
/// - Jump operations (JMP, indirect JMP)
/// - Y register operations (STY, LDY)
/// - Compare operations (CPY, CPX)
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

/// Addressing modes for Group 3 instructions
///
/// Similar to Group 2 modes but with differences:
/// - No accumulator mode
/// - Different unused/illegal combinations
/// - Limited indexed operations
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

/// Comparison and branch operations
///
/// These conditional branch instructions test processor status flags:
/// - BPL/BMI: Tests negative flag
/// - BVC/BVS: Tests overflow flag
/// - BCC/BCS: Tests carry flag
/// - BNE/BEQ: Tests zero flag
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

/// Subroutine and interrupt operations
///
/// These instructions handle subroutine calls and interrupts:
/// - BRK: Software interrupt
/// - JSR: Jump to subroutine
/// - RTI: Return from interrupt
/// - RTS: Return from subroutine
#[derive(Clone, Copy, Debug, Eq, PartialEq, Primitive)]
pub enum OpsSubroutine {
    BRK = 0x00,
    JSR = 0x20,
    RTI = 0x40,
    RTS = 0x60,
}

/// Single byte instructions using addressing mode 0
///
/// These instructions require no operands and include:
/// - Stack operations (PHP, PLP, PHA, PLA)
/// - Register transfers (TAY, TYA)
/// - Register operations (DEY, INY, INX)
/// - Status flag operations (CLC, SEC, CLI, SEI, CLV, CLD, SED)
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

/// Single byte instructions using addressing mode 2
///
/// These instructions require no operands and include:
/// - Register transfers (TXA, TXS, TAX, TSX)
/// - X register operations (DEX)
/// - No operation (NOP)
#[derive(Clone, Copy, Debug, Eq, PartialEq, Primitive)]
pub enum OpsSingleByte2 {
    TXA = 0x8A,
    TXS = 0x9A,
    TAX = 0xAA,
    TSX = 0xBA,
    DEX = 0xCA,
    NOP = 0xEA,
}

/// Converts a Group 1 operation into its corresponding instruction operation
///
/// # Arguments
/// * `op` - The Group 1 operation to convert
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

/// Converts a Group 2 operation into its corresponding instruction operation
///
/// # Arguments
/// * `op` - The Group 2 operation to convert
/// * `addr_mode` - The addressing mode, needed for accumulator vs memory operations
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

/// Converts a Group 3 operation into its corresponding instruction operation
///
/// # Arguments
/// * `op` - The Group 3 operation to convert
/// * `addr_mode` - The addressing mode, needed to distinguish between immediate and memory BIT operations
///
/// # Returns
/// The appropriate instruction operation for the Group 3 operation
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

/// Converts a conditional branch operation into its corresponding instruction operation
///
/// # Arguments
/// * `op` - The conditional branch operation to convert
///
/// # Returns
/// The appropriate branch instruction operation
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

/// Converts a single byte mode 0 operation into its corresponding instruction operation
///
/// # Arguments
/// * `op` - The single byte mode 0 operation to convert
///
/// # Returns
/// The appropriate instruction operation for stack, register transfer, and flag operations
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

/// Converts a single byte mode 2 operation into its corresponding instruction operation
///
/// # Arguments
/// * `op` - The single byte mode 2 operation to convert
///
/// # Returns
/// The appropriate instruction operation for register transfers and NOP
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

/// Determines the sequence mode for Group 1 instructions based on operation and addressing mode
///
/// # Arguments
/// * `op` - The Group 1 operation
/// * `addr_mode` - The addressing mode
///
/// # Returns
/// The appropriate instruction sequence mode for executing the instruction
fn sequence_mode_g1(op: OpsG1, addr_mode: AddrModeG1) -> InstructionSequenceMode {
    match addr_mode {
        AddrModeG1::ZeroPage => InstructionSequenceMode::ZeroPage,
        AddrModeG1::Immediate => InstructionSequenceMode::Immediate,
        AddrModeG1::Absolute => InstructionSequenceMode::Absolute,
        AddrModeG1::ZeroPageIndxIndirect => InstructionSequenceMode::ZeroPageIdxIndirect,
        AddrModeG1::ZeroPageIdx => InstructionSequenceMode::ZeroPageIdx,
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

/// Determines the sequence mode for Group 2 instructions based on operation and addressing mode
///
/// # Arguments
/// * `op` - The Group 2 operation
/// * `addr_mode` - The addressing mode
///
/// # Returns
/// The appropriate instruction sequence mode for executing the instruction, handling
/// read-modify-write operations differently from regular reads/writes
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
                AddrModeG2::ZeroPageIdx => InstructionSequenceMode::ZeroPageIdx,
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

/// Determines the sequence mode for Group 3 instructions based on operation and addressing mode
///
/// # Arguments
/// * `op` - The Group 3 operation
/// * `addr_mode` - The addressing mode
///
/// # Returns
/// The appropriate instruction sequence mode for executing the instruction, with
/// special handling for jump operations
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
            AddrModeG3::ZeroPageIdx => InstructionSequenceMode::ZeroPageIdx, // Illegal for BIT, Jumps and Cp
            AddrModeG3::AbsoluteIdxX => InstructionSequenceMode::AbsoluteIdxRead, // Only legal for LDY
        },
    }
}

/// Determines the sequence mode for subroutine and interrupt instructions
///
/// # Arguments
/// * `op` - The subroutine operation (BRK, JSR, RTI, RTS)
///
/// # Returns
/// The appropriate instruction sequence mode for handling stack operations
/// and program counter modifications
fn sequence_mode_subroutine(op: OpsSubroutine) -> InstructionSequenceMode {
    match op {
        OpsSubroutine::BRK => InstructionSequenceMode::Break,
        OpsSubroutine::JSR => InstructionSequenceMode::JumpSubroutine,
        OpsSubroutine::RTI => InstructionSequenceMode::ReturnInterrupt,
        OpsSubroutine::RTS => InstructionSequenceMode::ReturnSubroutine,
    }
}

/// Determines the sequence mode for single byte mode 0 instructions
///
/// # Arguments
/// * `op` - The single byte mode 0 operation
///
/// # Returns
/// The appropriate instruction sequence mode, distinguishing between
/// stack operations and implied addressing instructions
fn sequence_mode_single_byte0(op: OpsSingleByte0) -> InstructionSequenceMode {
    match op {
        OpsSingleByte0::PHP | OpsSingleByte0::PHA => InstructionSequenceMode::Push,
        OpsSingleByte0::PLP | OpsSingleByte0::PLA => InstructionSequenceMode::Pull,
        _ => InstructionSequenceMode::Implied,
    }
}

/// Gets the index register used by a Group 1 instruction
///
/// # Arguments
/// * `addr_mode` - The addressing mode
///
/// # Returns
/// Some(IndexRegister) if the instruction uses indexing, None otherwise
fn index_reg_g1(addr_mode: AddrModeG1) -> Option<IndexRegister> {
    match addr_mode {
        AddrModeG1::ZeroPageIdx | AddrModeG1::ZeroPageIndxIndirect | AddrModeG1::AbsoluteIdxX => {
            Some(IndexRegister::X)
        }
        AddrModeG1::ZeroPageIndirectIdx | AddrModeG1::AbsoluteIdxY => Some(IndexRegister::Y),
        _ => None,
    }
}

/// Gets the index register used by a Group 2 instruction
///
/// # Arguments
/// * `op` - The Group 2 operation
/// * `addr_mode` - The addressing mode
///
/// # Returns
/// Some(IndexRegister) if the instruction uses indexing, None otherwise.
/// Note that STX/LDX use Y register while other operations use X register.
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

/// Gets the index register used by a Group 3 instruction
///
/// # Arguments
/// * `addr_mode` - The addressing mode
///
/// # Returns
/// Some(IndexRegister::X) for zero page indexed or absolute indexed modes,
/// None otherwise
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

/// Checks if a Group 1 instruction combination is illegal
///
/// # Arguments
/// * `op` - The Group 1 operation
/// * `addr_mode` - The addressing mode
///
/// # Returns
/// true if the combination is illegal, false otherwise
fn illegal_instruction_g1(op: OpsG1, addr_mode: AddrModeG1) -> bool {
    (op == OpsG1::STA) && (addr_mode == AddrModeG1::Immediate)
}

/// Checks if a Group 2 instruction combination is illegal
///
/// # Arguments
/// * `op` - The Group 2 operation
/// * `addr_mode` - The addressing mode
///
/// # Returns
/// true if the combination is illegal according to the 6502 specification,
/// false otherwise
fn illegal_instruction_g2(op: OpsG2, addr_mode: AddrModeG2) -> bool {
    ((op == OpsG2::STX) && (addr_mode == AddrModeG2::AbsoluteIdxX))
        || ((op != OpsG2::LDX) && (addr_mode == AddrModeG2::Immediate))
        || (matches!(op, OpsG2::STX | OpsG2::LDX | OpsG2::DEC | OpsG2::INC)
            && (addr_mode == AddrModeG2::Accumulator))
        || matches!(addr_mode, AddrModeG2::Unused1 | AddrModeG2::Unused2)
}

/// Checks if a Group 3 instruction combination is illegal
///
/// # Arguments
/// * `op` - The Group 3 operation
/// * `addr_mode` - The addressing mode
///
/// # Returns
/// true if the combination is illegal according to the 6502 specification,
/// false otherwise
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

/// Decodes a raw opcode byte into a fully specified instruction
///
/// # Arguments
/// * `opcode` - The raw opcode byte to decode
///
/// # Returns
/// A decoded `Instruction` containing the operation type, addressing mode,
/// and any required index registers. Returns a NOP instruction for illegal opcodes
/// unless undocumented opcodes are enabled.
pub fn decode(opcode: u8) -> Instruction {
    let mut decoded_instruction = Instruction::default();
    match opcode & OPCODE_GROUP_MASK {
        1 => {
            let op = OpsG1::from_u8(opcode & OPCODE_G123_OP_MASK)
                .expect("Trying to decode wrong opcode");
            let addr_mode = AddrModeG1::from_u8(opcode & OPCODE_G123_ADDR_MASK)
                .expect("Trying to decode wrong opcode");
            if illegal_instruction_g1(op, addr_mode) == false {
                let operation = instr_op_g1(op);
                let sequence = sequence_mode_g1(op, addr_mode);
                let index = index_reg_g1(addr_mode);
                decoded_instruction = Instruction::new(sequence, operation, index);
            } else if cfg!(feature = "undoc_opcodes") {
                todo!();
            }
        }
        2 => {
            let op = OpsG2::from_u8(opcode & OPCODE_G123_OP_MASK)
                .expect("Trying to decode wrong opcode");
            let addr_mode = AddrModeG2::from_u8(opcode & OPCODE_G123_ADDR_MASK)
                .expect("Trying to decode wrong opcode");
            if illegal_instruction_g2(op, addr_mode) == false {
                let operation = instr_op_g2(op, addr_mode);
                let sequence = sequence_mode_g2(op, addr_mode);
                let index = index_reg_g2(op, addr_mode);
                decoded_instruction = Instruction::new(sequence, operation, index);
            } else if let Some(op) = OpsSingleByte2::from_u8(opcode) {
                let operation = instr_op_single_byte2(op);
                decoded_instruction = Instruction::Implied(operation.try_into().unwrap());
            } else if cfg!(feature = "undoc_opcodes") {
                todo!();
            }
        }
        0 => {
            let op = OpsG3::from_u8(opcode & OPCODE_G123_OP_MASK)
                .expect("Trying to decode wrong opcode");
            let addr_mode = AddrModeG3::from_u8(opcode & OPCODE_G123_ADDR_MASK)
                .expect("Trying to decode wrong opcode");
            if illegal_instruction_g3(op, addr_mode) == false {
                let operation = instr_op_g3(op, addr_mode);
                let sequence = sequence_mode_g3(op, addr_mode);
                let index = index_reg_g3(addr_mode);
                decoded_instruction = Instruction::new(sequence, operation, index);
            } else if let Some(op) = OpsCompBranch::from_u8(opcode) {
                let operation = instr_op_cond_branch(op);
                decoded_instruction = Instruction::Relative(operation.try_into().unwrap());
            } else if let Some(op) = OpsSubroutine::from_u8(opcode) {
                let sequence = sequence_mode_subroutine(op);
                decoded_instruction = Instruction::new(sequence, InstructionOp::Nop, None);
            } else if let Some(op) = OpsSingleByte0::from_u8(opcode) {
                let operation = instr_op_single_byte0(op);
                let sequence = sequence_mode_single_byte0(op);
                decoded_instruction = Instruction::new(sequence, operation, None);
            } else if cfg!(feature = "undoc_opcodes") {
                todo!();
            }
        }
        _ => (),
    };

    decoded_instruction
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
