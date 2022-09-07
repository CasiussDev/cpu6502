use crate::registers::{Reg8, StatusReg, StatusRegFlags};

pub fn update_status_nz(result: i8, status_register: &mut StatusReg) {
    if result < 0 {
        status_register.set_flags(StatusRegFlags::NEGATIVE);
        status_register.reset_flags(StatusRegFlags::ZERO);
    } else {
        status_register.reset_flags(StatusRegFlags::NEGATIVE);

        if result == 0 {
            status_register.set_flags(StatusRegFlags::ZERO);
        } else {
            status_register.reset_flags(StatusRegFlags::ZERO);
        }
    }
}

pub fn update_status_carry_add(carry: bool, status_register: &mut StatusReg) {
    if carry {
        status_register.set_flags(StatusRegFlags::CARRY);
    } else {
        status_register.reset_flags(StatusRegFlags::CARRY);
    }
}

pub fn update_status_carry_sub(carry: bool, status_register: &mut StatusReg) {
    if carry {
        status_register.reset_flags(StatusRegFlags::CARRY);
    } else {
        status_register.set_flags(StatusRegFlags::CARRY);
    }
}

pub fn update_status_v(overflow: bool, status_register: &mut StatusReg) {
    if overflow {
        status_register.set_flags(StatusRegFlags::OVERFLOW);
    } else {
        status_register.reset_flags(StatusRegFlags::OVERFLOW);
    }
}

pub fn add(accumulator: &mut Reg8, operand: &Reg8, status_register: &mut StatusReg) {
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

pub fn sub(accumulator: &mut Reg8, operand: &Reg8, status_register: &mut StatusReg) {
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

    fn perform_operands_add_overflowflagcorrect(mut op1: Reg8, op2: Reg8, expect_overflow: bool) {
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

                perform_operands_add_overflowflagcorrect(accumulator, operand, false);
            }
        }

        for op1 in 1..=i8::MAX {
            for op2 in (i8::MAX - op1 + 1)..=i8::MAX {
                let accumulator = Reg8::new_i8(op1);
                let operand = Reg8::new_i8(op2);

                perform_operands_add_overflowflagcorrect(accumulator, operand, true);
            }
        }
    }

    #[test]
    fn signednegativeoperands_add_overflowflagcorrect() {
        for op1 in (i8::MIN + 1)..=-1 {
            for op2 in (i8::MIN - op1 + 1)..=-1 {
                let accumulator = Reg8::new_i8(op1);
                let operand = Reg8::new_i8(op2);

                perform_operands_add_overflowflagcorrect(accumulator, operand, false);
            }
        }

        for op1 in i8::MIN..=-1 {
            for op2 in i8::MIN..=(i8::MIN - op1 - 1) {
                let accumulator = Reg8::new_i8(op1);
                let operand = Reg8::new_i8(op2);

                perform_operands_add_overflowflagcorrect(accumulator, operand, true);
            }
        }
    }

    #[test]
    fn differentsignoperands_add_overflowflagcorrect() {
        for op1 in 0..=i8::MAX {
            for op2 in i8::MIN..=-1 {
                let accumulator = Reg8::new_i8(op1);
                let operand = Reg8::new_i8(op2);

                perform_operands_add_overflowflagcorrect(accumulator, operand, false);
                perform_operands_add_overflowflagcorrect(operand, accumulator, false);
            }
        }
    }

    fn perform_operands_sub_overflowflagcorrect(mut op1: Reg8, op2: Reg8, expect_overflow: bool) {
        let mut status_register = StatusReg::default();
        status_register.set_flags(StatusRegFlags::CARRY);
        super::sub(&mut op1, &op2, &mut status_register);

        assert_eq!(
            status_register.are_all_flags_set(StatusRegFlags::OVERFLOW),
            expect_overflow
        );
    }

    #[test]
    fn signedpositiveoperands_sub_overflowflagcorrect() {
        for op1 in 0..=i8::MAX {
            for op2 in 0..=i8::MAX {
                let accumulator = Reg8::new_i8(op1);
                let operand = Reg8::new_i8(op2);

                perform_operands_sub_overflowflagcorrect(accumulator, operand, false);
            }
        }
    }

    #[test]
    fn signednegativeoperands_sub_overflowflagcorrect() {
        for op1 in i8::MIN..=-1 {
            for op2 in i8::MIN..=-1 {
                let accumulator = Reg8::new_i8(op1);
                let operand = Reg8::new_i8(op2);

                perform_operands_sub_overflowflagcorrect(accumulator, operand, false);
            }
        }
    }

    #[test]
    fn differentsignoperands_sub_overflowflagcorrect() {
        for op1 in 0..i8::MAX {
            for op2 in (i8::MIN + op1 + 1)..=-1 {
                let accumulator = Reg8::new_i8(op1);
                let operand = Reg8::new_i8(op2);

                perform_operands_sub_overflowflagcorrect(accumulator, operand, false);
            }
        }

        for op1 in i8::MIN..=-1 {
            for op2 in 0..=(i8::MAX + op1 + 1) {
                let accumulator = Reg8::new_i8(op1);
                let operand = Reg8::new_i8(op2);

                perform_operands_sub_overflowflagcorrect(accumulator, operand, false);
            }
        }
    }

    #[test]
    fn operand_inc_resultscorrect() {
        for op1 in 0..u8::MAX {
            // GIVEN
            let mut accumulator = Reg8::new(op1);
            let mut status_register = StatusReg::default();

            // random garbage to check the value only changes in n and z
            status_register.set_u8(op1);
            let old_status_register = status_register;

            //WHEN
            super::inc(&mut accumulator, &mut status_register);

            // THEN
            assert_eq!(accumulator.get_u8(), op1.wrapping_add(1));
            assert_eq!(
                status_register.get_u8() & 0x7D,
                old_status_register.get_u8() & 0x7D
            );
            assert_eq!(
                status_register.are_all_flags_set(StatusRegFlags::ZERO),
                accumulator.get_u8() == 0
            );
            assert_eq!(
                status_register.are_all_flags_set(StatusRegFlags::NEGATIVE),
                accumulator.get_i8().is_negative() == true
            );
        }
    }

    #[test]
    fn operand_dec_resultscorrect() {
        for op1 in 0..u8::MAX {
            // GIVEN
            let mut accumulator = Reg8::new(op1);
            let mut status_register = StatusReg::default();

            // random garbage to check the value only changes in n and z
            status_register.set_u8(op1);
            let old_status_register = status_register;

            //WHEN
            super::dec(&mut accumulator, &mut status_register);

            // THEN
            assert_eq!(accumulator.get_u8(), op1.wrapping_sub(1));
            assert_eq!(
                status_register.get_u8() & 0x7D,
                old_status_register.get_u8() & 0x7D
            );
            assert_eq!(
                status_register.are_all_flags_set(StatusRegFlags::ZERO),
                accumulator.get_u8() == 0
            );
            assert_eq!(
                status_register.are_all_flags_set(StatusRegFlags::NEGATIVE),
                accumulator.get_i8().is_negative() == true
            );
        }
    }

    #[test]
    fn operands_and_resultscorrect() {
        for op1 in 0..u8::MAX {
            for op2 in 0..u8::MAX {
                // GIVEN
                let mut status_register = StatusReg::default();

                let mut accumulator = Reg8::new(op1);

                let operand = Reg8::new(op2);

                // WHEN
                super::and(&mut accumulator, &operand, &mut status_register);

                // THEN
                assert_eq!(accumulator.get_u8(), op1 & op2);
                assert_eq!(
                    status_register.are_all_flags_set(StatusRegFlags::ZERO),
                    accumulator.get_u8() == 0
                );
                assert_eq!(
                    status_register.are_all_flags_set(StatusRegFlags::NEGATIVE),
                    accumulator.get_i8().is_negative() == true
                );
            }
        }
    }

    #[test]
    fn operands_or_resultscorrect() {
        for op1 in 0..u8::MAX {
            for op2 in 0..u8::MAX {
                // GIVEN
                let mut status_register = StatusReg::default();

                let mut accumulator = Reg8::new(op1);

                let operand = Reg8::new(op2);

                // WHEN
                super::or(&mut accumulator, &operand, &mut status_register);

                // THEN
                assert_eq!(accumulator.get_u8(), op1 | op2);
                assert_eq!(
                    status_register.are_all_flags_set(StatusRegFlags::ZERO),
                    accumulator.get_u8() == 0
                );
                assert_eq!(
                    status_register.are_all_flags_set(StatusRegFlags::NEGATIVE),
                    accumulator.get_i8().is_negative() == true
                );
            }
        }
    }

    #[test]
    fn operands_xor_resultscorrect() {
        for op1 in 0..u8::MAX {
            for op2 in 0..u8::MAX {
                // GIVEN
                let mut status_register = StatusReg::default();

                let mut accumulator = Reg8::new(op1);

                let operand = Reg8::new(op2);

                // WHEN
                super::xor(&mut accumulator, &operand, &mut status_register);

                // THEN
                assert_eq!(accumulator.get_u8(), op1 ^ op2);
                assert_eq!(
                    status_register.are_all_flags_set(StatusRegFlags::ZERO),
                    accumulator.get_u8() == 0
                );
                assert_eq!(
                    status_register.are_all_flags_set(StatusRegFlags::NEGATIVE),
                    accumulator.get_i8().is_negative() == true
                );
            }
        }
    }
}
