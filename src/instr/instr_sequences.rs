use crate::instr;
use crate::registers::{IndexRegister, SelectedRegister16, SelectedRegister8, StatusRegFlags};
use std::{collections, slice};
use strum_macros::EnumDiscriminants;

use crate::instr::{
    BranchOperation, ImplicitOperation, MemoryModifyOperation, PullStackOperation,
    PushStackOperation, RegisterMemoryOperation,
};
#[cfg(test)]
use strum::IntoEnumIterator;
#[cfg(any(test, feature = "gen_write_cycle_query"))]
use strum_macros::{Display, EnumIter};

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
#[cfg_attr(test, derive(EnumIter))]
pub enum InstructionSequenceMode {
    FetchInstr,
    Break,
    StartIrq,
    StartNmi,
    Reset,
    ReturnInterrupt,

    JumpSubroutine,
    ReturnSubroutine,

    Push,
    Pull,
    Implied,
    Immediate,

    AbsoluteJump,
    Absolute,
    AbsoluteReadModifyWrite,

    ZeroPage,
    ZeroPageReadModifyWrite,

    ZeroPageIdx,
    ZeroPageIdxReadModifyWrite,

    AbsoluteIdxRead,
    AbsoluteIdxReadModifyWrite,
    AbsoluteIdxWrite,

    Relative,

    ZeroPageIdxIndirect,
    ZeroPageIdxIndirectReadModifyWrite,

    ZeroPageIndirectIdxRead,
    ZeroPageIndirectIdxReadModifyWrite,
    ZeroPageIndirectIdxWrite,

    AbsoluteIndirectJump,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default, EnumDiscriminants)]
#[cfg_attr(feature = "gen_write_cycle_query", derive(EnumIter))]
//#[cfg_attr(feature = "gen_write_cycle_query", strum_discriminants(derive(EnumIter)))]
pub enum InstructionSequenceMode2 {
    #[default]
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

impl From<InstructionSequenceMode2> for InstructionSequenceMode {
    fn from(mode: InstructionSequenceMode2) -> Self {
        match mode {
            InstructionSequenceMode2::FetchInstr => InstructionSequenceMode::FetchInstr,
            InstructionSequenceMode2::Break => InstructionSequenceMode::Break,
            InstructionSequenceMode2::StartIrq => InstructionSequenceMode::StartIrq,
            InstructionSequenceMode2::StartNmi => InstructionSequenceMode::StartNmi,
            InstructionSequenceMode2::Reset => InstructionSequenceMode::Reset,
            InstructionSequenceMode2::ReturnInterrupt => InstructionSequenceMode::ReturnInterrupt,
            InstructionSequenceMode2::JumpSubroutine => InstructionSequenceMode::JumpSubroutine,
            InstructionSequenceMode2::ReturnSubroutine => InstructionSequenceMode::ReturnSubroutine,
            InstructionSequenceMode2::Push(_) => InstructionSequenceMode::Push,
            InstructionSequenceMode2::Pull(_) => InstructionSequenceMode::Pull,
            InstructionSequenceMode2::Implied(_) => InstructionSequenceMode::Implied,
            InstructionSequenceMode2::Immediate(_) => InstructionSequenceMode::Immediate,
            InstructionSequenceMode2::AbsoluteJump => InstructionSequenceMode::AbsoluteJump,
            InstructionSequenceMode2::Absolute(_) => InstructionSequenceMode::Absolute,
            InstructionSequenceMode2::AbsoluteReadModifyWrite(_) => {
                InstructionSequenceMode::AbsoluteReadModifyWrite
            }
            InstructionSequenceMode2::ZeroPage(_) => InstructionSequenceMode::ZeroPage,
            InstructionSequenceMode2::ZeroPageReadModifyWrite(_) => {
                InstructionSequenceMode::ZeroPageReadModifyWrite
            }
            InstructionSequenceMode2::ZeroPageIdx(_, _) => InstructionSequenceMode::ZeroPageIdx,
            InstructionSequenceMode2::ZeroPageIdxReadModifyWrite(_, _) => {
                InstructionSequenceMode::ZeroPageIdxReadModifyWrite
            }
            InstructionSequenceMode2::AbsoluteIdxRead(_, _) => {
                InstructionSequenceMode::AbsoluteIdxRead
            }
            InstructionSequenceMode2::AbsoluteIdxReadModifyWrite(_, _) => {
                InstructionSequenceMode::AbsoluteIdxReadModifyWrite
            }
            InstructionSequenceMode2::AbsoluteIdxWrite(_, _) => {
                InstructionSequenceMode::AbsoluteIdxWrite
            }
            InstructionSequenceMode2::Relative(_) => InstructionSequenceMode::Relative,
            InstructionSequenceMode2::ZeroPageIdxIndirect(_, _) => {
                InstructionSequenceMode::ZeroPageIdxIndirect
            }
            InstructionSequenceMode2::ZeroPageIdxIndirectReadModifyWrite(_, _) => {
                InstructionSequenceMode::ZeroPageIdxIndirectReadModifyWrite
            }
            InstructionSequenceMode2::ZeroPageIndirectIdxRead(_, _) => {
                InstructionSequenceMode::ZeroPageIndirectIdxRead
            }
            InstructionSequenceMode2::ZeroPageIndirectIdxReadModifyWrite(_, _) => {
                InstructionSequenceMode::ZeroPageIndirectIdxReadModifyWrite
            }
            InstructionSequenceMode2::ZeroPageIndirectIdxWrite(_, _) => {
                InstructionSequenceMode::ZeroPageIndirectIdxWrite
            }
            InstructionSequenceMode2::AbsoluteIndirectJump => {
                InstructionSequenceMode::AbsoluteIndirectJump
            }
        }
    }
}