use crate::registers::register_file::{SelectedRegister16, SelectedRegister8};
use crate::registers::{IndexRegister, RegisterFile, StatusRegFlags};
use crate::{alu, pinout};

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
        op: alu::UnaryOp,
        reg: SelectedRegister8,
    },
    AluBinaryOp {
        op: alu::BinaryOp,
        operand: SelectedRegister8,
    },
    AluCompareIndex {
        index: SelectedRegister8,
    },
    SetStatusFlag {
        flag: StatusRegFlags,
    },
    ClearStatusFlag {
        flag: StatusRegFlags,
    },
    BitInstr,
    BitInstrImmediate,
    TakeConditionalBranch {
        flag_to_test: StatusRegFlags,
        branch_if_set: bool,
    },
    PushFlagToTmp {
        flag: StatusRegFlags,
    },
    PopFlagFromTmp {
        flag: StatusRegFlags,
    },
    AddIndexToAddress,
    FixAddress,
    RunOperation,

    YieldClock,
    FinishInstruction,

    FixAddressOrRunOpAndFinish,
    FixAddressOrIncrementPC,
}

pub type MicroInstructionsVector = Vec<MicroInstruction>;

fn execute_alu_unary(op: alu::UnaryOp, selected_reg: SelectedRegister8, regs: &mut RegisterFile) {
    let mut status = regs.status;
    let reg = regs.get_selected_register8(selected_reg);
    match op {
        alu::UnaryOp::Inc => alu::inc(reg, &mut status),
        alu::UnaryOp::Dec => alu::dec(reg, &mut status),
        alu::UnaryOp::Asl => alu::shift_left(reg, &mut status),
        alu::UnaryOp::Lsr => alu::shift_right(reg, &mut status),
        alu::UnaryOp::Rol => alu::rotate_left(reg, &mut status),
        alu::UnaryOp::Ror => alu::rotate_right(reg, &mut status),
    };
    regs.status = status;
}

fn execute_alu_binary(op: alu::BinaryOp, operand: SelectedRegister8, regs: &mut RegisterFile) {
    //let mut status = regs.status;
    let operand = regs.get_copy_selected_register8(operand);
    match op {
        alu::BinaryOp::Add => alu::add(&mut regs.a, &operand, &mut regs.status),
        alu::BinaryOp::Sub => alu::sub(&mut regs.a, &operand, &mut regs.status),
        alu::BinaryOp::And => alu::and(&mut regs.a, &operand, &mut regs.status),
        alu::BinaryOp::Or => alu::or(&mut regs.a, &operand, &mut regs.status),
        alu::BinaryOp::Xor => alu::xor(&mut regs.a, &operand, &mut regs.status),
        alu::BinaryOp::Cmp => alu::cmp(&regs.a, &operand, &mut regs.status),
    }
}

fn execute_alu_compare_index(operand: SelectedRegister8, regs: &mut RegisterFile) {
    match operand {
        SelectedRegister8::X => alu::cmp(&regs.x, &regs.tmp, &mut regs.status),
        SelectedRegister8::Y => alu::cmp(&regs.y, &regs.tmp, &mut regs.status),
        _ => panic!("trying to call index comparison using an invalid register"),
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum ExecutionStatus {
    YieldClock,
    Continue,
    RunOp,
    WaitMemory { dst: Option<SelectedRegister8> },
    RunOpAndFinish,
    FinishInstruction,
}

pub fn execute(
    micro_instr: MicroInstruction,
    index_reg: Option<IndexRegister>,
    regs: &mut RegisterFile,
    pins: &mut pinout::Pinout,
) -> ExecutionStatus {
    match micro_instr {
        MicroInstruction::Fetch => {
            pins.set_address_output(regs.pc.get_u16());
            pins.set_data_direction(pinout::DataDirectionMode::Read);
            regs.pc.inc();
            return ExecutionStatus::WaitMemory {
                dst: Some(SelectedRegister8::IR),
            };
        }
        MicroInstruction::ReadPC { dst, increment } => {
            pins.set_address_output(regs.pc.get_u16());
            pins.set_data_direction(pinout::DataDirectionMode::Read);
            if increment {
                regs.pc.inc();
            };
            return ExecutionStatus::WaitMemory { dst: Some(dst) };
        }
        MicroInstruction::ReadAddress { dst } => {
            pins.set_address_output(regs.addr.get_u16());
            pins.set_data_direction(pinout::DataDirectionMode::Read);
            return ExecutionStatus::WaitMemory { dst: Some(dst) };
        }
        MicroInstruction::WriteAddress { src } => {
            pins.set_address_output(regs.addr.get_u16());
            pins.set_data_direction(pinout::DataDirectionMode::Write);
            let data = regs.get_copy_selected_register8(src).get_u8();
            pins.set_data_output(data);
            return ExecutionStatus::WaitMemory { dst: None };
        }
        MicroInstruction::CopyRegister { dst, src } => {
            let src = regs.get_copy_selected_register8(src);
            regs.set_selected_register8(dst, src.get_u8());
        }
        MicroInstruction::CopyRegister16 { dst, src } => {
            let src = regs.get_copy_selected_register16(src);
            regs.set_selected_register16(dst, src);
        }
        MicroInstruction::ZeroRegister { dst } => regs.set_selected_register8(dst, 0),
        MicroInstruction::IncrementRegister { dst } => {
            //let dst = regs.get_selected_register8(dst);
            //dst.inc();
            let mut dst_reg = regs.get_copy_selected_register8(dst);
            dst_reg.inc();
            regs.set_selected_register8(dst, dst_reg.get_u8());
        }
        MicroInstruction::IncrementRegister16 { dst } => {
            let dst = regs.get_selected_register16(dst);
            dst.inc();
        }
        MicroInstruction::DecrementRegister { dst } => {
            let dst = regs.get_selected_register8(dst);
            dst.dec();
        }
        MicroInstruction::AluUnaryOp { op, reg } => execute_alu_unary(op, reg, regs),
        MicroInstruction::AluBinaryOp { op, operand } => execute_alu_binary(op, operand, regs),
        MicroInstruction::AluCompareIndex { index } => execute_alu_compare_index(index, regs),
        MicroInstruction::SetStatusFlag { flag } => {
            regs.status.set_flags(flag);
        }
        MicroInstruction::ClearStatusFlag { flag } => {
            regs.status.clear_flags(flag);
        }
        MicroInstruction::AddIndexToAddress => {
            assert!(
                index_reg.is_some(),
                "index register not specified for MicroInstruction::AddIndexToAddress"
            );
            let index_reg = index_reg.unwrap();
            let index_reg = regs.get_copy_selected_register8(index_reg.into());
            let addr_low = regs.addr.get_low_u8();
            let addr_low = addr_low.wrapping_add(index_reg.get_u8());
            regs.addr.set_low_u8(addr_low);
        }
        MicroInstruction::FixAddress => {
            let addr_low_value = regs.addr.get_low_u8();
            let index_value = regs
                .get_copy_selected_register8(
                    index_reg.expect("Index register not specified").into(),
                )
                .get_u8();

            if index_value > addr_low_value {
                let addr_high_value = regs.addr.get_high_u8().wrapping_add(1);
                regs.addr.set_high_u8(addr_high_value);
            }
        }
        MicroInstruction::RunOperation => return ExecutionStatus::RunOp,
        MicroInstruction::YieldClock => return ExecutionStatus::YieldClock,
        MicroInstruction::FinishInstruction => return ExecutionStatus::FinishInstruction,
        MicroInstruction::FixAddressOrRunOpAndFinish => {
            let addr_low_value = regs.addr.get_low_u8();
            let index_value = regs
                .get_copy_selected_register8(
                    index_reg.expect("Index register not specified").into(),
                )
                .get_u8();

            if index_value > addr_low_value {
                let addr_high_value = regs.addr.get_high_u8().wrapping_add(1);
                regs.addr.set_high_u8(addr_high_value);
            } else {
                return ExecutionStatus::RunOpAndFinish;
            }
        }
        MicroInstruction::FixAddressOrIncrementPC => {
            let addr_low_value = regs.addr.get_low_u8();
            let index_value = regs
                .get_copy_selected_register8(
                    index_reg.expect("Index register not specified").into(),
                )
                .get_u8();

            if index_value > addr_low_value {
                let addr_high_value = regs.addr.get_high_u8().wrapping_add(1);
                regs.addr.set_high_u8(addr_high_value);
            } else {
                regs.pc.inc();
            }
        }
        MicroInstruction::BitInstr => {
            let msb = regs.tmp.get_u8() & 0x80;
            let smsb = regs.tmp.get_u8() & 0x40;
            regs.status.update_flags(StatusRegFlags::NEGATIVE, msb != 0);
            regs.status
                .update_flags(StatusRegFlags::OVERFLOW, smsb != 0);
            alu::bit_compare(regs.a, regs.tmp, &mut regs.status);
        }
        MicroInstruction::BitInstrImmediate => {
            alu::bit_compare(regs.a, regs.tmp, &mut regs.status);
        }
        MicroInstruction::TakeConditionalBranch {
            flag_to_test,
            branch_if_set,
        } => {
            let must_branch = regs.status.are_all_flags_set(flag_to_test) == branch_if_set;
            if must_branch {
                let (pc_low_byte, carry) = regs.pc.get_low_u8().overflowing_add(regs.tmp.get_u8());
                regs.pc.set_low_u8(pc_low_byte);
                if carry == false {
                    return ExecutionStatus::FinishInstruction;
                }
            } else {
                regs.pc.inc();
                return ExecutionStatus::FinishInstruction;
            }
        }
        MicroInstruction::PushFlagToTmp { flag } => {
            if regs.status.are_all_flags_set(flag) {
                regs.tmp.set_u8(1);
            } else {
                regs.tmp.set_u8(0);
            }
        }
        MicroInstruction::PopFlagFromTmp { flag } => {
            let flag_value = regs.tmp.get_u8() != 0;
            regs.status.update_flags(flag, flag_value);
        }
    }

    ExecutionStatus::Continue
}
