mod test_computer;

use cpu6502::YieldStatus;
use log::trace;
use std::{fs, io};
use std::io::{BufRead, Write};
use crate::test_computer::TestComputer;

extern crate disasm6502;
extern crate simplelog;

fn run(computer: &mut TestComputer, num_cycles: u128) {
    let mut log_file =
        fs::File::create("testdata/output.log.txt").expect("cannot open output log file");

    let mut instr_log = String::new();

    computer.cpu.reset();

    while computer.cpu.get_cycle_count_since_reset() < num_cycles {
        let status = computer.cpu.run();

        if computer.cpu.has_decoded() {
            writeln!(
                log_file,
                "{:<25}{} CYC:{}",
                instr_log,
                computer.cpu.get_regs_as_log_line(),
                computer.cpu.get_cycle_count_since_reset()
            )
            .expect("cannot write to file");
        }

        if status == YieldStatus::WaitingMemory {
            let addr = computer.cpu.read_address_pins();
            let write_to_memory = computer.cpu.read_writing_to_memory_pin();

            if write_to_memory {
                computer.memory[addr as usize] = computer.cpu.read_data_pins();
                trace!(
                    "\t\t\tWrite Memory[{:04X}] = {:02X}",
                    addr,
                    computer.cpu.read_data_pins()
                );
            } else {
                let old_instr_ready = computer.cpu.get_instr_ready();
                computer.cpu.set_data_pins(computer.memory[addr as usize]);
                trace!(
                    "\t\t\tRead Memory[{:04X}] = {:02X}",
                    addr,
                    computer.cpu.read_data_pins()
                );

                if (old_instr_ready == false) && computer.cpu.get_instr_ready() {
                    let instructions = disasm6502::from_addr_array(
                        &computer.memory[(addr as usize)..(addr as usize + 6)],
                        addr,
                    )
                    .expect("could not decode instr");
                    let decoded = instructions.first().expect("empty instr vector");

                    instr_log =
                        format!("{:04X} {} {}", addr, decoded.as_hex_str(), decoded.as_str());
                }
            }
        }
    }
}

fn check_results() {
    let reference_file = fs::File::open("testdata/reference.log.txt").expect("could not open reference.log.txt");
    let output_file = fs::File::open("testdata/output.log.txt").expect("could not open output.log.txt");

    let reference_lines = io::BufReader::new(reference_file).lines();
    let output_lines = io::BufReader::new(output_file).lines();

    for (reference, output) in reference_lines.zip(output_lines) {
        assert_eq!(reference.unwrap(), output.unwrap());
    }
}

#[test]
fn nes_rom_test() {
    let log_config = simplelog::ConfigBuilder::new()
        .set_max_level(log::LevelFilter::Off)
        .set_time_level(log::LevelFilter::Off)
        .set_thread_level(log::LevelFilter::Off)
        .set_target_level(log::LevelFilter::Off)
        .set_location_level(log::LevelFilter::Off)
        .build();

    let trace_file = fs::File::create("testdata/trace.log.txt").expect("cannot open trace file");
    simplelog::WriteLogger::init(log::LevelFilter::Trace, log_config, trace_file).unwrap();

    let mut computer = TestComputer::new();
    computer.load_rom();

    run(&mut computer, 14579);

    check_results();
}
