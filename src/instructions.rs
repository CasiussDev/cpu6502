use crate::alu::{AluBinaryOp, AluUnaryOp};
use crate::registers::register_file::SelectedRegister;
use crate::registers::StatusRegFlags;
use std::collections::HashMap;

#[cfg(test)]
use strum::IntoEnumIterator;
#[cfg(test)]
use strum_macros::EnumIter;

#[allow(dead_code)]
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum MicroInstruction {
    Fetch,
    ReadPC {
        dst: SelectedRegister,
        increment: bool,
    },
    ReadAddress {
        dst: SelectedRegister,
    },
    WriteAddress {
        src: SelectedRegister,
    },
    CopyRegister {
        dst: SelectedRegister,
        src: SelectedRegister,
    },
    ZeroRegister {
        dst: SelectedRegister,
    },
    IncrementRegister {
        dst: SelectedRegister,
    },
    DecrementRegister {
        dst: SelectedRegister,
    },
    AluUnaryOp {
        op: AluUnaryOp,
        reg: SelectedRegister,
    },
    AluBinaryOp {
        op: AluBinaryOp,
        dst: SelectedRegister,
        src: SelectedRegister,
    },
    SetStatusFlag {
        flag: StatusRegFlags,
    },
    ClearStatusFlag {
        flag: StatusRegFlags,
    },
    //AddIndexToAddress {
    //    index: SelectedRegister,
    //},
    AddIndexToAddress,
    FixAddress,
    RunOperation,

    YieldClock,

    FinishOrFixAddress,
}

#[allow(dead_code)]
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
#[cfg_attr(test, derive(EnumIter))]
pub enum InstructionOp {
    Nop,
    IncrementX,
    IncrementY,
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
}

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

    IdxIndirect,
    IdxIndirectReadModifyWrite,

    IndirectIdx,
    IndirectIdxReadModifyWrite,

    AbsoluteIndirect,
}

type MicroInstructionsVector = std::vec::Vec<MicroInstruction>;
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
                MicroInstruction::RunOperation,
                MicroInstruction::ReadPC {
                    dst: SelectedRegister::Discard,
                    increment: false,
                },
                MicroInstruction::YieldClock,
            ],
        ),
        (
            InstructionSequenceMode::Immediate,
            vec![MicroInstruction::RunOperation, MicroInstruction::YieldClock],
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
                MicroInstruction::FinishOrFixAddress,
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
    ]);

    sequences
}

type OpsMap = std::collections::HashMap<InstructionOp, MicroInstructionsVector>;

#[allow(dead_code)]
pub fn create_instructionops_sequences() -> OpsMap {
    let ops = HashMap::from([
        (InstructionOp::Nop, vec![]),
        (
            InstructionOp::IncrementX,
            vec![MicroInstruction::AluUnaryOp {
                op: AluUnaryOp::Inc,
                reg: SelectedRegister::X,
            }],
        ),
        (
            InstructionOp::IncrementY,
            vec![MicroInstruction::AluUnaryOp {
                op: AluUnaryOp::Inc,
                reg: SelectedRegister::Y,
            }],
        ),
        (
            InstructionOp::DecrementX,
            vec![MicroInstruction::AluUnaryOp {
                op: AluUnaryOp::Dec,
                reg: SelectedRegister::X,
            }],
        ),
        (
            InstructionOp::DecrementY,
            vec![MicroInstruction::AluUnaryOp {
                op: AluUnaryOp::Dec,
                reg: SelectedRegister::Y,
            }],
        ),
        (
            InstructionOp::ClearCarry,
            vec![MicroInstruction::ClearStatusFlag {
                flag: StatusRegFlags::CARRY,
            }],
        ),
        (
            InstructionOp::SetCarry,
            vec![MicroInstruction::SetStatusFlag {
                flag: StatusRegFlags::CARRY,
            }],
        ),
        (
            InstructionOp::ClearDecimal,
            vec![MicroInstruction::ClearStatusFlag {
                flag: StatusRegFlags::DECIMAL,
            }],
        ),
        (
            InstructionOp::SetDecimal,
            vec![MicroInstruction::SetStatusFlag {
                flag: StatusRegFlags::DECIMAL,
            }],
        ),
        (
            InstructionOp::ClearInterruptDisable,
            vec![MicroInstruction::ClearStatusFlag {
                flag: StatusRegFlags::IRQ_DISABLE,
            }],
        ),
        (
            InstructionOp::SetInterruptDisable,
            vec![MicroInstruction::SetStatusFlag {
                flag: StatusRegFlags::IRQ_DISABLE,
            }],
        ),
        (
            InstructionOp::ClearOverflow,
            vec![MicroInstruction::ClearStatusFlag {
                flag: StatusRegFlags::OVERFLOW,
            }],
        ),
        (
            InstructionOp::SetOverflow,
            vec![MicroInstruction::SetStatusFlag {
                flag: StatusRegFlags::OVERFLOW,
            }],
        ),
        (
            InstructionOp::TransferAccumulatorToX,
            vec![MicroInstruction::CopyRegister {
                src: SelectedRegister::A,
                dst: SelectedRegister::X,
            }],
        ),
        (
            InstructionOp::TransferAccumulatorToY,
            vec![MicroInstruction::CopyRegister {
                src: SelectedRegister::A,
                dst: SelectedRegister::Y,
            }],
        ),
        (
            InstructionOp::TransferStackPtrToX,
            vec![MicroInstruction::CopyRegister {
                src: SelectedRegister::SP,
                dst: SelectedRegister::X,
            }],
        ),
        (
            InstructionOp::TransferXToAccumulator,
            vec![MicroInstruction::CopyRegister {
                src: SelectedRegister::X,
                dst: SelectedRegister::A,
            }],
        ),
        (
            InstructionOp::TransferYToAccumulator,
            vec![MicroInstruction::CopyRegister {
                src: SelectedRegister::Y,
                dst: SelectedRegister::A,
            }],
        ),
        (
            InstructionOp::TransferXToStackPtr,
            vec![MicroInstruction::CopyRegister {
                src: SelectedRegister::X,
                dst: SelectedRegister::SP,
            }],
        ),
        (
            InstructionOp::PushA,
            vec![MicroInstruction::WriteAddress {
                src: SelectedRegister::A,
            }],
        ),
        (
            InstructionOp::PushStatus,
            vec![MicroInstruction::WriteAddress {
                src: SelectedRegister::Status,
            }],
        ),
        (
            InstructionOp::PullA,
            vec![MicroInstruction::ReadAddress {
                dst: SelectedRegister::A,
            }],
        ),
        (
            InstructionOp::PullStatus,
            vec![MicroInstruction::ReadAddress {
                dst: SelectedRegister::Status,
            }],
        ),
    ]);

    ops
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

    #[test]
    fn check_all_ops_implemented() {
        let instruction_ops = create_instructionops_sequences();
        for op in InstructionOp::iter() {
            assert!(
                instruction_ops.contains_key(&op),
                "Operation {:?} not implemented",
                op,
            );
        }
    }
}
