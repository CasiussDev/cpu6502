#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DataDirectionMode {
    Read,
    Write,
}

#[derive(Clone, Copy, Default, Debug)]
pub struct Pinout {
    data: u8,
    address: u16,
    mode: DataDirectionMode,
    irq: bool,
    nmi: bool,
}

impl Default for DataDirectionMode {
    fn default() -> Self {
        Self::Read
    }
}

impl Pinout {
    pub fn reset(&mut self) {
        self.mode = DataDirectionMode::Read;
        self.address = 0x00FF;
    }

    pub fn set_address_output(&mut self, address: u16) {
        self.address = address;
    }

    pub fn set_data_direction(&mut self, direction: DataDirectionMode) {
        self.mode = direction;
    }

    pub fn set_data_output(&mut self, data: u8) {
        assert!(
            self.mode == DataDirectionMode::Write,
            "CPU attempting to set data pins value while reading mode"
        );

        self.data = data;
    }

    pub fn set_data_input(&mut self, data: u8) {
        assert!(
            self.mode == DataDirectionMode::Read,
            "CPU attempting to set data pins value while writing mode"
        );

        self.data = data;
    }

    pub fn get_data(&self) -> u8 {
        self.data
    }

    pub fn get_address(&self) -> u16 {
        self.address
    }

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
