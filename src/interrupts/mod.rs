#[derive(Clone, Copy, Default, Debug)]
pub struct Interrupts {
    irq: bool,
    nmi: bool,
}

impl Interrupts {
    pub fn reset(&mut self) {}

    pub fn set_irq_input(&mut self) {
        self.irq = true;
    }

    pub fn clear_irq_input(&mut self) {
        self.irq = false;
    }

    pub fn set_nmi_input(&mut self) {
        self.nmi = true;
    }

    pub fn clear_nmi_input(&mut self) {
        self.nmi = false;
    }

    pub fn is_irq_set(&self) -> bool {
        self.irq
    }

    pub fn is_nmi_set(&self) -> bool {
        self.nmi
    }
}
