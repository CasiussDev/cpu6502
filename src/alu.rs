use crate::registers::{Reg8, StatusReg, StatusRegFlags};

pub fn update_status_nz(result: i8, status_register: &mut StatusReg) {
    if result < 0 {
        status_register.set_flags(StatusRegFlags::NEGATIVE);
    } else {
        status_register.reset_flags(StatusRegFlags::NEGATIVE);

        if result == 0 {
            status_register.set_flags(StatusRegFlags::ZERO);
        } else {
            status_register.reset_flags(StatusRegFlags::ZERO);
        }
    }
}

pub fn update_status_carry_add(op1: u8, result: u8, status_register: &mut StatusReg) {
    if op1 > result {
        status_register.set_flags(StatusRegFlags::CARRY);
    } else {
        status_register.reset_flags(StatusRegFlags::CARRY);
    }
}

pub fn update_status_carry_sub(op1: u8, result: u8, status_register: &mut StatusReg) {
    if op1 < result {
        status_register.reset_flags(StatusRegFlags::CARRY);
    } else {
        status_register.set_flags(StatusRegFlags::CARRY);
    }
}

pub fn update_status_v(op1: i8, op2: i8, result: i8, status_register: &mut StatusReg) {
    if (op1.is_negative() == op2.is_negative()) && (op1.is_negative() != result.is_negative()) {
        status_register.set_flags(StatusRegFlags::OVERFLOW);
    } else {
        status_register.reset_flags(StatusRegFlags::OVERFLOW);
    }
}

pub fn add(accumulator: &mut Reg8, operand: &Reg8, status_register: &mut StatusReg) {
    let mut result = accumulator.get_u8().wrapping_add(operand.get_u8());

    if status_register.are_all_flags_set(StatusRegFlags::CARRY) {
        result = result.wrapping_add(1);
    }

    update_status_v(
        accumulator.get_i8(),
        operand.get_i8(),
        result as i8,
        status_register,
    );
    update_status_carry_add(accumulator.get_u8(), result, status_register);
    update_status_nz(result as i8, status_register);

    accumulator.set_u8(result);
}

pub fn sub(accumulator: &mut Reg8, operand: &Reg8, status_register: &mut StatusReg) {
    let mut result = accumulator.get_i8().wrapping_sub(operand.get_i8());

    if status_register.are_all_flags_set(StatusRegFlags::CARRY) == false {
        result = result.wrapping_sub(1);
    }

    update_status_v(
        accumulator.get_i8(),
        operand.get_i8(),
        result,
        status_register,
    );
    update_status_carry_sub(accumulator.get_u8(), result as u8, status_register);
    update_status_nz(result, status_register);

    accumulator.set_i8(result);
}

pub fn inc(src_dst: &mut Reg8, status_register: &mut StatusReg) {
    let result = src_dst.get_u8().wrapping_add(1);

    update_status_nz(result as i8, status_register);

    src_dst.set_u8(result);
}

pub fn dec(src_dst: &mut Reg8, status_register: &mut StatusReg) {
    let result = src_dst.get_i8().wrapping_sub(1);

    update_status_nz(result as i8, status_register);

    src_dst.set_i8(result);
}

pub fn and(accumulator: &mut Reg8, operand: &Reg8, status_register: &mut StatusReg) {
    let result = accumulator.get_u8() & operand.get_u8();

    update_status_nz(result as i8, status_register);

    accumulator.set_u8(result);
}

pub fn or(accumulator: &mut Reg8, operand: &Reg8, status_register: &mut StatusReg) {
    let result = accumulator.get_u8() | operand.get_u8();

    update_status_nz(result as i8, status_register);

    accumulator.set_u8(result);
}

pub fn xor(accumulator: &mut Reg8, operand: &Reg8, status_register: &mut StatusReg) {
    let result = accumulator.get_u8() ^ operand.get_u8();

    update_status_nz(result as i8, status_register);

    accumulator.set_u8(result);
}

#[cfg(test)]
mod tests {
    use crate::registers::{Reg8, StatusReg, StatusRegFlags};

    #[test]
    fn operands_addwithoutcarry_resultscorrect() {
        for op1 in 0..u8::MAX {
            for op2 in 0..=(u8::MAX - op1) {
                // GIVEN
                let mut status_register = StatusReg::default();

                let mut accumulator = Reg8::new(op1);
                let old_accumulator_value = op1;

                let operand = Reg8::new(op2);

                // WHEN
                super::add(&mut accumulator, &operand, &mut status_register);

                // THEN
                assert_eq!(
                    accumulator.get_u8(),
                    old_accumulator_value + operand.get_u8()
                );
                assert!(!status_register.are_all_flags_set(StatusRegFlags::CARRY));
                assert_eq!(
                    status_register.are_all_flags_set(StatusRegFlags::ZERO),
                    accumulator.get_u8() == 0
                );
                assert_eq!(
                    status_register.are_all_flags_set(StatusRegFlags::NEGATIVE),
                    accumulator.get_i8() < 0
                );
            }
        }
    }

    #[test]
    fn operands_addwithcarry_resultscorrect() {
        for op1 in 0..255 {
            for op2 in 0..(u8::MAX - op1) {
                // GIVEN
                let mut status_register = StatusReg::default();
                status_register.set_flags(StatusRegFlags::CARRY);

                let mut accumulator = Reg8::new(op1);
                let old_accumulator_value = op1;

                let operand = Reg8::new(op2);

                // WHEN
                super::add(&mut accumulator, &operand, &mut status_register);

                // THEN
                assert_eq!(
                    accumulator.get_u8(),
                    old_accumulator_value + operand.get_u8() + 1
                );
                assert!(!status_register.are_all_flags_set(StatusRegFlags::CARRY));
                assert_eq!(
                    status_register.are_all_flags_set(StatusRegFlags::ZERO),
                    accumulator.get_u8() == 0
                );
                assert_eq!(
                    status_register.are_all_flags_set(StatusRegFlags::NEGATIVE),
                    accumulator.get_i8() < 0
                );
            }
        }
    }

    #[test]
    fn operands_subwithoutborrow_resultscorrect() {
        for op1 in 0..u8::MAX {
            for op2 in 0..=op1 {
                // GIVEN
                let mut status_register = StatusReg::default();
                status_register.set_flags(StatusRegFlags::CARRY); // bit set for no borrow in sub

                let mut accumulator = Reg8::new(op1);
                let old_accumulator_value = op1;

                let operand = Reg8::new(op2);

                // WHEN
                super::sub(&mut accumulator, &operand, &mut status_register);

                // THEN
                assert_eq!(
                    accumulator.get_u8(),
                    old_accumulator_value - operand.get_u8()
                );
                assert!(status_register.are_all_flags_set(StatusRegFlags::CARRY));
                assert_eq!(
                    status_register.are_all_flags_set(StatusRegFlags::ZERO),
                    accumulator.get_u8() == 0
                );
                assert_eq!(
                    status_register.are_all_flags_set(StatusRegFlags::NEGATIVE),
                    accumulator.get_i8() < 0
                );
            }
        }
    }

    #[test]
    fn operands_subwithborrow_resultscorrect() {
        for op1 in 0..u8::MAX {
            for op2 in 0..op1 {
                // GIVEN
                let mut status_register = StatusReg::default();
                status_register.reset_flags(StatusRegFlags::CARRY); // bit reset for borrow in sub

                let mut accumulator = Reg8::new(op1);
                let old_accumulator_value = op1;

                let operand = Reg8::new(op2);

                // WHEN
                super::sub(&mut accumulator, &operand, &mut status_register);

                // THEN
                assert_eq!(
                    accumulator.get_u8(),
                    old_accumulator_value - operand.get_u8() - 1
                );
                assert!(status_register.are_all_flags_set(StatusRegFlags::CARRY));
                assert_eq!(
                    status_register.are_all_flags_set(StatusRegFlags::ZERO),
                    accumulator.get_u8() == 0
                );
                assert_eq!(
                    status_register.are_all_flags_set(StatusRegFlags::NEGATIVE),
                    accumulator.get_i8() < 0
                );
            }
        }
    }

    #[test]
    fn operands_addcausecarry_resultscorrect() {
        for op1 in 1..u8::MAX {
            for op2 in (u8::MAX - op1 + 1)..=u8::MAX {
                // GIVEN
                let mut status_register = StatusReg::default();

                let mut accumulator = Reg8::new(op1);
                let old_accumulator_value = op1;

                let operand = Reg8::new(op2);

                // WHEN
                super::add(&mut accumulator, &operand, &mut status_register);

                // THEN
                assert_eq!(
                    accumulator.get_u8(),
                    old_accumulator_value.wrapping_add(operand.get_u8())
                );
                assert!(status_register.are_all_flags_set(StatusRegFlags::CARRY));
                assert_eq!(
                    status_register.are_all_flags_set(StatusRegFlags::ZERO),
                    accumulator.get_u8() == 0
                );
                assert_eq!(
                    status_register.are_all_flags_set(StatusRegFlags::NEGATIVE),
                    accumulator.get_i8() < 0
                );
            }
        }
    }

    #[test]
    fn operands_subcauseborrow_resultscorrect() {
        for op1 in 0..u8::MAX {
            for op2 in (op1 + 1)..=u8::MAX {
                // GIVEN
                let mut status_register = StatusReg::default();
                status_register.set_flags(StatusRegFlags::CARRY);

                let mut accumulator = Reg8::new(op1);
                let old_accumulator_value = op1;

                let operand = Reg8::new(op2);

                // WHEN
                super::sub(&mut accumulator, &operand, &mut status_register);

                // THEN
                assert_eq!(
                    accumulator.get_u8(),
                    old_accumulator_value.wrapping_sub(operand.get_u8())
                );
                assert!(!status_register.are_all_flags_set(StatusRegFlags::CARRY));
                assert_eq!(
                    status_register.are_all_flags_set(StatusRegFlags::ZERO),
                    accumulator.get_u8() == 0
                );
                assert_eq!(
                    status_register.are_all_flags_set(StatusRegFlags::NEGATIVE),
                    accumulator.get_i8() < 0
                );
            }
        }
    }

    fn perform_signedpositiveoperands_add_overflowflagcorrect(
        mut op1: Reg8,
        op2: Reg8,
        expect_overflow: bool,
    ) {
        let mut status_register = StatusReg::default();
        super::add(&mut op1, &op2, &mut status_register);

        assert_eq!(
            status_register.are_all_flags_set(StatusRegFlags::OVERFLOW),
            expect_overflow
        );
    }

    #[test]
    fn signedpositiveoperands_add_overflowflagcorrect() {
        for op1 in 0..=i8::MAX {
            for op2 in 0..=(i8::MAX - op1) {
                let accumulator = Reg8::new_i8(op1);
                let operand = Reg8::new_i8(op2);

                perform_signedpositiveoperands_add_overflowflagcorrect(accumulator, operand, false);
            }
        }

        for op1 in 1..=i8::MAX {
            for op2 in (i8::MAX - op1 + 1)..=i8::MAX {
                let accumulator = Reg8::new_i8(op1);
                let operand = Reg8::new_i8(op2);

                perform_signedpositiveoperands_add_overflowflagcorrect(accumulator, operand, true);
            }
        }
    }

    #[test]
    fn signednegativeoperands_add_overflowflagcorrect() {
        for op1 in (i8::MIN + 1)..=-1 {
            for op2 in (i8::MIN - op1 + 1)..=-1 {
                let accumulator = Reg8::new_i8(op1);
                let operand = Reg8::new_i8(op2);

                perform_signedpositiveoperands_add_overflowflagcorrect(accumulator, operand, false);
            }
        }

        for op1 in i8::MIN..=-1 {
            for op2 in i8::MIN..=(i8::MIN - op1 - 1) {
                let accumulator = Reg8::new_i8(op1);
                let operand = Reg8::new_i8(op2);

                perform_signedpositiveoperands_add_overflowflagcorrect(accumulator, operand, true);
            }
        }
    }

    #[test]
    fn differentsignoperands_add_overflowflagcorrect() {
        for op1 in 0..=i8::MAX {
            for op2 in i8::MIN..=-1 {
                let accumulator = Reg8::new_i8(op1);
                let operand = Reg8::new_i8(op2);

                perform_signedpositiveoperands_add_overflowflagcorrect(accumulator, operand, false);
                perform_signedpositiveoperands_add_overflowflagcorrect(operand, accumulator, false);
            }
        }
    }
}
