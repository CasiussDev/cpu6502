use crate::registers::register_file::SelectedRegister;
use std::collections::HashMap;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
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

    FinishOrFixAddress,
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

type MicroInstructionsVector = std::vec::Vec<MicroInstruction>;
type SequenceMap = std::collections::HashMap<InstructionSequenceMode, MicroInstructionsVector>;

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
            vec![
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
