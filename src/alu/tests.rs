use crate::alu;
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
            alu::add(&mut accumulator, &operand, &mut status_register);

            // THEN
            assert_eq!(
                accumulator.to_u8(),
                old_accumulator_value + operand.to_u8()
            );
            assert!(!status_register.are_all_flags_set(StatusRegFlags::CARRY));
            assert_eq!(
                status_register.are_all_flags_set(StatusRegFlags::ZERO),
                accumulator.to_u8() == 0
            );
            assert_eq!(
                status_register.are_all_flags_set(StatusRegFlags::NEGATIVE),
                accumulator.to_i8() < 0
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
            status_register.set_flags(StatusRegFlags::DECIMAL);
            status_register.set_flags(StatusRegFlags::CARRY);

            let mut accumulator = Reg8::new(op1);
            let old_accumulator_value = op1;

            let operand = Reg8::new(op2);

            // WHEN
            alu::add(&mut accumulator, &operand, &mut status_register);

            // THEN
            assert_eq!(
                accumulator.to_u8(),
                old_accumulator_value + operand.to_u8() + 1
            );
            assert!(!status_register.are_all_flags_set(StatusRegFlags::CARRY));
            assert_eq!(
                status_register.are_all_flags_set(StatusRegFlags::ZERO),
                accumulator.to_u8() == 0
            );
            assert_eq!(
                status_register.are_all_flags_set(StatusRegFlags::NEGATIVE),
                accumulator.to_i8() < 0
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
            alu::sub(&mut accumulator, &operand, &mut status_register);

            // THEN
            assert_eq!(
                accumulator.to_u8(),
                old_accumulator_value - operand.to_u8()
            );
            assert!(status_register.are_all_flags_set(StatusRegFlags::CARRY));
            assert_eq!(
                status_register.are_all_flags_set(StatusRegFlags::ZERO),
                accumulator.to_u8() == 0
            );
            assert_eq!(
                status_register.are_all_flags_set(StatusRegFlags::NEGATIVE),
                accumulator.to_i8() < 0
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
            status_register.clear_flags(StatusRegFlags::CARRY); // bit reset for borrow in sub

            let mut accumulator = Reg8::new(op1);
            let old_accumulator_value = op1;

            let operand = Reg8::new(op2);

            // WHEN
            alu::sub(&mut accumulator, &operand, &mut status_register);

            // THEN
            assert_eq!(
                accumulator.to_u8(),
                old_accumulator_value - operand.to_u8() - 1
            );
            assert!(status_register.are_all_flags_set(StatusRegFlags::CARRY));
            assert_eq!(
                status_register.are_all_flags_set(StatusRegFlags::ZERO),
                accumulator.to_u8() == 0
            );
            assert_eq!(
                status_register.are_all_flags_set(StatusRegFlags::NEGATIVE),
                accumulator.to_i8() < 0
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
            alu::add(&mut accumulator, &operand, &mut status_register);

            // THEN
            assert_eq!(
                accumulator.to_u8(),
                old_accumulator_value.wrapping_add(operand.to_u8())
            );
            assert!(status_register.are_all_flags_set(StatusRegFlags::CARRY));
            assert_eq!(
                status_register.are_all_flags_set(StatusRegFlags::ZERO),
                accumulator.to_u8() == 0
            );
            assert_eq!(
                status_register.are_all_flags_set(StatusRegFlags::NEGATIVE),
                accumulator.to_i8() < 0
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
            alu::sub(&mut accumulator, &operand, &mut status_register);

            // THEN
            assert_eq!(
                accumulator.to_u8(),
                old_accumulator_value.wrapping_sub(operand.to_u8())
            );
            assert!(!status_register.are_all_flags_set(StatusRegFlags::CARRY));
            assert_eq!(
                status_register.are_all_flags_set(StatusRegFlags::ZERO),
                accumulator.to_u8() == 0
            );
            assert_eq!(
                status_register.are_all_flags_set(StatusRegFlags::NEGATIVE),
                accumulator.to_i8() < 0
            );
        }
    }
}

fn perform_operands_add_overflowflagcorrect(mut op1: Reg8, op2: Reg8, expect_overflow: bool) {
    let mut status_register = StatusReg::default();
    alu::add(&mut op1, &op2, &mut status_register);

    assert_eq!(
        status_register.are_all_flags_set(StatusRegFlags::OVERFLOW),
        expect_overflow
    );
}

#[test]
fn signedpositiveoperands_add_overflowflagcorrect() {
    for op1 in 0..=i8::MAX {
        for op2 in 0..=(i8::MAX - op1) {
            let accumulator = op1.into();
            let operand = op2.into();

            perform_operands_add_overflowflagcorrect(accumulator, operand, false);
        }
    }

    for op1 in 1..=i8::MAX {
        for op2 in (i8::MAX - op1 + 1)..=i8::MAX {
            let accumulator = op1.into();
            let operand = op2.into();

            perform_operands_add_overflowflagcorrect(accumulator, operand, true);
        }
    }
}

#[test]
fn signednegativeoperands_add_overflowflagcorrect() {
    for op1 in (i8::MIN + 1)..=-1 {
        for op2 in (i8::MIN - op1 + 1)..=-1 {
            let accumulator = op1.into();
            let operand = op2.into();

            perform_operands_add_overflowflagcorrect(accumulator, operand, false);
        }
    }

    for op1 in i8::MIN..=-1 {
        for op2 in i8::MIN..=(i8::MIN - op1 - 1) {
            let accumulator = op1.into();
            let operand = op2.into();

            perform_operands_add_overflowflagcorrect(accumulator, operand, true);
        }
    }
}

#[test]
fn differentsignoperands_add_overflowflagcorrect() {
    for op1 in 0..=i8::MAX {
        for op2 in i8::MIN..=-1 {
            let accumulator = op1.into();
            let operand = op2.into();

            perform_operands_add_overflowflagcorrect(accumulator, operand, false);
            perform_operands_add_overflowflagcorrect(operand, accumulator, false);
        }
    }
}

fn perform_operands_sub_overflowflagcorrect(mut op1: Reg8, op2: Reg8, expect_overflow: bool) {
    let mut status_register = StatusReg::default();
    status_register.set_flags(StatusRegFlags::CARRY);
    alu::sub(&mut op1, &op2, &mut status_register);

    assert_eq!(
        status_register.are_all_flags_set(StatusRegFlags::OVERFLOW),
        expect_overflow
    );
}

#[test]
fn signedpositiveoperands_sub_overflowflagcorrect() {
    for op1 in 0..=i8::MAX {
        for op2 in 0..=i8::MAX {
            let accumulator = op1.into();
            let operand = op2.into();

            perform_operands_sub_overflowflagcorrect(accumulator, operand, false);
        }
    }
}

#[test]
fn signednegativeoperands_sub_overflowflagcorrect() {
    for op1 in i8::MIN..=-1 {
        for op2 in i8::MIN..=-1 {
            let accumulator = op1.into();
            let operand = op2.into();

            perform_operands_sub_overflowflagcorrect(accumulator, operand, false);
        }
    }
}

#[test]
fn differentsignoperands_sub_overflowflagcorrect() {
    for op1 in 0..i8::MAX {
        for op2 in (i8::MIN + op1 + 1)..=-1 {
            let accumulator = op1.into();
            let operand = op2.into();

            perform_operands_sub_overflowflagcorrect(accumulator, operand, false);
        }
    }

    for op1 in i8::MIN..=-1 {
        for op2 in 0..=(i8::MAX + op1 + 1) {
            let accumulator = op1.into();
            let operand = op2.into();

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
        alu::inc(&mut accumulator, &mut status_register);

        // THEN
        assert_eq!(accumulator.to_u8(), op1.wrapping_add(1));
        assert_eq!(
            status_register.to_u8() & 0x7D,
            old_status_register.to_u8() & 0x7D
        );
        assert_eq!(
            status_register.are_all_flags_set(StatusRegFlags::ZERO),
            accumulator.to_u8() == 0
        );
        assert_eq!(
            status_register.are_all_flags_set(StatusRegFlags::NEGATIVE),
            accumulator.to_i8().is_negative()
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
        alu::dec(&mut accumulator, &mut status_register);

        // THEN
        assert_eq!(accumulator.to_u8(), op1.wrapping_sub(1));
        assert_eq!(
            status_register.to_u8() & 0x7D,
            old_status_register.to_u8() & 0x7D
        );
        assert_eq!(
            status_register.are_all_flags_set(StatusRegFlags::ZERO),
            accumulator.to_u8() == 0
        );
        assert_eq!(
            status_register.are_all_flags_set(StatusRegFlags::NEGATIVE),
            accumulator.to_i8().is_negative()
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
            alu::and(&mut accumulator, &operand, &mut status_register);

            // THEN
            assert_eq!(accumulator.to_u8(), op1 & op2);
            assert_eq!(
                status_register.are_all_flags_set(StatusRegFlags::ZERO),
                accumulator.to_u8() == 0
            );
            assert_eq!(
                status_register.are_all_flags_set(StatusRegFlags::NEGATIVE),
                accumulator.to_i8().is_negative()
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
            alu::or(&mut accumulator, &operand, &mut status_register);

            // THEN
            assert_eq!(accumulator.to_u8(), op1 | op2);
            assert_eq!(
                status_register.are_all_flags_set(StatusRegFlags::ZERO),
                accumulator.to_u8() == 0
            );
            assert_eq!(
                status_register.are_all_flags_set(StatusRegFlags::NEGATIVE),
                accumulator.to_i8().is_negative()
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
            alu::xor(&mut accumulator, &operand, &mut status_register);

            // THEN
            assert_eq!(accumulator.to_u8(), op1 ^ op2);
            assert_eq!(
                status_register.are_all_flags_set(StatusRegFlags::ZERO),
                accumulator.to_u8() == 0
            );
            assert_eq!(
                status_register.are_all_flags_set(StatusRegFlags::NEGATIVE),
                accumulator.to_i8().is_negative()
            );
        }
    }
}
