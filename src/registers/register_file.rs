use core::fmt;
use std::fmt::Formatter;
use crate::registers::StatusRegFlags;
use super::{Reg8, Reg16};
use super::StatusReg;
use rand::distributions::Distribution;
use rand::distributions::Uniform;
use rand::rngs::ThreadRng;

#[derive(Clone, Copy, Default)]
pub struct RegisterFile {
    pub a: Reg8,
    pub x: Reg8,
    pub y: Reg8,
    pub sp: Reg8,
    pub pc: Reg16,
    pub status: StatusReg
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
        write!(f, "\t\tX: {:?}", self.x)?;
        write!(f, "\t\tY: {:?}", self.y)?;
        writeln!(f, "")?;
        write!(f, "\tSP: {:?}", self.sp)?;
        write!(f, "\tPC: {:?}", self.pc)?;
        writeln!(f, "")?;
        write!(f, "\tSR: {:?}", self.status)?;
        writeln!(f, "")
    }
}

#[cfg(test)]
mod tests {
    use rand::distributions::Uniform;
    use crate::registers::RegisterFile;

    #[test]
    fn debug() {
        let mut rng = rand::thread_rng();
        let uniform = Uniform::new_inclusive( 0_u16, u16::MAX);

        let mut zero_file = RegisterFile::default();
        zero_file.reset();
        let mut random_file = RegisterFile::default();
        random_file.reset_random(&mut rng, &uniform);

        println!("{:?} {:?}", zero_file, random_file);
    }
}