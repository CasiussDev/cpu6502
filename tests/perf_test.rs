mod test_computer;

use crate::test_computer::TestComputer;
use cpu6502::YieldStatus;

extern crate disasm6502;

const NES_CLOCKS_SECOND: u128 = 1789773;

fn run(computer: &mut TestComputer, num_cycles: u128) {
    computer.cpu.reset();

    while computer.cpu.get_cycle_count_since_reset() < num_cycles {
        let status = computer.cpu.run();

        if status == YieldStatus::WaitingMemory {
            let addr = computer.cpu.read_address_pins();
            let write_to_memory = computer.cpu.read_writing_to_memory_pin();

            if write_to_memory {
                computer.memory[addr as usize] = computer.cpu.read_data_pins();
            } else {
                computer.cpu.set_data_pins(computer.memory[addr as usize]);
            }
        }
    }
}

#[test]
//#[ignore]
fn perf_test() {
    let mut computer = TestComputer::new();
    computer.load_rom();

    run(&mut computer, 60 * NES_CLOCKS_SECOND);
}
