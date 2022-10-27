use cpu6502::YieldStatus;
use std::fmt::Debug;
use std::fs;
use std::io;
use std::io::{Read, Seek, SeekFrom, Write};

extern crate disasm6502;

const START_PROG_ADDR_HIGH: u8 = 0xC0;
const START_PROG_ADDR_LOW: u8 = 0x00;
const START_PROG_PTR_HIGH: usize = 0xFFFD;
const START_PROG_PTR_LOW: usize = 0xFFFC;

const NUM_ROM_BYTES: usize = 0x4000;
const ROM_OFFSET: usize = 0x0010;

const ROM_DESTINATION: usize = 0x8000;

const MEMORY_64K: usize = (u16::MAX as usize) + 1;

struct TestComputer {
    cpu: cpu6502::Cpu,
    memory: [u8; MEMORY_64K],
}

impl TestComputer {
    fn load_rom(&mut self) {
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

    pub fn run(&mut self, num_cycles: u64) {
        let mut log_file =
            fs::File::create("testdata/output.log.txt").expect("cannot open output file");

        let mut instr_log = String::new();

        self.cpu.reset();

        while self.cpu.get_cycle_count_since_reset() < num_cycles {
            let status = self.cpu.run();
            //assert_eq!(status, YieldStatus::WaitingMemory);

            if self.cpu.has_decoded() {
                writeln!(
                    log_file,
                    "{}\t{} CYC:{}",
                    instr_log,
                    self.cpu.get_regs_as_log_line(),
                    self.cpu.get_cycle_count_since_reset()
                )
                .expect("cannot write to file");
            }

            if status == YieldStatus::WaitingMemory {
                let addr = self.cpu.read_address_pins();
                let write_to_memory = self.cpu.read_writing_to_memory_pin();

                if write_to_memory {
                    self.memory[addr as usize] = self.cpu.read_data_pins();
                } else {
                    let old_instr_ready = self.cpu.get_instr_ready();
                    self.cpu.set_data_pins(self.memory[addr as usize]);
                    if (old_instr_ready == false) && self.cpu.get_instr_ready() {
                        let instructions = disasm6502::from_addr_array(
                            &self.memory[(addr as usize)..(addr as usize + 6)],
                            addr,
                        )
                        .expect("could not decode instr");
                        let decoded = instructions.first().expect("empty instr vector");

                        instr_log =
                            format!("{:04X} {} {}", addr, decoded.as_hex_str(), decoded.as_str());
                        //println!("{:04X} {} {}", addr, decoded.as_hex_str(), decoded.as_str());
                    }
                }
            }

            //let status = self.cpu.run();
            //assert_eq!(status, YieldStatus::ClockFinished);
        }
    }
}

#[test]
fn nes_rom_test() {
    let mut computer = TestComputer {
        cpu: Default::default(),
        memory: [0; MEMORY_64K],
    };
    computer.load_rom();

    computer.run(14575);
    //computer.run(150);
}
