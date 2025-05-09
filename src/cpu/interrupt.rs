#[derive(Clone, Copy, Default, Debug)]
pub struct Interrupts {
    irq_input: bool,
    nmi_input: bool,
    new_nmi: bool,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum InterruptType {
    NonMaskableInterrupt,
    Interrupt,
}

pub const NMINTERRUPT_ADDR_LOW: u16 = 0xFFFA;
pub const NMINTERRUPT_ADDR_HIGH: u16 = 0xFFFB;

pub const PROGRAM_START_ADDR_LOW: u16 = 0xFFFC;
pub const PROGRAM_START_ADDR_HIGH: u16 = 0xFFFD;

pub const INTERRUPT_ADDR_LOW: u16 = 0xFFFE;
pub const INTERRUPT_ADDR_HIGH: u16 = 0xFFFF;

#[derive(Clone, Copy)]
pub(crate) enum InterruptVector {
    NonMaskableInterrupt,
    Interrupt,
    ProgramStart,
}

#[derive(Clone, Copy)]
pub(crate) enum InterruptVectorAddrBytePos {
    Low,
    High,
}

impl InterruptVector {
    pub fn addr_low_byte(&self) -> u16 {
        match self {
            InterruptVector::NonMaskableInterrupt => NMINTERRUPT_ADDR_LOW,
            InterruptVector::Interrupt => INTERRUPT_ADDR_LOW,
            InterruptVector::ProgramStart => PROGRAM_START_ADDR_LOW,
        }
    }

    pub fn addr_high_byte(&self) -> u16 {
        match self {
            InterruptVector::NonMaskableInterrupt => NMINTERRUPT_ADDR_HIGH,
            InterruptVector::Interrupt => INTERRUPT_ADDR_HIGH,
            InterruptVector::ProgramStart => PROGRAM_START_ADDR_HIGH,
        }
    }

    pub fn addr(&self, byte_pos: InterruptVectorAddrBytePos) -> u16 {
        match byte_pos {
            InterruptVectorAddrBytePos::Low => self.addr_low_byte(),
            InterruptVectorAddrBytePos::High => self.addr_high_byte(),
        }
    }
}

impl Interrupts {
    pub fn reset(&mut self) {}

    pub fn set_irq_input(&mut self) {
        self.irq_input = true;
    }

    pub fn set_irq_input_value(&mut self, value: bool) {
        self.irq_input = value;
    }

    pub fn clear_irq_input(&mut self) {
        self.irq_input = false;
    }

    pub fn set_nmi_input(&mut self) {
        if !self.nmi_input {
            self.nmi_input = true;
            self.new_nmi = true;
        }
    }

    pub fn set_nmi_input_value(&mut self, value: bool) {
        if value && !self.nmi_input {
            self.new_nmi = true;
        }
        self.nmi_input = value;
    }

    pub fn clear_nmi_input(&mut self) {
        self.nmi_input = false;
    }

    pub fn is_irq_set(&self) -> bool {
        self.irq_input
    }

    pub fn waiting_nmi(&mut self) -> bool {
        let ret = self.new_nmi;
        self.new_nmi = false;
        ret
    }
}

pub fn waiting_interrupt(
    prev_waiting_interrupt: Option<InterruptType>,
    pins: &mut Interrupts,
    irq_disabled: bool,
) -> Option<InterruptType> {
    if pins.waiting_nmi() || (prev_waiting_interrupt == Some(InterruptType::NonMaskableInterrupt)) {
        Some(InterruptType::NonMaskableInterrupt)
    } else if pins.is_irq_set() && !irq_disabled {
        Some(InterruptType::Interrupt)
    } else {
        None
    }
}
