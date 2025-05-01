#[cfg(test)]
mod tests;

use crate::registers::{Reg8, StatusReg, StatusRegFlags};

pub fn update_status_nz(result: i8, status_register: &mut StatusReg) {
    if result < 0 {
        status_register.set_flags(StatusRegFlags::NEGATIVE);
        status_register.clear_flags(StatusRegFlags::ZERO);
    } else {
        status_register.clear_flags(StatusRegFlags::NEGATIVE);
        status_register.update_flags(StatusRegFlags::ZERO, result == 0);
    }
}

pub fn update_status_carry_add(carry: bool, status_register: &mut StatusReg) {
    status_register.update_flags(StatusRegFlags::CARRY, carry);
}

pub fn update_status_carry_sub(carry: bool, status_register: &mut StatusReg) {
    status_register.update_flags(StatusRegFlags::CARRY, !carry);
}

pub fn update_status_v(overflow: bool, status_register: &mut StatusReg) {
    status_register.update_flags(StatusRegFlags::OVERFLOW, overflow);
}

pub fn add(accumulator: &mut Reg8, operand: &Reg8, status_register: &mut StatusReg) {
    if cfg!(feature = "decimal") && status_register.are_all_flags_set(StatusRegFlags::DECIMAL) {
        todo!();
    } else {
        // Compilers are smart enough to do a single addition!
        let (mut result, mut carry) = accumulator.to_u8().overflowing_add(operand.to_u8());
        let (_, mut overflow) = accumulator.to_i8().overflowing_add(operand.to_i8());

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

pub fn cmp(accumulator: &Reg8, operand: &Reg8, status_register: &mut StatusReg) {
    let (result, carry) = accumulator.to_u8().overflowing_sub(operand.to_u8());

    update_status_carry_sub(carry, status_register);
    update_status_nz(result as i8, status_register);
}

pub fn sub(accumulator: &mut Reg8, operand: &Reg8, status_register: &mut StatusReg) {
    if cfg!(feature = "decimal") && status_register.are_all_flags_set(StatusRegFlags::DECIMAL) {
        todo!();
    } else {
        let (mut result, mut overflow) = accumulator.to_i8().overflowing_sub(operand.to_i8());
        let (_, mut carry) = accumulator.to_u8().overflowing_sub(operand.to_u8());

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

        accumulator.set_i8(result);
    }
}

pub fn inc(src_dst: &mut Reg8, status_register: &mut StatusReg) {
    if cfg!(feature = "decimal") && status_register.are_all_flags_set(StatusRegFlags::DECIMAL) {
        todo!();
    } else {
        let result = src_dst.to_u8().wrapping_add(1);

        update_status_nz(result as i8, status_register);

        src_dst.set_u8(result);
    }
}

pub fn dec(src_dst: &mut Reg8, status_register: &mut StatusReg) {
    if cfg!(feature = "decimal") && status_register.are_all_flags_set(StatusRegFlags::DECIMAL) {
        todo!();
    } else {
        let result = src_dst.to_i8().wrapping_sub(1);

        update_status_nz(result as i8, status_register);

        src_dst.set_i8(result);
    }
}

pub fn shift_left(src_dst: &mut Reg8, status_register: &mut StatusReg) {
    let msb = src_dst.to_u8() & 0x80;
    src_dst.shift_left();

    update_status_nz(src_dst.to_i8(), status_register);
    status_register.update_flags(StatusRegFlags::CARRY, msb != 0);
}

pub fn shift_right(src_dst: &mut Reg8, status_register: &mut StatusReg) {
    let lsb = src_dst.to_u8() & 0x01;
    src_dst.shift_right();

    update_status_nz(src_dst.to_i8(), status_register);
    status_register.update_flags(StatusRegFlags::CARRY, lsb != 0);
}

pub fn rotate_left(src_dst: &mut Reg8, status_register: &mut StatusReg) {
    let old_carry_bit_set = status_register.are_all_flags_set(StatusRegFlags::CARRY);

    let msb = src_dst.to_u8() & 0x80;
    src_dst.shift_left();

    if old_carry_bit_set {
        src_dst.inc();
    }

    update_status_nz(src_dst.to_i8(), status_register);
    status_register.update_flags(StatusRegFlags::CARRY, msb != 0);
}

pub fn rotate_right(src_dst: &mut Reg8, status_register: &mut StatusReg) {
    let old_carry_bit_set = status_register.are_all_flags_set(StatusRegFlags::CARRY);

    let lsb = src_dst.to_u8() & 0x01;
    src_dst.shift_right();

    if old_carry_bit_set {
        let result = src_dst.to_u8() | 0x80;
        src_dst.set_u8(result);
    }

    update_status_nz(src_dst.to_i8(), status_register);
    status_register.update_flags(StatusRegFlags::CARRY, lsb != 0);
}

pub fn and(accumulator: &mut Reg8, operand: &Reg8, status_register: &mut StatusReg) {
    let result = accumulator.to_u8() & operand.to_u8();

    update_status_nz(result as i8, status_register);

    accumulator.set_u8(result);
}

pub fn or(accumulator: &mut Reg8, operand: &Reg8, status_register: &mut StatusReg) {
    let result = accumulator.to_u8() | operand.to_u8();

    update_status_nz(result as i8, status_register);

    accumulator.set_u8(result);
}

pub fn xor(accumulator: &mut Reg8, operand: &Reg8, status_register: &mut StatusReg) {
    let result = accumulator.to_u8() ^ operand.to_u8();

    update_status_nz(result as i8, status_register);

    accumulator.set_u8(result);
}

pub fn bit_compare(accumulator: Reg8, operand: Reg8, status_register: &mut StatusReg) {
    let and = accumulator.to_u8() & operand.to_u8();
    status_register.update_flags(StatusRegFlags::ZERO, and == 0);
}
