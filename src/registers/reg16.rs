use std::fmt;
use std::fmt::Formatter;

/// Represents a 16-bit CPU register.
///
/// Provides methods for accessing and modifying the register value,
/// as well as its high and low 8-bit parts. Supports conversions
/// between `u16`, `i16`, and the register type.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub struct Reg16 {
    value: u16,
}

impl From<u16> for Reg16 {
    /// Creates a `Reg16` from a `u16` value.
    fn from(value: u16) -> Self {
        Self { value }
    }
}

impl From<Reg16> for u16 {
    /// Converts a `Reg16` into a `u16` value.
    fn from(value: Reg16) -> Self {
        value.value
    }
}

impl From<i16> for Reg16 {
    /// Creates a `Reg16` from an `i16` value (bitwise cast).
    fn from(value: i16) -> Self {
        Self {
            value: value as u16,
        }
    }
}

impl From<Reg16> for i16 {
    /// Converts a `Reg16` into an `i16` value (bitwise cast).
    fn from(value: Reg16) -> Self {
        value.value as i16
    }
}

impl Reg16 {
    /// Resets the register value to 0.
    pub fn reset(&mut self) {
        self.value = 0;
    }

    /// Creates a new `Reg16` with the specified `u16` value.
    pub fn new(value: u16) -> Self {
        Self { value }
    }

    /// Returns the current value as `u16`.
    pub fn to_u16(self) -> u16 {
        self.into()
    }

    /// Sets the register value from a `u16`.
    pub fn set_u16(&mut self, value: u16) {
        self.value = value;
    }

    /// Returns the current value as `i16`.
    #[allow(dead_code)]
    pub fn to_i16(self) -> i16 {
        self.into()
    }

    /// Sets the register value from an `i16`.
    #[allow(dead_code)]
    pub fn set_i16(&mut self, value: i16) {
        self.value = value as u16;
    }

    /// Returns the low 8 bits of the register as `u8`.
    pub fn low_u8(&self) -> u8 {
        self.value as u8
    }

    /// Sets the low 8 bits of the register from a `u8`.
    pub fn set_low_u8(&mut self, value: u8) {
        self.value &= 0xFF00;
        self.value |= value as u16;
    }

    /// Returns the low 8 bits of the register as `i8`.
    #[allow(dead_code)]
    pub fn low_i8(&self) -> i8 {
        self.value as i8
    }

    /// Sets the low 8 bits of the register from an `i8`.
    #[allow(dead_code)]
    pub fn set_low_i8(&mut self, value: i8) {
        self.value &= 0xFF00;
        self.value |= (value as u8) as u16;
    }

    /// Returns the high 8 bits of the register as `u8`.
    pub fn high_u8(&self) -> u8 {
        self.value.to_be_bytes()[0]
    }

    /// Sets the high 8 bits of the register from a `u8`.
    pub fn set_high_u8(&mut self, value: u8) {
        self.value &= 0x00FF;
        self.value |= (value as u16) << 8;
    }

    /// Returns the high 8 bits of the register as `i8`.
    #[allow(dead_code)]
    pub fn high_i8(&self) -> i8 {
        self.value.to_be_bytes()[0] as i8
    }

    /// Sets the high 8 bits of the register from an `i8`.
    #[allow(dead_code)]
    pub fn set_high_i8(&mut self, value: i8) {
        self.value &= 0x00FF;
        self.value |= (value as u16) << 8;
    }

    /// Increments the register value by 1, wrapping on overflow.
    pub fn inc(&mut self) {
        self.value = self.value.wrapping_add(1);
    }

    /// Decrements the register value by 1, wrapping on underflow.
    #[allow(dead_code)]
    pub fn dec(&mut self) {
        self.value = self.value.wrapping_sub(1);
    }
}

/// Formats the register for debugging, showing hexadecimal, unsigned, and signed values.
impl fmt::Debug for Reg16 {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:#06X} ( {}, {} )",
            self.value, self.value, self.value as i16
        )
    }
}
