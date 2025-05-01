use crate::instr;
use crate::registers::{IndexRegister, SelectedRegister16, SelectedRegister8, StatusRegFlags};
use std::{collections, slice};
use strum_macros::EnumDiscriminants;

use crate::instr::{BranchOperation, ImplicitOperation, InstructionOp, MemoryModifyOperation, PullStackOperation, PushStackOperation, RegisterMemoryOperation};
#[cfg(test)]
use strum::IntoEnumIterator;
#[cfg(any(test, feature = "gen_write_cycle_query"))]
use strum_macros::{Display, EnumIter};

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default, EnumDiscriminants)]
#[cfg_attr(feature = "gen_write_cycle_query", derive(EnumIter))]
#[strum_discriminants(name(InstructionSequenceMode))]
#[strum_discriminants(derive(Default))]
pub enum Instruction {
    #[default]
    #[strum_discriminants(default)]
    FetchInstr,
    Break,
    StartIrq,
    StartNmi,
    Reset,
    ReturnInterrupt,

    JumpSubroutine,
    ReturnSubroutine,

    Push(PushStackOperation),
    Pull(PullStackOperation),
    Implied(ImplicitOperation),
    Immediate(RegisterMemoryOperation),

    AbsoluteJump,
    Absolute(RegisterMemoryOperation),
    AbsoluteReadModifyWrite(MemoryModifyOperation),

    ZeroPage(RegisterMemoryOperation),
    ZeroPageReadModifyWrite(MemoryModifyOperation),

    ZeroPageIdx(RegisterMemoryOperation, IndexRegister),
    ZeroPageIdxReadModifyWrite(MemoryModifyOperation, IndexRegister),

    AbsoluteIdxRead(RegisterMemoryOperation, IndexRegister),
    AbsoluteIdxReadModifyWrite(MemoryModifyOperation, IndexRegister),
    AbsoluteIdxWrite(RegisterMemoryOperation, IndexRegister),

    Relative(BranchOperation),

    ZeroPageIdxIndirect(RegisterMemoryOperation, IndexRegister),
    ZeroPageIdxIndirectReadModifyWrite(MemoryModifyOperation, IndexRegister),

    ZeroPageIndirectIdxRead(RegisterMemoryOperation, IndexRegister),
    ZeroPageIndirectIdxReadModifyWrite(MemoryModifyOperation, IndexRegister),
    ZeroPageIndirectIdxWrite(RegisterMemoryOperation, IndexRegister),

    AbsoluteIndirectJump,
}

impl From<Instruction> for (InstructionSequenceMode, InstructionOp, IndexRegister) {
    fn from(sequence: Instruction) -> Self {
        match sequence {
            Instruction::Reset => (
                InstructionSequenceMode::Reset,
                InstructionOp::default(),
                IndexRegister::default(),
            ),
            Instruction::FetchInstr => (
                InstructionSequenceMode::FetchInstr,
                InstructionOp::default(),
                IndexRegister::default(),
            ),
            Instruction::StartNmi => (
                InstructionSequenceMode::StartNmi,
                InstructionOp::default(),
                IndexRegister::default(),
            ),
            Instruction::StartIrq => (
                InstructionSequenceMode::StartIrq,
                InstructionOp::default(),
                IndexRegister::default(),
            ),
            Instruction::Break => (
                InstructionSequenceMode::Break,
                InstructionOp::default(),
                IndexRegister::default(),
            ),
            Instruction::ReturnInterrupt => (
                InstructionSequenceMode::ReturnInterrupt,
                InstructionOp::default(),
                IndexRegister::default(),
            ),
            Instruction::JumpSubroutine => (
                InstructionSequenceMode::JumpSubroutine,
                InstructionOp::default(),
                IndexRegister::default(),
            ),
            Instruction::ReturnSubroutine => (
                InstructionSequenceMode::ReturnSubroutine,
                InstructionOp::default(),
                IndexRegister::default(),
            ),
            Instruction::Push(op) => (
                InstructionSequenceMode::Push,
                InstructionOp::from(op),
                IndexRegister::default(),
            ),
            Instruction::Pull(op) => (
                InstructionSequenceMode::Pull,
                InstructionOp::from(op),
                IndexRegister::default(),
            ),
            Instruction::Implied(op) => (
                InstructionSequenceMode::Implied,
                InstructionOp::from(op),
                IndexRegister::default(),
            ),
            Instruction::Immediate(op) => (
                InstructionSequenceMode::Immediate,
                InstructionOp::from(op),
                IndexRegister::default(),
            ),
            Instruction::AbsoluteJump => (
                InstructionSequenceMode::AbsoluteJump,
                InstructionOp::default(),
                IndexRegister::default(),
            ),
            Instruction::Absolute(op) => (
                InstructionSequenceMode::Absolute,
                InstructionOp::from(op),
                IndexRegister::default(),
            ),
            Instruction::AbsoluteReadModifyWrite(op) => (
                InstructionSequenceMode::AbsoluteReadModifyWrite,
                InstructionOp::from(op),
                IndexRegister::default(),
            ),
            Instruction::ZeroPage(op) => (
                InstructionSequenceMode::ZeroPage,
                InstructionOp::from(op),
                IndexRegister::default(),
            ),
            Instruction::ZeroPageReadModifyWrite(op) => (
                InstructionSequenceMode::ZeroPageReadModifyWrite,
                InstructionOp::from(op),
                IndexRegister::default(),
            ),
            Instruction::ZeroPageIdx(op, idx) => (
                InstructionSequenceMode::ZeroPageIdx,
                InstructionOp::from(op),
                idx,
            ),
            Instruction::ZeroPageIdxReadModifyWrite(op, idx) => (
                InstructionSequenceMode::ZeroPageIdxReadModifyWrite,
                InstructionOp::from(op),
                idx,
            ),
            Instruction::AbsoluteIdxRead(op, idx) => (
                InstructionSequenceMode::AbsoluteIdxRead,
                InstructionOp::from(op),
                idx,
            ),
            Instruction::AbsoluteIdxReadModifyWrite(op, idx) => (
                InstructionSequenceMode::AbsoluteIdxReadModifyWrite,
                InstructionOp::from(op),
                idx,
            ),
            Instruction::AbsoluteIdxWrite(op, idx) => (
                InstructionSequenceMode::AbsoluteIdxWrite,
                InstructionOp::from(op),
                idx,
            ),
            Instruction::Relative(op) => (
                InstructionSequenceMode::Relative,
                InstructionOp::from(op),
                IndexRegister::default(),
            ),
            Instruction::ZeroPageIdxIndirect(op, idx) => (
                InstructionSequenceMode::ZeroPageIdxIndirect,
                InstructionOp::from(op),
                idx,
            ),
            Instruction::ZeroPageIdxIndirectReadModifyWrite(op, idx) => (
                InstructionSequenceMode::ZeroPageIdxIndirectReadModifyWrite,
                InstructionOp::from(op),
                idx,
            ),
            Instruction::ZeroPageIndirectIdxRead(op, idx) => (
                InstructionSequenceMode::ZeroPageIndirectIdxRead,
                InstructionOp::from(op),
                idx,
            ),
            Instruction::ZeroPageIndirectIdxReadModifyWrite(op, idx) => (
                InstructionSequenceMode::ZeroPageIndirectIdxReadModifyWrite,
                InstructionOp::from(op),
                idx,
            ),
            Instruction::ZeroPageIndirectIdxWrite(op, idx) => (
                InstructionSequenceMode::ZeroPageIndirectIdxWrite,
                InstructionOp::from(op),
                idx,
            ),
            Instruction::AbsoluteIndirectJump => (
                InstructionSequenceMode::AbsoluteIndirectJump,
                InstructionOp::default(),
                IndexRegister::default(),
            ),
        }
    }
}