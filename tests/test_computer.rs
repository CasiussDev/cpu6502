use log::trace;
use std::io::{Read, Seek, SeekFrom};
use std::{fs, io};

pub const START_PROG_ADDR_HIGH: u8 = 0xC0;
pub const START_PROG_ADDR_LOW: u8 = 0x00;
pub const START_PROG_PTR_HIGH: usize = 0xFFFD;
pub const START_PROG_PTR_LOW: usize = 0xFFFC;

pub const NUM_ROM_BYTES: usize = 0x4000;
pub const ROM_OFFSET: usize = 0x0010;

pub const ROM_DESTINATION: usize = 0x8000;

pub const MEMORY_64K: usize = (u16::MAX as usize) + 1;

pub struct TestComputer {
    pub cpu: cpu6502::Cpu,
    pub memory: [u8; MEMORY_64K],
}

impl TestComputer {
    pub fn new() -> Self {
        Self {
            cpu: Default::default(),
            memory: [0; MEMORY_64K],
        }
    }

    pub fn load_rom(&mut self) {
        let mut rom_file = fs::File::open("testdata/nestest.nes").expect("could not load rom file");
        rom_file
            .seek(SeekFrom::Start(ROM_OFFSET as u64))
            .expect("error in seek call");
        let rom_file = rom_file.take(NUM_ROM_BYTES as u64);

        let mut rom_reader = io::BufReader::new(rom_file);

        let mut read_rom_content = Vec::<u8>::with_capacity(NUM_ROM_BYTES);

        rom_reader
            //.take(NUM_ROM_BYTES)
            .read_to_end(&mut read_rom_content)
            .expect("error reading rom");

        assert_eq!(read_rom_content.len(), NUM_ROM_BYTES);

        let ranges = vec![
            ROM_DESTINATION..(ROM_DESTINATION + NUM_ROM_BYTES),
            (ROM_DESTINATION + NUM_ROM_BYTES)..(ROM_DESTINATION + NUM_ROM_BYTES * 2),
        ];

        for i in 0..2 {
            let dst = &mut self.memory[ranges[i].clone()];
            dst.copy_from_slice(&read_rom_content);
        }

        self.memory[START_PROG_PTR_HIGH] = START_PROG_ADDR_HIGH;
        self.memory[START_PROG_PTR_LOW] = START_PROG_ADDR_LOW;
    }
}
