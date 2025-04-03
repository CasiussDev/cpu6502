mod test_computer;

use crate::test_computer::TestComputer;

extern crate disasm6502;

const NES_CLOCKS_SECOND: u128 = 1789773;

fn run(computer: &mut TestComputer, num_cycles: u128) {
    computer.cpu.reset();

    while computer.cpu.cycle_count_since_reset() < num_cycles {
        computer.cpu.run(&mut computer.memory);
    }
}

#[test]
//#[ignore]
fn perf_test() {
    let mut computer = TestComputer::new();
    computer.load_rom();

    run(&mut computer, 180 * NES_CLOCKS_SECOND);
}
