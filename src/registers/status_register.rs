use bitflags::bitflags;
use std::fmt;
use std::fmt::Formatter;

bitflags! {
    #[derive(Default)]
    pub struct StatusRegFlags: u8 {
        const CARRY =       0b00000001;
        const ZERO =        0b00000010;
        const IRQ_DISABLE = 0b00000100;
        const DECIMAL =     0b00001000;
        const BREAK =       0b00010000;
        const UNUSED =      0b00100000;
        const OVERFLOW =    0b01000000;
        const NEGATIVE =    0b10000000;
        const STARTUP =     Self::UNUSED.bits | Self::BREAK.bits | Self::IRQ_DISABLE.bits;
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Default)]
pub struct StatusReg {
    flags: StatusRegFlags,
}

#[allow(dead_code)]
impl StatusReg {
    pub fn new_from_u8(flags: u8) -> Self {
        Self {
            flags: unsafe { StatusRegFlags::from_bits_unchecked(flags) },
        }
    }

    pub fn reset(&mut self) {
        self.flags.bits = 0;
    }

    pub fn get_u8(&self) -> u8 {
        self.flags.bits
    }

    pub fn set_u8(&mut self, value: u8) {
        // All combinations are accepted, even if some flags could be ignored
        self.flags = unsafe { StatusRegFlags::from_bits_unchecked(value) };
    }

    pub fn set_flags(&mut self, flags_to_set: StatusRegFlags) {
        self.flags.insert(flags_to_set);
    }

    pub fn clear_flags(&mut self, flags_to_clear: StatusRegFlags) {
        self.flags.remove(flags_to_clear);
    }

    pub fn are_all_flags_set(&self, flags_to_check: StatusRegFlags) -> bool {
        self.flags.contains(flags_to_check)
    }

    pub fn are_any_flags_set(&self, flags_to_check: StatusRegFlags) -> bool {
        self.flags.intersects(flags_to_check)
    }
}

impl fmt::Debug for StatusReg {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
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
