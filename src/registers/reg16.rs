use std::fmt;
use std::fmt::Formatter;

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub struct Reg16 {
    value: u16,
}

impl From<u16> for Reg16 {
    fn from(value: u16) -> Self {
        Self { value }
    }
}

impl From<Reg16> for u16 {
    fn from(value: Reg16) -> Self {
        value.value
    }
}

impl From<i16> for Reg16 {
    fn from(value: i16) -> Self {
        Self {
            value: value as u16,
        }
    }
}

impl From<Reg16> for i16 {
    fn from(value: Reg16) -> Self {
        value.value as i16
    }
}

impl Reg16 {
    pub fn reset(&mut self) {
        self.value = 0;
    }

    pub fn new(value: u16) -> Self {
        Self { value }
    }

    pub fn to_u16(self) -> u16 {
        self.into()
    }

    pub fn set_u16(&mut self, value: u16) {
        self.value = value;
    }

    #[allow(dead_code)]
    pub fn to_i16(self) -> i16 {
        self.into()
    }

    #[allow(dead_code)]
    pub fn set_i16(&mut self, value: i16) {
        self.value = value as u16;
    }

    pub fn low_u8(&self) -> u8 {
        self.value as u8
    }

    pub fn set_low_u8(&mut self, value: u8) {
        self.value &= 0xFF00;
        self.value |= value as u16;
    }

    #[allow(dead_code)]
    pub fn low_i8(&self) -> i8 {
        self.value as i8
    }

    #[allow(dead_code)]
    pub fn set_low_i8(&mut self, value: i8) {
        self.value &= 0xFF00;
        self.value |= (value as u8) as u16;
    }

    pub fn high_u8(&self) -> u8 {
        self.value.to_be_bytes()[0]
    }

    pub fn set_high_u8(&mut self, value: u8) {
        self.value &= 0x00FF;
        self.value |= (value as u16) << 8;
    }

    #[allow(dead_code)]
    pub fn high_i8(&self) -> i8 {
        self.value.to_be_bytes()[0] as i8
    }

    #[allow(dead_code)]
    pub fn set_high_i8(&mut self, value: i8) {
        self.value &= 0x00FF;
        self.value |= (value as u16) << 8;
    }

    pub fn inc(&mut self) {
        self.value = self.value.wrapping_add(1);
    }

    #[allow(dead_code)]
    pub fn dec(&mut self) {
        self.value = self.value.wrapping_sub(1);
    }
}

impl fmt::Debug for Reg16 {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:#06X} ( {}, {} )",
            self.value, self.value, self.value as i16
        )
    }
}