#[cfg(not(debug_assertions))]
mod test_computer;

#[cfg(not(debug_assertions))]
mod inner {
    use super::test_computer::TestComputer;

    const NES_CLOCKS_SECOND: u64 = 1789773;

    fn run(computer: &mut TestComputer, num_cycles: u64) {
        computer.cpu.reset();

        while computer.cpu.cycle_count_since_reset() < num_cycles {
            computer.cpu.run(&mut computer.memory);
        }
    }

    #[test]
    #[cfg(not(debug_assertions))]
    fn perf_test() {
        let mut computer = TestComputer::new();
        computer.load_rom();

        run(&mut computer, 180 * NES_CLOCKS_SECOND);
    }
}
