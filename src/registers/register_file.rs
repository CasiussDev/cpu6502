use super::StatusReg;
use super::{Reg16, Reg8};
use crate::registers::StatusRegFlags;
use std::fmt;
use std::fmt::Formatter;

#[cfg(feature = "random")]
use rand::distributions::{Distribution, Uniform};
#[cfg(feature = "random")]
use rand::rngs::ThreadRng;

#[allow(dead_code)]
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum SelectedRegister8 {
    A = 0xF0,
    X,
    Y,
    SP,
    Status,
    IR,
    Tmp,
    PCHigh,
    PCLow,
    AddrHigh,
    AddrLow,

    // "virtual" registers
    StackPage = 0x01,
    Discard,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum SelectedRegister16 {
    PC,
    Addr,

    // "virtual" registers
    NMInterruptAddrLow = 0xFFFA,
    NMInterruptAddHigh = 0xFFFB,
    ProgramStartAddrLow = 0xFFFC,
    ProgramStartAddrHigh = 0xFFFD,
    InterruptAddrLow = 0xFFFE,
    InterruptAddrHigh = 0xFFFF,
    #[allow(dead_code)]
    Discard,
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub struct RegisterFile {
    pub a: Reg8,
    pub x: Reg8,
    pub y: Reg8,
    pub sp: Reg8,
    pub pc: Reg16,
    pub status: StatusReg,

    // implementation non-visible registers
    pub ir: Reg8,
    pub tmp: Reg8,
    pub addr: Reg16,
}

#[allow(dead_code)]
impl RegisterFile {
    // TO DO: NES specific?
    pub fn powerup(&mut self) {
        self.status.set_u8(StatusRegFlags::STARTUP.bits());
        self.a.reset();
        self.x.reset();
        self.y.reset();
        self.sp.set_u8(0xFD); // Stack init at 0xFF, then decremented by 3
    }

    pub fn reset(&mut self) {
        self.status.set_flags(StatusRegFlags::IRQ_DISABLE);

        self.sp.dec();
        self.sp.dec();
        self.sp.dec();
    }

    #[cfg(feature = "random")]
    pub fn reset_random(&mut self, rng: &mut ThreadRng, uniform: &Uniform<u16>) {
        self.a.set_u8(uniform.sample(rng) as u8);
        self.x.set_u8(uniform.sample(rng) as u8);
        self.y.set_u8(uniform.sample(rng) as u8);
        self.sp.set_u8(uniform.sample(rng) as u8);
        self.pc.set_u16(uniform.sample(rng));
        self.status.set_u8(uniform.sample(rng) as u8);
        self.status.set_flags(StatusRegFlags::IRQ_DISABLE);
    }

    pub fn get_copy_selected_register16(&self, selection: SelectedRegister16) -> Reg16 {
        match selection {
            SelectedRegister16::Addr => self.addr,
            SelectedRegister16::PC => self.pc,
            //virtual registers
            v => Reg16 { value: v as u16 },
        }
    }

    pub fn get_selected_register16(&mut self, selection: SelectedRegister16) -> &mut Reg16 {
        match selection {
            SelectedRegister16::Addr => &mut self.addr,
            SelectedRegister16::PC => &mut self.pc,
            //virtual registers
            _ => panic!("trying to get a mutable ref of a virtual register"),
        }
    }

    pub fn get_selected_register8(&mut self, selection: SelectedRegister8) -> &mut Reg8 {
        match selection {
            SelectedRegister8::A => &mut self.a,
            SelectedRegister8::X => &mut self.x,
            SelectedRegister8::Y => &mut self.y,
            SelectedRegister8::SP => &mut self.sp,
            SelectedRegister8::Status => todo!(),
            SelectedRegister8::IR => &mut self.ir,
            SelectedRegister8::Tmp => &mut self.tmp,
            _ => todo!(),
        }
    }

    pub fn get_copy_selected_register8(&self, selection: SelectedRegister8) -> Reg8 {
        match selection {
            SelectedRegister8::A => self.a,
            SelectedRegister8::X => self.x,
            SelectedRegister8::Y => self.y,
            SelectedRegister8::SP => self.sp,
            SelectedRegister8::Status => Reg8 {
                value: self.status.get_u8(),
            },
            SelectedRegister8::IR => self.ir,
            SelectedRegister8::Tmp => self.tmp,
            SelectedRegister8::PCHigh => Reg8 {
                value: self.pc.get_high_u8(),
            },
            SelectedRegister8::PCLow => Reg8 {
                value: self.pc.get_low_u8(),
            },
            SelectedRegister8::AddrHigh => Reg8 {
                value: self.addr.get_high_u8(),
            },
            SelectedRegister8::AddrLow => Reg8 {
                value: self.addr.get_low_u8(),
            },
            SelectedRegister8::StackPage => Reg8 {
                value: SelectedRegister8::StackPage as u8,
            },
            SelectedRegister8::Discard => Reg8::default(),
        }
    }

    pub fn get_copy_status_register(&self) -> StatusReg {
        self.status
    }

    pub fn set_selected_register16(&mut self, selection: SelectedRegister16, reg: Reg16) {
        assert_ne!(
            selection,
            SelectedRegister16::NMInterruptAddHigh,
            "Attempting to write a read only register"
        );
        assert_ne!(
            selection,
            SelectedRegister16::NMInterruptAddrLow,
            "Attempting to write a read only register"
        );
        assert_ne!(
            selection,
            SelectedRegister16::InterruptAddrHigh,
            "Attempting to write a read only register"
        );
        assert_ne!(
            selection,
            SelectedRegister16::InterruptAddrLow,
            "Attempting to write a read only register"
        );
        assert_ne!(
            selection,
            SelectedRegister16::ProgramStartAddrHigh,
            "Attempting to write a read only register"
        );
        assert_ne!(
            selection,
            SelectedRegister16::ProgramStartAddrLow,
            "Attempting to write a read only register"
        );

        match selection {
            SelectedRegister16::Addr => self.addr = reg,
            SelectedRegister16::PC => self.pc = reg,
            _ => (),
        }
    }

    pub fn set_selected_register8(&mut self, selection: SelectedRegister8, reg: Reg8) {
        assert_ne!(
            selection,
            SelectedRegister8::StackPage,
            "Attempting to write a read only register"
        );

        match selection {
            SelectedRegister8::A => self.a = reg,
            SelectedRegister8::X => self.x = reg,
            SelectedRegister8::Y => self.y = reg,
            SelectedRegister8::SP => self.sp = reg,
            SelectedRegister8::Status => self.status.set_u8(reg.value),
            SelectedRegister8::IR => self.ir = reg,
            SelectedRegister8::Tmp => self.tmp = reg,
            SelectedRegister8::PCHigh => self.pc.set_high_u8(reg.value),
            SelectedRegister8::PCLow => self.pc.set_low_u8(reg.value),
            SelectedRegister8::AddrHigh => self.addr.set_high_u8(reg.value),
            SelectedRegister8::AddrLow => self.addr.set_low_u8(reg.value),
            SelectedRegister8::StackPage => (),
            SelectedRegister8::Discard => (),
        };
    }
}

impl fmt::Debug for RegisterFile {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "\nRegister File\n")?;
        write!(f, "\tA: {:?}", self.a)?;
        write!(f, "\tX: {:?}", self.x)?;
        write!(f, "\tY: {:?}", self.y)?;
        writeln!(f)?;
        write!(f, "\tSP: {:?}", self.sp)?;
        write!(f, "\tPC: {:?}", self.pc)?;
        writeln!(f)?;
        write!(f, "\tStatus: {:?}", self.status)?;
        writeln!(f)
    }
}

#[cfg(test)]
mod tests {
    use crate::registers::{Reg8, RegisterFile, SelectedRegister8};

    #[cfg(feature = "random")]
    use crate::registers::StatusRegFlags;
    #[cfg(feature = "random")]
    use rand::distributions::Uniform;

    //#[test]
    //fn registerfiles_reset_resultscorrect() {
    //    // GIVEN
    //    let mut zero_file = RegisterFile::default();
    //
    //    // WHEN
    //    zero_file.reset();
    //
    //    // THEN
    //    let mut status = StatusReg::new_from_u8(0);
    //    status.set_flags(StatusRegFlags::IRQ_DISABLE);
    //    assert_eq!(
    //        zero_file,
    //        RegisterFile {
    //            a: Reg8 { value: 0 },
    //            x: Reg8 { value: 0 },
    //            y: Reg8 { value: 0 },
    //            sp: Reg8 { value: 0 },
    //            pc: Reg16 { value: 0 },
    //            status,
    //        }
    //    )
    //}

    #[test]
    #[cfg(feature = "random")]
    fn registerfile_resetrandom_irqdisabled() {
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
                .are_all_flags_set(StatusRegFlags::IRQ_DISABLE));
        }
    }

    #[test]
    fn registerfile_sethighlowbytes_pccontainscorrectaddress() {
        // GIVEN
        let mut register_file = RegisterFile::default();

        // WHEN
        register_file.set_selected_register8(SelectedRegister8::PCHigh, Reg8 { value: 0xCA });
        register_file.set_selected_register8(SelectedRegister8::PCLow, Reg8 { value: 0xFE });

        register_file.set_selected_register8(SelectedRegister8::AddrHigh, Reg8 { value: 0xFA });
        register_file.set_selected_register8(SelectedRegister8::AddrLow, Reg8 { value: 0xCE });

        // THEN
        assert_eq!(register_file.pc.get_u16(), 0xCAFE);
        assert_eq!(register_file.addr.get_u16(), 0xFACE);
    }
}
