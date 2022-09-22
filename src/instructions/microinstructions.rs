use crate::alu::{AluBinaryOp, AluUnaryOp};
use crate::registers::register_file::SelectedRegister;
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
        operand: SelectedRegister,
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