mod helpers;
mod instr;
mod op;

use super::*;

struct MockMemory {
    data: [u8; 65536],
}

impl MemorySpace for MockMemory {
    fn read(&mut self, addr: u16) -> u8 {
        self.data[addr as usize]
    }

    fn write(&mut self, value: u8, addr: u16) {
        self.data[addr as usize] = value;
    }
}
