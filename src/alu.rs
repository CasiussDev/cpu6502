mod tests;

use crate::registers::{Reg8, StatusReg, StatusRegFlags};

#[allow(dead_code)]
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum AluUnaryOp {
    Inc,
    Dec,
}

#[allow(dead_code)]
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum AluBinaryOp {
    Add,
    Sub,
    And,
    Or,
    Xor,
    Cmp,
}

#[allow(dead_code)]
pub fn update_status_nz(result: i8, status_register: &mut StatusReg) {
    if result < 0 {
        status_register.set_flags(StatusRegFlags::NEGATIVE);
        status_register.clear_flags(StatusRegFlags::ZERO);
    } else {
        status_register.clear_flags(StatusRegFlags::NEGATIVE);

        if result == 0 {
            status_register.set_flags(StatusRegFlags::ZERO);
        } else {
            status_register.clear_flags(StatusRegFlags::ZERO);
        }
    }
}

#[allow(dead_code)]
pub fn update_status_carry_add(carry: bool, status_register: &mut StatusReg) {
    if carry {
        status_register.set_flags(StatusRegFlags::CARRY);
    } else {
        status_register.clear_flags(StatusRegFlags::CARRY);
    }
}

#[allow(dead_code)]
pub fn update_status_carry_sub(carry: bool, status_register: &mut StatusReg) {
    if carry {
        status_register.clear_flags(StatusRegFlags::CARRY);
    } else {
        status_register.set_flags(StatusRegFlags::CARRY);
    }
}

#[allow(dead_code)]
pub fn update_status_v(overflow: bool, status_register: &mut StatusReg) {
    if overflow {
        status_register.set_flags(StatusRegFlags::OVERFLOW);
    } else {
        status_register.clear_flags(StatusRegFlags::OVERFLOW);
    }
}

#[allow(dead_code)]
pub fn add(accumulator: &mut Reg8, operand: &Reg8, status_register: &mut StatusReg) {
    if cfg!(feature = "decimal") && status_register.are_all_flags_set(StatusRegFlags::DECIMAL) {
        todo!();
    } else {
        // Compilers are smart enough to do a single addition!
        let (mut result, mut carry) = accumulator.get_u8().overflowing_add(operand.get_u8());
        let (_, mut overflow) = accumulator.get_i8().overflowing_add(operand.get_i8());

        if status_register.are_all_flags_set(StatusRegFlags::CARRY) {
            let (_, inc_overflow) = (result as i8).overflowing_add(1);
            let (new_result, inc_carry) = result.overflowing_add(1);
            result = new_result;
            carry |= inc_carry;
            overflow |= inc_overflow;
        }

        update_status_v(overflow, status_register);
        update_status_carry_add(carry, status_register);
        update_status_nz(result as i8, status_register);

        accumulator.set_u8(result);
    }
}

#[allow(dead_code)]
pub fn cmp(accumulator: &Reg8, operand: &Reg8, status_register: &mut StatusReg) -> i8 {
    let (mut result, mut overflow) = accumulator.get_i8().overflowing_sub(operand.get_i8());
    let (_, mut carry) = accumulator.get_u8().overflowing_sub(operand.get_u8());

    if status_register.are_all_flags_set(StatusRegFlags::CARRY) == false {
        let (_, dec_carry) = (result as u8).overflowing_sub(1);
        let (new_result, dec_overflow) = result.overflowing_sub(1);
        result = new_result;
        overflow |= dec_overflow;
        carry |= dec_carry;
    }

    update_status_v(overflow, status_register);
    update_status_carry_sub(carry, status_register);
    update_status_nz(result, status_register);

    result
}

#[allow(dead_code)]
pub fn sub(accumulator: &mut Reg8, operand: &Reg8, status_register: &mut StatusReg) {
    if cfg!(feature = "decimal") && status_register.are_all_flags_set(StatusRegFlags::DECIMAL) {
        todo!();
    } else {
        let result = cmp(accumulator, operand, status_register);
        accumulator.set_i8(result);
    }
}

#[allow(dead_code)]
pub fn inc(src_dst: &mut Reg8, status_register: &mut StatusReg) {
    if cfg!(feature = "decimal") && status_register.are_all_flags_set(StatusRegFlags::DECIMAL) {
        todo!();
    } else {
        let result = src_dst.get_u8().wrapping_add(1);

        update_status_nz(result as i8, status_register);

        src_dst.set_u8(result);
    }
}

#[allow(dead_code)]
pub fn dec(src_dst: &mut Reg8, status_register: &mut StatusReg) {
    if cfg!(feature = "decimal") && status_register.are_all_flags_set(StatusRegFlags::DECIMAL) {
        todo!();
    } else {
        let result = src_dst.get_i8().wrapping_sub(1);

        update_status_nz(result as i8, status_register);

        src_dst.set_i8(result);
    }
}

#[allow(dead_code)]
pub fn and(accumulator: &mut Reg8, operand: &Reg8, status_register: &mut StatusReg) {
    let result = accumulator.get_u8() & operand.get_u8();

    update_status_nz(result as i8, status_register);

    accumulator.set_u8(result);
}

#[allow(dead_code)]
pub fn or(accumulator: &mut Reg8, operand: &Reg8, status_register: &mut StatusReg) {
    let result = accumulator.get_u8() | operand.get_u8();

    update_status_nz(result as i8, status_register);

    accumulator.set_u8(result);
}

#[allow(dead_code)]
pub fn xor(accumulator: &mut Reg8, operand: &Reg8, status_register: &mut StatusReg) {
    let result = accumulator.get_u8() ^ operand.get_u8();

    update_status_nz(result as i8, status_register);

    accumulator.set_u8(result);
}
