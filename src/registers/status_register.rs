use bitflags::bitflags;
use core::fmt;

bitflags! {
    /// Represents the different flags in the 6502 status register.
    #[derive(Default, Copy, Clone, PartialEq, Eq, Debug)]
    pub struct StatusRegFlags: u8 {
        /// Carry flag (bit 0) - Set if operation produced a carry or if A >= M in comparison
        const CARRY =       0b00000001;

        /// Zero flag (bit 1) - Set if the result of the last operation was zero
        const ZERO =        0b00000010;

        /// Interrupt disable flag (bit 2) - When set, the CPU will not respond to IRQs
        const IRQ_DISABLE = 0b00000100;

        /// Decimal mode flag (bit 3) - When set, the CPU uses binary-coded decimal arithmetic
        const DECIMAL =     0b00001000;

        /// Break command flag (bit 4) - Set when a BRK instruction is executed
        const BREAK =       0b00010000;

        /// Unused flag (bit 5) - Always set in the status register
        const UNUSED =      0b00100000;

        /// Overflow flag (bit 6) - Set when arithmetic operation results in invalid 2's complement result
        const OVERFLOW =    0b01000000;

        /// Negative flag (bit 7) - Set if the result of the last operation had bit 7 set (negative in 2's complement)
        const NEGATIVE =    0b10000000;

        /// Default state at startup: UNUSED, BREAK and IRQ_DISABLE are set
        const STARTUP =     Self::UNUSED.bits() | Self::BREAK.bits() | Self::IRQ_DISABLE.bits();
    }
}

/// Represents the 6502 processor status register.
///
/// The status register contains flags that indicate the state of the processor
/// after arithmetic and logical operations. These flags can be modified directly
/// by instructions like SEC (Set Carry) or indirectly by operations that affect the
/// result flags (like ADC, SBC, etc.).
///
/// The status register is automatically modified during interrupts and can be
/// pushed to and pulled from the stack with PHP and PLP instructions.
#[derive(Copy, Clone, PartialEq, Eq, Default)]
pub struct StatusReg {
    /// The status flags, packed into a single byte
    flags: StatusRegFlags,
}

impl From<u8> for StatusReg {
    fn from(value: u8) -> Self {
        Self {
            flags: StatusRegFlags::from_bits_truncate(value),
        }
    }
}

impl From<StatusReg> for u8 {
    fn from(value: StatusReg) -> Self {
        value.flags.bits()
    }
}

impl StatusReg {
    /// Resets the status register to its initial state.
    ///
    /// This sets the register to have the UNUSED bit set (0x20) and IRQ_DISABLE (0x04),
    /// which is the state of the status register after a processor reset.
    pub fn reset(&mut self) {
        self.flags = StatusRegFlags::from_bits_truncate(0x24);
    }

    /// Converts the status register to a u8 value.
    ///
    /// # Returns
    ///
    /// The status register value as an 8-bit unsigned integer
    pub fn to_u8(self) -> u8 {
        self.into()
    }

    /// Sets the status register from a raw u8 value.
    ///
    /// # Arguments
    ///
    /// * `value` - The raw byte to set as the new status register value
    pub fn set_u8(&mut self, value: u8) {
        // All combinations are accepted, even if some flags could be ignored
        self.flags = StatusRegFlags::from_bits_truncate(value);
    }

    /// Sets specific flags in the status register.
    ///
    /// # Arguments
    ///
    /// * `flags_to_set` - The flags to be set (set to 1)
    pub fn set_flags(&mut self, flags_to_set: StatusRegFlags) {
        self.flags.insert(flags_to_set);
    }

    /// Clears specific flags in the status register.
    ///
    /// # Arguments
    ///
    /// * `flags_to_clear` - The flags to be cleared (set to 0)
    pub fn clear_flags(&mut self, flags_to_clear: StatusRegFlags) {
        self.flags.remove(flags_to_clear);
    }

    /// Updates specific flags based on a boolean value.
    ///
    /// This is a convenient method to either set or clear flags based
    /// on a condition.
    ///
    /// # Arguments
    ///
    /// * `flags_to_update` - The flags to be updated
    /// * `value` - If true, the flags will be set; if false, they will be cleared
    pub fn update_flags(&mut self, flags_to_update: StatusRegFlags, value: bool) {
        if value {
            self.set_flags(flags_to_update);
        } else {
            self.clear_flags(flags_to_update);
        }
    }

    /// Checks if all specified flags are set.
    ///
    /// # Arguments
    ///
    /// * `flags_to_check` - The flags to check
    ///
    /// # Returns
    ///
    /// `true` if all specified flags are set, `false` otherwise
    pub fn are_all_flags_set(&self, flags_to_check: StatusRegFlags) -> bool {
        self.flags.contains(flags_to_check)
    }

    /// Checks if any of the specified flags are set.
    ///
    /// # Arguments
    ///
    /// * `flags_to_check` - The flags to check
    ///
    /// # Returns
    ///
    /// `true` if at least one of the specified flags is set, `false` otherwise
    pub fn are_any_flags_set(&self, flags_to_check: StatusRegFlags) -> bool {
        self.flags.intersects(flags_to_check)
    }

    /// Checks if the negative flag (N) is set.
    ///
    /// # Returns
    ///
    /// `true` if the negative flag is set, `false` otherwise
    pub fn negative(&self) -> bool {
        self.flags.contains(StatusRegFlags::NEGATIVE)
    }

    /// Checks if the carry flag (C) is set.
    ///
    /// # Returns
    ///
    /// `true` if the carry flag is set, `false` otherwise
    pub fn carry(&self) -> bool {
        self.flags.contains(StatusRegFlags::CARRY)
    }

    /// Checks if the zero flag (Z) is set.
    ///
    /// # Returns
    ///
    /// `true` if the zero flag is set, `false` otherwise
    pub fn zero(&self) -> bool {
        self.flags.contains(StatusRegFlags::ZERO)
    }

    /// Checks if the overflow flag (V) is set.
    ///
    /// # Returns
    ///
    /// `true` if the overflow flag is set, `false` otherwise
    pub fn overflow(&self) -> bool {
        self.flags.contains(StatusRegFlags::OVERFLOW)
    }

    /// Checks if the interrupt disable flag (I) is set.
    ///
    /// # Returns
    ///
    /// `true` if the interrupt disable flag is set, `false` otherwise
    pub fn irq_disable(&self) -> bool {
        self.flags.contains(StatusRegFlags::IRQ_DISABLE)
    }
}

/// Implements the Debug trait for displaying the status register in a human-readable format.
///
/// The output displays each flag with its name letter and value (1 or 0).
/// For example: 'n: 0   v: 0   -: 1   b: 0   d: 0   i: 1   z: 0   c: 0'
///
/// The flag letters are:
/// - n: Negative
/// - v: Overflow
/// - -: Unused
/// - b: Break
/// - d: Decimal
/// - i: Interrupt disable
/// - z: Zero
/// - c: Carry
impl fmt::Debug for StatusReg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let letters = ['n', 'v', '-', 'b', 'd', 'i', 'z', 'c'];
        let flags = [
            StatusRegFlags::NEGATIVE,
            StatusRegFlags::OVERFLOW,
            StatusRegFlags::UNUSED,
            StatusRegFlags::BREAK,
            StatusRegFlags::DECIMAL,
            StatusRegFlags::IRQ_DISABLE,
            StatusRegFlags::ZERO,
            StatusRegFlags::CARRY,
        ];

        let is_set = |flag| {
            if self.flags.contains(flag) {
                1_u8
            } else {
                0_u8
            }
        };

        for (flag, letter) in flags.into_iter().zip(letters) {
            write!(f, "{}: {}   ", letter, is_set(flag))?;
        }

        write!(f, "")
    }
}

#[cfg(test)]
mod tests {
    use crate::registers::status_register::{StatusReg, StatusRegFlags};

    #[test]
    fn debug() {
        let mut reg = StatusReg::default();
        reg.set_flags(StatusRegFlags::ZERO | StatusRegFlags::CARRY);
        println!("{:?}", reg);
    }
}
