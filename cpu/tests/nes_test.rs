extern crate disasm6502;

#[cfg(feature = "logging")]
extern crate simplelog;

mod test_computer;

use crate::test_computer::TestComputer;
use std::io::{BufRead, Write};
use std::{fs, io};

#[cfg(feature = "logging")]
use log::trace;
use cpu6502::FetchedInstr;

fn run(computer: &mut TestComputer, num_cycles: u128) {
    let log_file =
        fs::File::create("testdata/output.log.txt").expect("cannot open output log file");

    let mut log_file = io::BufWriter::new(log_file);

    let mut instr_log = String::new();

    computer.cpu.reset();

    while computer.cpu.cycle_count_since_reset() < num_cycles {
        let status = computer.cpu.run(&mut computer.memory);

        if let FetchedInstr::Some(addr) = computer.cpu.fetched_instr() {
            let instructions = disasm6502::from_addr_array(
                &computer.memory[(addr as usize)..(addr as usize + 6)],
                addr,
            )
            .expect("could not decode instr");
            let decoded = instructions.first().expect("empty instr vector");

            instr_log = format!("{:04X} {} {}", addr, decoded.as_hex_str(), decoded.as_str());

            writeln!(
                log_file,
                "{:<25}{} CYC:{}",
                instr_log,
                computer.cpu.regs_as_log_line(),
                computer.cpu.cycle_count_since_reset()
            )
            .expect("cannot write to file");
        }
    }

    log_file.flush().expect("could not flush file writer");
}

fn check_results() {
    let reference_file_len = fs::metadata("testdata/reference.log.txt")
        .expect("could not read metadata of reference.log.txt")
        .len();
    let output_file_len = fs::metadata("testdata/output.log.txt")
        .expect("could not read metadata of reference.log.txt")
        .len();

    assert_eq!(output_file_len, reference_file_len);

    let reference_file =
        fs::File::open("testdata/reference.log.txt").expect("could not open reference.log.txt");
    let output_file =
        fs::File::open("testdata/output.log.txt").expect("could not open output.log.txt");

    let reference_lines = io::BufReader::new(reference_file).lines();
    let output_lines = io::BufReader::new(output_file).lines();

    for (reference, output) in reference_lines.zip(output_lines) {
        assert_eq!(reference.unwrap(), output.unwrap());
    }
}

#[test]
fn nes_rom_test() {
    #[cfg(feature = "logging")]
    {
        let log_config = simplelog::ConfigBuilder::new()
            .set_max_level(log::LevelFilter::Off)
            .set_time_level(log::LevelFilter::Off)
            .set_thread_level(log::LevelFilter::Off)
            .set_target_level(log::LevelFilter::Off)
            .set_location_level(log::LevelFilter::Off)
            .build();

        let trace_file =
            fs::File::create("testdata/trace.log.txt").expect("cannot open trace file");
        simplelog::WriteLogger::init(log::LevelFilter::Trace, log_config, trace_file).unwrap();
    }

    let mut computer = TestComputer::new();
    computer.load_rom();

    run(&mut computer, 14575);

    check_results();
}
