use crate::instr;
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

type SequenceMap = HashMap<InstructionSequenceMode, instr::MicroInstructionsVector>;

//static SEQUENCES_DEFS: Lazy<SequenceMap> = Lazy::new(|| { create_instruction_mode_sequences() });
lazy_static! {
    static ref MODES_SEQUENCES_DEFS: SequenceMap = create_instruction_mode_sequences();
}

pub fn get_sequences_map() -> &'static SequenceMap {
    &MODES_SEQUENCES_DEFS
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
                instr::MicroInstruction::ReadPC {
                    dst: SelectedRegister8::Discard,
                    increment: true,
                },
                instr::MicroInstruction::YieldClock,
                // Next clock
                instr::MicroInstruction::CopyRegister {
                    dst: SelectedRegister8::AddrHigh,
                    src: SelectedRegister8::StackPage,
                },
                instr::MicroInstruction::CopyRegister {
                    dst: SelectedRegister8::AddrLow,
                    src: SelectedRegister8::SP,
                },
                instr::MicroInstruction::WriteAddress {
                    src: SelectedRegister8::PCHigh,
                },
                instr::MicroInstruction::DecrementRegister {
                    dst: SelectedRegister8::SP,
                },
                instr::MicroInstruction::YieldClock,
                // Next clock
                instr::MicroInstruction::CopyRegister {
                    dst: SelectedRegister8::AddrHigh,
                    src: SelectedRegister8::StackPage,
                },
                instr::MicroInstruction::CopyRegister {
                    dst: SelectedRegister8::AddrLow,
                    src: SelectedRegister8::SP,
                },
                instr::MicroInstruction::WriteAddress {
                    src: SelectedRegister8::PCLow,
                },
                instr::MicroInstruction::DecrementRegister {
                    dst: SelectedRegister8::SP,
                },
                instr::MicroInstruction::YieldClock,
                // Next clock
                instr::MicroInstruction::SetStatusFlag {
                    flag: StatusRegFlags::BREAK,
                },
                instr::MicroInstruction::CopyRegister {
                    dst: SelectedRegister8::AddrHigh,
                    src: SelectedRegister8::StackPage,
                },
                instr::MicroInstruction::CopyRegister {
                    dst: SelectedRegister8::AddrLow,
                    src: SelectedRegister8::SP,
                },
                instr::MicroInstruction::WriteAddress {
                    src: SelectedRegister8::Status,
                },
                instr::MicroInstruction::DecrementRegister {
                    dst: SelectedRegister8::SP,
                },
                instr::MicroInstruction::YieldClock,
                // Next clock
                instr::MicroInstruction::CopyRegister16 {
                    dst: SelectedRegister16::Addr,
                    src: SelectedRegister16::InterruptAddrLow,
                },
                instr::MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::PCLow,
                },
                instr::MicroInstruction::YieldClock,
                // Next clock
                instr::MicroInstruction::CopyRegister16 {
                    dst: SelectedRegister16::Addr,
                    src: SelectedRegister16::InterruptAddrHigh,
                },
                instr::MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::PCHigh,
                },
                instr::MicroInstruction::YieldClock,
                // Next clock
            ],
        ),
        (
            InstructionSequenceMode::ReturnInterrupt,
            vec![
                instr::MicroInstruction::ReadPC {
                    dst: SelectedRegister8::Discard,
                    increment: false,
                },
                instr::MicroInstruction::YieldClock,
                // Next clock
                instr::MicroInstruction::IncrementRegister {
                    dst: SelectedRegister8::SP,
                },
                instr::MicroInstruction::YieldClock,
                // Next clock
                instr::MicroInstruction::CopyRegister {
                    dst: SelectedRegister8::AddrHigh,
                    src: SelectedRegister8::StackPage,
                },
                instr::MicroInstruction::CopyRegister {
                    dst: SelectedRegister8::AddrLow,
                    src: SelectedRegister8::SP,
                },
                instr::MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::Status,
                },
                instr::MicroInstruction::IncrementRegister {
                    dst: SelectedRegister8::SP,
                },
                instr::MicroInstruction::YieldClock,
                // Next clock
                instr::MicroInstruction::CopyRegister {
                    dst: SelectedRegister8::AddrHigh,
                    src: SelectedRegister8::StackPage,
                },
                instr::MicroInstruction::CopyRegister {
                    dst: SelectedRegister8::AddrLow,
                    src: SelectedRegister8::SP,
                },
                instr::MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::PCLow,
                },
                instr::MicroInstruction::IncrementRegister {
                    dst: SelectedRegister8::SP,
                },
                instr::MicroInstruction::YieldClock,
                // Next clock
                instr::MicroInstruction::CopyRegister {
                    dst: SelectedRegister8::AddrHigh,
                    src: SelectedRegister8::StackPage,
                },
                instr::MicroInstruction::CopyRegister {
                    dst: SelectedRegister8::AddrLow,
                    src: SelectedRegister8::SP,
                },
                instr::MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::PCHigh,
                },
                instr::MicroInstruction::YieldClock,
            ],
        ),
        (
            InstructionSequenceMode::Push,
            vec![
                instr::MicroInstruction::ReadPC {
                    dst: SelectedRegister8::Discard,
                    increment: false,
                },
                instr::MicroInstruction::YieldClock,
                // Next clock
                instr::MicroInstruction::CopyRegister {
                    dst: SelectedRegister8::AddrHigh,
                    src: SelectedRegister8::StackPage,
                },
                instr::MicroInstruction::CopyRegister {
                    dst: SelectedRegister8::AddrLow,
                    src: SelectedRegister8::SP,
                },
                instr::MicroInstruction::RunOperation,
                instr::MicroInstruction::DecrementRegister {
                    dst: SelectedRegister8::SP,
                },
                instr::MicroInstruction::YieldClock,
            ],
        ),
        (
            InstructionSequenceMode::Pull,
            vec![
                instr::MicroInstruction::ReadPC {
                    dst: SelectedRegister8::Discard,
                    increment: false,
                },
                instr::MicroInstruction::YieldClock,
                // Next clock
                instr::MicroInstruction::IncrementRegister {
                    dst: SelectedRegister8::SP,
                },
                instr::MicroInstruction::YieldClock,
                // Next clock
                instr::MicroInstruction::CopyRegister {
                    dst: SelectedRegister8::AddrHigh,
                    src: SelectedRegister8::StackPage,
                },
                instr::MicroInstruction::CopyRegister {
                    dst: SelectedRegister8::AddrLow,
                    src: SelectedRegister8::SP,
                },
                instr::MicroInstruction::RunOperation,
                instr::MicroInstruction::YieldClock,
            ],
        ),
        (
            InstructionSequenceMode::JumpSubroutine,
            vec![
                instr::MicroInstruction::ReadPC {
                    dst: SelectedRegister8::Tmp,
                    increment: true,
                },
                instr::MicroInstruction::YieldClock,
                // Next clock
                instr::MicroInstruction::YieldClock,
                // Next clock
                instr::MicroInstruction::CopyRegister {
                    dst: SelectedRegister8::AddrHigh,
                    src: SelectedRegister8::StackPage,
                },
                instr::MicroInstruction::CopyRegister {
                    dst: SelectedRegister8::AddrLow,
                    src: SelectedRegister8::SP,
                },
                instr::MicroInstruction::WriteAddress {
                    src: SelectedRegister8::PCHigh,
                },
                instr::MicroInstruction::DecrementRegister {
                    dst: SelectedRegister8::SP,
                },
                instr::MicroInstruction::YieldClock,
                // Next clock
                instr::MicroInstruction::CopyRegister {
                    dst: SelectedRegister8::AddrHigh,
                    src: SelectedRegister8::StackPage,
                },
                instr::MicroInstruction::CopyRegister {
                    dst: SelectedRegister8::AddrLow,
                    src: SelectedRegister8::SP,
                },
                instr::MicroInstruction::WriteAddress {
                    src: SelectedRegister8::PCLow,
                },
                instr::MicroInstruction::DecrementRegister {
                    dst: SelectedRegister8::SP,
                },
                instr::MicroInstruction::YieldClock,
                // Next clock
                instr::MicroInstruction::ReadPC {
                    dst: SelectedRegister8::PCHigh,
                    increment: true,
                },
                instr::MicroInstruction::CopyRegister {
                    dst: SelectedRegister8::PCLow,
                    src: SelectedRegister8::Tmp,
                },
                instr::MicroInstruction::YieldClock,
            ],
        ),
        (
            InstructionSequenceMode::ReturnSubroutine,
            vec![
                instr::MicroInstruction::ReadPC {
                    dst: SelectedRegister8::Discard,
                    increment: false,
                },
                instr::MicroInstruction::YieldClock,
                // Next clock
                instr::MicroInstruction::IncrementRegister {
                    dst: SelectedRegister8::SP,
                },
                instr::MicroInstruction::YieldClock,
                // Next clock
                instr::MicroInstruction::CopyRegister {
                    dst: SelectedRegister8::AddrHigh,
                    src: SelectedRegister8::StackPage,
                },
                instr::MicroInstruction::CopyRegister {
                    dst: SelectedRegister8::AddrLow,
                    src: SelectedRegister8::SP,
                },
                instr::MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::Tmp,
                },
                instr::MicroInstruction::IncrementRegister {
                    dst: SelectedRegister8::SP,
                },
                instr::MicroInstruction::YieldClock,
                // Next clock
                instr::MicroInstruction::CopyRegister {
                    dst: SelectedRegister8::AddrHigh,
                    src: SelectedRegister8::StackPage,
                },
                instr::MicroInstruction::CopyRegister {
                    dst: SelectedRegister8::AddrLow,
                    src: SelectedRegister8::SP,
                },
                instr::MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::PCHigh,
                },
                instr::MicroInstruction::CopyRegister {
                    dst: SelectedRegister8::PCLow,
                    src: SelectedRegister8::Tmp,
                },
                instr::MicroInstruction::YieldClock,
                // Next clock
                instr::MicroInstruction::IncrementRegister16 {
                    dst: SelectedRegister16::PC,
                },
                instr::MicroInstruction::YieldClock,
            ],
        ),
        (
            InstructionSequenceMode::Implied,
            vec![
                instr::MicroInstruction::ReadPC {
                    dst: SelectedRegister8::Discard,
                    increment: false,
                },
                instr::MicroInstruction::RunOperation,
                instr::MicroInstruction::YieldClock,
            ],
        ),
        (
            InstructionSequenceMode::Immediate,
            vec![
                instr::MicroInstruction::CopyRegister {
                    dst: SelectedRegister8::AddrLow,
                    src: SelectedRegister8::PCLow,
                },
                instr::MicroInstruction::CopyRegister {
                    dst: SelectedRegister8::AddrHigh,
                    src: SelectedRegister8::PCHigh,
                },
                instr::MicroInstruction::IncrementRegister16 {
                    dst: SelectedRegister16::PC,
                },
                instr::MicroInstruction::RunOperation,
                instr::MicroInstruction::YieldClock,
            ],
        ),
        (
            InstructionSequenceMode::AbsoluteJump,
            vec![
                instr::MicroInstruction::ReadPC {
                    dst: SelectedRegister8::AddrLow,
                    increment: true,
                },
                instr::MicroInstruction::YieldClock,
                // Next clock
                instr::MicroInstruction::ReadPC {
                    dst: SelectedRegister8::PCHigh,
                    increment: true,
                },
                instr::MicroInstruction::CopyRegister {
                    dst: SelectedRegister8::PCLow,
                    src: SelectedRegister8::AddrLow,
                },
                instr::MicroInstruction::YieldClock,
            ],
        ),
        (
            InstructionSequenceMode::Absolute,
            vec![
                instr::MicroInstruction::ReadPC {
                    dst: SelectedRegister8::AddrLow,
                    increment: true,
                },
                instr::MicroInstruction::YieldClock,
                // Next clock
                instr::MicroInstruction::ReadPC {
                    dst: SelectedRegister8::AddrHigh,
                    increment: true,
                },
                instr::MicroInstruction::YieldClock,
                // Next clock
                instr::MicroInstruction::RunOperation,
                instr::MicroInstruction::YieldClock,
            ],
        ),
        (
            InstructionSequenceMode::AbsoluteReadModifyWrite,
            vec![
                instr::MicroInstruction::ReadPC {
                    dst: SelectedRegister8::AddrLow,
                    increment: true,
                },
                instr::MicroInstruction::YieldClock,
                // Next clock
                instr::MicroInstruction::ReadPC {
                    dst: SelectedRegister8::AddrHigh,
                    increment: true,
                },
                instr::MicroInstruction::YieldClock,
                // Next clock
                instr::MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::Tmp,
                },
                instr::MicroInstruction::YieldClock,
                // Next clock
                instr::MicroInstruction::WriteAddress {
                    src: SelectedRegister8::Tmp,
                },
                instr::MicroInstruction::RunOperation,
                instr::MicroInstruction::YieldClock,
                // Next clock
                instr::MicroInstruction::WriteAddress {
                    src: SelectedRegister8::Tmp,
                },
                instr::MicroInstruction::YieldClock,
            ],
        ),
        (
            InstructionSequenceMode::ZeroPage,
            vec![
                instr::MicroInstruction::ZeroRegister {
                    dst: SelectedRegister8::AddrHigh,
                },
                instr::MicroInstruction::ReadPC {
                    dst: SelectedRegister8::AddrLow,
                    increment: true,
                },
                instr::MicroInstruction::YieldClock,
                // Next clock
                instr::MicroInstruction::RunOperation,
                instr::MicroInstruction::YieldClock,
            ],
        ),
        (
            InstructionSequenceMode::ZeroPageReadModifyWrite,
            vec![
                instr::MicroInstruction::ZeroRegister {
                    dst: SelectedRegister8::AddrHigh,
                },
                instr::MicroInstruction::ReadPC {
                    dst: SelectedRegister8::AddrLow,
                    increment: true,
                },
                instr::MicroInstruction::YieldClock,
                // Next clock
                instr::MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::Tmp,
                },
                instr::MicroInstruction::YieldClock,
                // Next clock
                instr::MicroInstruction::WriteAddress {
                    src: SelectedRegister8::Tmp,
                },
                instr::MicroInstruction::RunOperation,
                instr::MicroInstruction::YieldClock,
                // Next clock
                instr::MicroInstruction::WriteAddress {
                    src: SelectedRegister8::Tmp,
                },
                instr::MicroInstruction::YieldClock,
            ],
        ),
        (
            InstructionSequenceMode::ZeroPageIndx,
            vec![
                instr::MicroInstruction::ZeroRegister {
                    dst: SelectedRegister8::AddrHigh,
                },
                instr::MicroInstruction::ReadPC {
                    dst: SelectedRegister8::AddrLow,
                    increment: true,
                },
                instr::MicroInstruction::YieldClock,
                // Next clock
                instr::MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::Discard,
                },
                instr::MicroInstruction::AddIndexToAddress,
                instr::MicroInstruction::YieldClock,
                // Next clock
                instr::MicroInstruction::RunOperation,
                instr::MicroInstruction::YieldClock,
            ],
        ),
        (
            InstructionSequenceMode::ZeroPageIdxReadModifyWrite,
            vec![
                instr::MicroInstruction::ZeroRegister {
                    dst: SelectedRegister8::AddrHigh,
                },
                instr::MicroInstruction::ReadPC {
                    dst: SelectedRegister8::AddrLow,
                    increment: true,
                },
                instr::MicroInstruction::YieldClock,
                // Next clock
                instr::MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::Discard,
                },
                instr::MicroInstruction::AddIndexToAddress,
                instr::MicroInstruction::YieldClock,
                // Next clock
                instr::MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::Tmp,
                },
                instr::MicroInstruction::YieldClock,
                // Next clock
                instr::MicroInstruction::WriteAddress {
                    src: SelectedRegister8::Tmp,
                },
                instr::MicroInstruction::RunOperation,
                instr::MicroInstruction::YieldClock,
                // Next clock
                instr::MicroInstruction::WriteAddress {
                    src: SelectedRegister8::Tmp,
                },
                instr::MicroInstruction::YieldClock,
            ],
        ),
        (
            InstructionSequenceMode::AbsoluteIdxRead,
            vec![
                instr::MicroInstruction::ReadPC {
                    dst: SelectedRegister8::AddrLow,
                    increment: true,
                },
                instr::MicroInstruction::YieldClock,
                // Next clock
                instr::MicroInstruction::ReadPC {
                    dst: SelectedRegister8::AddrHigh,
                    increment: true,
                },
                instr::MicroInstruction::YieldClock,
                // Next clock
                instr::MicroInstruction::AddIndexToAddress,
                instr::MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::Tmp,
                },
                instr::MicroInstruction::FixAddressOrRunOpAndFinish,
                instr::MicroInstruction::YieldClock,
                // Next clock
                instr::MicroInstruction::RunOperation,
                instr::MicroInstruction::YieldClock,
                // Next clock
            ],
        ),
        (
            InstructionSequenceMode::AbsoluteIdxReadModifyWrite,
            vec![
                instr::MicroInstruction::ReadPC {
                    dst: SelectedRegister8::AddrLow,
                    increment: true,
                },
                instr::MicroInstruction::YieldClock,
                // Next clock
                instr::MicroInstruction::ReadPC {
                    dst: SelectedRegister8::AddrHigh,
                    increment: true,
                },
                instr::MicroInstruction::YieldClock,
                instr::MicroInstruction::AddIndexToAddress,
                // Next clock
                instr::MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::Discard,
                },
                instr::MicroInstruction::FixAddress,
                instr::MicroInstruction::YieldClock,
                // Next clock
                instr::MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::Tmp,
                },
                instr::MicroInstruction::YieldClock,
                // Next clock
                instr::MicroInstruction::WriteAddress {
                    src: SelectedRegister8::Tmp,
                },
                instr::MicroInstruction::RunOperation,
                instr::MicroInstruction::YieldClock,
                // Next clock
                instr::MicroInstruction::WriteAddress {
                    src: SelectedRegister8::Tmp,
                },
                instr::MicroInstruction::YieldClock,
                // Next clock
            ],
        ),
        (
            InstructionSequenceMode::AbsoluteIdxWrite,
            vec![
                instr::MicroInstruction::ReadPC {
                    dst: SelectedRegister8::AddrLow,
                    increment: true,
                },
                instr::MicroInstruction::YieldClock,
                // Next clock
                instr::MicroInstruction::ReadPC {
                    dst: SelectedRegister8::AddrHigh,
                    increment: true,
                },
                instr::MicroInstruction::YieldClock,
                // Next clock
                instr::MicroInstruction::AddIndexToAddress,
                instr::MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::Tmp,
                },
                instr::MicroInstruction::FixAddress,
                instr::MicroInstruction::YieldClock,
                // Next clock
                instr::MicroInstruction::RunOperation,
                instr::MicroInstruction::YieldClock,
                // Next clock
            ],
        ),
        (
            InstructionSequenceMode::Relative,
            vec![
                instr::MicroInstruction::ReadPC {
                    dst: SelectedRegister8::Tmp,
                    increment: true,
                },
                instr::MicroInstruction::YieldClock,
                // Next clock
                instr::MicroInstruction::ReadPC {
                    dst: SelectedRegister8::IR,
                    increment: false,
                },
                instr::MicroInstruction::RunOperation,
                instr::MicroInstruction::YieldClock,
                // Next clock
                instr::MicroInstruction::ReadPC {
                    dst: SelectedRegister8::IR,
                    increment: false,
                },
                instr::MicroInstruction::FixAddressOrIncrementPC,
                instr::MicroInstruction::YieldClock,
            ],
        ),
        (
            InstructionSequenceMode::ZeroPageIdxIndirect,
            vec![
                instr::MicroInstruction::ZeroRegister {
                    dst: SelectedRegister8::AddrHigh,
                },
                instr::MicroInstruction::ReadPC {
                    dst: SelectedRegister8::AddrLow,
                    increment: true,
                },
                instr::MicroInstruction::YieldClock,
                // Next Clock
                instr::MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::Discard,
                },
                instr::MicroInstruction::AddIndexToAddress,
                instr::MicroInstruction::YieldClock,
                // Next Clock
                instr::MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::Tmp,
                },
                instr::MicroInstruction::YieldClock,
                // Next Clock
                instr::MicroInstruction::IncrementRegister {
                    dst: SelectedRegister8::AddrLow,
                },
                instr::MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::AddrHigh,
                },
                instr::MicroInstruction::CopyRegister {
                    dst: SelectedRegister8::AddrLow,
                    src: SelectedRegister8::Tmp,
                },
                instr::MicroInstruction::YieldClock,
                // Next Clock
                instr::MicroInstruction::RunOperation,
                instr::MicroInstruction::YieldClock,
            ],
        ),
        (
            InstructionSequenceMode::ZeroPageIdxIndirectReadModifyWrite,
            vec![
                instr::MicroInstruction::ZeroRegister {
                    dst: SelectedRegister8::AddrHigh,
                },
                instr::MicroInstruction::ReadPC {
                    dst: SelectedRegister8::AddrLow,
                    increment: true,
                },
                instr::MicroInstruction::YieldClock,
                // Next Clock
                instr::MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::Discard,
                },
                instr::MicroInstruction::AddIndexToAddress,
                instr::MicroInstruction::YieldClock,
                // Next Clock
                instr::MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::Tmp,
                },
                instr::MicroInstruction::YieldClock,
                // Next Clock
                instr::MicroInstruction::IncrementRegister {
                    dst: SelectedRegister8::AddrLow,
                },
                instr::MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::AddrHigh,
                },
                instr::MicroInstruction::CopyRegister {
                    dst: SelectedRegister8::AddrLow,
                    src: SelectedRegister8::Tmp,
                },
                instr::MicroInstruction::YieldClock,
                // Next Clock
                instr::MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::Tmp,
                },
                instr::MicroInstruction::YieldClock,
                // Next Clock
                instr::MicroInstruction::WriteAddress {
                    src: SelectedRegister8::Tmp,
                },
                instr::MicroInstruction::RunOperation,
                instr::MicroInstruction::YieldClock,
                // Next Clock
                instr::MicroInstruction::WriteAddress {
                    src: SelectedRegister8::Tmp,
                },
                instr::MicroInstruction::YieldClock,
            ],
        ),
        (
            InstructionSequenceMode::ZeroPageIndirectIdxRead,
            vec![
                instr::MicroInstruction::ZeroRegister {
                    dst: SelectedRegister8::AddrHigh,
                },
                instr::MicroInstruction::ReadPC {
                    dst: SelectedRegister8::AddrLow,
                    increment: true,
                },
                instr::MicroInstruction::YieldClock,
                // Next Clock
                instr::MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::Tmp,
                },
                instr::MicroInstruction::YieldClock,
                // Next Clock
                instr::MicroInstruction::IncrementRegister {
                    dst: SelectedRegister8::AddrLow,
                },
                instr::MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::AddrHigh,
                },
                instr::MicroInstruction::CopyRegister {
                    dst: SelectedRegister8::AddrLow,
                    src: SelectedRegister8::Tmp,
                },
                instr::MicroInstruction::AddIndexToAddress,
                instr::MicroInstruction::YieldClock,
                // Next Clock
                instr::MicroInstruction::FixAddressOrRunOpAndFinish,
                instr::MicroInstruction::YieldClock,
                // Next Clock
                instr::MicroInstruction::RunOperation,
                instr::MicroInstruction::YieldClock,
                // Next Clock
            ],
        ),
        (
            InstructionSequenceMode::ZeroPageIndirectIdxReadModifyWrite,
            vec![
                instr::MicroInstruction::ZeroRegister {
                    dst: SelectedRegister8::AddrHigh,
                },
                instr::MicroInstruction::ReadPC {
                    dst: SelectedRegister8::AddrLow,
                    increment: true,
                },
                instr::MicroInstruction::YieldClock,
                // Next Clock
                instr::MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::Tmp,
                },
                instr::MicroInstruction::YieldClock,
                // Next Clock
                instr::MicroInstruction::IncrementRegister {
                    dst: SelectedRegister8::AddrLow,
                },
                instr::MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::AddrHigh,
                },
                instr::MicroInstruction::CopyRegister {
                    dst: SelectedRegister8::AddrLow,
                    src: SelectedRegister8::Tmp,
                },
                instr::MicroInstruction::AddIndexToAddress,
                instr::MicroInstruction::YieldClock,
                // Next Clock
                instr::MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::Discard,
                },
                instr::MicroInstruction::FixAddress,
                instr::MicroInstruction::YieldClock,
                // Next Clock
                instr::MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::Tmp,
                },
                instr::MicroInstruction::YieldClock,
                // Next Clock
                instr::MicroInstruction::WriteAddress {
                    src: SelectedRegister8::Tmp,
                },
                instr::MicroInstruction::RunOperation,
                instr::MicroInstruction::YieldClock,
                // Next Clock
                instr::MicroInstruction::WriteAddress {
                    src: SelectedRegister8::Tmp,
                },
                instr::MicroInstruction::YieldClock,
                // Next Clock
            ],
        ),
        (
            InstructionSequenceMode::ZeroPageIndirectIdxWrite,
            vec![
                instr::MicroInstruction::ZeroRegister {
                    dst: SelectedRegister8::AddrHigh,
                },
                instr::MicroInstruction::ReadPC {
                    dst: SelectedRegister8::AddrLow,
                    increment: true,
                },
                instr::MicroInstruction::YieldClock,
                // Next Clock
                instr::MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::Tmp,
                },
                instr::MicroInstruction::YieldClock,
                // Next Clock
                instr::MicroInstruction::IncrementRegister {
                    dst: SelectedRegister8::AddrLow,
                },
                instr::MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::AddrHigh,
                },
                instr::MicroInstruction::CopyRegister {
                    dst: SelectedRegister8::AddrLow,
                    src: SelectedRegister8::Tmp,
                },
                instr::MicroInstruction::AddIndexToAddress,
                instr::MicroInstruction::YieldClock,
                // Next Clock
                instr::MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::Discard,
                },
                instr::MicroInstruction::FixAddress,
                instr::MicroInstruction::YieldClock,
                // Next Clock
                instr::MicroInstruction::RunOperation,
                instr::MicroInstruction::YieldClock,
                // Next Clock
            ],
        ),
        (
            InstructionSequenceMode::AbsoluteIndirectJump,
            vec![
                instr::MicroInstruction::ReadPC {
                    dst: SelectedRegister8::Tmp,
                    increment: true,
                },
                instr::MicroInstruction::YieldClock,
                // Next Clock
                instr::MicroInstruction::ReadPC {
                    dst: SelectedRegister8::AddrHigh,
                    increment: true,
                },
                instr::MicroInstruction::CopyRegister {
                    dst: SelectedRegister8::AddrLow,
                    src: SelectedRegister8::Tmp,
                },
                instr::MicroInstruction::YieldClock,
                // Next Clock
                instr::MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::Tmp,
                },
                instr::MicroInstruction::YieldClock,
                // Next Clock
                instr::MicroInstruction::IncrementRegister {
                    dst: SelectedRegister8::AddrLow,
                },
                instr::MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::PCHigh,
                },
                instr::MicroInstruction::CopyRegister {
                    dst: SelectedRegister8::PCLow,
                    src: SelectedRegister8::Tmp,
                },
                instr::MicroInstruction::YieldClock,
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
            let last_micro_instr = sequence.last().unwrap_or_else(|| {
                panic!(
                    "Sequence mode {:?} does not have any instr::MicroInstruction",
                    mode
                )
            });
            assert_eq!(
                *last_micro_instr,
                instr::MicroInstruction::YieldClock,
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
                    .position(|&instr| instr == instr::MicroInstruction::RunOperation);

                if let Some(position) = runop_position {
                    let last_memory_read_before_op = sequence[..position]
                        .iter()
                        .rev()
                        .find(|&instr| matches!(instr, instr::MicroInstruction::ReadAddress { .. }));

                    assert_eq!(
                        last_memory_read_before_op,
                        Some(&instr::MicroInstruction::ReadAddress {
                            dst: SelectedRegister8::Tmp
                        })
                    );

                    let next_memory_write_after_op = sequence[position..]
                        .iter()
                        .find(|&instr| matches!(instr, instr::MicroInstruction::WriteAddress { .. }));

                    assert_eq!(
                        next_memory_write_after_op,
                        Some(&instr::MicroInstruction::WriteAddress {
                            src: SelectedRegister8::Tmp
                        })
                    );
                }
            }
        }
    }
}
