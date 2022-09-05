use super::StatusReg;
use super::{Reg16, Reg8};
use crate::registers::StatusRegFlags;
use rand::distributions::Distribution;
use rand::distributions::Uniform;
use rand::rngs::ThreadRng;
use std::fmt;
use std::fmt::Formatter;

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub struct RegisterFile {
    pub a: Reg8,
    pub x: Reg8,
    pub y: Reg8,
    pub sp: Reg8,
    pub pc: Reg16,
    pub status: StatusReg,
}

impl RegisterFile {
    pub fn reset(&mut self) {
        self.a.reset();
        self.x.reset();
        self.y.reset();
        self.sp.reset();
        self.pc.reset();
        self.status.reset();
        self.status.set_flags(StatusRegFlags::IRQ_DISABLE);
    }

    pub fn reset_random(&mut self, rng: &mut ThreadRng, uniform: &Uniform<u16>) {
        self.a.set_u8(uniform.sample(rng) as u8);
        self.x.set_u8(uniform.sample(rng) as u8);
        self.y.set_u8(uniform.sample(rng) as u8);
        self.sp.set_u8(uniform.sample(rng) as u8);
        self.pc.set_u16(uniform.sample(rng));
        self.status.set_u8(uniform.sample(rng) as u8);
        self.status.set_flags(StatusRegFlags::IRQ_DISABLE);
    }
}

impl fmt::Debug for RegisterFile {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "\nRegister File\n")?;
        write!(f, "\tA: {:?}", self.a)?;
        write!(f, "\tX: {:?}", self.x)?;
        write!(f, "\tY: {:?}", self.y)?;
        writeln!(f, "")?;
        write!(f, "\tSP: {:?}", self.sp)?;
        write!(f, "\tPC: {:?}", self.pc)?;
        writeln!(f, "")?;
        write!(f, "\tStatus: {:?}", self.status)?;
        writeln!(f, "")
    }
}

#[cfg(test)]
mod tests {
    use crate::registers::{Reg16, Reg8, RegisterFile, StatusReg, StatusRegFlags};
    use rand::distributions::Uniform;

    #[test]
    fn registerfiles_reset_resultscorrect() {
        // GIVEN
        let mut zero_file = RegisterFile::default();

        // WHEN
        zero_file.reset();

        // THEN
        let mut status = StatusReg::new_from_u8(0);
        status.set_flags(StatusRegFlags::IRQ_DISABLE);
        assert_eq!(
            zero_file,
            RegisterFile {
                a: Reg8 { value: 0 },
                x: Reg8 { value: 0 },
                y: Reg8 { value: 0 },
                sp: Reg8 { value: 0 },
                pc: Reg16 { value: 0 },
                status,
            }
        )
    }

    #[test]
    fn registerfiles_resetrandom_irqdisabled() {
        let mut rng = rand::thread_rng();
        let uniform = Uniform::new_inclusive(0_u16, u16::MAX);

        // GIVEN
        let mut random_file = RegisterFile::default();

        for _ in 0..50 {
            // WHEN
            random_file.reset_random(&mut rng, &uniform);

            // THEN
            assert!(random_file
                .status
                .are_flags_set(StatusRegFlags::IRQ_DISABLE));
        }
    }
}
