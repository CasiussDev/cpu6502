use super::StatusReg;
use super::{Reg16, Reg8};
#[cfg(feature = "logging")]
use arrayvec::ArrayString;
use std::fmt;
#[cfg(feature = "logging")]
use std::fmt::Write;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub enum IndexRegister {
    #[default]
    X,
    Y,
}

/// Represents the complete set of registers for the 6502 CPU.
///
/// This structure holds all registers used by the 6502 processor, including:
/// - General purpose registers: A (accumulator), X and Y (index registers)
/// - Special purpose registers: SP (stack pointer), PC (program counter)
/// - Status register: contains processor status flags (carry, zero, etc.)
/// - Implementation-specific internal registers used during instruction execution
///
/// The register file serves as the core state of the CPU, maintaining all the
/// necessary data for instruction execution, memory addressing, and program flow control.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub struct RegisterFile {
    /// Accumulator register (A): primary register for arithmetic and logical operations
    pub a: Reg8,
    /// X index register: used for indexed addressing and as a counter
    pub x: Reg8,
    /// Y index register: used for indexed addressing and as a counter
    pub y: Reg8,
    /// Stack pointer (SP): points to the next available location on the stack (in page 1)
    pub sp: Reg8,
    /// Program counter (PC): holds the address of the next instruction to execute
    pub pc: Reg16,
    /// Processor status register: contains flag bits indicating CPU state
    pub status: StatusReg,

    // implementation non-visible registers
    /// Instruction register (IR): holds the opcode of the current instruction
    pub ir: Reg8,
    /// Temporary register: used internally during instruction execution
    pub tmp: Reg8,
    /// Address register: holds memory addresses during instruction execution
    pub addr: Reg16,
}

impl RegisterFile {
    /// Resets all registers to their default values.
    ///
    /// This method resets the state of the CPU registers to a clean default state,
    /// which is useful for initialization or when simulating a CPU reset.
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

    /// Retrieves the value of a specified index register as an 8-bit unsigned integer.
    ///
    /// This method allows access to the values of the X and Y index registers,
    /// which are used for indexed addressing modes and as counters in various instructions.
    ///
    /// # Arguments
    ///
    /// * `index_reg` - The index register to retrieve (X or Y)
    ///
    /// # Returns
    ///
    /// The value of the specified index register as an 8-bit unsigned integer
    pub fn index_register_u8(&self, index_reg: IndexRegister) -> u8 {
        match index_reg {
            IndexRegister::X => self.x.to_u8(),
            IndexRegister::Y => self.y.to_u8(),
        }
    }

    /// Generates a text representation of the current state of all registers.
    ///
    /// This method formats the current state of all the main CPU registers
    /// (accumulator, index registers, stack pointer, program counter, and status register)
    /// into a readable text string to facilitate logging or debugging.
    ///
    /// # Arguments
    /// * `dst` - An ArrayString buffer where the formatted output will be stored
    ///
    /// # Returns
    /// A fmt::Result indicating whether the formatting succeeded
    ///
    /// # Format
    /// The output has the format: `A:XX X:XX Y:XX P:XX SP:XX`
    ///
    /// This method is only available when the "logging" feature
    /// is enabled.
    #[cfg(feature = "logging")]
    pub fn as_log_line(&self, dst: &mut ArrayString<32>) -> fmt::Result {
        write!(
            dst,
            "\t\tA:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X}",
            self.a.to_u8(),
            self.x.to_u8(),
            self.y.to_u8(),
            self.status.to_u8(),
            self.sp.to_u8()
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
