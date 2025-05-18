//! Arithmetic Logic Unit (ALU) implementation for the 6502 CPU
//!
//! This module contains the implementation of all arithmetic and logical operations
//! performed by the 6502 processor. The ALU is responsible for:
//!
//! - Arithmetic operations (addition, subtraction, increment, decrement)
//! - Logical operations (AND, OR, XOR, bit tests)
//! - Shift and rotate operations
//! - Flag management based on operation results
//!
//! Each function in this module corresponds to a specific operation or instruction
//! in the 6502 instruction set, with appropriate handling of processor status flags
//! such as carry, overflow, zero, and negative.
//!
//! Most operations take registers as input and modify both the registers and status
//! flags according to the 6502 specification. The implementation supports both
//! binary and decimal mode arithmetic, though decimal mode is only enabled when
//! the "decimal" feature is activated (not yet implemented).

#[cfg(test)]
mod tests;

use crate::registers::{Reg8, StatusReg, StatusRegFlags};

/// Updates the Negative and Zero flags based on operation result
///
/// This function is called after most operations to set the N and Z flags
/// correctly based on the result value.
///
/// # Arguments
///
/// * `result` - The signed 8-bit result value to check
/// * `status_register` - The processor status register to update
///
/// # Effects
///
/// * Sets the NEGATIVE flag if `result` is negative (bit 7 is set)
/// * Sets the ZERO flag if `result` is zero
pub fn update_status_nz(result: i8, status_register: &mut StatusReg) {
    if result < 0 {
        status_register.set_flags(StatusRegFlags::NEGATIVE);
        status_register.clear_flags(StatusRegFlags::ZERO);
    } else {
        status_register.clear_flags(StatusRegFlags::NEGATIVE);
        status_register.update_flags(StatusRegFlags::ZERO, result == 0);
    }
}

/// Updates the Carry flag after an addition operation
///
/// # Arguments
///
/// * `carry` - Whether the operation produced a carry out
/// * `status_register` - The processor status register to update
///
/// # Effects
///
/// * Sets the CARRY flag if `carry` is true
pub fn update_status_carry_add(carry: bool, status_register: &mut StatusReg) {
    status_register.update_flags(StatusRegFlags::CARRY, carry);
}

/// Updates the Carry flag after a subtraction operation
///
/// For subtraction operations, the 6502 inverts the carry flag logic:
/// the CARRY flag is set when NO borrow is needed and cleared when a borrow IS needed.
///
/// # Arguments
///
/// * `carry` - Whether the subtraction produced a borrow
/// * `status_register` - The processor status register to update
///
/// # Effects
///
/// * Sets the CARRY flag if NO borrow occurred (`carry` is false)
/// * Clears the CARRY flag if a borrow occurred (`carry` is true)
pub fn update_status_carry_sub(carry: bool, status_register: &mut StatusReg) {
    status_register.update_flags(StatusRegFlags::CARRY, !carry);
}

/// Updates the Overflow flag after an arithmetic operation
///
/// The Overflow flag indicates whether an arithmetic operation resulted
/// in a value too large or too small to fit in a signed 8-bit value.
///
/// # Arguments
///
/// * `overflow` - Whether the operation produced an overflow
/// * `status_register` - The processor status register to update
///
/// # Effects
///
/// * Sets the OVERFLOW flag if `overflow` is true
pub fn update_status_v(overflow: bool, status_register: &mut StatusReg) {
    status_register.update_flags(StatusRegFlags::OVERFLOW, overflow);
}

/// Performs addition with carry (ADC instruction)
///
/// Adds the operand and the carry flag to the accumulator, updating all
/// relevant status flags. This implements the ADC instruction in the 6502.
///
/// # Arguments
///
/// * `accumulator` - The A register, modified in-place
/// * `operand` - The value to add to the accumulator
/// * `status_register` - The processor status register, updated based on result
///
/// # Effects
///
/// * Modifies the accumulator with the addition result
/// * Updates the N, Z, C, and V flags based on the result
/// * In decimal mode (when the "decimal" feature is enabled), performs BCD addition (not yet implemented)
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

/// Compares a register with a memory value (CMP, CPX, CPY instructions)
///
/// Performs a subtraction without modifying the register, only updating
/// the status flags. This is used for the CMP, CPX, and CPY instructions.
///
/// # Arguments
///
/// * `accumulator` - The register to compare (A, X, or Y)
/// * `operand` - The memory value to compare against
/// * `status_register` - The processor status register to update
///
/// # Effects
///
/// * Updates the N, Z, and C flags based on the comparison:
///   - Z flag is set if the values are equal
///   - C flag is set if register >= memory value
///   - N flag is set based on the result's sign bit
pub fn cmp(accumulator: &Reg8, operand: &Reg8, status_register: &mut StatusReg) {
    let (result, carry) = accumulator.to_u8().overflowing_sub(operand.to_u8());

    update_status_carry_sub(carry, status_register);
    update_status_nz(result as i8, status_register);
}

/// Performs subtraction with borrow (SBC instruction)
///
/// Subtracts the operand and the NOT of the carry flag from the accumulator,
/// updating all relevant status flags. This implements the SBC instruction.
///
/// # Arguments
///
/// * `accumulator` - The A register, modified in-place
/// * `operand` - The value to subtract from the accumulator
/// * `status_register` - The processor status register, updated based on result
///
/// # Effects
///
/// * Modifies the accumulator with the subtraction result
/// * Updates the N, Z, C, and V flags based on the result
/// * In decimal mode (when the "decimal" feature is enabled), performs BCD subtraction (not yet implemented)
pub fn sub(accumulator: &mut Reg8, operand: &Reg8, status_register: &mut StatusReg) {
    if cfg!(feature = "decimal") && status_register.are_all_flags_set(StatusRegFlags::DECIMAL) {
        todo!();
    } else {
        let (mut result, mut overflow) = accumulator.to_i8().overflowing_sub(operand.to_i8());
        let (_, mut carry) = accumulator.to_u8().overflowing_sub(operand.to_u8());

        if !status_register.are_all_flags_set(StatusRegFlags::CARRY) {
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

/// Increments a register or memory location (INC, INX, INY instructions)
///
/// Adds one to the specified operand and updates status flags accordingly.
///
/// # Arguments
///
/// * `src_dst` - The register to increment
/// * `status_register` - The processor status register to update
///
/// # Effects
///
/// * Increments the value in `src_dst`
/// * Updates the N and Z flags based on the result
/// * In decimal mode (when the "decimal" feature is enabled), performs BCD increment (not yet implemented)
pub fn inc(src_dst: &mut Reg8, status_register: &mut StatusReg) {
    if cfg!(feature = "decimal") && status_register.are_all_flags_set(StatusRegFlags::DECIMAL) {
        todo!();
    } else {
        let result = src_dst.to_u8().wrapping_add(1);

        update_status_nz(result as i8, status_register);

        src_dst.set_u8(result);
    }
}

/// Decrements a register or memory location (DEC, DEX, DEY instructions)
///
/// Subtracts one from the specified operand and updates status flags accordingly.
///
/// # Arguments
///
/// * `src_dst` - The register or memory location to decrement
/// * `status_register` - The processor status register to update
///
/// # Effects
///
/// * Decrements the value in `src_dst`
/// * Updates the N and Z flags based on the result
/// * In decimal mode (when the "decimal" feature is enabled), performs BCD decrement (not yet implemented)
pub fn dec(src_dst: &mut Reg8, status_register: &mut StatusReg) {
    if cfg!(feature = "decimal") && status_register.are_all_flags_set(StatusRegFlags::DECIMAL) {
        todo!();
    } else {
        let result = src_dst.to_i8().wrapping_sub(1);

        update_status_nz(result, status_register);

        src_dst.set_i8(result);
    }
}

/// Performs a logical shift left (ASL instruction)
///
/// Shifts all bits in the operand one position to the left, setting bit 0 to 0
/// and placing the original bit 7 in the carry flag.
///
/// # Arguments
///
/// * `src_dst` - The register or memory location to shift
/// * `status_register` - The processor status register to update
///
/// # Effects
///
/// * Shifts the value in `src_dst` left by one bit
/// * Updates the N and Z flags based on the result
/// * Sets the C flag to the value of the original bit 7
pub fn shift_left(src_dst: &mut Reg8, status_register: &mut StatusReg) {
    let msb = src_dst.to_u8() & 0x80;
    src_dst.shift_left();

    update_status_nz(src_dst.to_i8(), status_register);
    status_register.update_flags(StatusRegFlags::CARRY, msb != 0);
}

/// Performs a logical shift right (LSR instruction)
///
/// Shifts all bits in the operand one position to the right, setting bit 7 to 0
/// and placing the original bit 0 in the carry flag.
///
/// # Arguments
///
/// * `src_dst` - The register or memory location to shift
/// * `status_register` - The processor status register to update
///
/// # Effects
///
/// * Shifts the value in `src_dst` right by one bit
/// * Updates the N and Z flags based on the result (N will always be clear)
/// * Sets the C flag to the value of the original bit 0
pub fn shift_right(src_dst: &mut Reg8, status_register: &mut StatusReg) {
    let lsb = src_dst.to_u8() & 0x01;
    src_dst.shift_right();

    update_status_nz(src_dst.to_i8(), status_register);
    status_register.update_flags(StatusRegFlags::CARRY, lsb != 0);
}

/// Performs a rotate left through carry (ROL instruction)
///
/// Shifts all bits in the operand one position to the left, moving the original
/// bit 7 into the carry flag and the original carry flag into bit 0.
///
/// # Arguments
///
/// * `src_dst` - The register or memory location to rotate
/// * `status_register` - The processor status register to update
///
/// # Effects
///
/// * Rotates the value in `src_dst` left by one bit through the carry flag
/// * Updates the N and Z flags based on the result
/// * Sets the C flag to the value of the original bit 7
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

/// Performs a rotate right through carry (ROR instruction)
///
/// Shifts all bits in the operand one position to the right, moving the original
/// bit 0 into the carry flag and the original carry flag into bit 7.
///
/// # Arguments
///
/// * `src_dst` - The register or memory location to rotate
/// * `status_register` - The processor status register to update
///
/// # Effects
///
/// * Rotates the value in `src_dst` right by one bit through the carry flag
/// * Updates the N and Z flags based on the result
/// * Sets the C flag to the value of the original bit 0
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

/// Performs a logical AND (AND instruction)
///
/// Performs a bitwise AND between the accumulator and the operand,
/// storing the result in the accumulator.
///
/// # Arguments
///
/// * `accumulator` - The A register, modified in-place
/// * `operand` - The value to AND with the accumulator
/// * `status_register` - The processor status register to update
///
/// # Effects
///
/// * Modifies the accumulator with the result of A & operand
/// * Updates the N and Z flags based on the result
pub fn and(accumulator: &mut Reg8, operand: &Reg8, status_register: &mut StatusReg) {
    let result = accumulator.to_u8() & operand.to_u8();

    update_status_nz(result as i8, status_register);

    accumulator.set_u8(result);
}

/// Performs a logical OR (ORA instruction)
///
/// Performs a bitwise OR between the accumulator and the operand,
/// storing the result in the accumulator.
///
/// # Arguments
///
/// * `accumulator` - The A register, modified in-place
/// * `operand` - The value to OR with the accumulator
/// * `status_register` - The processor status register to update
///
/// # Effects
///
/// * Modifies the accumulator with the result of A | operand
/// * Updates the N and Z flags based on the result
pub fn or(accumulator: &mut Reg8, operand: &Reg8, status_register: &mut StatusReg) {
    let result = accumulator.to_u8() | operand.to_u8();

    update_status_nz(result as i8, status_register);

    accumulator.set_u8(result);
}

/// Performs a logical exclusive OR (EOR instruction)
///
/// Performs a bitwise XOR between the accumulator and the operand,
/// storing the result in the accumulator.
///
/// # Arguments
///
/// * `accumulator` - The A register, modified in-place
/// * `operand` - The value to XOR with the accumulator
/// * `status_register` - The processor status register to update
///
/// # Effects
///
/// * Modifies the accumulator with the result of A ^ operand
/// * Updates the N and Z flags based on the result
pub fn xor(accumulator: &mut Reg8, operand: &Reg8, status_register: &mut StatusReg) {
    let result = accumulator.to_u8() ^ operand.to_u8();

    update_status_nz(result as i8, status_register);

    accumulator.set_u8(result);
}

/// Tests bits in memory against the accumulator (BIT instruction)
///
/// Performs a bitwise AND between the accumulator and the operand, but
/// only updates status flags without storing the result.
///
/// This is different from most 6502 operations in that:
/// - Bits 7 and 6 of the operand are copied directly to the N and V flags
/// - The Z flag is based on the result of A & operand
///
/// # Arguments
///
/// * `accumulator` - The A register (not modified)
/// * `operand` - The memory value to test against the accumulator
/// * `status_register` - The processor status register to update
///
/// # Effects
///
/// * Sets the Z flag if the result of A & operand is zero
/// * Copies bit 7 of the operand to the N flag
/// * Copies bit 6 of the operand to the V flag
pub fn bit_compare(accumulator: Reg8, operand: Reg8, status_register: &mut StatusReg) {
    let and = accumulator.to_u8() & operand.to_u8();
    status_register.update_flags(StatusRegFlags::ZERO, and == 0);
}
