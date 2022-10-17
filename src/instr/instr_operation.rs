use crate::alu;
use crate::instr;
use crate::registers::{SelectedRegister8, StatusRegFlags};
use lazy_static::lazy_static;
use std::collections;

#[cfg(test)]
use strum::IntoEnumIterator;
#[cfg(test)]
use strum_macros::EnumIter;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
#[cfg_attr(test, derive(EnumIter))]
pub enum InstructionOp {
    Nop,
    IncrementMemory,
    IncrementX,
    IncrementY,
    DecrementMemory,
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
    Or,
    And,
    Xor,
    Add,
    Sub,
    Cmp,
    Cpx,
    Cpy,
    Bit,
    BitImmediate,
    ShiftLeftA,
    ShiftRightA,
    RotateLeftA,
    RotateRightA,
    ShiftLeftMemory,
    ShiftRightMemory,
    RotateLeftMemory,
    RotateRightMemory,
    StoreA,
    LoadA,
    StoreX,
    LoadX,
    StoreY,
    LoadY,
    BranchPlus,
    BranchMinus,
    BranchOverflowClear,
    BranchOverflowSet,
    BranchCarryClear,
    BranchCarrySet,
    BranchNotEqual,
    BranchEqual,
}

type OpsMap = collections::HashMap<InstructionOp, instr::MicroInstructionsVector>;

lazy_static! {
    static ref OPS_SEQUENCES_DEFS: OpsMap = create_instructionops_sequences();
}

pub fn get_ops_map() -> &'static OpsMap {
    &OPS_SEQUENCES_DEFS
}

impl Default for InstructionOp {
    fn default() -> Self {
        InstructionOp::Nop
    }
}

pub fn create_instructionops_sequences() -> OpsMap {
    collections::HashMap::from([
        (InstructionOp::Nop, vec![]),
        (
            InstructionOp::IncrementMemory,
            vec![instr::MicroInstruction::AluUnaryOp {
                op: alu::UnaryOp::Inc,
                reg: SelectedRegister8::Tmp,
            }],
        ),
        (
            InstructionOp::IncrementX,
            vec![instr::MicroInstruction::AluUnaryOp {
                op: alu::UnaryOp::Inc,
                reg: SelectedRegister8::X,
            }],
        ),
        (
            InstructionOp::IncrementY,
            vec![instr::MicroInstruction::AluUnaryOp {
                op: alu::UnaryOp::Inc,
                reg: SelectedRegister8::Y,
            }],
        ),
        (
            InstructionOp::DecrementMemory,
            vec![instr::MicroInstruction::AluUnaryOp {
                op: alu::UnaryOp::Dec,
                reg: SelectedRegister8::Tmp,
            }],
        ),
        (
            InstructionOp::DecrementX,
            vec![instr::MicroInstruction::AluUnaryOp {
                op: alu::UnaryOp::Dec,
                reg: SelectedRegister8::X,
            }],
        ),
        (
            InstructionOp::DecrementY,
            vec![instr::MicroInstruction::AluUnaryOp {
                op: alu::UnaryOp::Dec,
                reg: SelectedRegister8::Y,
            }],
        ),
        (
            InstructionOp::ShiftLeftA,
            vec![instr::MicroInstruction::AluUnaryOp {
                op: alu::UnaryOp::Asl,
                reg: SelectedRegister8::A,
            }],
        ),
        (
            InstructionOp::ShiftRightA,
            vec![instr::MicroInstruction::AluUnaryOp {
                op: alu::UnaryOp::Lsr,
                reg: SelectedRegister8::A,
            }],
        ),
        (
            InstructionOp::RotateLeftA,
            vec![instr::MicroInstruction::AluUnaryOp {
                op: alu::UnaryOp::Rol,
                reg: SelectedRegister8::A,
            }],
        ),
        (
            InstructionOp::RotateRightA,
            vec![instr::MicroInstruction::AluUnaryOp {
                op: alu::UnaryOp::Ror,
                reg: SelectedRegister8::A,
            }],
        ),
        (
            InstructionOp::ShiftLeftMemory,
            vec![instr::MicroInstruction::AluUnaryOp {
                op: alu::UnaryOp::Asl,
                reg: SelectedRegister8::Tmp,
            }],
        ),
        (
            InstructionOp::ShiftRightMemory,
            vec![instr::MicroInstruction::AluUnaryOp {
                op: alu::UnaryOp::Lsr,
                reg: SelectedRegister8::Tmp,
            }],
        ),
        (
            InstructionOp::RotateLeftMemory,
            vec![instr::MicroInstruction::AluUnaryOp {
                op: alu::UnaryOp::Rol,
                reg: SelectedRegister8::Tmp,
            }],
        ),
        (
            InstructionOp::RotateRightMemory,
            vec![instr::MicroInstruction::AluUnaryOp {
                op: alu::UnaryOp::Ror,
                reg: SelectedRegister8::Tmp,
            }],
        ),
        (
            InstructionOp::ClearCarry,
            vec![instr::MicroInstruction::ClearStatusFlag {
                flag: StatusRegFlags::CARRY,
            }],
        ),
        (
            InstructionOp::SetCarry,
            vec![instr::MicroInstruction::SetStatusFlag {
                flag: StatusRegFlags::CARRY,
            }],
        ),
        (
            InstructionOp::ClearDecimal,
            vec![instr::MicroInstruction::ClearStatusFlag {
                flag: StatusRegFlags::DECIMAL,
            }],
        ),
        (
            InstructionOp::SetDecimal,
            vec![instr::MicroInstruction::SetStatusFlag {
                flag: StatusRegFlags::DECIMAL,
            }],
        ),
        (
            InstructionOp::ClearInterruptDisable,
            vec![instr::MicroInstruction::ClearStatusFlag {
                flag: StatusRegFlags::IRQ_DISABLE,
            }],
        ),
        (
            InstructionOp::SetInterruptDisable,
            vec![instr::MicroInstruction::SetStatusFlag {
                flag: StatusRegFlags::IRQ_DISABLE,
            }],
        ),
        (
            InstructionOp::ClearOverflow,
            vec![instr::MicroInstruction::ClearStatusFlag {
                flag: StatusRegFlags::OVERFLOW,
            }],
        ),
        (
            InstructionOp::SetOverflow,
            vec![instr::MicroInstruction::SetStatusFlag {
                flag: StatusRegFlags::OVERFLOW,
            }],
        ),
        (
            InstructionOp::TransferAccumulatorToX,
            vec![instr::MicroInstruction::CopyRegister {
                src: SelectedRegister8::A,
                dst: SelectedRegister8::X,
            }],
        ),
        (
            InstructionOp::TransferAccumulatorToY,
            vec![instr::MicroInstruction::CopyRegister {
                src: SelectedRegister8::A,
                dst: SelectedRegister8::Y,
            }],
        ),
        (
            InstructionOp::TransferStackPtrToX,
            vec![instr::MicroInstruction::CopyRegister {
                src: SelectedRegister8::SP,
                dst: SelectedRegister8::X,
            }],
        ),
        (
            InstructionOp::TransferXToAccumulator,
            vec![instr::MicroInstruction::CopyRegister {
                src: SelectedRegister8::X,
                dst: SelectedRegister8::A,
            }],
        ),
        (
            InstructionOp::TransferYToAccumulator,
            vec![instr::MicroInstruction::CopyRegister {
                src: SelectedRegister8::Y,
                dst: SelectedRegister8::A,
            }],
        ),
        (
            InstructionOp::TransferXToStackPtr,
            vec![instr::MicroInstruction::CopyRegister {
                src: SelectedRegister8::X,
                dst: SelectedRegister8::SP,
            }],
        ),
        (
            InstructionOp::PushA,
            vec![instr::MicroInstruction::WriteAddress {
                src: SelectedRegister8::A,
            }],
        ),
        (
            InstructionOp::PushStatus,
            vec![instr::MicroInstruction::WriteAddress {
                src: SelectedRegister8::Status,
            }],
        ),
        (
            InstructionOp::PullA,
            vec![instr::MicroInstruction::ReadAddress {
                dst: SelectedRegister8::A,
            }],
        ),
        (
            InstructionOp::PullStatus,
            vec![instr::MicroInstruction::ReadAddress {
                dst: SelectedRegister8::Status,
            }],
        ),
        (
            InstructionOp::Or,
            vec![
                instr::MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::Tmp,
                },
                instr::MicroInstruction::AluBinaryOp {
                    op: alu::BinaryOp::Or,
                    operand: SelectedRegister8::Tmp,
                },
            ],
        ),
        (
            InstructionOp::And,
            vec![
                instr::MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::Tmp,
                },
                instr::MicroInstruction::AluBinaryOp {
                    op: alu::BinaryOp::And,
                    operand: SelectedRegister8::Tmp,
                },
            ],
        ),
        (
            InstructionOp::Xor,
            vec![
                instr::MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::Tmp,
                },
                instr::MicroInstruction::AluBinaryOp {
                    op: alu::BinaryOp::Xor,
                    operand: SelectedRegister8::Tmp,
                },
            ],
        ),
        (
            InstructionOp::Add,
            vec![
                instr::MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::Tmp,
                },
                instr::MicroInstruction::AluBinaryOp {
                    op: alu::BinaryOp::Add,
                    operand: SelectedRegister8::Tmp,
                },
            ],
        ),
        (
            InstructionOp::Sub,
            vec![
                instr::MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::Tmp,
                },
                instr::MicroInstruction::AluBinaryOp {
                    op: alu::BinaryOp::Sub,
                    operand: SelectedRegister8::Tmp,
                },
            ],
        ),
        (
            InstructionOp::Cmp,
            vec![
                instr::MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::Tmp,
                },
                instr::MicroInstruction::AluBinaryOp {
                    op: alu::BinaryOp::Cmp,
                    operand: SelectedRegister8::Tmp,
                },
            ],
        ),
        (
            InstructionOp::Cpx,
            vec![
                instr::MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::Tmp,
                },
                instr::MicroInstruction::AluCompareIndex {
                    index: SelectedRegister8::X,
                },
            ],
        ),
        (
            InstructionOp::Cpy,
            vec![
                instr::MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::Tmp,
                },
                instr::MicroInstruction::AluCompareIndex {
                    index: SelectedRegister8::Y,
                },
            ],
        ),
        (
            InstructionOp::StoreA,
            vec![instr::MicroInstruction::WriteAddress {
                src: SelectedRegister8::A,
            }],
        ),
        (
            InstructionOp::LoadA,
            vec![instr::MicroInstruction::ReadAddress {
                dst: SelectedRegister8::A,
            }],
        ),
        (
            InstructionOp::StoreX,
            vec![instr::MicroInstruction::WriteAddress {
                src: SelectedRegister8::X,
            }],
        ),
        (
            InstructionOp::LoadX,
            vec![instr::MicroInstruction::ReadAddress {
                dst: SelectedRegister8::X,
            }],
        ),
        (
            InstructionOp::StoreY,
            vec![instr::MicroInstruction::WriteAddress {
                src: SelectedRegister8::Y,
            }],
        ),
        (
            InstructionOp::LoadY,
            vec![instr::MicroInstruction::ReadAddress {
                dst: SelectedRegister8::Y,
            }],
        ),
        (InstructionOp::Bit, vec![instr::MicroInstruction::BitInstr]),
        (
            InstructionOp::BitImmediate,
            vec![instr::MicroInstruction::BitInstrImmediate],
        ),
        (
            InstructionOp::BranchPlus,
            vec![instr::MicroInstruction::TakeConditionalBranch {
                flag_to_test: StatusRegFlags::NEGATIVE,
                branch_if_set: false,
            }],
        ),
        (
            InstructionOp::BranchMinus,
            vec![instr::MicroInstruction::TakeConditionalBranch {
                flag_to_test: StatusRegFlags::NEGATIVE,
                branch_if_set: true,
            }],
        ),
        (
            InstructionOp::BranchOverflowClear,
            vec![instr::MicroInstruction::TakeConditionalBranch {
                flag_to_test: StatusRegFlags::OVERFLOW,
                branch_if_set: false,
            }],
        ),
        (
            InstructionOp::BranchOverflowSet,
            vec![instr::MicroInstruction::TakeConditionalBranch {
                flag_to_test: StatusRegFlags::OVERFLOW,
                branch_if_set: true,
            }],
        ),
        (
            InstructionOp::BranchCarryClear,
            vec![instr::MicroInstruction::TakeConditionalBranch {
                flag_to_test: StatusRegFlags::CARRY,
                branch_if_set: false,
            }],
        ),
        (
            InstructionOp::BranchCarrySet,
            vec![instr::MicroInstruction::TakeConditionalBranch {
                flag_to_test: StatusRegFlags::CARRY,
                branch_if_set: true,
            }],
        ),
        (
            InstructionOp::BranchNotEqual,
            vec![instr::MicroInstruction::TakeConditionalBranch {
                flag_to_test: StatusRegFlags::ZERO,
                branch_if_set: false,
            }],
        ),
        (
            InstructionOp::BranchEqual,
            vec![instr::MicroInstruction::TakeConditionalBranch {
                flag_to_test: StatusRegFlags::ZERO,
                branch_if_set: true,
            }],
        ),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_all_ops_implemented() {
        let instruction_ops = &*OPS_SEQUENCES_DEFS;
        for op in InstructionOp::iter() {
            assert!(
                instruction_ops.contains_key(&op),
                "Operation {:?} not implemented",
                op,
            );
        }
    }
}
