#[derive(Clone, Copy, Default, Debug)]
pub struct Interrupts {
    irq_input: bool,
    nmi_input: bool,
    new_nmi: bool,
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
        if self.nmi_input == false {
            self.nmi_input = true;
            self.new_nmi = true;
        }
    }

    pub fn set_nmi_input_value(&mut self, value: bool) {
        if value && (self.nmi_input == false) {
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
