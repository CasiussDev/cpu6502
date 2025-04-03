use bitflags::bitflags;
use std::fmt;

bitflags! {
    #[derive(Default, Copy, Clone, PartialEq, Eq, Debug)]
    pub struct StatusRegFlags: u8 {
        const CARRY =       0b00000001;
        const ZERO =        0b00000010;
        const IRQ_DISABLE = 0b00000100;
        const DECIMAL =     0b00001000;
        const BREAK =       0b00010000;
        const UNUSED =      0b00100000;
        const OVERFLOW =    0b01000000;
        const NEGATIVE =    0b10000000;
        const STARTUP =     Self::UNUSED.bits() | Self::BREAK.bits() | Self::IRQ_DISABLE.bits();
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Default)]
pub struct StatusReg {
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
    pub fn reset(&mut self) {
        self.flags = StatusRegFlags::from_bits_truncate(0x24);
    }

    pub fn to_u8(self) -> u8 {
        self.into()
    }

    pub fn set_u8(&mut self, value: u8) {
        // All combinations are accepted, even if some flags could be ignored
        self.flags = StatusRegFlags::from_bits_truncate(value);
    }

    pub fn set_flags(&mut self, flags_to_set: StatusRegFlags) {
        self.flags.insert(flags_to_set);
    }

    pub fn clear_flags(&mut self, flags_to_clear: StatusRegFlags) {
        self.flags.remove(flags_to_clear);
    }

    pub fn update_flags(&mut self, flags_to_update: StatusRegFlags, value: bool) {
        if value == true {
            self.set_flags(flags_to_update);
        } else {
            self.clear_flags(flags_to_update);
        }
    }

    pub fn are_all_flags_set(&self, flags_to_check: StatusRegFlags) -> bool {
        self.flags.contains(flags_to_check)
    }

    pub fn are_any_flags_set(&self, flags_to_check: StatusRegFlags) -> bool {
        self.flags.intersects(flags_to_check)
    }

    pub fn negative(&self) -> bool {
        self.flags.contains(StatusRegFlags::NEGATIVE)
    }

    pub fn carry(&self) -> bool {
        self.flags.contains(StatusRegFlags::CARRY)
    }

    pub fn zero(&self) -> bool {
        self.flags.contains(StatusRegFlags::ZERO)
    }

    pub fn overflow(&self) -> bool {
        self.flags.contains(StatusRegFlags::OVERFLOW)
    }
}

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
