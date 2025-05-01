use crate::alu;
use crate::instr;
use crate::registers::{ReferenceableRegister8, SelectedRegister8, StatusRegFlags};
use std::convert::TryFrom;
use std::{collections, slice};

#[cfg(test)]
use strum::IntoEnumIterator;
#[cfg(any(test, feature = "gen_write_cycle_query"))]
use strum_macros::EnumIter;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[cfg_attr(feature = "gen_write_cycle_query", derive(EnumIter))]
pub enum ImplicitOperation {
    #[default]
    Nop,
    ShiftLeftA,
    ShiftRightA,
    RotateLeftA,
    RotateRightA,
    //IncrementMemory,
    IncrementX,
    IncrementY,
    //DecrementMemory,
    DecrementX,
    DecrementY,
    ClearCarry,
    SetCarry,
    ClearDecimal,
    SetDecimal,
    ClearInterruptDisable,
    SetInterruptDisable,
    ClearOverflow,
    SetOverflow,
    TransferAccumulatorToX,
    TransferAccumulatorToY,
    TransferStackPtrToX,
    TransferXToAccumulator,
    TransferYToAccumulator,
    TransferXToStackPtr,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[cfg_attr(feature = "gen_write_cycle_query", derive(EnumIter))]
pub enum BranchOperation {
    #[default]
    BranchPlus,
    BranchMinus,
    BranchOverflowClear,
    BranchOverflowSet,
    BranchCarryClear,
    BranchCarrySet,
    BranchNotEqual,
    BranchEqual,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[cfg_attr(feature = "gen_write_cycle_query", derive(EnumIter))]
pub enum MemoryModifyOperation {
    #[default]
    IncrementMemory,
    DecrementMemory,
    ShiftLeftMemory,
    ShiftRightMemory,
    RotateLeftMemory,
    RotateRightMemory,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[cfg_attr(feature = "gen_write_cycle_query", derive(EnumIter))]
pub enum RegisterMemoryOperation {
    #[default]
    StoreA,
    LoadA,
    StoreX,
    LoadX,
    StoreY,
    LoadY,
    Bit,
    BitImmediate,
    Or,
    And,
    Xor,
    Add,
    Sub,
    Cmp,
    Cpx,
    Cpy,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[cfg_attr(feature = "gen_write_cycle_query", derive(EnumIter))]
pub enum PushStackOperation {
    #[default]
    PushA,
    PushStatus,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[cfg_attr(feature = "gen_write_cycle_query", derive(EnumIter))]
pub enum PullStackOperation {
    #[default]
    PullA,
    PullStatus,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Default)]
#[cfg_attr(test, derive(EnumIter))]
pub enum InstructionOp {
    #[default]
    Nop,
    IncrementMemory,
    IncrementX,
    IncrementY,
    DecrementMemory,
    DecrementX,
    DecrementY,
    ClearCarry,
    SetCarry,
    ClearDecimal,
    SetDecimal,
    ClearInterruptDisable,
    SetInterruptDisable,
    ClearOverflow,
    SetOverflow,
    TransferAccumulatorToX,
    TransferAccumulatorToY,
    TransferStackPtrToX,
    TransferXToAccumulator,
    TransferYToAccumulator,
    TransferXToStackPtr,
    PushA,
    PushStatus,
    PullA,
    PullStatus,
    Or,
    And,
    Xor,
    Add,
    Sub,
    Cmp,
    Cpx,
    Cpy,
    Bit,
    BitImmediate,
    ShiftLeftA,
    ShiftRightA,
    RotateLeftA,
    RotateRightA,
    ShiftLeftMemory,
    ShiftRightMemory,
    RotateLeftMemory,
    RotateRightMemory,
    StoreA,
    LoadA,
    StoreX,
    LoadX,
    StoreY,
    LoadY,
    BranchPlus,
    BranchMinus,
    BranchOverflowClear,
    BranchOverflowSet,
    BranchCarryClear,
    BranchCarrySet,
    BranchNotEqual,
    BranchEqual,
}

impl From<ImplicitOperation> for InstructionOp {
    fn from(op: ImplicitOperation) -> Self {
        match op {
            ImplicitOperation::Nop => InstructionOp::Nop,
            ImplicitOperation::ShiftLeftA => InstructionOp::ShiftLeftA,
            ImplicitOperation::ShiftRightA => InstructionOp::ShiftRightA,
            ImplicitOperation::RotateLeftA => InstructionOp::RotateLeftA,
            ImplicitOperation::RotateRightA => InstructionOp::RotateRightA,
            ImplicitOperation::IncrementX => InstructionOp::IncrementX,
            ImplicitOperation::IncrementY => InstructionOp::IncrementY,
            ImplicitOperation::DecrementX => InstructionOp::DecrementX,
            ImplicitOperation::DecrementY => InstructionOp::DecrementY,
            ImplicitOperation::ClearCarry => InstructionOp::ClearCarry,
            ImplicitOperation::SetCarry => InstructionOp::SetCarry,
            ImplicitOperation::ClearDecimal => InstructionOp::ClearDecimal,
            ImplicitOperation::SetDecimal => InstructionOp::SetDecimal,
            ImplicitOperation::ClearInterruptDisable => InstructionOp::ClearInterruptDisable,
            ImplicitOperation::SetInterruptDisable => InstructionOp::SetInterruptDisable,
            ImplicitOperation::ClearOverflow => InstructionOp::ClearOverflow,
            ImplicitOperation::SetOverflow => InstructionOp::SetOverflow,
            ImplicitOperation::TransferAccumulatorToX => InstructionOp::TransferAccumulatorToX,
            ImplicitOperation::TransferAccumulatorToY => InstructionOp::TransferAccumulatorToY,
            ImplicitOperation::TransferStackPtrToX => InstructionOp::TransferStackPtrToX,
            ImplicitOperation::TransferXToAccumulator => InstructionOp::TransferXToAccumulator,
            ImplicitOperation::TransferYToAccumulator => InstructionOp::TransferYToAccumulator,
            ImplicitOperation::TransferXToStackPtr => InstructionOp::TransferXToStackPtr,
        }
    }
}

impl From<BranchOperation> for InstructionOp {
    fn from(op: BranchOperation) -> Self {
        match op {
            BranchOperation::BranchPlus => InstructionOp::BranchPlus,
            BranchOperation::BranchMinus => InstructionOp::BranchMinus,
            BranchOperation::BranchOverflowClear => InstructionOp::BranchOverflowClear,
            BranchOperation::BranchOverflowSet => InstructionOp::BranchOverflowSet,
            BranchOperation::BranchCarryClear => InstructionOp::BranchCarryClear,
            BranchOperation::BranchCarrySet => InstructionOp::BranchCarrySet,
            BranchOperation::BranchNotEqual => InstructionOp::BranchNotEqual,
            BranchOperation::BranchEqual => InstructionOp::BranchEqual,
        }
    }
}

impl From<MemoryModifyOperation> for InstructionOp {
    fn from(op: MemoryModifyOperation) -> Self {
        match op {
            MemoryModifyOperation::IncrementMemory => InstructionOp::IncrementMemory,
            MemoryModifyOperation::DecrementMemory => InstructionOp::DecrementMemory,
            MemoryModifyOperation::ShiftLeftMemory => InstructionOp::ShiftLeftMemory,
            MemoryModifyOperation::ShiftRightMemory => InstructionOp::ShiftRightMemory,
            MemoryModifyOperation::RotateLeftMemory => InstructionOp::RotateLeftMemory,
            MemoryModifyOperation::RotateRightMemory => InstructionOp::RotateRightMemory,
        }
    }
}

impl From<RegisterMemoryOperation> for InstructionOp {
    fn from(op: RegisterMemoryOperation) -> Self {
        match op {
            RegisterMemoryOperation::StoreA => InstructionOp::StoreA,
            RegisterMemoryOperation::LoadA => InstructionOp::LoadA,
            RegisterMemoryOperation::StoreX => InstructionOp::StoreX,
            RegisterMemoryOperation::LoadX => InstructionOp::LoadX,
            RegisterMemoryOperation::StoreY => InstructionOp::StoreY,
            RegisterMemoryOperation::LoadY => InstructionOp::LoadY,
            RegisterMemoryOperation::Bit => InstructionOp::Bit,
            RegisterMemoryOperation::BitImmediate => InstructionOp::BitImmediate,
            RegisterMemoryOperation::Or => InstructionOp::Or,
            RegisterMemoryOperation::And => InstructionOp::And,
            RegisterMemoryOperation::Xor => InstructionOp::Xor,
            RegisterMemoryOperation::Add => InstructionOp::Add,
            RegisterMemoryOperation::Sub => InstructionOp::Sub,
            RegisterMemoryOperation::Cmp => InstructionOp::Cmp,
            RegisterMemoryOperation::Cpx => InstructionOp::Cpx,
            RegisterMemoryOperation::Cpy => InstructionOp::Cpy,
        }
    }
}

impl From<PushStackOperation> for InstructionOp {
    fn from(op: PushStackOperation) -> Self {
        match op {
            PushStackOperation::PushA => InstructionOp::PushA,
            PushStackOperation::PushStatus => InstructionOp::PushStatus,
        }
    }
}

impl From<PullStackOperation> for InstructionOp {
    fn from(op: PullStackOperation) -> Self {
        match op {
            PullStackOperation::PullA => InstructionOp::PullA,
            PullStackOperation::PullStatus => InstructionOp::PullStatus,
        }
    }
}

impl TryFrom<InstructionOp> for ImplicitOperation {
    type Error = &'static str;
    fn try_from(op: InstructionOp) -> Result<Self, Self::Error> {
        match op {
            InstructionOp::Nop => Ok(ImplicitOperation::Nop),
            InstructionOp::ShiftLeftA => Ok(ImplicitOperation::ShiftLeftA),
            InstructionOp::ShiftRightA => Ok(ImplicitOperation::ShiftRightA),
            InstructionOp::RotateLeftA => Ok(ImplicitOperation::RotateLeftA),
            InstructionOp::RotateRightA => Ok(ImplicitOperation::RotateRightA),
            InstructionOp::IncrementX => Ok(ImplicitOperation::IncrementX),
            InstructionOp::IncrementY => Ok(ImplicitOperation::IncrementY),
            InstructionOp::DecrementX => Ok(ImplicitOperation::DecrementX),
            InstructionOp::DecrementY => Ok(ImplicitOperation::DecrementY),
            InstructionOp::ClearCarry => Ok(ImplicitOperation::ClearCarry),
            InstructionOp::SetCarry => Ok(ImplicitOperation::SetCarry),
            InstructionOp::ClearDecimal => Ok(ImplicitOperation::ClearDecimal),
            InstructionOp::SetDecimal => Ok(ImplicitOperation::SetDecimal),
            InstructionOp::ClearInterruptDisable => Ok(ImplicitOperation::ClearInterruptDisable),
            InstructionOp::SetInterruptDisable => Ok(ImplicitOperation::SetInterruptDisable),
            InstructionOp::ClearOverflow => Ok(ImplicitOperation::ClearOverflow),
            InstructionOp::SetOverflow => Ok(ImplicitOperation::SetOverflow),
            InstructionOp::TransferAccumulatorToX => Ok(ImplicitOperation::TransferAccumulatorToX),
            InstructionOp::TransferAccumulatorToY => Ok(ImplicitOperation::TransferAccumulatorToY),
            InstructionOp::TransferStackPtrToX => Ok(ImplicitOperation::TransferStackPtrToX),
            InstructionOp::TransferXToAccumulator => Ok(ImplicitOperation::TransferXToAccumulator),
            InstructionOp::TransferYToAccumulator => Ok(ImplicitOperation::TransferYToAccumulator),
            InstructionOp::TransferXToStackPtr => Ok(ImplicitOperation::TransferXToStackPtr),
            _ => Err("No matching ImplicitOperation"),
        }
    }
}

impl TryFrom<InstructionOp> for BranchOperation {
    type Error = &'static str;
    fn try_from(op: InstructionOp) -> Result<Self, Self::Error> {
        match op {
            InstructionOp::BranchPlus => Ok(BranchOperation::BranchPlus),
            InstructionOp::BranchMinus => Ok(BranchOperation::BranchMinus),
            InstructionOp::BranchOverflowClear => Ok(BranchOperation::BranchOverflowClear),
            InstructionOp::BranchOverflowSet => Ok(BranchOperation::BranchOverflowSet),
            InstructionOp::BranchCarryClear => Ok(BranchOperation::BranchCarryClear),
            InstructionOp::BranchCarrySet => Ok(BranchOperation::BranchCarrySet),
            InstructionOp::BranchNotEqual => Ok(BranchOperation::BranchNotEqual),
            InstructionOp::BranchEqual => Ok(BranchOperation::BranchEqual),
            _ => Err("No matching BranchOperation"),
        }
    }
}

impl TryFrom<InstructionOp> for MemoryModifyOperation {
    type Error = &'static str;
    fn try_from(op: InstructionOp) -> Result<Self, Self::Error> {
        match op {
            InstructionOp::IncrementMemory => Ok(MemoryModifyOperation::IncrementMemory),
            InstructionOp::DecrementMemory => Ok(MemoryModifyOperation::DecrementMemory),
            InstructionOp::ShiftLeftMemory => Ok(MemoryModifyOperation::ShiftLeftMemory),
            InstructionOp::ShiftRightMemory => Ok(MemoryModifyOperation::ShiftRightMemory),
            InstructionOp::RotateLeftMemory => Ok(MemoryModifyOperation::RotateLeftMemory),
            InstructionOp::RotateRightMemory => Ok(MemoryModifyOperation::RotateRightMemory),
            _ => Err("No matching MemoryModifyOperation"),
        }
    }
}

impl TryFrom<InstructionOp> for RegisterMemoryOperation {
    type Error = &'static str;
    fn try_from(op: InstructionOp) -> Result<Self, Self::Error> {
        match op {
            InstructionOp::StoreA => Ok(RegisterMemoryOperation::StoreA),
            InstructionOp::LoadA => Ok(RegisterMemoryOperation::LoadA),
            InstructionOp::StoreX => Ok(RegisterMemoryOperation::StoreX),
            InstructionOp::LoadX => Ok(RegisterMemoryOperation::LoadX),
            InstructionOp::StoreY => Ok(RegisterMemoryOperation::StoreY),
            InstructionOp::LoadY => Ok(RegisterMemoryOperation::LoadY),
            InstructionOp::Bit => Ok(RegisterMemoryOperation::Bit),
            InstructionOp::BitImmediate => Ok(RegisterMemoryOperation::BitImmediate),
            InstructionOp::Or => Ok(RegisterMemoryOperation::Or),
            InstructionOp::And => Ok(RegisterMemoryOperation::And),
            InstructionOp::Xor => Ok(RegisterMemoryOperation::Xor),
            InstructionOp::Add => Ok(RegisterMemoryOperation::Add),
            InstructionOp::Sub => Ok(RegisterMemoryOperation::Sub),
            InstructionOp::Cmp => Ok(RegisterMemoryOperation::Cmp),
            InstructionOp::Cpx => Ok(RegisterMemoryOperation::Cpx),
            InstructionOp::Cpy => Ok(RegisterMemoryOperation::Cpy),
            _ => Err("No matching RegisterMemoryOperation"),
        }
    }
}

impl TryFrom<InstructionOp> for PushStackOperation {
    type Error = &'static str;
    fn try_from(op: InstructionOp) -> Result<Self, Self::Error> {
        match op {
            InstructionOp::PushA => Ok(PushStackOperation::PushA),
            InstructionOp::PushStatus => Ok(PushStackOperation::PushStatus),
            _ => Err("No matching PushStackOperation"),
        }
    }
}

impl TryFrom<InstructionOp> for PullStackOperation {
    type Error = &'static str;
    fn try_from(op: InstructionOp) -> Result<Self, Self::Error> {
        match op {
            InstructionOp::PullA => Ok(PullStackOperation::PullA),
            InstructionOp::PullStatus => Ok(PullStackOperation::PullStatus),
            _ => Err("No matching PullStackOperation"),
        }
    }
}
