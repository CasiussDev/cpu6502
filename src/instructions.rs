use crate::registers::register_file::SelectedRegister;
use std::collections::HashMap;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum MicroInstructions {
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
        reg: SelectedRegister,
    },
    AluBinaryOp {
        dst: SelectedRegister,
        src: SelectedRegister,
    },
    //AddIndexToAddress {
    //    index: SelectedRegister,
    //},
    AddIndexToAddress,
    FixAddress,
    RunOperation,

    YieldClock,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
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

type MicroInstructionsVector = std::vec::Vec<MicroInstructions>;
type SequenceMap = std::collections::HashMap<InstructionSequenceMode, MicroInstructionsVector>;

pub fn create_instruction_mode_sequences() -> SequenceMap {
    //let sequences = SequenceMap::new();
    let sequences = HashMap::from([
        (
            InstructionSequenceMode::Implied,
            vec![
                MicroInstructions::RunOperation,
                MicroInstructions::ReadPC {
                    dst: SelectedRegister::Discard,
                    increment: false,
                },
                MicroInstructions::YieldClock,
            ],
        ),
        (
            InstructionSequenceMode::Immediate,
            vec![
                MicroInstructions::RunOperation,
                MicroInstructions::YieldClock,
            ],
        ),
        (
            InstructionSequenceMode::AbsoluteJump,
            vec![
                MicroInstructions::ReadPC {
                    dst: SelectedRegister::AddrLow,
                    increment: true,
                },
                MicroInstructions::YieldClock,
                // Next clock
                MicroInstructions::ReadPC {
                    dst: SelectedRegister::PCHigh,
                    increment: true,
                },
                MicroInstructions::CopyRegister {
                    dst: SelectedRegister::PCLow,
                    src: SelectedRegister::AddrLow,
                },
                MicroInstructions::YieldClock,
            ],
        ),
        (
            InstructionSequenceMode::Absolute,
            vec![
                MicroInstructions::ReadPC {
                    dst: SelectedRegister::AddrLow,
                    increment: true,
                },
                MicroInstructions::YieldClock,
                // Next clock
                MicroInstructions::ReadPC {
                    dst: SelectedRegister::AddrHigh,
                    increment: true,
                },
                MicroInstructions::YieldClock,
                // Next clock
                MicroInstructions::RunOperation,
                MicroInstructions::YieldClock,
            ],
        ),
        (
            InstructionSequenceMode::AbsoluteReadModifyWrite,
            vec![
                MicroInstructions::ReadPC {
                    dst: SelectedRegister::AddrLow,
                    increment: true,
                },
                MicroInstructions::YieldClock,
                // Next clock
                MicroInstructions::ReadPC {
                    dst: SelectedRegister::AddrHigh,
                    increment: true,
                },
                MicroInstructions::YieldClock,
                // Next clock
                MicroInstructions::ReadAddress {
                    dst: SelectedRegister::Tmp,
                },
                MicroInstructions::YieldClock,
                // Next clock
                MicroInstructions::WriteAddress {
                    src: SelectedRegister::Tmp,
                },
                MicroInstructions::RunOperation,
                MicroInstructions::YieldClock,
                // Next clock
                MicroInstructions::WriteAddress {
                    src: SelectedRegister::Tmp,
                },
                MicroInstructions::YieldClock,
            ],
        ),
        (
            InstructionSequenceMode::ZeroPage,
            vec![
                MicroInstructions::ZeroRegister {
                    dst: SelectedRegister::AddrHigh,
                },
                MicroInstructions::ReadPC {
                    dst: SelectedRegister::AddrLow,
                    increment: true,
                },
                MicroInstructions::YieldClock,
                // Next clock
                MicroInstructions::RunOperation,
                MicroInstructions::YieldClock,
            ],
        ),
        (
            InstructionSequenceMode::ZeroPageReadModifyWrite,
            vec![
                MicroInstructions::ZeroRegister {
                    dst: SelectedRegister::AddrHigh,
                },
                MicroInstructions::ReadPC {
                    dst: SelectedRegister::AddrLow,
                    increment: true,
                },
                MicroInstructions::YieldClock,
                // Next clock
                MicroInstructions::ReadAddress {
                    dst: SelectedRegister::Tmp,
                },
                MicroInstructions::YieldClock,
                // Next clock
                MicroInstructions::WriteAddress {
                    src: SelectedRegister::Tmp,
                },
                MicroInstructions::RunOperation,
                MicroInstructions::YieldClock,
                // Next clock
                MicroInstructions::WriteAddress {
                    src: SelectedRegister::Tmp,
                },
                MicroInstructions::YieldClock,
            ],
        ),
        (
            InstructionSequenceMode::ZeroPageIndx,
            vec![
                MicroInstructions::ZeroRegister {
                    dst: SelectedRegister::AddrHigh,
                },
                MicroInstructions::ReadPC {
                    dst: SelectedRegister::AddrLow,
                    increment: true,
                },
                MicroInstructions::YieldClock,
                // Next clock
                MicroInstructions::ReadAddress {
                    dst: SelectedRegister::Discard,
                },
                MicroInstructions::AddIndexToAddress,
                MicroInstructions::YieldClock,
                // Next clock
                MicroInstructions::RunOperation,
                MicroInstructions::YieldClock,
            ],
        ),
        (
            InstructionSequenceMode::ZeroPageReadModifyWrite,
            vec![
                MicroInstructions::ZeroRegister {
                    dst: SelectedRegister::AddrHigh,
                },
                MicroInstructions::ReadPC {
                    dst: SelectedRegister::AddrLow,
                    increment: true,
                },
                MicroInstructions::YieldClock,
                // Next clock
                MicroInstructions::ReadAddress {
                    dst: SelectedRegister::Discard,
                },
                MicroInstructions::AddIndexToAddress,
                MicroInstructions::YieldClock,
                // Next clock
                MicroInstructions::ReadAddress {
                    dst: SelectedRegister::Tmp,
                },
                MicroInstructions::YieldClock,
                // Next clock
                MicroInstructions::WriteAddress {
                    src: SelectedRegister::Tmp,
                },
                MicroInstructions::RunOperation,
                MicroInstructions::YieldClock,
                // Next clock
                MicroInstructions::WriteAddress {
                    src: SelectedRegister::Tmp,
                },
                MicroInstructions::YieldClock,
            ],
        ),
        (
            InstructionSequenceMode::AbsoluteIdxWrite,
            vec![
                MicroInstructions::ReadPC {
                    dst: SelectedRegister::AddrLow,
                    increment: true,
                },
                MicroInstructions::YieldClock,
                // Next clock
                MicroInstructions::ReadPC {
                    dst: SelectedRegister::AddrHigh,
                    increment: true,
                },
                MicroInstructions::YieldClock,
                // Next clock
                MicroInstructions::AddIndexToAddress,
                MicroInstructions::ReadAddress {
                    dst: SelectedRegister::Tmp,
                },
                MicroInstructions::YieldClock,
                // Next clock
                MicroInstructions::FixAddress,
                MicroInstructions::RunOperation,
                MicroInstructions::YieldClock,
                // Next clock
            ],
        ),
    ]);

    sequences
}
