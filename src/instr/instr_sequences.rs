use crate::registers::IndexRegister;
use strum_macros::EnumDiscriminants;

use crate::instr::{
    BranchOperation, ImplicitOperation, InstructionOp, MemoryModifyOperation, PullStackOperation,
    PushStackOperation, RegisterMemoryOperation,
};
#[cfg(feature = "gen_write_cycle_query")]
use strum_macros::EnumIter;

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

impl Instruction {
    pub fn new(
        mode: InstructionSequenceMode,
        operation: InstructionOp,
        index: Option<IndexRegister>,
    ) -> Self {
        match mode {
            InstructionSequenceMode::Immediate => {
                Instruction::Immediate(operation.try_into().unwrap())
            }
            InstructionSequenceMode::ZeroPage => {
                Instruction::ZeroPage(operation.try_into().unwrap())
            }
            InstructionSequenceMode::Absolute => {
                Instruction::Absolute(operation.try_into().unwrap())
            }
            InstructionSequenceMode::ZeroPageIdxIndirect => {
                Instruction::ZeroPageIdxIndirect(operation.try_into().unwrap(), index.unwrap())
            }
            InstructionSequenceMode::ZeroPageIdx => {
                Instruction::ZeroPageIdx(operation.try_into().unwrap(), index.unwrap())
            }
            InstructionSequenceMode::AbsoluteIdxRead => {
                Instruction::AbsoluteIdxRead(operation.try_into().unwrap(), index.unwrap())
            }
            InstructionSequenceMode::AbsoluteIdxWrite => {
                Instruction::AbsoluteIdxWrite(operation.try_into().unwrap(), index.unwrap())
            }
            InstructionSequenceMode::ZeroPageIndirectIdxRead => {
                Instruction::ZeroPageIndirectIdxRead(operation.try_into().unwrap(), index.unwrap())
            }
            InstructionSequenceMode::ZeroPageIndirectIdxWrite => {
                Instruction::ZeroPageIndirectIdxWrite(operation.try_into().unwrap(), index.unwrap())
            }
            InstructionSequenceMode::ZeroPageReadModifyWrite => {
                Instruction::ZeroPageReadModifyWrite(operation.try_into().unwrap())
            }
            InstructionSequenceMode::AbsoluteReadModifyWrite => {
                Instruction::AbsoluteReadModifyWrite(operation.try_into().unwrap())
            }
            InstructionSequenceMode::ZeroPageIdxReadModifyWrite => {
                Instruction::ZeroPageIdxReadModifyWrite(
                    operation.try_into().unwrap(),
                    index.unwrap(),
                )
            }
            InstructionSequenceMode::AbsoluteIdxReadModifyWrite => {
                Instruction::AbsoluteIdxReadModifyWrite(
                    operation.try_into().unwrap(),
                    index.unwrap(),
                )
            }
            InstructionSequenceMode::Implied => Instruction::Implied(operation.try_into().unwrap()),
            InstructionSequenceMode::AbsoluteJump => Instruction::AbsoluteJump,
            InstructionSequenceMode::AbsoluteIndirectJump => Instruction::AbsoluteIndirectJump,
            InstructionSequenceMode::Relative => {
                Instruction::Relative(operation.try_into().unwrap())
            }
            InstructionSequenceMode::Push => Instruction::Push(operation.try_into().unwrap()),
            InstructionSequenceMode::Pull => Instruction::Pull(operation.try_into().unwrap()),
            InstructionSequenceMode::Break => Instruction::Break,
            InstructionSequenceMode::JumpSubroutine => Instruction::JumpSubroutine,
            InstructionSequenceMode::ReturnInterrupt => Instruction::ReturnInterrupt,
            InstructionSequenceMode::ReturnSubroutine => Instruction::ReturnSubroutine,
            InstructionSequenceMode::FetchInstr => Instruction::FetchInstr,
            InstructionSequenceMode::StartIrq => Instruction::StartIrq,
            InstructionSequenceMode::StartNmi => Instruction::StartNmi,
            InstructionSequenceMode::Reset => Instruction::Reset,
            InstructionSequenceMode::ZeroPageIdxIndirectReadModifyWrite => {
                Instruction::ZeroPageIdxIndirectReadModifyWrite(
                    operation.try_into().unwrap(),
                    index.unwrap(),
                )
            }
            InstructionSequenceMode::ZeroPageIndirectIdxReadModifyWrite => {
                Instruction::ZeroPageIndirectIdxReadModifyWrite(
                    operation.try_into().unwrap(),
                    index.unwrap(),
                )
            }
        }
    }
}
