//! Utilities for testing the 6502 CPU implementation with ROM loading and memory setup.
//!
//! This module provides the `TestComputer` struct, which encapsulates a CPU instance and
//! a 64KB memory array. It includes helper methods for initializing the test environment
//! and loading NES ROM files for integration testing.

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

/// Represents a test computer with a 6502 CPU and 64KB of memory.
///
/// This struct is used for integration testing, providing methods to
/// initialize the CPU and load NES ROM files into memory.
pub struct TestComputer {
    /// The 6502 CPU instance under test.
    pub cpu: cpu6502::Cpu,
    /// The 64KB memory array used by the CPU.
    pub memory: [u8; MEMORY_64K],
}

impl Default for TestComputer {
    /// Creates a new `TestComputer` with a fresh CPU and zeroed memory.
    fn default() -> Self {
        Self {
            cpu: cpu6502::Cpu::new(),
            memory: [0; MEMORY_64K],
        }
    }
}

impl TestComputer {
    /// Constructs a new `TestComputer` using the default implementation.
    pub fn new() -> Self {
        Self::default()
    }

    /// Loads a NES ROM file into memory and patches the reset vector.
    ///
    /// This method reads the ROM file `testdata/nestest.nes`, skips the header,
    /// and loads the PRG ROM into the appropriate memory locations. It also
    /// sets the reset vector and applies a code patch to execute the program
    /// in an infinite loop.
    ///
    /// # Panics
    ///
    /// Panics if the ROM file cannot be opened, read, or if the ROM size is incorrect.
    pub fn load_rom(&mut self) {
        // Open the NES ROM file for reading.
        let mut rom_file = fs::File::open("testdata/nestest.nes").expect("could not load rom file");
        // Seek past the 16-byte NES header.
        rom_file
            .seek(SeekFrom::Start(ROM_OFFSET as u64))
            .expect("error in seek call");
        // Limit reading to the PRG ROM size (16KB).
        let rom_file = rom_file.take(NUM_ROM_BYTES as u64);

        // Wrap the file in a buffered reader.
        let mut rom_reader = io::BufReader::new(rom_file);

        // Prepare a buffer to hold the ROM contents.
        let mut read_rom_content = Vec::<u8>::with_capacity(NUM_ROM_BYTES);

        // Read the PRG ROM data into the buffer.
        rom_reader
            //.take(NUM_ROM_BYTES)
            .read_to_end(&mut read_rom_content)
            .expect("error reading rom");

        // Ensure the ROM size is as expected.
        assert_eq!(read_rom_content.len(), NUM_ROM_BYTES);

        // Define the memory ranges to copy the ROM into (mirrored).
        let ranges = [
            ROM_DESTINATION..(ROM_DESTINATION + NUM_ROM_BYTES),
            (ROM_DESTINATION + NUM_ROM_BYTES)..(ROM_DESTINATION + NUM_ROM_BYTES * 2),
        ];

        // Copy the ROM data into both ranges.
        for range in ranges {
            let dst = &mut self.memory[range];
            dst.copy_from_slice(&read_rom_content);
        }

        // Set the reset vector to the program start address.
        self.memory[START_PROG_PTR_HIGH] = START_PROG_ADDR_HIGH;
        self.memory[START_PROG_PTR_LOW] = START_PROG_ADDR_LOW;

        // Patch the code at $C6BD to create an infinite loop for testing.
        let code_patch: Vec<u8> = vec![
            0x68, 0x68, 0x68, 0x68, // 4 PLA
            0xA9, 0x24, // LDA #$24
            0x48, // PHA
            0x28, // PLA
            0x4C, 0xF5, 0xC5, // JMP $C5F5
        ];

        // Write the code patch into memory at $C6BD.
        self.memory[0xC6BD..(0xC6BD + code_patch.len())].copy_from_slice(&code_patch);
    }
}
