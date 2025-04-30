use crate::instr;
use crate::registers::{IndexRegister, SelectedRegister16, SelectedRegister8, StatusRegFlags};
//use once_cell::unsync::Lazy;
use enum_map::{enum_map, Enum};
use lazy_static::lazy_static;
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

#[derive(Copy, Clone, PartialEq, Eq, Hash, Enum, Debug)]
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

type SequenceMap = collections::HashMap<InstructionSequenceMode, instr::MicroInstructionsVector>;
type SequenceModeEnumMap =
    enum_map::EnumMap<InstructionSequenceMode, instr::MicroInstructionsVector>;

lazy_static! {
    static ref MODES_SEQUENCES_DEFS: SequenceMap = create_instruction_mode_sequences();
}

lazy_static! {
    static ref MODES_SEQUENCES_ENUM_MAP: SequenceModeEnumMap =
        create_instruction_mode_sequences_enum_map();
}

pub fn sequence_for_mode(
    mode: InstructionSequenceMode,
) -> slice::Iter<'static, instr::MicroInstruction> {
    if cfg!(feature = "enummaps") {
        MODES_SEQUENCES_ENUM_MAP[mode].iter()
    } else {
        MODES_SEQUENCES_DEFS.get(&mode).unwrap().iter()
    }
}

pub fn sequence_for_mode_map(
    mode: InstructionSequenceMode,
) -> slice::Iter<'static, instr::MicroInstruction> {
    MODES_SEQUENCES_DEFS.get(&mode).map(|x| x.iter()).unwrap()
}

impl Default for InstructionSequenceMode {
    fn default() -> Self {
        InstructionSequenceMode::Implied
    }
}

pub fn create_instruction_mode_sequences_enum_map() -> SequenceModeEnumMap {
    enum_map! {
        InstructionSequenceMode::FetchInstr => sequence_for_mode_map(InstructionSequenceMode::FetchInstr).cloned().collect(),
        InstructionSequenceMode::Break => sequence_for_mode_map(InstructionSequenceMode::Break).cloned().collect(),
        InstructionSequenceMode::StartIrq => sequence_for_mode_map(InstructionSequenceMode::StartIrq).cloned().collect(),
        InstructionSequenceMode::StartNmi => sequence_for_mode_map(InstructionSequenceMode::StartNmi).cloned().collect(),
        InstructionSequenceMode::Reset => sequence_for_mode_map(InstructionSequenceMode::Reset).cloned().collect(),
        InstructionSequenceMode::ReturnInterrupt => sequence_for_mode_map(InstructionSequenceMode::ReturnInterrupt).cloned().collect(),

        InstructionSequenceMode::JumpSubroutine => sequence_for_mode_map(InstructionSequenceMode::JumpSubroutine).cloned().collect(),
        InstructionSequenceMode::ReturnSubroutine => sequence_for_mode_map(InstructionSequenceMode::ReturnSubroutine).cloned().collect(),

        InstructionSequenceMode::Push => sequence_for_mode_map(InstructionSequenceMode::Push).cloned().collect(),
        InstructionSequenceMode::Pull => sequence_for_mode_map(InstructionSequenceMode::Pull).cloned().collect(),
        InstructionSequenceMode::Implied => sequence_for_mode_map(InstructionSequenceMode::Implied).cloned().collect(),
        InstructionSequenceMode::Immediate => sequence_for_mode_map(InstructionSequenceMode::Immediate).cloned().collect(),

        InstructionSequenceMode::AbsoluteJump => sequence_for_mode_map(InstructionSequenceMode::AbsoluteJump).cloned().collect(),
        InstructionSequenceMode::Absolute => sequence_for_mode_map(InstructionSequenceMode::Absolute).cloned().collect(),
        InstructionSequenceMode::AbsoluteReadModifyWrite => sequence_for_mode_map(InstructionSequenceMode::AbsoluteReadModifyWrite).cloned().collect(),

        InstructionSequenceMode::ZeroPage => sequence_for_mode_map(InstructionSequenceMode::ZeroPage).cloned().collect(),
        InstructionSequenceMode::ZeroPageReadModifyWrite => sequence_for_mode_map(InstructionSequenceMode::ZeroPageReadModifyWrite).cloned().collect(),

        InstructionSequenceMode::ZeroPageIdx => sequence_for_mode_map(InstructionSequenceMode::ZeroPageIdx).cloned().collect(),
        InstructionSequenceMode::ZeroPageIdxReadModifyWrite => sequence_for_mode_map(InstructionSequenceMode::ZeroPageIdxReadModifyWrite).cloned().collect(),

        InstructionSequenceMode::AbsoluteIdxRead => sequence_for_mode_map(InstructionSequenceMode::AbsoluteIdxRead).cloned().collect(),
        InstructionSequenceMode::AbsoluteIdxReadModifyWrite => sequence_for_mode_map(InstructionSequenceMode::AbsoluteIdxReadModifyWrite).cloned().collect(),
        InstructionSequenceMode::AbsoluteIdxWrite => sequence_for_mode_map(InstructionSequenceMode::AbsoluteIdxWrite).cloned().collect(),

        InstructionSequenceMode::Relative => sequence_for_mode_map(InstructionSequenceMode::Relative).cloned().collect(),

        InstructionSequenceMode::ZeroPageIdxIndirect => sequence_for_mode_map(InstructionSequenceMode::ZeroPageIdxIndirect).cloned().collect(),
        InstructionSequenceMode::ZeroPageIdxIndirectReadModifyWrite => sequence_for_mode_map(InstructionSequenceMode::ZeroPageIdxIndirectReadModifyWrite).cloned().collect(),

        InstructionSequenceMode::ZeroPageIndirectIdxRead => sequence_for_mode_map(InstructionSequenceMode::ZeroPageIndirectIdxRead).cloned().collect(),
        InstructionSequenceMode::ZeroPageIndirectIdxReadModifyWrite => sequence_for_mode_map(InstructionSequenceMode::ZeroPageIndirectIdxReadModifyWrite).cloned().collect(),
        InstructionSequenceMode::ZeroPageIndirectIdxWrite => sequence_for_mode_map(InstructionSequenceMode::ZeroPageIndirectIdxWrite).cloned().collect(),

        InstructionSequenceMode::AbsoluteIndirectJump => sequence_for_mode_map(InstructionSequenceMode::AbsoluteIndirectJump).cloned().collect(),
    }
}

pub fn create_instruction_mode_sequences() -> SequenceMap {
    let mut sequences_map = collections::HashMap::from([
        (
            InstructionSequenceMode::FetchInstr,
            vec![
                instr::MicroInstruction::Fetch,
                instr::MicroInstruction::YieldClock,
            ],
        ),
        (
            InstructionSequenceMode::Reset,
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
                instr::MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::Discard,
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
                instr::MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::Discard,
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
                    src: SelectedRegister8::Status,
                },
                instr::MicroInstruction::DecrementRegister {
                    dst: SelectedRegister8::SP,
                },
                instr::MicroInstruction::YieldClock,
                // Next clock
                instr::MicroInstruction::CopyRegister16 {
                    dst: SelectedRegister16::Addr,
                    src: SelectedRegister16::ProgramStartAddrLow,
                },
                instr::MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::PCLow,
                },
                instr::MicroInstruction::YieldClock,
                // Next clock
                instr::MicroInstruction::CopyRegister16 {
                    dst: SelectedRegister16::Addr,
                    src: SelectedRegister16::ProgramStartAddrHigh,
                },
                instr::MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::PCHigh,
                },
                instr::MicroInstruction::YieldClock,
                // Next clock
            ],
        ),
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
                instr::MicroInstruction::SetStatusFlag {
                    flag: StatusRegFlags::IRQ_DISABLE,
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
            InstructionSequenceMode::StartIrq,
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
                    flag: StatusRegFlags::IRQ_DISABLE,
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
            InstructionSequenceMode::StartNmi,
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
                    src: SelectedRegister16::NMInterruptAddrLow,
                },
                instr::MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::PCLow,
                },
                instr::MicroInstruction::YieldClock,
                // Next clock
                instr::MicroInstruction::CopyRegister16 {
                    dst: SelectedRegister16::Addr,
                    src: SelectedRegister16::NMInterruptAddHigh,
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
                instr::MicroInstruction::PushFlagToTmp {
                    //flag: StatusRegFlags::IRQ_DISABLE,
                    flag: StatusRegFlags::BREAK | StatusRegFlags::UNUSED,
                },
                instr::MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::Status,
                },
                instr::MicroInstruction::PopFlagFromTmp {
                    //flag: StatusRegFlags::IRQ_DISABLE ,
                    flag: StatusRegFlags::BREAK | StatusRegFlags::UNUSED,
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
                instr::MicroInstruction::CopyRegister {
                    dst: SelectedRegister8::AddrHigh,
                    src: SelectedRegister8::StackPage,
                },
                instr::MicroInstruction::CopyRegister {
                    dst: SelectedRegister8::AddrLow,
                    src: SelectedRegister8::SP,
                },
                instr::MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::Discard,
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
                    increment: false,
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
            InstructionSequenceMode::ZeroPageIdx,
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
                // Next clock
                instr::MicroInstruction::AddIndexToAddress,
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
                instr::MicroInstruction::WriteAddress {
                    src: SelectedRegister8::Tmp,
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
    ]);

    for (mode, m_instrs) in sequences_map.iter_mut() {
        let last = m_instrs.last_mut();
        assert!(last.is_some(), "Sequence mode {:?} is empty", mode); //use this instead of expect to format

        // SAFETY: just asserted it's not None
        let last = unsafe { last.unwrap_unchecked() };
        assert!(matches!(
            *last,
            instr::MicroInstruction::YieldClock | instr::MicroInstruction::FinishInstruction
        ));
        *last = instr::MicroInstruction::FinishInstruction;
    }

    sequences_map
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instr;

    #[test]
    fn instructionsequences_checklastmicroinstruction_yieldclock() {
        for (mode, m_instrs) in MODES_SEQUENCES_DEFS.iter() {
            let last = *m_instrs.last().expect("Sequence mode is empty");
            assert_eq!(
                last,
                instr::MicroInstruction::FinishInstruction,
                "Sequence mode {:?} doesn't end with FinishInstruction",
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
                    let last_memory_read_before_op =
                        sequence[..position].iter().rev().find(|&instr| {
                            matches!(instr, instr::MicroInstruction::ReadAddress { .. })
                        });

                    assert_eq!(
                        last_memory_read_before_op,
                        Some(&instr::MicroInstruction::ReadAddress {
                            dst: SelectedRegister8::Tmp
                        })
                    );

                    let next_memory_write_after_op = sequence[position..].iter().find(|&instr| {
                        matches!(instr, instr::MicroInstruction::WriteAddress { .. })
                    });

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

    #[test]
    fn check_no_consecutive_yield() {
        let sequences = &*MODES_SEQUENCES_DEFS;

        for (mode, m_instrs) in sequences {
            assert!(
                m_instrs.windows(2).all(|w| !matches!(
                    w[0],
                    instr::MicroInstruction::YieldClock
                        | instr::MicroInstruction::FinishInstruction
                ) || !matches!(
                    w[1],
                    instr::MicroInstruction::YieldClock
                        | instr::MicroInstruction::FinishInstruction
                )),
                "Sequence mode {:?} has consecutive YieldClock's",
                mode
            );
        }
    }
}
