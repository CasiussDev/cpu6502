#[cfg(feature = "disassembly")]
extern crate disasm6502;

#[cfg(feature = "logging")]
extern crate simplelog;

mod test_computer;

#[cfg(feature = "disassembly")]
use crate::test_computer::TestComputer;
#[cfg(feature = "disassembly")]
use std::io::{BufRead, Write};
#[cfg(feature = "disassembly")]
use std::{fs, io};
#[cfg(feature = "disassembly")]
const REFERENCE_FILE: &'static str = "testdata/reference.6502log";
#[cfg(feature = "disassembly")]
const OUTPUT_FILE: &'static str = "testdata/output.6502log";

#[cfg(feature = "disassembly")]
fn run(computer: &mut TestComputer, num_cycles: u128) {
    let log_file = fs::File::create(OUTPUT_FILE).expect("cannot open output log file");

    let mut log_file = io::BufWriter::new(log_file);

    let mut instr_log;

    computer.cpu.reset();

    while computer.cpu.cycle_count_since_reset() < num_cycles {
        computer.cpu.run(&mut computer.memory);

        if let Some(addr) = computer.cpu.fetched_instr_addr() {
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

#[cfg(feature = "disassembly")]
fn check_results() {
    let reference_file_len = fs::metadata(REFERENCE_FILE)
        .expect("could not read metadata of {REFERENCE_FILE}")
        .len();
    let output_file_len = fs::metadata(OUTPUT_FILE)
        .expect("could not read metadata of {OUTPUT_FILE}")
        .len();

    assert_eq!(output_file_len, reference_file_len);

    let reference_file = fs::File::open(REFERENCE_FILE).expect("could not open {REFERENCE_FILE}");
    let output_file = fs::File::open(OUTPUT_FILE).expect("could not open {OUTPUT_FILE}");

    let reference_lines = io::BufReader::new(reference_file).lines();
    let output_lines = io::BufReader::new(output_file).lines();

    for (reference, output) in reference_lines.zip(output_lines) {
        assert_eq!(
            reference.expect("cannot unwrap reference trace line"),
            output.expect("cannot unwrap output trace line")
        );
    }
}

#[test]
#[cfg(feature = "disassembly")]
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
        simplelog::WriteLogger::init(log::LevelFilter::Trace, log_config, trace_file)
            .expect("WriteLogger::init error");
    }

    let mut computer = TestComputer::new();
    computer.load_rom();

    run(&mut computer, 14575);

    check_results();
}
