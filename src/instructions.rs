use crate::registers::register_file::SelectedRegister;
use std::collections::HashMap;
use crate::alu::{AluBinaryOp, AluUnaryOp};
use crate::registers::StatusRegFlags;

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
pub enum InstructionOp {
    INX,
    INY,
    DEX,
    DEY,
}

#[allow(dead_code)]
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
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
        (
            InstructionOp::INX,
            vec![MicroInstruction::AluUnaryOp {
                op: AluUnaryOp::Inc,
                reg: SelectedRegister::X,
            }],
        ),
        (
            InstructionOp::INY,
            vec![MicroInstruction::AluUnaryOp {
                op: AluUnaryOp::Inc,
                reg: SelectedRegister::Y,
            }],
        ),
        (
            InstructionOp::DEX,
            vec![MicroInstruction::AluUnaryOp {
                op: AluUnaryOp::Dec,
                reg: SelectedRegister::X,
            }],
        ),
        (
            InstructionOp::DEY,
            vec![MicroInstruction::AluUnaryOp {
                op: AluUnaryOp::Dec,
                reg: SelectedRegister::Y,
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
}
