use super::StatusReg;
use super::{Reg16, Reg8};
use std::fmt;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
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

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum IndexRegister {
    X = SelectedRegister8::X as isize,
    Y = SelectedRegister8::Y as isize,
}

impl From<IndexRegister> for SelectedRegister8 {
    fn from(index_reg: IndexRegister) -> Self {
        match index_reg {
            IndexRegister::X => SelectedRegister8::X,
            IndexRegister::Y => SelectedRegister8::Y,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
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

impl RegisterFile {
    pub fn reset(&mut self) {
        self.a.reset();
        self.x.reset();
        self.y.reset();
        self.pc.reset();
        self.sp.reset();
        self.ir.reset();
        self.status.reset();

        self.addr.set_u16(0x00FF);
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

    pub fn set_selected_register16(&mut self, selection: SelectedRegister16, reg: Reg16) {
        debug_assert_ne!(
            selection,
            SelectedRegister16::NMInterruptAddHigh,
            "Attempting to write a read only register"
        );
        debug_assert_ne!(
            selection,
            SelectedRegister16::NMInterruptAddrLow,
            "Attempting to write a read only register"
        );
        debug_assert_ne!(
            selection,
            SelectedRegister16::InterruptAddrHigh,
            "Attempting to write a read only register"
        );
        debug_assert_ne!(
            selection,
            SelectedRegister16::InterruptAddrLow,
            "Attempting to write a read only register"
        );
        debug_assert_ne!(
            selection,
            SelectedRegister16::ProgramStartAddrHigh,
            "Attempting to write a read only register"
        );
        debug_assert_ne!(
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

    pub fn set_selected_register8(&mut self, selection: SelectedRegister8, value: u8) {
        debug_assert_ne!(
            selection,
            SelectedRegister8::StackPage,
            "Attempting to write a read only register"
        );

        let reg = Reg8::new(value);
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

    pub fn as_log_line(&self) -> String {
        format!(
            "\t\tA:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X}",
            self.a.get_u8(),
            self.x.get_u8(),
            self.y.get_u8(),
            self.status.get_u8(),
            self.sp.get_u8()
        )
    }
}

impl fmt::Debug for RegisterFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
    use crate::registers::{RegisterFile, SelectedRegister8};

    #[test]
    fn registerfile_sethighlowbytes_pccontainscorrectaddress() {
        // GIVEN
        let mut register_file = RegisterFile::default();

        // WHEN
        register_file.set_selected_register8(SelectedRegister8::PCHigh, 0xCA);
        register_file.set_selected_register8(SelectedRegister8::PCLow, 0xFE);

        register_file.set_selected_register8(SelectedRegister8::AddrHigh, 0xFA);
        register_file.set_selected_register8(SelectedRegister8::AddrLow, 0xCE);

        // THEN
        assert_eq!(register_file.pc.get_u16(), 0xCAFE);
        assert_eq!(register_file.addr.get_u16(), 0xFACE);
    }
}
