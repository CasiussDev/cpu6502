use core::fmt;
use core::fmt::Formatter;

/// Represents an 8-bit register in the 6502 CPU.
///
/// This structure encapsulates an 8-bit value and provides methods to interact
/// with it both as an unsigned (u8) and signed (i8) value, matching the 6502's
/// capability to interpret register contents in both ways depending on the operation.
///
/// The register supports operations common to the 6502 instruction set such as
/// increments, decrements, and bit shifts, with appropriate wrapping behavior
/// to mimic the processor's behavior.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub struct Reg8 {
    value: u8,
}

impl From<u8> for Reg8 {
    fn from(value: u8) -> Self {
        Self { value }
    }
}

impl From<Reg8> for u8 {
    fn from(value: Reg8) -> Self {
        value.value
    }
}

impl From<i8> for Reg8 {
    fn from(value: i8) -> Self {
        Self { value: value as u8 }
    }
}

impl From<Reg8> for i8 {
    fn from(value: Reg8) -> Self {
        value.value as i8
    }
}

impl Reg8 {
    /// Resets the register value to zero.
    pub fn reset(&mut self) {
        self.value = 0;
    }

    /// Creates a new register with the specified value.
    pub fn new(value: u8) -> Self {
        Self { value }
    }

    /// Retrieves the register value as an unsigned 8-bit integer.
    pub fn to_u8(self) -> u8 {
        self.into()
    }

    /// Sets the register value from an unsigned 8-bit integer.
    pub fn set_u8(&mut self, value: u8) {
        self.value = value;
    }

    /// Retrieves the register value as a signed 8-bit integer.
    pub fn to_i8(self) -> i8 {
        self.into()
    }

    /// Sets the register value from a signed 8-bit integer.
    pub fn set_i8(&mut self, value: i8) {
        self.value = value as u8;
    }

    /// Increments the register value by one with wrapping.
    ///
    /// This method simulates the behavior of the 6502 increment instructions,
    /// where incrementing 0xFF results in 0x00 due to 8-bit register width.
    pub fn inc(&mut self) {
        self.value = self.value.wrapping_add(1);
    }

    /// Decrements the register value by one with wrapping.
    ///
    /// This method simulates the behavior of the 6502 decrement instructions,
    /// where decrementing 0x00 results in 0xFF due to 8-bit register width.
    pub fn dec(&mut self) {
        self.value = self.value.wrapping_sub(1);
    }

    /// Performs a logical shift left on the register value.
    ///
    /// Shifts all bits in the register to the left by one position.
    /// Bit 0 is set to 0, and bit 7 is discarded.
    pub fn shift_left(&mut self) {
        self.value <<= 1;
    }

    /// Performs a logical shift right on the register value.
    ///
    /// Shifts all bits in the register to the right by one position.
    /// Bit 7 is set to 0, and bit 0 is discarded.
    pub fn shift_right(&mut self) {
        self.value >>= 1;
    }
}

impl fmt::Debug for Reg8 {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:#04X} ( {}, {} )",
            self.value, self.value, self.value as i8
        )
    }
}
