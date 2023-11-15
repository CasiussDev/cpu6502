use crate::MemorySpace;
use log::debug;

pub struct LoggingMemory<'a, T> {
    inner: &'a mut T,
}

impl<'a, T> LoggingMemory<'a, T>
where
    T: MemorySpace,
{
    pub fn new(inner: &'a mut T) -> Self {
        Self { inner }
    }

    fn read_inner(&mut self, addr: u16) -> u8 {
        self.inner.read(addr)
    }
}

impl<'a, T> MemorySpace for LoggingMemory<'a, T>
where
    T: MemorySpace,
{
    fn read(&mut self, addr: u16) -> u8 {
        let data = self.inner.read(addr);
        debug!("\t\t\tRead Memory[{:04X}] = {:02X}", addr, data);
        data
    }

    fn write(&mut self, data: u8, addr: u16) {
        debug!("\t\t\tWrite Memory[{:04X}] = {:02X}", addr, data,);
        self.inner.write(data, addr);
    }

    fn read_array(&mut self, addr: u16, out: &mut [u8]) {
        self.inner.read_array(addr, out);
    }
}
