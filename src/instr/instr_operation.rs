use crate::alu;
use crate::instr;
use crate::registers::{ReferenceableRegister8, SelectedRegister8, StatusRegFlags};
use enum_map::{enum_map, Enum};
use lazy_static::lazy_static;
use std::{collections, slice};

#[cfg(test)]
use strum::IntoEnumIterator;
#[cfg(test)]
use strum_macros::EnumIter;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Enum, Debug)]
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
type SequenceOpEnumMap = enum_map::EnumMap<InstructionOp, instr::MicroInstructionsVector>;

lazy_static! {
    static ref OPS_SEQUENCES_DEFS: OpsMap = create_instructionops_sequences();
}

lazy_static! {
    static ref OPS_SEQUENCES_ENUM_MAP: SequenceOpEnumMap =
        create_instruction_op_sequences_enum_map();
}

pub fn sequence_for_op(op: InstructionOp) -> slice::Iter<'static, instr::MicroInstruction> {
    //OPS_SEQUENCES_DEFS.get(&op).map(|x| x.iter()).unwrap()
    if cfg!(feature = "enummaps") {
        OPS_SEQUENCES_ENUM_MAP[op].iter()
    } else {
        OPS_SEQUENCES_DEFS.get(&op).unwrap().iter()
    }
}

pub fn sequence_for_op_map(op: InstructionOp) -> slice::Iter<'static, instr::MicroInstruction> {
    OPS_SEQUENCES_DEFS.get(&op).map(|x| x.iter()).unwrap()
}

impl Default for InstructionOp {
    fn default() -> Self {
        InstructionOp::Nop
    }
}

pub fn create_instruction_op_sequences_enum_map() -> SequenceOpEnumMap {
    enum_map! {
        InstructionOp::Nop => sequence_for_op_map(InstructionOp::Nop).cloned().collect(),
        InstructionOp::IncrementMemory => sequence_for_op_map(InstructionOp::IncrementMemory).cloned().collect(),
        InstructionOp::IncrementX => sequence_for_op_map(InstructionOp::IncrementX).cloned().collect(),
        InstructionOp::IncrementY => sequence_for_op_map(InstructionOp::IncrementY).cloned().collect(),
        InstructionOp::DecrementMemory => sequence_for_op_map(InstructionOp::DecrementMemory).cloned().collect(),
        InstructionOp::DecrementX => sequence_for_op_map(InstructionOp::DecrementX).cloned().collect(),
        InstructionOp::DecrementY => sequence_for_op_map(InstructionOp::DecrementY).cloned().collect(),
        InstructionOp::ClearCarry => sequence_for_op_map(InstructionOp::ClearCarry).cloned().collect(),
        InstructionOp::SetCarry => sequence_for_op_map(InstructionOp::SetCarry).cloned().collect(),
        InstructionOp::ClearDecimal => sequence_for_op_map(InstructionOp::ClearDecimal).cloned().collect(),
        InstructionOp::SetDecimal => sequence_for_op_map(InstructionOp::SetDecimal).cloned().collect(),
        InstructionOp::ClearInterruptDisable => sequence_for_op_map(InstructionOp::ClearInterruptDisable).cloned().collect(),
        InstructionOp::SetInterruptDisable => sequence_for_op_map(InstructionOp::SetInterruptDisable).cloned().collect(),
        InstructionOp::ClearOverflow => sequence_for_op_map(InstructionOp::ClearOverflow).cloned().collect(),
        InstructionOp::SetOverflow => sequence_for_op_map(InstructionOp::SetOverflow).cloned().collect(),
        InstructionOp::TransferAccumulatorToX => sequence_for_op_map(InstructionOp::TransferAccumulatorToX).cloned().collect(),
        InstructionOp::TransferAccumulatorToY => sequence_for_op_map(InstructionOp::TransferAccumulatorToY).cloned().collect(),
        InstructionOp::TransferStackPtrToX => sequence_for_op_map(InstructionOp::TransferStackPtrToX).cloned().collect(),
        InstructionOp::TransferXToAccumulator => sequence_for_op_map(InstructionOp::TransferXToAccumulator).cloned().collect(),
        InstructionOp::TransferYToAccumulator => sequence_for_op_map(InstructionOp::TransferYToAccumulator).cloned().collect(),
        InstructionOp::TransferXToStackPtr => sequence_for_op_map(InstructionOp::TransferXToStackPtr).cloned().collect(),
        InstructionOp::PushA => sequence_for_op_map(InstructionOp::PushA).cloned().collect(),
        InstructionOp::PushStatus => sequence_for_op_map(InstructionOp::PushStatus).cloned().collect(),
        InstructionOp::PullA => sequence_for_op_map(InstructionOp::PullA).cloned().collect(),
        InstructionOp::PullStatus => sequence_for_op_map(InstructionOp::PullStatus).cloned().collect(),
        InstructionOp::Or => sequence_for_op_map(InstructionOp::Or).cloned().collect(),
        InstructionOp::And => sequence_for_op_map(InstructionOp::And).cloned().collect(),
        InstructionOp::Xor => sequence_for_op_map(InstructionOp::Xor).cloned().collect(),
        InstructionOp::Add => sequence_for_op_map(InstructionOp::Add).cloned().collect(),
        InstructionOp::Sub => sequence_for_op_map(InstructionOp::Sub).cloned().collect(),
        InstructionOp::Cmp => sequence_for_op_map(InstructionOp::Cmp).cloned().collect(),
        InstructionOp::Cpx => sequence_for_op_map(InstructionOp::Cpx).cloned().collect(),
        InstructionOp::Cpy => sequence_for_op_map(InstructionOp::Cpy).cloned().collect(),
        InstructionOp::Bit => sequence_for_op_map(InstructionOp::Bit).cloned().collect(),
        InstructionOp::BitImmediate => sequence_for_op_map(InstructionOp::BitImmediate).cloned().collect(),
        InstructionOp::ShiftLeftA => sequence_for_op_map(InstructionOp::ShiftLeftA).cloned().collect(),
        InstructionOp::ShiftRightA => sequence_for_op_map(InstructionOp::ShiftRightA).cloned().collect(),
        InstructionOp::RotateLeftA => sequence_for_op_map(InstructionOp::RotateLeftA).cloned().collect(),
        InstructionOp::RotateRightA => sequence_for_op_map(InstructionOp::RotateRightA).cloned().collect(),
        InstructionOp::ShiftLeftMemory => sequence_for_op_map(InstructionOp::ShiftLeftMemory).cloned().collect(),
        InstructionOp::ShiftRightMemory => sequence_for_op_map(InstructionOp::ShiftRightMemory).cloned().collect(),
        InstructionOp::RotateLeftMemory => sequence_for_op_map(InstructionOp::RotateLeftMemory).cloned().collect(),
        InstructionOp::RotateRightMemory => sequence_for_op_map(InstructionOp::RotateRightMemory).cloned().collect(),
        InstructionOp::StoreA => sequence_for_op_map(InstructionOp::StoreA).cloned().collect(),
        InstructionOp::LoadA => sequence_for_op_map(InstructionOp::LoadA).cloned().collect(),
        InstructionOp::StoreX => sequence_for_op_map(InstructionOp::StoreX).cloned().collect(),
        InstructionOp::LoadX => sequence_for_op_map(InstructionOp::LoadX).cloned().collect(),
        InstructionOp::StoreY => sequence_for_op_map(InstructionOp::StoreY).cloned().collect(),
        InstructionOp::LoadY => sequence_for_op_map(InstructionOp::LoadY).cloned().collect(),
        InstructionOp::BranchPlus => sequence_for_op_map(InstructionOp::BranchPlus).cloned().collect(),
        InstructionOp::BranchMinus => sequence_for_op_map(InstructionOp::BranchMinus).cloned().collect(),
        InstructionOp::BranchOverflowClear => sequence_for_op_map(InstructionOp::BranchOverflowClear).cloned().collect(),
        InstructionOp::BranchOverflowSet => sequence_for_op_map(InstructionOp::BranchOverflowSet).cloned().collect(),
        InstructionOp::BranchCarryClear => sequence_for_op_map(InstructionOp::BranchCarryClear).cloned().collect(),
        InstructionOp::BranchCarrySet => sequence_for_op_map(InstructionOp::BranchCarrySet).cloned().collect(),
        InstructionOp::BranchNotEqual => sequence_for_op_map(InstructionOp::BranchNotEqual).cloned().collect(),
        InstructionOp::BranchEqual => sequence_for_op_map(InstructionOp::BranchEqual).cloned().collect(),
    }
}

pub fn create_instructionops_sequences() -> OpsMap {
    collections::HashMap::from([
        (InstructionOp::Nop, vec![]),
        (
            InstructionOp::IncrementMemory,
            vec![instr::MicroInstruction::AluUnaryOp {
                op: alu::UnaryOp::Inc,
                reg: ReferenceableRegister8::Tmp,
            }],
        ),
        (
            InstructionOp::IncrementX,
            vec![instr::MicroInstruction::AluUnaryOp {
                op: alu::UnaryOp::Inc,
                reg: ReferenceableRegister8::X,
            }],
        ),
        (
            InstructionOp::IncrementY,
            vec![instr::MicroInstruction::AluUnaryOp {
                op: alu::UnaryOp::Inc,
                reg: ReferenceableRegister8::Y,
            }],
        ),
        (
            InstructionOp::DecrementMemory,
            vec![instr::MicroInstruction::AluUnaryOp {
                op: alu::UnaryOp::Dec,
                reg: ReferenceableRegister8::Tmp,
            }],
        ),
        (
            InstructionOp::DecrementX,
            vec![instr::MicroInstruction::AluUnaryOp {
                op: alu::UnaryOp::Dec,
                reg: ReferenceableRegister8::X,
            }],
        ),
        (
            InstructionOp::DecrementY,
            vec![instr::MicroInstruction::AluUnaryOp {
                op: alu::UnaryOp::Dec,
                reg: ReferenceableRegister8::Y,
            }],
        ),
        (
            InstructionOp::ShiftLeftA,
            vec![instr::MicroInstruction::AluUnaryOp {
                op: alu::UnaryOp::Asl,
                reg: ReferenceableRegister8::A,
            }],
        ),
        (
            InstructionOp::ShiftRightA,
            vec![instr::MicroInstruction::AluUnaryOp {
                op: alu::UnaryOp::Lsr,
                reg: ReferenceableRegister8::A,
            }],
        ),
        (
            InstructionOp::RotateLeftA,
            vec![instr::MicroInstruction::AluUnaryOp {
                op: alu::UnaryOp::Rol,
                reg: ReferenceableRegister8::A,
            }],
        ),
        (
            InstructionOp::RotateRightA,
            vec![instr::MicroInstruction::AluUnaryOp {
                op: alu::UnaryOp::Ror,
                reg: ReferenceableRegister8::A,
            }],
        ),
        (
            InstructionOp::ShiftLeftMemory,
            vec![instr::MicroInstruction::AluUnaryOp {
                op: alu::UnaryOp::Asl,
                reg: ReferenceableRegister8::Tmp,
            }],
        ),
        (
            InstructionOp::ShiftRightMemory,
            vec![instr::MicroInstruction::AluUnaryOp {
                op: alu::UnaryOp::Lsr,
                reg: ReferenceableRegister8::Tmp,
            }],
        ),
        (
            InstructionOp::RotateLeftMemory,
            vec![instr::MicroInstruction::AluUnaryOp {
                op: alu::UnaryOp::Rol,
                reg: ReferenceableRegister8::Tmp,
            }],
        ),
        (
            InstructionOp::RotateRightMemory,
            vec![instr::MicroInstruction::AluUnaryOp {
                op: alu::UnaryOp::Ror,
                reg: ReferenceableRegister8::Tmp,
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
            vec![
                instr::MicroInstruction::CopyRegister {
                    src: SelectedRegister8::A,
                    dst: SelectedRegister8::X,
                },
                instr::MicroInstruction::UpdateStatusFlagsNZ {
                    reg: SelectedRegister8::X,
                },
            ],
        ),
        (
            InstructionOp::TransferAccumulatorToY,
            vec![
                instr::MicroInstruction::CopyRegister {
                    src: SelectedRegister8::A,
                    dst: SelectedRegister8::Y,
                },
                instr::MicroInstruction::UpdateStatusFlagsNZ {
                    reg: SelectedRegister8::Y,
                },
            ],
        ),
        (
            InstructionOp::TransferStackPtrToX,
            vec![
                instr::MicroInstruction::CopyRegister {
                    src: SelectedRegister8::SP,
                    dst: SelectedRegister8::X,
                },
                instr::MicroInstruction::UpdateStatusFlagsNZ {
                    reg: SelectedRegister8::X,
                },
            ],
        ),
        (
            InstructionOp::TransferXToAccumulator,
            vec![
                instr::MicroInstruction::CopyRegister {
                    src: SelectedRegister8::X,
                    dst: SelectedRegister8::A,
                },
                instr::MicroInstruction::UpdateStatusFlagsNZ {
                    reg: SelectedRegister8::A,
                },
            ],
        ),
        (
            InstructionOp::TransferYToAccumulator,
            vec![
                instr::MicroInstruction::CopyRegister {
                    src: SelectedRegister8::Y,
                    dst: SelectedRegister8::A,
                },
                instr::MicroInstruction::UpdateStatusFlagsNZ {
                    reg: SelectedRegister8::A,
                },
            ],
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
            vec![
                instr::MicroInstruction::CopyRegister {
                    src: SelectedRegister8::Status,
                    dst: SelectedRegister8::Tmp,
                },
                instr::MicroInstruction::SetFlagsTmp {
                    flags: StatusRegFlags::BREAK | StatusRegFlags::UNUSED,
                },
                instr::MicroInstruction::WriteAddress {
                    src: SelectedRegister8::Tmp,
                },
            ],
        ),
        (
            InstructionOp::PullA,
            vec![
                instr::MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::A,
                },
                instr::MicroInstruction::UpdateStatusFlagsNZ {
                    reg: SelectedRegister8::A,
                },
            ],
        ),
        (
            InstructionOp::PullStatus,
            vec![
                instr::MicroInstruction::PushFlagToTmp {
                    flag: StatusRegFlags::BREAK | StatusRegFlags::UNUSED,
                },
                instr::MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::Status,
                },
                instr::MicroInstruction::PopFlagFromTmp {
                    flag: StatusRegFlags::BREAK | StatusRegFlags::UNUSED,
                },
            ],
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
            vec![
                instr::MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::A,
                },
                instr::MicroInstruction::UpdateStatusFlagsNZ {
                    reg: SelectedRegister8::A,
                },
            ],
        ),
        (
            InstructionOp::StoreX,
            vec![instr::MicroInstruction::WriteAddress {
                src: SelectedRegister8::X,
            }],
        ),
        (
            InstructionOp::LoadX,
            vec![
                instr::MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::X,
                },
                instr::MicroInstruction::UpdateStatusFlagsNZ {
                    reg: SelectedRegister8::X,
                },
            ],
        ),
        (
            InstructionOp::StoreY,
            vec![instr::MicroInstruction::WriteAddress {
                src: SelectedRegister8::Y,
            }],
        ),
        (
            InstructionOp::LoadY,
            vec![
                instr::MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::Y,
                },
                instr::MicroInstruction::UpdateStatusFlagsNZ {
                    reg: SelectedRegister8::Y,
                },
            ],
        ),
        (
            InstructionOp::Bit,
            vec![
                instr::MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::Tmp,
                },
                instr::MicroInstruction::BitInstr,
            ],
        ),
        (
            InstructionOp::BitImmediate,
            vec![
                instr::MicroInstruction::ReadAddress {
                    dst: SelectedRegister8::Tmp,
                },
                instr::MicroInstruction::BitInstrImmediate,
            ],
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
