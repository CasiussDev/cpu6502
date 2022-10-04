use crate::alu;
use crate::alu::{AluBinaryOp, AluUnaryOp};
use crate::pinout::{DataDirectionMode, Pinout};
use crate::registers::register_file::{SelectedRegister16, SelectedRegister8};
use crate::registers::{Reg8, RegisterFile, StatusRegFlags};

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

fn execute_alu_unary(op: AluUnaryOp, selected_reg: SelectedRegister8, regs: &mut RegisterFile) {
    let mut status = regs.status;
    let reg = regs.get_selected_register8(selected_reg);
    match op {
        AluUnaryOp::Inc => alu::inc(reg, &mut status),
        AluUnaryOp::Dec => alu::dec(reg, &mut status),
    };
    regs.status = status;
}

fn execute_alu_binary(op: AluBinaryOp, operand: SelectedRegister8, regs: &mut RegisterFile) {
    //let mut status = regs.status;
    let operand = regs.get_copy_selected_register8(operand);
    match op {
        AluBinaryOp::Add => alu::add(&mut regs.a, &operand, &mut regs.status),
        AluBinaryOp::Sub => alu::sub(&mut regs.a, &operand, &mut regs.status),
        AluBinaryOp::And => alu::and(&mut regs.a, &operand, &mut regs.status),
        AluBinaryOp::Or => alu::or(&mut regs.a, &operand, &mut regs.status),
        AluBinaryOp::Xor => alu::xor(&mut regs.a, &operand, &mut regs.status),
        AluBinaryOp::Cmp => {
            alu::cmp(&mut regs.a, &operand, &mut regs.status);
        }
    }
}

pub enum ExecutionStatus {
    YieldClock,
    Continue,
    RunOp,
    WaitMemory { dst: Option<SelectedRegister8> },
}
#[allow(dead_code)]
pub fn execute(
    micro_instr: MicroInstruction,
    index_reg: Option<SelectedRegister8>,
    regs: &mut RegisterFile,
    pins: &mut Pinout,
) -> ExecutionStatus {
    match micro_instr {
        MicroInstruction::Fetch => {
            pins.set_address_output(regs.pc.get_u16());
            pins.set_data_direction(DataDirectionMode::Read);
            regs.pc.inc();
            return ExecutionStatus::WaitMemory {
                dst: Some(SelectedRegister8::IR),
            };
        }
        MicroInstruction::ReadPC { dst, increment } => {
            pins.set_address_output(regs.pc.get_u16());
            pins.set_data_direction(DataDirectionMode::Read);
            if increment {
                regs.pc.inc();
            };
            return ExecutionStatus::WaitMemory { dst: Some(dst) };
        }
        MicroInstruction::ReadAddress { dst } => {
            pins.set_address_output(regs.addr.get_u16());
            pins.set_data_direction(DataDirectionMode::Read);
            return ExecutionStatus::WaitMemory { dst: Some(dst) };
        }
        MicroInstruction::WriteAddress { src } => {
            pins.set_address_output(regs.addr.get_u16());
            pins.set_data_direction(DataDirectionMode::Write);
            let data = regs.get_copy_selected_register8(src).get_u8();
            pins.set_data_output(data);
            return ExecutionStatus::WaitMemory { dst: None };
        }
        MicroInstruction::CopyRegister { dst, src } => {
            let src = regs.get_copy_selected_register8(src);
            regs.set_selected_register8(dst, src);
        }
        MicroInstruction::CopyRegister16 { dst, src } => {
            let src = regs.get_copy_selected_register16(src);
            regs.set_selected_register16(dst, src);
        }
        MicroInstruction::ZeroRegister { dst } => regs.set_selected_register8(dst, Reg8::default()),
        MicroInstruction::IncrementRegister { dst } => {
            let dst = regs.get_selected_register8(dst);
            dst.inc();
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
            assert!(
                (index_reg == SelectedRegister8::X) || (index_reg == SelectedRegister8::Y),
                "using a wrong register for index"
            );
            let index_reg = regs.get_copy_selected_register8(index_reg);
            let addr_low = regs.addr.get_low_u8();
            let addr_low = addr_low.wrapping_add(index_reg.get_u8());
            regs.addr.set_low_u8(addr_low);
        }
        MicroInstruction::FixAddress => {}
        MicroInstruction::RunOperation => return ExecutionStatus::RunOp,
        MicroInstruction::YieldClock => return ExecutionStatus::YieldClock,
        MicroInstruction::FixAddressOrRunOpAndFinish => {}
        MicroInstruction::FixAddressOrIncrementPC => {}
    }

    ExecutionStatus::Continue
}
