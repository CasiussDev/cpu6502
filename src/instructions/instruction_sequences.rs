use crate::instructions::microinstructions::{MicroInstruction, MicroInstructionsVector};
use crate::registers::{SelectedRegister16, SelectedRegister8, StatusRegFlags};
//use once_cell::unsync::Lazy;
use lazy_static::lazy_static;
use std::collections::HashMap;

#[cfg(test)]
use strum::IntoEnumIterator;
#[cfg(test)]
use strum_macros::EnumIter;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
#[cfg_attr(test, derive(EnumIter))]
pub enum InstructionSequenceMode {
    Break,
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

    ZeroPageIndx,
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

type SequenceMap = HashMap<InstructionSequenceMode, MicroInstructionsVector>;

//static SEQUENCES_DEFS: Lazy<SequenceMap> = Lazy::new(|| { create_instruction_mode_sequences() });
lazy_static! {
    static ref MODES_SEQUENCES_DEFS: SequenceMap = create_instruction_mode_sequences();
}

impl Default for InstructionSequenceMode {
    fn default() -> Self {
        InstructionSequenceMode::Implied
    }
}

pub fn create_instruction_mode_sequences() -> SequenceMap {
    HashMap::from([
        (
            InstructionSequenceMode::Break,
            vec![
                MicroInstruction::ReadPC {
                    dst: SelectedRegister8::Discard,
                    increment: true,
                },
                MicroInstruction::YieldClock,
                // Next clock
                MicroInstruction::CopyRegister {
                    dst: SelectedRegister8::AddrHigh,
                    src: SelectedRegister8::StackPage,
                },
                MicroInstruction::CopyRegister {
                    dst: SelectedRegister8::AddrLow,
                    src: SelectedRegister8::SP,
                },
                MicroInstruction::WriteAddress {
                    src: SelectedRegister8::PCHigh,
                },
                MicroInstruction::DecrementRegister {
                    dst: SelectedRegister8::SP,
                },
                MicroInstruction::YieldClock,
                // Next clock
                MicroInstruction::CopyRegister {
                    dst: SelectedRegister8::AddrHigh,
                    src: SelectedRegister8::StackPage,
                },
                MicroInstruction::CopyRegister {
                    dst: SelectedRegister8::AddrLow,
                    src: SelectedRegister8::SP,
                },
                MicroInstruction::WriteAddress {
                    src: SelectedRegister8::PCLow,
                },
                MicroInstruction::DecrementRegister {
                    dst: SelectedRegister8::SP,
                },
                MicroInstruction::YieldClock,
                // Next clock
                MicroInstruction::SetStatusFlag {
                    flag: StatusRegFlags::BREAK,
                },
                MicroInstruction::CopyRegister {
                    dst: SelectedRegister8::AddrHigh,
                    src: SelectedRegister8::StackPage,
                },
                MicroInstruction::CopyRegister {
                    dst: SelectedRegister8::AddrLow,
                    src: SelectedRegister8::SP,
                },
                MicroInstruction::WriteAddress {
                    src: SelectedRegister8::Status,
                },
                MicroInstruction::DecrementRegister {
                    dst: SelectedRegister8::SP,
                },
                MicroInstruction::YieldClock,
                // Next clock
                MicroInstruction::CopyRegister16 {
                    dst: SelectedRegister16::Addr,
                    src: SelectedRegister16::InterruptAddrLow,
                },
                MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::PCLow,
                },
                MicroInstruction::YieldClock,
                // Next clock
                MicroInstruction::CopyRegister16 {
                    dst: SelectedRegister16::Addr,
                    src: SelectedRegister16::InterruptAddrHigh,
                },
                MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::PCHigh,
                },
                MicroInstruction::YieldClock,
                // Next clock
            ],
        ),
        (
            InstructionSequenceMode::ReturnInterrupt,
            vec![
                MicroInstruction::ReadPC {
                    dst: SelectedRegister8::Discard,
                    increment: false,
                },
                MicroInstruction::YieldClock,
                // Next clock
                MicroInstruction::IncrementRegister {
                    dst: SelectedRegister8::SP,
                },
                MicroInstruction::YieldClock,
                // Next clock
                MicroInstruction::CopyRegister {
                    dst: SelectedRegister8::AddrHigh,
                    src: SelectedRegister8::StackPage,
                },
                MicroInstruction::CopyRegister {
                    dst: SelectedRegister8::AddrLow,
                    src: SelectedRegister8::SP,
                },
                MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::Status,
                },
                MicroInstruction::IncrementRegister {
                    dst: SelectedRegister8::SP,
                },
                MicroInstruction::YieldClock,
                // Next clock
                MicroInstruction::CopyRegister {
                    dst: SelectedRegister8::AddrHigh,
                    src: SelectedRegister8::StackPage,
                },
                MicroInstruction::CopyRegister {
                    dst: SelectedRegister8::AddrLow,
                    src: SelectedRegister8::SP,
                },
                MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::PCLow,
                },
                MicroInstruction::IncrementRegister {
                    dst: SelectedRegister8::SP,
                },
                MicroInstruction::YieldClock,
                // Next clock
                MicroInstruction::CopyRegister {
                    dst: SelectedRegister8::AddrHigh,
                    src: SelectedRegister8::StackPage,
                },
                MicroInstruction::CopyRegister {
                    dst: SelectedRegister8::AddrLow,
                    src: SelectedRegister8::SP,
                },
                MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::PCHigh,
                },
                MicroInstruction::YieldClock,
            ],
        ),
        (
            InstructionSequenceMode::Push,
            vec![
                MicroInstruction::ReadPC {
                    dst: SelectedRegister8::Discard,
                    increment: false,
                },
                MicroInstruction::YieldClock,
                // Next clock
                MicroInstruction::CopyRegister {
                    dst: SelectedRegister8::AddrHigh,
                    src: SelectedRegister8::StackPage,
                },
                MicroInstruction::CopyRegister {
                    dst: SelectedRegister8::AddrLow,
                    src: SelectedRegister8::SP,
                },
                MicroInstruction::RunOperation,
                MicroInstruction::DecrementRegister {
                    dst: SelectedRegister8::SP,
                },
                MicroInstruction::YieldClock,
            ],
        ),
        (
            InstructionSequenceMode::Pull,
            vec![
                MicroInstruction::ReadPC {
                    dst: SelectedRegister8::Discard,
                    increment: false,
                },
                MicroInstruction::YieldClock,
                // Next clock
                MicroInstruction::IncrementRegister {
                    dst: SelectedRegister8::SP,
                },
                MicroInstruction::YieldClock,
                // Next clock
                MicroInstruction::CopyRegister {
                    dst: SelectedRegister8::AddrHigh,
                    src: SelectedRegister8::StackPage,
                },
                MicroInstruction::CopyRegister {
                    dst: SelectedRegister8::AddrLow,
                    src: SelectedRegister8::SP,
                },
                MicroInstruction::RunOperation,
                MicroInstruction::YieldClock,
            ],
        ),
        (
            InstructionSequenceMode::JumpSubroutine,
            vec![
                MicroInstruction::ReadPC {
                    dst: SelectedRegister8::Tmp,
                    increment: true,
                },
                MicroInstruction::YieldClock,
                // Next clock
                MicroInstruction::YieldClock,
                // Next clock
                MicroInstruction::CopyRegister {
                    dst: SelectedRegister8::AddrHigh,
                    src: SelectedRegister8::StackPage,
                },
                MicroInstruction::CopyRegister {
                    dst: SelectedRegister8::AddrLow,
                    src: SelectedRegister8::SP,
                },
                MicroInstruction::WriteAddress {
                    src: SelectedRegister8::PCHigh,
                },
                MicroInstruction::DecrementRegister {
                    dst: SelectedRegister8::SP,
                },
                MicroInstruction::YieldClock,
                // Next clock
                MicroInstruction::CopyRegister {
                    dst: SelectedRegister8::AddrHigh,
                    src: SelectedRegister8::StackPage,
                },
                MicroInstruction::CopyRegister {
                    dst: SelectedRegister8::AddrLow,
                    src: SelectedRegister8::SP,
                },
                MicroInstruction::WriteAddress {
                    src: SelectedRegister8::PCLow,
                },
                MicroInstruction::DecrementRegister {
                    dst: SelectedRegister8::SP,
                },
                MicroInstruction::YieldClock,
                // Next clock
                MicroInstruction::ReadPC {
                    dst: SelectedRegister8::PCHigh,
                    increment: true,
                },
                MicroInstruction::CopyRegister {
                    dst: SelectedRegister8::PCLow,
                    src: SelectedRegister8::Tmp,
                },
                MicroInstruction::YieldClock,
            ],
        ),
        (
            InstructionSequenceMode::ReturnSubroutine,
            vec![
                MicroInstruction::ReadPC {
                    dst: SelectedRegister8::Discard,
                    increment: false,
                },
                MicroInstruction::YieldClock,
                // Next clock
                MicroInstruction::IncrementRegister {
                    dst: SelectedRegister8::SP,
                },
                MicroInstruction::YieldClock,
                // Next clock
                MicroInstruction::CopyRegister {
                    dst: SelectedRegister8::AddrHigh,
                    src: SelectedRegister8::StackPage,
                },
                MicroInstruction::CopyRegister {
                    dst: SelectedRegister8::AddrLow,
                    src: SelectedRegister8::SP,
                },
                MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::Tmp,
                },
                MicroInstruction::IncrementRegister {
                    dst: SelectedRegister8::SP,
                },
                MicroInstruction::YieldClock,
                // Next clock
                MicroInstruction::CopyRegister {
                    dst: SelectedRegister8::AddrHigh,
                    src: SelectedRegister8::StackPage,
                },
                MicroInstruction::CopyRegister {
                    dst: SelectedRegister8::AddrLow,
                    src: SelectedRegister8::SP,
                },
                MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::PCHigh,
                },
                MicroInstruction::CopyRegister {
                    dst: SelectedRegister8::PCLow,
                    src: SelectedRegister8::Tmp,
                },
                MicroInstruction::YieldClock,
                // Next clock
                MicroInstruction::IncrementRegister16 {
                    dst: SelectedRegister16::PC,
                },
                MicroInstruction::YieldClock,
            ],
        ),
        (
            InstructionSequenceMode::Implied,
            vec![
                MicroInstruction::ReadPC {
                    dst: SelectedRegister8::Discard,
                    increment: false,
                },
                MicroInstruction::RunOperation,
                MicroInstruction::YieldClock,
            ],
        ),
        (
            InstructionSequenceMode::Immediate,
            vec![
                MicroInstruction::CopyRegister {
                    dst: SelectedRegister8::AddrLow,
                    src: SelectedRegister8::PCLow,
                },
                MicroInstruction::CopyRegister {
                    dst: SelectedRegister8::AddrHigh,
                    src: SelectedRegister8::PCHigh,
                },
                MicroInstruction::IncrementRegister16 {
                    dst: SelectedRegister16::PC,
                },
                MicroInstruction::RunOperation,
                MicroInstruction::YieldClock,
            ],
        ),
        (
            InstructionSequenceMode::AbsoluteJump,
            vec![
                MicroInstruction::ReadPC {
                    dst: SelectedRegister8::AddrLow,
                    increment: true,
                },
                MicroInstruction::YieldClock,
                // Next clock
                MicroInstruction::ReadPC {
                    dst: SelectedRegister8::PCHigh,
                    increment: true,
                },
                MicroInstruction::CopyRegister {
                    dst: SelectedRegister8::PCLow,
                    src: SelectedRegister8::AddrLow,
                },
                MicroInstruction::YieldClock,
            ],
        ),
        (
            InstructionSequenceMode::Absolute,
            vec![
                MicroInstruction::ReadPC {
                    dst: SelectedRegister8::AddrLow,
                    increment: true,
                },
                MicroInstruction::YieldClock,
                // Next clock
                MicroInstruction::ReadPC {
                    dst: SelectedRegister8::AddrHigh,
                    increment: true,
                },
                MicroInstruction::YieldClock,
                // Next clock
                MicroInstruction::RunOperation,
                MicroInstruction::YieldClock,
            ],
        ),
        (
            InstructionSequenceMode::AbsoluteReadModifyWrite,
            vec![
                MicroInstruction::ReadPC {
                    dst: SelectedRegister8::AddrLow,
                    increment: true,
                },
                MicroInstruction::YieldClock,
                // Next clock
                MicroInstruction::ReadPC {
                    dst: SelectedRegister8::AddrHigh,
                    increment: true,
                },
                MicroInstruction::YieldClock,
                // Next clock
                MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::Tmp,
                },
                MicroInstruction::YieldClock,
                // Next clock
                MicroInstruction::WriteAddress {
                    src: SelectedRegister8::Tmp,
                },
                MicroInstruction::RunOperation,
                MicroInstruction::YieldClock,
                // Next clock
                MicroInstruction::WriteAddress {
                    src: SelectedRegister8::Tmp,
                },
                MicroInstruction::YieldClock,
            ],
        ),
        (
            InstructionSequenceMode::ZeroPage,
            vec![
                MicroInstruction::ZeroRegister {
                    dst: SelectedRegister8::AddrHigh,
                },
                MicroInstruction::ReadPC {
                    dst: SelectedRegister8::AddrLow,
                    increment: true,
                },
                MicroInstruction::YieldClock,
                // Next clock
                MicroInstruction::RunOperation,
                MicroInstruction::YieldClock,
            ],
        ),
        (
            InstructionSequenceMode::ZeroPageReadModifyWrite,
            vec![
                MicroInstruction::ZeroRegister {
                    dst: SelectedRegister8::AddrHigh,
                },
                MicroInstruction::ReadPC {
                    dst: SelectedRegister8::AddrLow,
                    increment: true,
                },
                MicroInstruction::YieldClock,
                // Next clock
                MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::Tmp,
                },
                MicroInstruction::YieldClock,
                // Next clock
                MicroInstruction::WriteAddress {
                    src: SelectedRegister8::Tmp,
                },
                MicroInstruction::RunOperation,
                MicroInstruction::YieldClock,
                // Next clock
                MicroInstruction::WriteAddress {
                    src: SelectedRegister8::Tmp,
                },
                MicroInstruction::YieldClock,
            ],
        ),
        (
            InstructionSequenceMode::ZeroPageIndx,
            vec![
                MicroInstruction::ZeroRegister {
                    dst: SelectedRegister8::AddrHigh,
                },
                MicroInstruction::ReadPC {
                    dst: SelectedRegister8::AddrLow,
                    increment: true,
                },
                MicroInstruction::YieldClock,
                // Next clock
                MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::Discard,
                },
                MicroInstruction::AddIndexToAddress,
                MicroInstruction::YieldClock,
                // Next clock
                MicroInstruction::RunOperation,
                MicroInstruction::YieldClock,
            ],
        ),
        (
            InstructionSequenceMode::ZeroPageIdxReadModifyWrite,
            vec![
                MicroInstruction::ZeroRegister {
                    dst: SelectedRegister8::AddrHigh,
                },
                MicroInstruction::ReadPC {
                    dst: SelectedRegister8::AddrLow,
                    increment: true,
                },
                MicroInstruction::YieldClock,
                // Next clock
                MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::Discard,
                },
                MicroInstruction::AddIndexToAddress,
                MicroInstruction::YieldClock,
                // Next clock
                MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::Tmp,
                },
                MicroInstruction::YieldClock,
                // Next clock
                MicroInstruction::WriteAddress {
                    src: SelectedRegister8::Tmp,
                },
                MicroInstruction::RunOperation,
                MicroInstruction::YieldClock,
                // Next clock
                MicroInstruction::WriteAddress {
                    src: SelectedRegister8::Tmp,
                },
                MicroInstruction::YieldClock,
            ],
        ),
        (
            InstructionSequenceMode::AbsoluteIdxRead,
            vec![
                MicroInstruction::ReadPC {
                    dst: SelectedRegister8::AddrLow,
                    increment: true,
                },
                MicroInstruction::YieldClock,
                // Next clock
                MicroInstruction::ReadPC {
                    dst: SelectedRegister8::AddrHigh,
                    increment: true,
                },
                MicroInstruction::YieldClock,
                // Next clock
                MicroInstruction::AddIndexToAddress,
                MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::Tmp,
                },
                MicroInstruction::FixAddressOrRunOpAndFinish,
                MicroInstruction::YieldClock,
                // Next clock
                MicroInstruction::RunOperation,
                MicroInstruction::YieldClock,
                // Next clock
            ],
        ),
        (
            InstructionSequenceMode::AbsoluteIdxReadModifyWrite,
            vec![
                MicroInstruction::ReadPC {
                    dst: SelectedRegister8::AddrLow,
                    increment: true,
                },
                MicroInstruction::YieldClock,
                // Next clock
                MicroInstruction::ReadPC {
                    dst: SelectedRegister8::AddrHigh,
                    increment: true,
                },
                MicroInstruction::YieldClock,
                MicroInstruction::AddIndexToAddress,
                // Next clock
                MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::Discard,
                },
                MicroInstruction::FixAddress,
                MicroInstruction::YieldClock,
                // Next clock
                MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::Tmp,
                },
                MicroInstruction::YieldClock,
                // Next clock
                MicroInstruction::WriteAddress {
                    src: SelectedRegister8::Tmp,
                },
                MicroInstruction::RunOperation,
                MicroInstruction::YieldClock,
                // Next clock
                MicroInstruction::WriteAddress {
                    src: SelectedRegister8::Tmp,
                },
                MicroInstruction::YieldClock,
                // Next clock
            ],
        ),
        (
            InstructionSequenceMode::AbsoluteIdxWrite,
            vec![
                MicroInstruction::ReadPC {
                    dst: SelectedRegister8::AddrLow,
                    increment: true,
                },
                MicroInstruction::YieldClock,
                // Next clock
                MicroInstruction::ReadPC {
                    dst: SelectedRegister8::AddrHigh,
                    increment: true,
                },
                MicroInstruction::YieldClock,
                // Next clock
                MicroInstruction::AddIndexToAddress,
                MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::Tmp,
                },
                MicroInstruction::FixAddress,
                MicroInstruction::YieldClock,
                // Next clock
                MicroInstruction::RunOperation,
                MicroInstruction::YieldClock,
                // Next clock
            ],
        ),
        (
            InstructionSequenceMode::Relative,
            vec![
                MicroInstruction::ReadPC {
                    dst: SelectedRegister8::Tmp,
                    increment: true,
                },
                MicroInstruction::YieldClock,
                // Next clock
                MicroInstruction::ReadPC {
                    dst: SelectedRegister8::IR,
                    increment: false,
                },
                MicroInstruction::RunOperation,
                MicroInstruction::YieldClock,
                // Next clock
                MicroInstruction::ReadPC {
                    dst: SelectedRegister8::IR,
                    increment: false,
                },
                MicroInstruction::FixAddressOrIncrementPC,
                MicroInstruction::YieldClock,
            ],
        ),
        (
            InstructionSequenceMode::ZeroPageIdxIndirect,
            vec![
                MicroInstruction::ZeroRegister {
                    dst: SelectedRegister8::AddrHigh,
                },
                MicroInstruction::ReadPC {
                    dst: SelectedRegister8::AddrLow,
                    increment: true,
                },
                MicroInstruction::YieldClock,
                // Next Clock
                MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::Discard,
                },
                MicroInstruction::AddIndexToAddress,
                MicroInstruction::YieldClock,
                // Next Clock
                MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::Tmp,
                },
                MicroInstruction::YieldClock,
                // Next Clock
                MicroInstruction::IncrementRegister {
                    dst: SelectedRegister8::AddrLow,
                },
                MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::AddrHigh,
                },
                MicroInstruction::CopyRegister {
                    dst: SelectedRegister8::AddrLow,
                    src: SelectedRegister8::Tmp,
                },
                MicroInstruction::YieldClock,
                // Next Clock
                MicroInstruction::RunOperation,
                MicroInstruction::YieldClock,
            ],
        ),
        (
            InstructionSequenceMode::ZeroPageIdxIndirectReadModifyWrite,
            vec![
                MicroInstruction::ZeroRegister {
                    dst: SelectedRegister8::AddrHigh,
                },
                MicroInstruction::ReadPC {
                    dst: SelectedRegister8::AddrLow,
                    increment: true,
                },
                MicroInstruction::YieldClock,
                // Next Clock
                MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::Discard,
                },
                MicroInstruction::AddIndexToAddress,
                MicroInstruction::YieldClock,
                // Next Clock
                MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::Tmp,
                },
                MicroInstruction::YieldClock,
                // Next Clock
                MicroInstruction::IncrementRegister {
                    dst: SelectedRegister8::AddrLow,
                },
                MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::AddrHigh,
                },
                MicroInstruction::CopyRegister {
                    dst: SelectedRegister8::AddrLow,
                    src: SelectedRegister8::Tmp,
                },
                MicroInstruction::YieldClock,
                // Next Clock
                MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::Tmp,
                },
                MicroInstruction::YieldClock,
                // Next Clock
                MicroInstruction::WriteAddress {
                    src: SelectedRegister8::Tmp,
                },
                MicroInstruction::RunOperation,
                MicroInstruction::YieldClock,
                // Next Clock
                MicroInstruction::WriteAddress {
                    src: SelectedRegister8::Tmp,
                },
                MicroInstruction::YieldClock,
            ],
        ),
        (
            InstructionSequenceMode::ZeroPageIndirectIdxRead,
            vec![
                MicroInstruction::ZeroRegister {
                    dst: SelectedRegister8::AddrHigh,
                },
                MicroInstruction::ReadPC {
                    dst: SelectedRegister8::AddrLow,
                    increment: true,
                },
                MicroInstruction::YieldClock,
                // Next Clock
                MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::Tmp,
                },
                MicroInstruction::YieldClock,
                // Next Clock
                MicroInstruction::IncrementRegister {
                    dst: SelectedRegister8::AddrLow,
                },
                MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::AddrHigh,
                },
                MicroInstruction::CopyRegister {
                    dst: SelectedRegister8::AddrLow,
                    src: SelectedRegister8::Tmp,
                },
                MicroInstruction::AddIndexToAddress,
                MicroInstruction::YieldClock,
                // Next Clock
                MicroInstruction::FixAddressOrRunOpAndFinish,
                MicroInstruction::YieldClock,
                // Next Clock
                MicroInstruction::RunOperation,
                MicroInstruction::YieldClock,
                // Next Clock
            ],
        ),
        (
            InstructionSequenceMode::ZeroPageIndirectIdxReadModifyWrite,
            vec![
                MicroInstruction::ZeroRegister {
                    dst: SelectedRegister8::AddrHigh,
                },
                MicroInstruction::ReadPC {
                    dst: SelectedRegister8::AddrLow,
                    increment: true,
                },
                MicroInstruction::YieldClock,
                // Next Clock
                MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::Tmp,
                },
                MicroInstruction::YieldClock,
                // Next Clock
                MicroInstruction::IncrementRegister {
                    dst: SelectedRegister8::AddrLow,
                },
                MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::AddrHigh,
                },
                MicroInstruction::CopyRegister {
                    dst: SelectedRegister8::AddrLow,
                    src: SelectedRegister8::Tmp,
                },
                MicroInstruction::AddIndexToAddress,
                MicroInstruction::YieldClock,
                // Next Clock
                MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::Discard,
                },
                MicroInstruction::FixAddress,
                MicroInstruction::YieldClock,
                // Next Clock
                MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::Tmp,
                },
                MicroInstruction::YieldClock,
                // Next Clock
                MicroInstruction::WriteAddress {
                    src: SelectedRegister8::Tmp,
                },
                MicroInstruction::RunOperation,
                MicroInstruction::YieldClock,
                // Next Clock
                MicroInstruction::WriteAddress {
                    src: SelectedRegister8::Tmp,
                },
                MicroInstruction::YieldClock,
                // Next Clock
            ],
        ),
        (
            InstructionSequenceMode::ZeroPageIndirectIdxWrite,
            vec![
                MicroInstruction::ZeroRegister {
                    dst: SelectedRegister8::AddrHigh,
                },
                MicroInstruction::ReadPC {
                    dst: SelectedRegister8::AddrLow,
                    increment: true,
                },
                MicroInstruction::YieldClock,
                // Next Clock
                MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::Tmp,
                },
                MicroInstruction::YieldClock,
                // Next Clock
                MicroInstruction::IncrementRegister {
                    dst: SelectedRegister8::AddrLow,
                },
                MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::AddrHigh,
                },
                MicroInstruction::CopyRegister {
                    dst: SelectedRegister8::AddrLow,
                    src: SelectedRegister8::Tmp,
                },
                MicroInstruction::AddIndexToAddress,
                MicroInstruction::YieldClock,
                // Next Clock
                MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::Discard,
                },
                MicroInstruction::FixAddress,
                MicroInstruction::YieldClock,
                // Next Clock
                MicroInstruction::RunOperation,
                MicroInstruction::YieldClock,
                // Next Clock
            ],
        ),
        (
            InstructionSequenceMode::AbsoluteIndirectJump,
            vec![
                MicroInstruction::ReadPC {
                    dst: SelectedRegister8::Tmp,
                    increment: true,
                },
                MicroInstruction::YieldClock,
                // Next Clock
                MicroInstruction::ReadPC {
                    dst: SelectedRegister8::AddrHigh,
                    increment: true,
                },
                MicroInstruction::CopyRegister {
                    dst: SelectedRegister8::AddrLow,
                    src: SelectedRegister8::Tmp,
                },
                MicroInstruction::YieldClock,
                // Next Clock
                MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::Tmp,
                },
                MicroInstruction::YieldClock,
                // Next Clock
                MicroInstruction::IncrementRegister {
                    dst: SelectedRegister8::AddrLow,
                },
                MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::PCHigh,
                },
                MicroInstruction::CopyRegister {
                    dst: SelectedRegister8::PCLow,
                    src: SelectedRegister8::Tmp,
                },
                MicroInstruction::YieldClock,
            ],
        ),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instructionsequences_checklastmicroinstruction_yieldclock() {
        for (mode, sequence) in MODES_SEQUENCES_DEFS.iter() {
            let last_microinstruction = sequence.last().unwrap_or_else(|| {
                panic!(
                    "Sequence mode {:?} does not have any microinstruction",
                    mode
                )
            });
            assert_eq!(
                *last_microinstruction,
                MicroInstruction::YieldClock,
                "Sequence mode {:?} does not finish with YieldClock",
                mode
            );
        }
    }

    #[test]
    fn check_all_instruction_modes_implemented() {
        let sequences = &*MODES_SEQUENCES_DEFS;
        for mode in InstructionSequenceMode::iter() {
            assert!(
                sequences.contains_key(&mode),
                "Mode {:?} not implemented",
                mode,
            )
        }
    }

    #[test]
    fn check_readmodifywrite_instructions_use_tmp_reg() {
        let sequences = &*MODES_SEQUENCES_DEFS;

        let modes = [
            InstructionSequenceMode::AbsoluteReadModifyWrite,
            InstructionSequenceMode::ZeroPageReadModifyWrite,
            InstructionSequenceMode::ZeroPageIdxReadModifyWrite,
            InstructionSequenceMode::AbsoluteIdxReadModifyWrite,
            InstructionSequenceMode::ZeroPageIdxIndirectReadModifyWrite,
            InstructionSequenceMode::ZeroPageIndirectIdxReadModifyWrite,
        ];

        for mode in &modes {
            if let Some(sequence) = sequences.get(mode) {
                let runop_position = sequence
                    .iter()
                    .position(|&instr| instr == MicroInstruction::RunOperation);

                if let Some(position) = runop_position {
                    let last_memory_read_before_op = sequence[..position]
                        .iter()
                        .rev()
                        .find(|&instr| matches!(instr, MicroInstruction::ReadAddress { .. }));

                    assert_eq!(
                        last_memory_read_before_op,
                        Some(&MicroInstruction::ReadAddress {
                            dst: SelectedRegister8::Tmp
                        })
                    );

                    let next_memory_write_after_op = sequence[position..]
                        .iter()
                        .find(|&instr| matches!(instr, MicroInstruction::WriteAddress { .. }));

                    assert_eq!(
                        next_memory_write_after_op,
                        Some(&MicroInstruction::WriteAddress {
                            src: SelectedRegister8::Tmp
                        })
                    );
                }
            }
        }
    }
}
