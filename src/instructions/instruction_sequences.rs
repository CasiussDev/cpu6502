use std::collections::HashMap;
use crate::instructions::microinstructions::{MicroInstruction, MicroInstructionsVector};
use crate::registers::SelectedRegister;

#[cfg(test)]
use strum::IntoEnumIterator;
#[cfg(test)]
use strum_macros::EnumIter;

#[allow(dead_code)]
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
#[cfg_attr(test, derive(EnumIter))]
pub enum InstructionSequenceMode {
    Break,
    ReturnInterrupt,
    ReturnSubroutine,
    Push,
    Pull,
    JumpSubroutine,
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

    AbsoluteIndirect,
}

impl Default for InstructionSequenceMode {
    fn default() -> Self {
        InstructionSequenceMode::Implied
    }
}

type SequenceMap = std::collections::HashMap<InstructionSequenceMode, MicroInstructionsVector>;

#[allow(dead_code)]
pub fn create_instruction_mode_sequences() -> SequenceMap {
    //let sequences = SequenceMap::new();
    let sequences = HashMap::from([
        (
            InstructionSequenceMode::Push,
            vec![
                MicroInstruction::ReadPC {
                    dst: SelectedRegister::Discard,
                    increment: false,
                },
                MicroInstruction::YieldClock,
                // Next clock
                MicroInstruction::ZeroRegister {
                    dst: SelectedRegister::AddrHigh,
                },
                MicroInstruction::CopyRegister {
                    dst: SelectedRegister::AddrLow,
                    src: SelectedRegister::SP,
                },
                MicroInstruction::RunOperation,
                MicroInstruction::DecrementRegister {
                    dst: SelectedRegister::SP,
                },
                MicroInstruction::YieldClock,
            ],
        ),
        (
            InstructionSequenceMode::Pull,
            vec![
                MicroInstruction::ReadPC {
                    dst: SelectedRegister::Discard,
                    increment: false,
                },
                MicroInstruction::YieldClock,
                // Next clock
                MicroInstruction::IncrementRegister {
                    dst: SelectedRegister::SP,
                },
                MicroInstruction::YieldClock,
                // Next clock
                MicroInstruction::ZeroRegister {
                    dst: SelectedRegister::AddrHigh,
                },
                MicroInstruction::CopyRegister {
                    dst: SelectedRegister::AddrLow,
                    src: SelectedRegister::SP,
                },
                MicroInstruction::RunOperation,
                MicroInstruction::YieldClock,
            ],
        ),
        (
            InstructionSequenceMode::Implied,
            vec![
                MicroInstruction::ReadPC {
                    dst: SelectedRegister::Discard,
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
                    dst: SelectedRegister::AddrLow,
                    src: SelectedRegister::PCLow,
                },
                MicroInstruction::CopyRegister {
                    dst: SelectedRegister::AddrHigh,
                    src: SelectedRegister::PCHigh,
                },
                MicroInstruction::IncrementRegister {
                    dst: SelectedRegister::PC,
                },
                MicroInstruction::RunOperation,
                MicroInstruction::YieldClock,
            ],
        ),
        (
            InstructionSequenceMode::AbsoluteJump,
            vec![
                MicroInstruction::ReadPC {
                    dst: SelectedRegister::AddrLow,
                    increment: true,
                },
                MicroInstruction::YieldClock,
                // Next clock
                MicroInstruction::ReadPC {
                    dst: SelectedRegister::PCHigh,
                    increment: true,
                },
                MicroInstruction::CopyRegister {
                    dst: SelectedRegister::PCLow,
                    src: SelectedRegister::AddrLow,
                },
                MicroInstruction::YieldClock,
            ],
        ),
        (
            InstructionSequenceMode::Absolute,
            vec![
                MicroInstruction::ReadPC {
                    dst: SelectedRegister::AddrLow,
                    increment: true,
                },
                MicroInstruction::YieldClock,
                // Next clock
                MicroInstruction::ReadPC {
                    dst: SelectedRegister::AddrHigh,
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
                    dst: SelectedRegister::AddrLow,
                    increment: true,
                },
                MicroInstruction::YieldClock,
                // Next clock
                MicroInstruction::ReadPC {
                    dst: SelectedRegister::AddrHigh,
                    increment: true,
                },
                MicroInstruction::YieldClock,
                // Next clock
                MicroInstruction::ReadAddress {
                    dst: SelectedRegister::Tmp,
                },
                MicroInstruction::YieldClock,
                // Next clock
                MicroInstruction::WriteAddress {
                    src: SelectedRegister::Tmp,
                },
                MicroInstruction::RunOperation,
                MicroInstruction::YieldClock,
                // Next clock
                MicroInstruction::WriteAddress {
                    src: SelectedRegister::Tmp,
                },
                MicroInstruction::YieldClock,
            ],
        ),
        (
            InstructionSequenceMode::ZeroPage,
            vec![
                MicroInstruction::ZeroRegister {
                    dst: SelectedRegister::AddrHigh,
                },
                MicroInstruction::ReadPC {
                    dst: SelectedRegister::AddrLow,
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
                    dst: SelectedRegister::AddrHigh,
                },
                MicroInstruction::ReadPC {
                    dst: SelectedRegister::AddrLow,
                    increment: true,
                },
                MicroInstruction::YieldClock,
                // Next clock
                MicroInstruction::ReadAddress {
                    dst: SelectedRegister::Tmp,
                },
                MicroInstruction::YieldClock,
                // Next clock
                MicroInstruction::WriteAddress {
                    src: SelectedRegister::Tmp,
                },
                MicroInstruction::RunOperation,
                MicroInstruction::YieldClock,
                // Next clock
                MicroInstruction::WriteAddress {
                    src: SelectedRegister::Tmp,
                },
                MicroInstruction::YieldClock,
            ],
        ),
        (
            InstructionSequenceMode::ZeroPageIndx,
            vec![
                MicroInstruction::ZeroRegister {
                    dst: SelectedRegister::AddrHigh,
                },
                MicroInstruction::ReadPC {
                    dst: SelectedRegister::AddrLow,
                    increment: true,
                },
                MicroInstruction::YieldClock,
                // Next clock
                MicroInstruction::ReadAddress {
                    dst: SelectedRegister::Discard,
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
                    dst: SelectedRegister::AddrHigh,
                },
                MicroInstruction::ReadPC {
                    dst: SelectedRegister::AddrLow,
                    increment: true,
                },
                MicroInstruction::YieldClock,
                // Next clock
                MicroInstruction::ReadAddress {
                    dst: SelectedRegister::Discard,
                },
                MicroInstruction::AddIndexToAddress,
                MicroInstruction::YieldClock,
                // Next clock
                MicroInstruction::ReadAddress {
                    dst: SelectedRegister::Tmp,
                },
                MicroInstruction::YieldClock,
                // Next clock
                MicroInstruction::WriteAddress {
                    src: SelectedRegister::Tmp,
                },
                MicroInstruction::RunOperation,
                MicroInstruction::YieldClock,
                // Next clock
                MicroInstruction::WriteAddress {
                    src: SelectedRegister::Tmp,
                },
                MicroInstruction::YieldClock,
            ],
        ),
        (
            InstructionSequenceMode::AbsoluteIdxRead,
            vec![
                MicroInstruction::ReadPC {
                    dst: SelectedRegister::AddrLow,
                    increment: true,
                },
                MicroInstruction::YieldClock,
                // Next clock
                MicroInstruction::ReadPC {
                    dst: SelectedRegister::AddrHigh,
                    increment: true,
                },
                MicroInstruction::YieldClock,
                // Next clock
                MicroInstruction::AddIndexToAddress,
                MicroInstruction::ReadAddress {
                    dst: SelectedRegister::Tmp,
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
                    dst: SelectedRegister::AddrLow,
                    increment: true,
                },
                MicroInstruction::YieldClock,
                // Next clock
                MicroInstruction::ReadPC {
                    dst: SelectedRegister::AddrHigh,
                    increment: true,
                },
                MicroInstruction::YieldClock,
                MicroInstruction::AddIndexToAddress,
                // Next clock
                MicroInstruction::ReadAddress {
                    dst: SelectedRegister::Discard,
                },
                MicroInstruction::FixAddress,
                MicroInstruction::YieldClock,
                // Next clock
                MicroInstruction::ReadAddress {
                    dst: SelectedRegister::Tmp,
                },
                MicroInstruction::YieldClock,
                // Next clock
                MicroInstruction::WriteAddress {
                    src: SelectedRegister::Tmp,
                },
                MicroInstruction::RunOperation,
                MicroInstruction::YieldClock,
                // Next clock
                MicroInstruction::WriteAddress {
                    src: SelectedRegister::Tmp,
                },
                MicroInstruction::YieldClock,
                // Next clock
            ],
        ),
        (
            InstructionSequenceMode::AbsoluteIdxWrite,
            vec![
                MicroInstruction::ReadPC {
                    dst: SelectedRegister::AddrLow,
                    increment: true,
                },
                MicroInstruction::YieldClock,
                // Next clock
                MicroInstruction::ReadPC {
                    dst: SelectedRegister::AddrHigh,
                    increment: true,
                },
                MicroInstruction::YieldClock,
                // Next clock
                MicroInstruction::AddIndexToAddress,
                MicroInstruction::ReadAddress {
                    dst: SelectedRegister::Tmp,
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
            InstructionSequenceMode::ZeroPageIdxIndirect,
            vec![
                MicroInstruction::ZeroRegister {
                    dst: SelectedRegister::AddrHigh,
                },
                MicroInstruction::ReadPC {
                    dst: SelectedRegister::AddrLow,
                    increment: true,
                },
                MicroInstruction::YieldClock,
                // Next Clock
                MicroInstruction::ReadAddress {
                    dst: SelectedRegister::Discard,
                },
                MicroInstruction::AddIndexToAddress,
                MicroInstruction::YieldClock,
                // Next Clock
                MicroInstruction::ReadAddress {
                    dst: SelectedRegister::Tmp,
                },
                MicroInstruction::YieldClock,
                // Next Clock
                MicroInstruction::IncrementRegister {
                    dst: SelectedRegister::Addr,
                },
                MicroInstruction::ReadAddress {
                    dst: SelectedRegister::AddrHigh,
                },
                MicroInstruction::CopyRegister {
                    dst: SelectedRegister::AddrLow,
                    src: SelectedRegister::Tmp,
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
                    dst: SelectedRegister::AddrHigh,
                },
                MicroInstruction::ReadPC {
                    dst: SelectedRegister::AddrLow,
                    increment: true,
                },
                MicroInstruction::YieldClock,
                // Next Clock
                MicroInstruction::ReadAddress {
                    dst: SelectedRegister::Discard,
                },
                MicroInstruction::AddIndexToAddress,
                MicroInstruction::YieldClock,
                // Next Clock
                MicroInstruction::ReadAddress {
                    dst: SelectedRegister::Tmp,
                },
                MicroInstruction::YieldClock,
                // Next Clock
                MicroInstruction::IncrementRegister {
                    dst: SelectedRegister::Addr,
                },
                MicroInstruction::ReadAddress {
                    dst: SelectedRegister::AddrHigh,
                },
                MicroInstruction::CopyRegister {
                    dst: SelectedRegister::AddrLow,
                    src: SelectedRegister::Tmp,
                },
                MicroInstruction::YieldClock,
                // Next Clock
                MicroInstruction::ReadAddress {
                    dst: SelectedRegister::Tmp,
                },
                MicroInstruction::YieldClock,
                // Next Clock
                MicroInstruction::WriteAddress {
                    src: SelectedRegister::Tmp,
                },
                MicroInstruction::RunOperation,
                MicroInstruction::YieldClock,
                // Next Clock
                MicroInstruction::WriteAddress {
                    src: SelectedRegister::Tmp,
                },
                MicroInstruction::YieldClock,
            ],
        ),
        (
            InstructionSequenceMode::ZeroPageIndirectIdxWrite,
            vec![
                MicroInstruction::ZeroRegister {
                    dst: SelectedRegister::AddrHigh,
                },
                MicroInstruction::ReadPC {
                    dst: SelectedRegister::AddrLow,
                    increment: true,
                },
                MicroInstruction::YieldClock,
                // Next Clock
                MicroInstruction::ReadAddress {
                    dst: SelectedRegister::Tmp,
                },
                MicroInstruction::YieldClock,
                // Next Clock
                MicroInstruction::IncrementRegister {
                    dst: SelectedRegister::Addr,
                },
                MicroInstruction::ReadAddress {
                    dst: SelectedRegister::AddrHigh,
                },
                MicroInstruction::CopyRegister {
                    dst: SelectedRegister::AddrLow,
                    src: SelectedRegister::Tmp,
                },
                MicroInstruction::AddIndexToAddress,
                MicroInstruction::YieldClock,
                // Next Clock
                MicroInstruction::ReadAddress {
                    dst: SelectedRegister::Discard,
                },
                MicroInstruction::FixAddress,
                MicroInstruction::YieldClock,
                // Next Clock
                MicroInstruction::RunOperation,
                MicroInstruction::YieldClock,
                // Next Clock
            ],
        ),
    ]);

    sequences
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instructionsequences_checklastmicroinstruction_yieldclock() {
        for (mode, sequence) in create_instruction_mode_sequences() {
            let last_microinstruction = sequence.last().unwrap_or_else(|| {
                panic!(
                    "Sequence mode {:?} does not have any microinstruction",
                    mode
                )
            });
            assert!(
                *last_microinstruction == MicroInstruction::YieldClock,
                "Sequence mode {:?} does not finish with YieldClock",
                mode
            );
        }
    }

    #[test]
    fn check_all_instruction_modes_implemented() {
        let sequence_modes = create_instruction_mode_sequences();
        for mode in InstructionSequenceMode::iter() {
            assert!(
                sequence_modes.contains_key(&mode),
                "Mode {:?} not implemented",
                mode,
            )
        }
    }
}