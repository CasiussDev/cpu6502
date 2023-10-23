use crate::alu;
use crate::memory::MemorySpace;
use crate::registers::register_file::{SelectedRegister16, SelectedRegister8};
use crate::registers::{IndexRegister, ReferenceableRegister8, RegisterFile, StatusRegFlags};
use std::slice;

#[cfg(feature = "logging")]
use log::trace;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
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
        reg: ReferenceableRegister8,
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
    UpdateStatusFlagsNZ {
        reg: SelectedRegister8,
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
    SetFlagsTmp {
        flags: StatusRegFlags,
    },
    //ClearFlagsTmp {
    //    flags: StatusRegFlags,
    //},
    AddIndexToAddress,
    FixAddress,
    RunOperation,

    YieldClock,
    FinishInstruction,

    FixAddressOrRunOpAndFinish,
    FixAddressOrIncrementPC,
}

const FINISH_INSTR: [MicroInstruction; 1] = [MicroInstruction::FinishInstruction; 1];

//#[cfg(feature = "integration_test")]
//pub type FetchedInstr = Option<u16>;
//
//#[cfg(not(feature = "integration_test"))]
//pub type FetchedInstr = Option<()>;

#[cfg(feature = "integration_test")]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum FetchedInstr {
    Some(u16),
    None,
    Invalidate,
}

#[cfg(not(feature = "integration_test"))]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[cfg(feature = "integration_test")]
pub enum FetchedInstr {
    Some(u16),
    None,
    Invalidate,
}

impl Default for FetchedInstr {
    fn default() -> Self {
        FetchedInstr::None
    }
}

impl FetchedInstr {
    pub fn is_some(&self) -> bool {
        (*self != FetchedInstr::Invalidate) && (*self != FetchedInstr::None)
    }
}

pub fn finish_instr_sequence() -> slice::Iter<'static, MicroInstruction> {
    FINISH_INSTR.iter()
}

pub type MicroInstructionsVector = Vec<MicroInstruction>;

fn execute_alu_unary(
    op: alu::UnaryOp,
    selected_reg: ReferenceableRegister8,
    regs: &mut RegisterFile,
) {
    let mut status = regs.status;
    let reg = regs.selected_register8(selected_reg);
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
    let operand = regs.copy_selected_register8(operand);
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
    RunOpAndFinish,
    FinishInstruction,
    FinishInstructionBranch,
}

impl Default for ExecutionStatus {
    fn default() -> Self {
        ExecutionStatus::Continue
    }
}

pub fn execute(
    micro_instr: MicroInstruction,
    index_reg: Option<IndexRegister>,
    regs: &mut RegisterFile,
    memory: &mut impl MemorySpace,
) -> (ExecutionStatus, FetchedInstr) {
    #[cfg(feature = "logging")]
    {
        trace!("\t{:?}", micro_instr);
    }
    let mut fetched_instr = FetchedInstr::None;
    match micro_instr {
        MicroInstruction::Fetch => {
            let instr = memory.read(regs.pc.to_u16());
            regs.ir.set_u8(instr);

            #[cfg(feature = "integration_test")]
            {
                fetched_instr = FetchedInstr::Some(regs.pc.to_u16());
            }
            #[cfg(not(feature = "integration_test"))]
            {
                fetched_instr = FetchedInstr::Some();
            }

            regs.pc.inc();
        }
        MicroInstruction::ReadPC { dst, increment } => {
            let operand = memory.read(regs.pc.to_u16());

            if dst == SelectedRegister8::IR {
                #[cfg(feature = "integration_test")]
                {
                    fetched_instr = FetchedInstr::Some(regs.pc.to_u16());
                }
                #[cfg(not(feature = "integration_test"))]
                {
                    fetched_instr = FetchedInstr::Some();
                }
            }

            if increment {
                regs.pc.inc();
            };

            regs.set_selected_register8(dst, operand);
        }
        MicroInstruction::ReadAddress { dst } => {
            let data = memory.read(regs.addr.to_u16());
            regs.set_selected_register8(dst, data);
        }
        MicroInstruction::WriteAddress { src } => {
            let data = regs.copy_selected_register8(src).to_u8();
            memory.write(data, regs.addr.to_u16());
        }
        MicroInstruction::CopyRegister { dst, src } => {
            let src = regs.copy_selected_register8(src);
            regs.set_selected_register8(dst, src.to_u8());
        }
        MicroInstruction::CopyRegister16 { dst, src } => {
            let src = regs.copy_selected_register16(src);
            regs.set_selected_register16(dst, src);
        }
        MicroInstruction::ZeroRegister { dst } => regs.set_selected_register8(dst, 0),
        MicroInstruction::IncrementRegister { dst } => {
            let mut dst_reg = regs.copy_selected_register8(dst);
            dst_reg.inc();
            regs.set_selected_register8(dst, dst_reg.to_u8());
        }
        MicroInstruction::IncrementRegister16 { dst } => {
            let dst = regs.selected_register16(dst);
            dst.inc();
        }
        MicroInstruction::DecrementRegister { dst } => {
            let mut dst_reg = regs.copy_selected_register8(dst);
            dst_reg.dec();
            regs.set_selected_register8(dst, dst_reg.to_u8());
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
            let index_reg = index_reg
                .expect("index register not specified for MicroInstruction::AddIndexToAddress")
                .into();
            let index_reg = regs.copy_selected_register8(index_reg);
            let addr_low = regs.addr.low_u8();
            let addr_low = addr_low.wrapping_add(index_reg.to_u8());
            regs.addr.set_low_u8(addr_low);
        }
        MicroInstruction::FixAddress => {
            let addr_low_value = regs.addr.low_u8();
            let index_value = regs
                .copy_selected_register8(
                    index_reg
                        .expect("Index register not specified for MicroInstruction::FixAddress")
                        .into(),
                )
                .to_u8();

            if index_value > addr_low_value {
                let addr_high_value = regs.addr.high_u8().wrapping_add(1);
                regs.addr.set_high_u8(addr_high_value);
            }
        }
        MicroInstruction::RunOperation => return (ExecutionStatus::RunOp, FetchedInstr::None),
        MicroInstruction::YieldClock => return (ExecutionStatus::YieldClock, FetchedInstr::None),
        MicroInstruction::FinishInstruction => {
            return (ExecutionStatus::FinishInstruction, FetchedInstr::None)
        }
        MicroInstruction::FixAddressOrRunOpAndFinish => {
            let addr_low_value = regs.addr.low_u8();
            let index_value = regs
                .copy_selected_register8(
                    index_reg
                        .expect("Index register not specified MicroInstruction::FixAddressOrRunOpAndFinish")
                        .into(),
                )
                .to_u8();

            if index_value > addr_low_value {
                let addr_high_value = regs.addr.high_u8().wrapping_add(1);
                regs.addr.set_high_u8(addr_high_value);
            } else {
                return (ExecutionStatus::RunOpAndFinish, FetchedInstr::None);
            }
        }
        MicroInstruction::FixAddressOrIncrementPC => {
            let addr_low_value = regs.pc.low_u8();
            let index_value = regs.tmp.to_u8();

            if index_value > addr_low_value {
                let addr_high_value = regs.pc.high_u8().wrapping_add(1);
                regs.pc.set_high_u8(addr_high_value);

                fetched_instr = FetchedInstr::Invalidate;
            } else {
                regs.pc.inc();
            }
        }
        MicroInstruction::BitInstr => {
            let msb = regs.tmp.to_u8() & 0x80;
            let smsb = regs.tmp.to_u8() & 0x40;
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
            #[cfg(feature = "logging")]
            {
                trace!("\tmust_branch is {must_branch} with {flag_to_test:?}");
            }
            if must_branch {
                let prev_pc_low_byte = regs.pc.low_u8();
                let (pc_low_byte, carry) = regs.pc.low_u8().overflowing_add(regs.tmp.to_u8());
                #[cfg(feature = "logging")]
                {
                    trace!("\tmust_branch is {must_branch} with {flag_to_test:?}");
                }
                regs.pc.set_low_u8(pc_low_byte);
                fetched_instr = FetchedInstr::Invalidate;
                if regs.tmp.to_i8() > 0 {
                    if regs.tmp.to_u8() > pc_low_byte {
                        let pc_high_byte = regs.pc.high_u8().wrapping_add(1);
                        regs.pc.set_high_u8(pc_high_byte);
                    }
                } else {
                    if prev_pc_low_byte < pc_low_byte {
                        let pc_high_byte = regs.pc.high_u8().wrapping_sub(1);
                        regs.pc.set_high_u8(pc_high_byte);
                    }
                }

                if carry == false {
                    return (ExecutionStatus::FinishInstructionBranch, fetched_instr);
                }
            } else {
                regs.pc.inc();
                return (ExecutionStatus::FinishInstruction, FetchedInstr::None);
            }
        }
        MicroInstruction::PushFlagToTmp { flag } => {
            regs.tmp.set_u8(regs.status.to_u8() & flag.bits());
        }
        MicroInstruction::PopFlagFromTmp { flag } => {
            let tmp_mask = flag.bits();
            let status_mask = !tmp_mask;

            let new_status = (regs.status.to_u8() & status_mask) | (regs.tmp.to_u8() & tmp_mask);
            regs.status.set_u8(new_status);
        }
        MicroInstruction::SetFlagsTmp { flags } => {
            let new_value = regs.tmp.to_u8() | flags.bits();
            regs.tmp.set_u8(new_value);
        }
        //MicroInstruction::ClearFlagsTmp { flags } => {
        //    let new_value = regs.tmp.to_u8() & !flags.bits();
        //    regs.tmp.set_u8(new_value);
        //}
        MicroInstruction::UpdateStatusFlagsNZ { reg } => {
            let value = regs.copy_selected_register8(reg).to_i8();
            alu::update_status_nz(value, &mut regs.status);
        }
    }

    (ExecutionStatus::Continue, fetched_instr)
}
