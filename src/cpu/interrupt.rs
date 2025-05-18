pub const NMINTERRUPT_ADDR_LOW: u16 = 0xFFFA;
pub const NMINTERRUPT_ADDR_HIGH: u16 = 0xFFFB;

pub const PROGRAM_START_ADDR_LOW: u16 = 0xFFFC;
pub const PROGRAM_START_ADDR_HIGH: u16 = 0xFFFD;

pub const INTERRUPT_ADDR_LOW: u16 = 0xFFFE;
pub const INTERRUPT_ADDR_HIGH: u16 = 0xFFFF;

/// Represents the state of hardware interrupt pins in a 6502 CPU
///
/// This structure maintains the state of both IRQ (Interrupt Request) and NMI
/// (Non-Maskable Interrupt) pins, as well as tracking when a new NMI has occurred.
/// Since NMI is edge-active, even if nmi_input is currently false we might need
/// to process an NMI interrupt.
/// In the 6502 architecture, IRQ can be masked by the interrupt disable flag,
/// while NMI will always be processed regardless of the flag state.
#[derive(Clone, Copy, Default, Debug)]
pub struct Interrupts {
    irq_input: bool,
    nmi_input: bool,
    new_nmi: bool,
}

/// Defines the types of interrupts that can occur in a 6502 CPU
///
/// - `NonMaskableInterrupt`: Cannot be disabled by the processor status register
/// - `Interrupt`: Standard interrupt that can be disabled by setting the interrupt disable flag
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum InterruptType {
    NonMaskableInterrupt,
    Interrupt,
}

/// Represents the different vector types used by the 6502 CPU for interrupt handling and initialization
///
/// The 6502 has three main vectors stored at fixed addresses in memory:
/// - `NonMaskableInterrupt`: Vector for NMI handling (0xFFFA-0xFFFB)
/// - `Interrupt`: Vector for IRQ/BRK handling (0xFFFE-0xFFFF)
/// - `ProgramStart`: Reset vector used for initialization (0xFFFC-0xFFFD)
///
/// Each vector is a 16-bit address that points to the handler routine.
#[derive(Clone, Copy)]
pub(crate) enum InterruptVector {
    NonMaskableInterrupt,
    Interrupt,
    ProgramStart,
}

/// Specifies which byte of an interrupt vector address to access
///
/// In the 6502, addresses are stored in little-endian format, meaning the low byte
/// comes first in memory. This enum is used to specify whether to access the low byte
/// or high byte of a 16-bit interrupt vector address.
#[derive(Clone, Copy)]
pub(crate) enum InterruptVectorAddrBytePos {
    /// The low byte of the vector address (first byte in memory)
    Low,
    /// The high byte of the vector address (second byte in memory)
    High,
}

impl InterruptVector {
    /// Returns the memory address where the low byte of this interrupt vector is stored
    ///
    /// Each interrupt vector requires two bytes in memory (for a 16-bit address),
    /// and this method returns the address of the first (low) byte.
    ///
    /// # Returns
    ///
    /// The memory address (0xFFFA, 0xFFFC, or 0xFFFE depending on the vector type)
    pub fn addr_low_byte(&self) -> u16 {
        match self {
            InterruptVector::NonMaskableInterrupt => NMINTERRUPT_ADDR_LOW,
            InterruptVector::Interrupt => INTERRUPT_ADDR_LOW,
            InterruptVector::ProgramStart => PROGRAM_START_ADDR_LOW,
        }
    }

    /// Returns the memory address where the high byte of this interrupt vector is stored
    ///
    /// Each interrupt vector requires two bytes in memory (for a 16-bit address),
    /// and this method returns the address of the second (high) byte.
    ///
    /// # Returns
    ///
    /// The memory address (0xFFFB, 0xFFFD, or 0xFFFF depending on the vector type)
    pub fn addr_high_byte(&self) -> u16 {
        match self {
            InterruptVector::NonMaskableInterrupt => NMINTERRUPT_ADDR_HIGH,
            InterruptVector::Interrupt => INTERRUPT_ADDR_HIGH,
            InterruptVector::ProgramStart => PROGRAM_START_ADDR_HIGH,
        }
    }

    /// Returns either the low or high byte address for this interrupt vector
    ///
    /// This is a convenience method that wraps `addr_low_byte` and `addr_high_byte`,
    /// allowing the caller to specify which byte of the vector they want to access.
    ///
    /// # Arguments
    ///
    /// * `byte_pos` - Which byte of the vector address to return (low or high)
    ///
    /// # Returns
    ///
    /// The memory address corresponding to the requested byte position for this vector
    pub fn addr(&self, byte_pos: InterruptVectorAddrBytePos) -> u16 {
        match byte_pos {
            InterruptVectorAddrBytePos::Low => self.addr_low_byte(),
            InterruptVectorAddrBytePos::High => self.addr_high_byte(),
        }
    }
}

impl Interrupts {
    /// Resets all interrupt pin states to their default inactive values
    ///
    /// This method is called during CPU reset sequence and sets all interrupt-related
    /// flags to false (inactive):
    /// - Clears the IRQ pin state
    /// - Clears the NMI pin state
    /// - Clears any pending NMI edge detection
    ///
    /// After calling this method, no interrupts will be pending or active until
    /// the corresponding pins are explicitly set.
    pub fn reset(&mut self) {
        self.irq_input = false;
        self.nmi_input = false;
        self.new_nmi = false;
    }

    /// Sets the IRQ input pin to active (true)
    ///
    /// The IRQ (Interrupt Request) pin is level-sensitive in the 6502,
    /// meaning an interrupt is requested as long as this pin is held low.
    pub fn set_irq_input(&mut self) {
        self.irq_input = true;
    }

    /// Sets the IRQ input pin to a specific value
    ///
    /// # Arguments
    ///
    /// * `value` - The new state for the IRQ pin
    pub fn set_irq_input_value(&mut self, value: bool) {
        self.irq_input = value;
    }

    /// Clears the IRQ input pin (sets it to inactive/false)
    ///
    /// This removes any pending standard interrupt request.
    pub fn clear_irq_input(&mut self) {
        self.irq_input = false;
    }

    /// Sets the NMI input pin to active, respecting edge sensitivity
    ///
    /// The NMI (Non-Maskable Interrupt) pin is edge-sensitive in the 6502,
    /// meaning it's triggered by a high-to-low transition. This method sets
    /// the new_nmi flag only if the nmi_input was previously false, simulating
    /// the edge detection behavior of the hardware.
    pub fn set_nmi_input(&mut self) {
        if !self.nmi_input {
            self.nmi_input = true;
            self.new_nmi = true;
        }
    }

    /// Sets the NMI input pin to a specific value, respecting edge sensitivity
    ///
    /// This sets the new_nmi flag only on a rising edge (false->true transition),
    /// simulating the edge detection behavior of the hardware.
    ///
    /// # Arguments
    ///
    /// * `value` - The new state for the NMI pin
    pub fn set_nmi_input_value(&mut self, value: bool) {
        if value && !self.nmi_input {
            self.new_nmi = true;
        }
        self.nmi_input = value;
    }

    /// Clears the NMI input pin (sets it to inactive/false)
    ///
    /// Note that this does not clear a pending NMI that has already been detected,
    /// as the edge has already been detected and stored in new_nmi.
    pub fn clear_nmi_input(&mut self) {
        self.nmi_input = false;
    }

    /// Checks if the IRQ pin is currently set (active)
    ///
    /// # Returns
    ///
    /// `true` if IRQ is currently active, `false` otherwise
    pub fn is_irq_set(&self) -> bool {
        self.irq_input
    }

    /// Checks and acknowledges any pending NMI interrupt
    ///
    /// This method returns the current state of the new_nmi flag (indicating
    /// if an NMI edge was detected) and then clears that flag. This simulates
    /// the CPU acknowledging and beginning to service the NMI.
    ///
    /// # Returns
    ///
    /// `true` if an NMI edge was detected and not yet processed, `false` otherwise
    pub fn waiting_nmi(&mut self) -> bool {
        let ret = self.new_nmi;
        self.new_nmi = false;
        ret
    }
}

/// Determines the type of interrupt waiting to be serviced by the CPU
///
/// This function implements the 6502's interrupt priority system:
/// 1. NMI (Non-Maskable Interrupt) has highest priority
/// 2. IRQ (Interrupt Request) has lower priority and can be disabled
///
/// The function checks for newly detected NMI edges, previously acknowledged NMIs
/// that are still being processed, and active IRQ signals (if not disabled).
///
/// # Arguments
///
/// * `prev_waiting_interrupt` - The previously determined waiting interrupt type (if any)
/// * `pins` - Current state of the hardware interrupt pins
/// * `irq_disabled` - Whether IRQ interrupts are currently disabled by the CPU's I flag
///
/// # Returns
///
/// * `Some(InterruptType::NonMaskableInterrupt)` - If an NMI is waiting to be serviced
/// * `Some(InterruptType::Interrupt)` - If an IRQ is waiting and not disabled
/// * `None` - If no interrupt is waiting or only a disabled IRQ is active
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
