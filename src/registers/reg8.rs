use std::fmt;
use std::fmt::Formatter;

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub struct Reg8 {
    value: u8,
}

impl Reg8 {
    pub fn reset(&mut self) {
        self.value = 0;
    }

    pub fn new(value: u8) -> Self {
        Self { value }
    }

    #[allow(dead_code)]
    pub fn new_i8(value: i8) -> Self {
        Self {
            value: (value as u8),
        }
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

    pub fn inc(&mut self) {
        self.value = self.value.wrapping_add(1);
    }

    pub fn dec(&mut self) {
        self.value = self.value.wrapping_sub(1);
    }

    pub fn shift_left(&mut self) {
        self.value <<= 1;
    }

    pub fn shift_right(&mut self) {
        self.value >>= 1;
    }
}

impl fmt::Debug for Reg8 {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:#04X} ( {}, {} )",
            self.value, self.value as u8, self.value as i8
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::registers::Reg8;
    use rand::distributions::Uniform;
    use rand::prelude::*;

    #[test]
    fn reg8_setvalue_getvaluecorrect() {
        let mut rng = thread_rng();
        let uniform = Uniform::new_inclusive(0_u8, u8::MAX);

        // GIVEN
        let mut register_u = Reg8::default();
        let mut register_i = Reg8::default();

        for _ in 0..10 {
            let value = uniform.sample(&mut rng);
            // WHEN
            register_u.set_u8(value);
            register_i.set_i8(value as i8);

            // THEN
            assert_eq!(register_u.get_u8(), value);
            assert_eq!(register_i.get_i8(), value as i8);
        }
    }
}
