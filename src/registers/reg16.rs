use std::fmt;
use std::fmt::Formatter;

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub struct Reg16 {
    value: u16,
}

impl Reg16 {
    pub fn reset(&mut self) {
        self.value = 0;
    }

    #[allow(dead_code)]
    pub fn new(value: u16) -> Self {
        Self { value }
    }

    #[allow(dead_code)]
    pub fn new_i16(value: i16) -> Self {
        Self {
            value: (value as u16),
        }
    }

    pub fn get_u16(&self) -> u16 {
        self.value as u16
    }

    pub fn set_u16(&mut self, value: u16) {
        self.value = value;
    }

    #[allow(dead_code)]
    pub fn get_i16(&self) -> i16 {
        self.value as i16
    }

    #[allow(dead_code)]
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

    #[allow(dead_code)]
    pub fn get_low_i8(&self) -> i8 {
        self.value as i8
    }

    #[allow(dead_code)]
    pub fn set_low_i8(&mut self, value: i8) {
        self.value &= 0xFF00;
        self.value |= (value as u8) as u16;
    }

    pub fn get_high_u8(&self) -> u8 {
        self.value.to_be_bytes()[0] as u8
    }

    pub fn set_high_u8(&mut self, value: u8) {
        self.value &= 0x00FF;
        self.value |= (value as u16) << 8;
    }

    #[allow(dead_code)]
    pub fn get_high_i8(&self) -> i8 {
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
            self.value, self.value as u16, self.value as i16
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::registers::Reg16;
    use rand::distributions::Uniform;
    use rand::prelude::*;

    #[test]
    fn reg16_setvalue_getvaluecorrect() {
        let mut rng = thread_rng();
        let uniform = Uniform::new_inclusive(0_u16, u16::MAX);

        // GIVEN
        let mut register_u = Reg16::default();
        let mut register_i = Reg16::default();

        for _ in 0..10 {
            let value = uniform.sample(&mut rng);
            // WHEN
            register_u.set_u16(value);
            register_i.set_i16(value as i16);

            // THEN
            assert_eq!(register_u.get_u16(), value);
            assert_eq!(register_i.get_i16(), value as i16);
        }
    }

    #[test]
    fn reg16_sethighlowvalue_getvaluecorrect() {
        let mut rng = thread_rng();
        let uniform = Uniform::new_inclusive(0_u16, u16::MAX);

        // GIVEN
        let mut register_u = Reg16::default();
        let mut register_i = Reg16::default();

        for _ in 0..10 {
            let value = uniform.sample(&mut rng);
            let high = ((value & 0xFF00) >> 8) as u8;
            let low = value as u8;
            // WHEN

            register_u.set_high_u8(high);
            register_u.set_low_u8(low);
            register_i.set_high_i8(high as i8);
            register_i.set_low_i8(low as i8);

            // THEN
            assert_eq!(register_u.get_u16(), value);
            assert_eq!(register_u.get_high_u8(), high);
            assert_eq!(register_u.get_low_u8(), low);
            assert_eq!(register_i.get_i16(), value as i16);
            assert_eq!(register_u.get_high_i8(), high as i8);
            assert_eq!(register_u.get_low_i8(), low as i8);
        }
    }
}
