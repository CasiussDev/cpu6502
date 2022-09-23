use crate::alu::{AluBinaryOp, AluUnaryOp};
use crate::registers::register_file::{SelectedRegister8, SelectedRegister16};
use crate::registers::StatusRegFlags;

#[allow(dead_code)]
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum MicroInstruction {
    Fetch,
    ReadPC {
        dst: SelectedRegister8,
        increment: bool,
    },
    ReadAddress {
        dst: SelectedRegister8,
    },
    WriteAddress {
        src: SelectedRegister8,
    },
    CopyRegister {
        dst: SelectedRegister8,
        src: SelectedRegister8,
    },
    CopyRegister16 {
        dst: SelectedRegister16,
        src: SelectedRegister16,
    },
    ZeroRegister {
        dst: SelectedRegister8,
    },
    IncrementRegister {
        dst: SelectedRegister8,
    },
    IncrementRegister16 {
        dst: SelectedRegister16,
    },
    DecrementRegister {
        dst: SelectedRegister8,
    },
    AluUnaryOp {
        op: AluUnaryOp,
        reg: SelectedRegister8,
    },
    AluBinaryOp {
        op: AluBinaryOp,
        operand: SelectedRegister8,
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

    FixAddressOrRunOpAndFinish,
    FixAddressOrIncrementPC,
}

pub type MicroInstructionsVector = std::vec::Vec<MicroInstruction>;