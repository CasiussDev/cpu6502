pub trait MemorySpace {
    fn read(&mut self, addr: u16) -> u8;
    fn write(&mut self, data: u8, addr: u16);
}

const MEMORY_64K: usize = (u16::MAX as usize) + 1;

impl MemorySpace for [u8; MEMORY_64K] {
    fn read(&mut self, addr: u16) -> u8 {
        self[addr as usize]
    }

    fn write(&mut self, data: u8, addr: u16) {
        self[addr as usize] = data;
    }
}
