use crate::alu::{AluBinaryOp, AluUnaryOp};
use crate::instructions::microinstructions::{MicroInstruction, MicroInstructionsVector};
use crate::registers::{SelectedRegister, StatusRegFlags};
use std::collections::HashMap;

#[cfg(test)]
use strum::IntoEnumIterator;
#[cfg(test)]
use strum_macros::EnumIter;

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
    Or,
    And,
    Xor,
    Add,
    Sub,
    Cmp,
    StoreA,
    LoadA,
}

impl Default for InstructionOp {
    fn default() -> Self {
        InstructionOp::Nop
    }
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
        (
            InstructionOp::Or,
            vec![
                MicroInstruction::ReadAddress {
                    dst: SelectedRegister::Tmp,
                },
                MicroInstruction::AluBinaryOp {
                    op: AluBinaryOp::Or,
                    operand: SelectedRegister::Tmp,
                },
            ],
        ),
        (
            InstructionOp::And,
            vec![
                MicroInstruction::ReadAddress {
                    dst: SelectedRegister::Tmp,
                },
                MicroInstruction::AluBinaryOp {
                    op: AluBinaryOp::And,
                    operand: SelectedRegister::Tmp,
                },
            ],
        ),
        (
            InstructionOp::Xor,
            vec![
                MicroInstruction::ReadAddress {
                    dst: SelectedRegister::Tmp,
                },
                MicroInstruction::AluBinaryOp {
                    op: AluBinaryOp::Xor,
                    operand: SelectedRegister::Tmp,
                },
            ],
        ),
        (
            InstructionOp::Add,
            vec![
                MicroInstruction::ReadAddress {
                    dst: SelectedRegister::Tmp,
                },
                MicroInstruction::AluBinaryOp {
                    op: AluBinaryOp::Add,
                    operand: SelectedRegister::Tmp,
                },
            ],
        ),
        (
            InstructionOp::Sub,
            vec![
                MicroInstruction::ReadAddress {
                    dst: SelectedRegister::Tmp,
                },
                MicroInstruction::AluBinaryOp {
                    op: AluBinaryOp::Sub,
                    operand: SelectedRegister::Tmp,
                },
            ],
        ),
        (
            InstructionOp::Cmp,
            vec![
                MicroInstruction::ReadAddress {
                    dst: SelectedRegister::Tmp,
                },
                MicroInstruction::AluBinaryOp {
                    op: AluBinaryOp::Cmp,
                    operand: SelectedRegister::Tmp,
                },
            ],
        ),
        (
            InstructionOp::StoreA,
            vec![MicroInstruction::WriteAddress {
                src: SelectedRegister::A,
            }],
        ),
        (
            InstructionOp::LoadA,
            vec![MicroInstruction::ReadAddress {
                dst: SelectedRegister::A,
            }],
        ),
    ]);

    ops
}

#[cfg(test)]
mod tests {
    use super::*;

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
