use crate::alu::{AluBinaryOp, AluUnaryOp};
use crate::instructions::microinstructions::{MicroInstruction, MicroInstructionsVector};
use crate::registers::{SelectedRegister8, StatusRegFlags};
use lazy_static::lazy_static;
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
}

type OpsMap = std::collections::HashMap<InstructionOp, MicroInstructionsVector>;

lazy_static! {
    static ref OPS_SEQUENCES_DEFS: OpsMap = create_instructionops_sequences();
}

impl Default for InstructionOp {
    fn default() -> Self {
        InstructionOp::Nop
    }
}

#[allow(dead_code)]
pub fn create_instructionops_sequences() -> OpsMap {
    let ops = HashMap::from([
        (InstructionOp::Nop, vec![]),
        (
            InstructionOp::IncrementMemory,
            vec![MicroInstruction::AluUnaryOp {
                op: AluUnaryOp::Inc,
                reg: SelectedRegister8::Tmp,
            }],
        ),
        (
            InstructionOp::IncrementX,
            vec![MicroInstruction::AluUnaryOp {
                op: AluUnaryOp::Inc,
                reg: SelectedRegister8::X,
            }],
        ),
        (
            InstructionOp::IncrementY,
            vec![MicroInstruction::AluUnaryOp {
                op: AluUnaryOp::Inc,
                reg: SelectedRegister8::Y,
            }],
        ),
        (
            InstructionOp::DecrementMemory,
            vec![MicroInstruction::AluUnaryOp {
                op: AluUnaryOp::Dec,
                reg: SelectedRegister8::Tmp,
            }],
        ),
        (
            InstructionOp::DecrementX,
            vec![MicroInstruction::AluUnaryOp {
                op: AluUnaryOp::Dec,
                reg: SelectedRegister8::X,
            }],
        ),
        (
            InstructionOp::DecrementY,
            vec![MicroInstruction::AluUnaryOp {
                op: AluUnaryOp::Dec,
                reg: SelectedRegister8::Y,
            }],
        ),
        (
            InstructionOp::ShiftLeftA,
            vec![MicroInstruction::AluUnaryOp {
                op: AluUnaryOp::Asl,
                reg: SelectedRegister8::A,
            }],
        ),
        (
            InstructionOp::ShiftRightA,
            vec![MicroInstruction::AluUnaryOp {
                op: AluUnaryOp::Lsr,
                reg: SelectedRegister8::A,
            }],
        ),
        (
            InstructionOp::RotateLeftA,
            vec![MicroInstruction::AluUnaryOp {
                op: AluUnaryOp::Rol,
                reg: SelectedRegister8::A,
            }],
        ),
        (
            InstructionOp::RotateRightA,
            vec![MicroInstruction::AluUnaryOp {
                op: AluUnaryOp::Ror,
                reg: SelectedRegister8::A,
            }],
        ),
        (
            InstructionOp::ShiftLeftMemory,
            vec![MicroInstruction::AluUnaryOp {
                op: AluUnaryOp::Asl,
                reg: SelectedRegister8::Tmp,
            }],
        ),
        (
            InstructionOp::ShiftRightMemory,
            vec![MicroInstruction::AluUnaryOp {
                op: AluUnaryOp::Lsr,
                reg: SelectedRegister8::Tmp,
            }],
        ),
        (
            InstructionOp::RotateLeftMemory,
            vec![MicroInstruction::AluUnaryOp {
                op: AluUnaryOp::Rol,
                reg: SelectedRegister8::Tmp,
            }],
        ),
        (
            InstructionOp::RotateRightMemory,
            vec![MicroInstruction::AluUnaryOp {
                op: AluUnaryOp::Ror,
                reg: SelectedRegister8::Tmp,
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
                src: SelectedRegister8::A,
                dst: SelectedRegister8::X,
            }],
        ),
        (
            InstructionOp::TransferAccumulatorToY,
            vec![MicroInstruction::CopyRegister {
                src: SelectedRegister8::A,
                dst: SelectedRegister8::Y,
            }],
        ),
        (
            InstructionOp::TransferStackPtrToX,
            vec![MicroInstruction::CopyRegister {
                src: SelectedRegister8::SP,
                dst: SelectedRegister8::X,
            }],
        ),
        (
            InstructionOp::TransferXToAccumulator,
            vec![MicroInstruction::CopyRegister {
                src: SelectedRegister8::X,
                dst: SelectedRegister8::A,
            }],
        ),
        (
            InstructionOp::TransferYToAccumulator,
            vec![MicroInstruction::CopyRegister {
                src: SelectedRegister8::Y,
                dst: SelectedRegister8::A,
            }],
        ),
        (
            InstructionOp::TransferXToStackPtr,
            vec![MicroInstruction::CopyRegister {
                src: SelectedRegister8::X,
                dst: SelectedRegister8::SP,
            }],
        ),
        (
            InstructionOp::PushA,
            vec![MicroInstruction::WriteAddress {
                src: SelectedRegister8::A,
            }],
        ),
        (
            InstructionOp::PushStatus,
            vec![MicroInstruction::WriteAddress {
                src: SelectedRegister8::Status,
            }],
        ),
        (
            InstructionOp::PullA,
            vec![MicroInstruction::ReadAddress {
                dst: SelectedRegister8::A,
            }],
        ),
        (
            InstructionOp::PullStatus,
            vec![MicroInstruction::ReadAddress {
                dst: SelectedRegister8::Status,
            }],
        ),
        (
            InstructionOp::Or,
            vec![
                MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::Tmp,
                },
                MicroInstruction::AluBinaryOp {
                    op: AluBinaryOp::Or,
                    operand: SelectedRegister8::Tmp,
                },
            ],
        ),
        (
            InstructionOp::And,
            vec![
                MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::Tmp,
                },
                MicroInstruction::AluBinaryOp {
                    op: AluBinaryOp::And,
                    operand: SelectedRegister8::Tmp,
                },
            ],
        ),
        (
            InstructionOp::Xor,
            vec![
                MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::Tmp,
                },
                MicroInstruction::AluBinaryOp {
                    op: AluBinaryOp::Xor,
                    operand: SelectedRegister8::Tmp,
                },
            ],
        ),
        (
            InstructionOp::Add,
            vec![
                MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::Tmp,
                },
                MicroInstruction::AluBinaryOp {
                    op: AluBinaryOp::Add,
                    operand: SelectedRegister8::Tmp,
                },
            ],
        ),
        (
            InstructionOp::Sub,
            vec![
                MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::Tmp,
                },
                MicroInstruction::AluBinaryOp {
                    op: AluBinaryOp::Sub,
                    operand: SelectedRegister8::Tmp,
                },
            ],
        ),
        (
            InstructionOp::Cmp,
            vec![
                MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::Tmp,
                },
                MicroInstruction::AluBinaryOp {
                    op: AluBinaryOp::Cmp,
                    operand: SelectedRegister8::Tmp,
                },
            ],
        ),
        (
            InstructionOp::StoreA,
            vec![MicroInstruction::WriteAddress {
                src: SelectedRegister8::A,
            }],
        ),
        (
            InstructionOp::LoadA,
            vec![MicroInstruction::ReadAddress {
                dst: SelectedRegister8::A,
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
