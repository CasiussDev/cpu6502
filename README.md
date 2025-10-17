# cpu6502

A fast, testable 6502 CPU core written in Rust (2021 edition). Designed as a reusable library for emulators and tools, with optional logging and code generators for decode and write-cycle analysis.

- Library-first API: `Cpu`, `MemorySpace`, and `new_basic_ram()` are re-exported at the crate root for easy use.
- Feature flags enable optional behaviors (logging, decimal mode, undocumented opcodes) and generator binaries.
- Includes an integration test that reproduces the classic NES `nestest` trace when the logging feature is enabled.


## Contents
- Getting started
- Features
- Build
- Testing
- Quick start example
- Generators (optional)
- Development notes


## Getting started
Prerequisites:
- Rust 1.70+ (Rust 2021 edition)
- Windows/Linux/macOS

Clone and build:
```
cargo build
```

Optional logging build:
```
cargo build --features "logging"
```


## Features
Enable features via `--features "..."`.

- `decimal`: Enable decimal (BCD) behavior for ADC/SBC. Useful if software under test relies on BCD.
- `undoc_opcodes`: Include unofficial 6502 opcodes in decode/execute. Leave off unless testing code that expects them.
- `logging`: Enable execution/disassembly logging (adds `Cpu::init_logging_*` and a `LoggingMemory` wrapper).
- `decode_logic`: Enable the canonical decoder logic and the `gen_decode_switch` binary.
- `gen_write_cycle_query`: Switch CPU internals to use a generated write-cycle table and enable the `gen_write_cycle_query` binary.


## Build
- Fast dev build (no features):
  - `cargo build`
- With logging:
  - `cargo build --features "logging"`
- With decoder generator support:
  - `cargo build --features "decode_logic"`
- With write-cycle generator support:
  - `cargo build --features "gen_write_cycle_query"`


## Testing
Default, fast code tests (excludes doctests):
```
cargo test --lib --bins --tests --benches
```

Full test run (includes doctests; may require writable filesystem for logging examples):
```
cargo test
```

NES logging integration test (requires feature and test assets in `testdata/`):
```
cargo test --features "logging" --test nes_test -- --nocapture
```
What it does: loads `testdata/nestest.nes`, runs for a fixed cycle budget, writes a trace to `testdata/output.6502log`, and compares to `testdata/reference.6502log`.

Working with features in tests:
- Decode logic path:
  - `cargo run --bin gen_decode_switch --features "decode_logic"`
  - `cargo test --features "decode_logic"`
- Write-cycle detection path:
  - `cargo run --bin gen_write_cycle_query --features "gen_write_cycle_query"`
  - `cargo test --features "gen_write_cycle_query"`

Note: On clean CI environments, two doctests (`Cpu::init_logging_trace`, `Cpu::init_logging_debug`) can fail if they cannot create files. Prefer the selective command above for routine development.


## Quick start example
A minimal smoke test that loads a tiny program and runs until it stores to memory:

```rust
// Cargo.toml
// [dependencies]
// cpu6502 = { path = "." }

fn main() {
    let mut ram = cpu6502::new_basic_ram();

    // Program at $8000: LDA #$05; ADC #$03; STA $0200
    ram[0x8000] = 0xA9; ram[0x8001] = 0x05; // LDA #$05
    ram[0x8002] = 0x69; ram[0x8003] = 0x03; // ADC #$03
    ram[0x8004] = 0x8D; ram[0x8005] = 0x00; ram[0x8006] = 0x02; // STA $0200

    // Reset vector → $8000
    ram[0xFFFC] = 0x00; ram[0xFFFD] = 0x80;

    let mut cpu = cpu6502::Cpu::new();

    // Run cycles until memory reflects the expected store or a safety bound is hit.
    for _ in 0..2000 {
        if ram[0x0200] == 0x08 { break; }
        cpu.run(&mut ram);
    }

    assert_eq!(ram[0x0200], 0x08);
}
```

Notes:
- `Cpu::run(&mut MemorySpace)` performs a single cycle. Many instructions take multiple cycles.
- The CPU powers up in the Reset instruction; the first program instruction runs after Reset sets PC from `$FFFC/$FFFD`.


## Generators (optional)
Two helper binaries live under `src/bin/` and are gated by features. They regenerate tables used in the CPU:

- `gen_decode_switch` (feature: `decode_logic`)
  - Regenerates `src/instr/opcodes/decode_switch.rs` with the full opcode→instruction match.
  - Run: `cargo run --bin gen_decode_switch --features "decode_logic"`
  - Requires `rustfmt` on `PATH` to format the generated file.

- `gen_write_cycle_query` (feature: `gen_write_cycle_query`)
  - Regenerates `src/cpu/write_cycle_query.rs` to answer: “does instruction X at step S write to memory?”
  - Run: `cargo run --bin gen_write_cycle_query --features "gen_write_cycle_query"`
  - Internals: Simulates each instruction step using `execute(...)` with a `MemorySpace` that records writes.


## Development notes
- Module layout
  - `cpu` (cpu_impl.rs): core CPU state machine; public API includes `Cpu::new`, `Cpu::run`, counters, IRQ/NMI control, and logging setup (when `logging` is enabled).
  - `instr`: instruction enum, decode logic, and per-cycle microcode. Decode switch may be generated (`decode_switch.rs`) or implemented via logic depending on features.
  - `alu`: flag‑correct arithmetic/logic helpers for instruction executors.
  - `memory`: `MemorySpace` trait, `BasicRam` implementation, and `new_basic_ram()` factory.
  - `registers`: register file and status flag helpers.

- Execution model
  - Prefer testing on instruction completion or externally observable state (e.g., memory/registers) rather than fixed cycle counts.

- Performance/CI
  - Default selective test run is fast (<100 ms on typical machines). Keep tests deterministic.


## License

Licensed under either of

- Apache License, Version 2.0 (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
