pub mod register_file;
pub mod status_register;

pub use register_file::*;
pub use status_register::*;

use std::fmt;
use std::fmt::Formatter;

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub struct Reg8 {
    value: u8,
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub struct Reg16 {
    value: u16,
}

impl Reg8 {
    pub fn reset(&mut self) {
        self.value = 0;
    }

    pub fn get_u8(&self) -> u8 {
        self.value as u8
    }

    pub fn set_u8(&mut self, value: u8) {
        self.value = value;
    }

    pub fn get_i8(&self) -> i8 {
        self.value as i8
    }

    pub fn set_i8(&mut self, value: i8) {
        self.value = value as u8;
    }
}

impl Reg16 {
    pub fn reset(&mut self) {
        self.value = 0;
    }

    pub fn get_u16(&self) -> u16 {
        self.value as u16
    }

    pub fn set_u16(&mut self, value: u16) {
        self.value = value;
    }

    pub fn get_i16(&self) -> i16 {
        self.value as i16
    }

    pub fn set_i16(&mut self, value: i16) {
        self.value = value as u16;
    }

    pub fn get_low_u8(&self) -> u8 {
        self.value as u8
    }

    pub fn set_low_u8(&mut self, value: u8) {
        self.value &= 0xFF00;
        self.value |= value as u16;
    }

    pub fn get_low_i8(&self) -> i8 {
        self.value as i8
    }

    pub fn set_low_i8(&mut self, value: i8) {
        self.value &= 0xFF00;
        self.value |= value as u16;
    }

    pub fn get_high_u8(&self) -> u8 {
        self.value.to_be_bytes()[0] as u8
    }

    pub fn set_high_u8(&mut self, value: u8) {
        self.value &= 0x00FF;
        self.value |= (value as u16) << 8;
    }

    pub fn get_high_i8(&self) -> i8 {
        self.value.to_be_bytes()[0] as i8
    }

    pub fn set_high_i8(&mut self, value: i8) {
        self.value &= 0x00FF;
        self.value |= (value as u16) << 8;
    }
}

impl fmt::Debug for Reg8 {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:#04X}", self.value)
    }
}

impl fmt::Debug for Reg16 {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:#06X}", self.value)
    }
}