# cpu6502

A cycle-accurate 6502 CPU emulator library written in Rust (2021 edition). Designed as a reusable, testable library for emulators, retro computing tools, and NES projects. It implements the complete 6502 instruction set with cycle-accurate timing and multi-cycle execution.

**Features:**
- Fully cycle-accurate instruction execution with multi-cycle addressing modes
- Optional instruction logging and disassembly for debugging
- Code generators for opcode decoding and write-cycle detection
- Support for decimal mode (BCD), undocumented opcodes, and custom memory implementations
- `no_std` compatible core with optional `std` features
- Includes NES integration test (nestest) for validation

**Quick facts:**
- Each instruction takes 2-7 cycles depending on addressing mode and page boundary crossing
- Memory accessed via trait, allowing custom implementations for logging, breakpoints, etc.
- Reset vectors at `$FFFC/$FFFD` control program startup
- Stack lives in page 0x01

## Contents
- [Getting started](#getting-started)
- [Architecture](#architecture)
- [Features](#features)
- [Build](#build)
- [Testing](#testing)
- [Quick start example](#quick-start-example)
- [Generators (optional)](#generators-optional)
- [Development notes](#development-notes)
- [No-std considerations](#no-std-considerations)


## Getting started

**Prerequisites:**
- Rust 1.70+ (Rust 2021 edition)
- Windows/Linux/macOS
- `rustfmt` (required if regenerating code generators)

**Clone and build:**
```bash
# Default build
cargo build

# With logging for debugging
cargo build --features "logging"
```


## Architecture

### High-Level Design
The 6502 is a multi-cycle CPU where each instruction takes multiple clock cycles. This library models that behavior exactly through a state machine:

- **Instruction Fetching**: CPU reads an opcode and decodes it into an `Instruction` enum
- **Multi-Cycle Execution**: Each instruction executes step-by-step via `instr_impl`, with each step returning `ClockEndStatus` indicating if more cycles are needed
- **Memory Abstraction**: The `MemorySpace` trait allows custom memory implementations (logging, breakpoints, etc.); `BasicRam` is the default 64KB array

### Module Organization
- **`cpu`** (`cpu_impl.rs`): Core CPU state machine and public API (`Cpu::new`, `Cpu::run`). Manages interrupt handling, instruction sequencing, performance counters, and optional logging.
- **`instr`**: Instruction processing:
  - `instr_operation`: What each instruction does (ADD, AND, branches, etc.)
  - `instr_sequences`: Addressing modes and instruction fetch/execute/store sequences
  - `opcodes`: Maps opcodes (0x00-0xFF) to `Instruction` enum variants
  - `instr_impl`: Cycle-by-cycle execution logic with unit tests in `instr_impl/tests/`
  - `disassemble`: Instruction disassembly (with `logging` feature only)
- **`alu`**: Flag-correct arithmetic/logic helpers (ADD, ADC, AND, shifts, rotations)
- **`memory`**: `MemorySpace` trait, `BasicRam` 64KB implementation, and `new_basic_ram()` factory
- **`registers`**: Register file (`A`, `X`, `Y`, `PC`, `SP`, status flags) and helpers like `IndexRegister`

### Execution Model
- `Cpu::run(&mut MemorySpace)` executes a **single cycle**, not a full instruction
- Instructions take 2-7 cycles depending on addressing mode:
  - Implied/Immediate: 2 cycles
  - Absolute: 3-4 cycles
  - Indexed addressing: 4-7 cycles (may add cycles for page boundary crossing)
- CPU starts in the Reset instruction; the first program instruction runs after Reset reads the reset vector from `$FFFC/$FFFD`
- Design philosophy: **Verify externally observable state** (registers, memory) in tests rather than cycle counts, unless testing specific multi-cycle or boundary crossing behavior

### Addressing Modes
All standard 6502 addressing modes are supported with cycle-accurate timing:
- **Implied/Implicit**: No memory operand
- **Immediate**: 8-bit operand in instruction
- **Absolute**: Full 16-bit address
- **Zero Page**: 8-bit address in page 0x00
- **Indexed (X/Y)**: Base address + index register; may add cycles crossing page boundary
- **Indirect**: Two-stage address lookup (includes the 6502's famous indirect addressing bug)
- **Relative**: Branch instructions use 8-bit signed offset from PC

The `instr_sequences` module encodes these patterns; each sequence progresses through fetch/modify/execute/store stages as needed.

## Features

Enable features via `cargo build --features "feature_name"` or in Cargo.toml:

| Feature | Purpose | Notes |
|---------|---------|-------|
| `std` (default) | Enables standard library | Required for most other features; can be disabled for `no_std` core |
| `logging` | Instruction tracing and disassembly | Adds `Cpu::init_logging_*` methods and `LoggingMemory` wrapper; uses `log` crate |
| `decimal` | Decimal (BCD) mode for ADC/SBC | For software that relies on BCD arithmetic |
| `undoc_opcodes` | Undocumented 6502 opcodes | Includes illegal/undocumented instructions |
| `decode_logic` | Canonical opcode decoder | Enables `gen_decode_switch` generator binary |
| `gen_write_cycle_query` | Write-cycle detection generator | Enables `gen_write_cycle_query` binary; changes CPU internals to use generated lookup |


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

### Fast development testing (recommended)
```bash
# Core tests only (excludes doctests)
cargo test --lib --bins --tests
```

### Complete test suite
```bash
# Full run including doctests (requires writable filesystem)
cargo test
```

### Feature-specific tests
```bash
# Test with decoder logic feature
cargo test --features "decode_logic"

# Test with write-cycle generator
cargo test --features "gen_write_cycle_query"

# Test with logging
cargo test --features "logging"
```

### Integration testing with NES ROM
```bash
# NES integration test (requires testdata/nestest.nes)
cargo test --features "logging" --test nes_test -- --nocapture
```

What it does:
- Loads `testdata/nestest.nes` (the classic NES CPU test ROM)
- Runs for a fixed cycle budget
- Writes instruction trace to `testdata/output.6502log`
- Compares against `testdata/reference.6502log` for validation

**CI Note:** On systems without writable filesystem, use `cargo test --lib --bins --tests` to skip doctests that depend on file I/O.

### Running individual tests
```bash
# Run a specific test module
cargo test --test test_computer

# Run a specific test in instr_impl
cargo test --lib instr::instr_impl::tests::test_lda
```

### Test structure
- **Unit tests** in ``src/instr/instr_impl/tests/` verify individual instructions over multiple cycles
- **Integration tests** in `tests/` verify program execution and state changes
- **Test helpers** in ``src/instr/instr_impl/tests/helpers.rs` set up CPU and memory for reproducible tests


## Quick start example

A minimal program that loads a tiny routine, runs it, and verifies the result:

```rust
use cpu6502::{Cpu, new_basic_ram};

fn main() {
    let mut ram = new_basic_ram();

    // Program at $8000: LDA #$05; ADC #$03; STA $0200
    ram[0x8000] = 0xA9; ram[0x8001] = 0x05; // LDA #$05     (2 cycles)
    ram[0x8002] = 0x69; ram[0x8003] = 0x03; // ADC #$03     (2 cycles)
    ram[0x8004] = 0x8D; ram[0x8005] = 0x00; ram[0x8006] = 0x02; // STA $0200 (4 cycles)

    // Reset vector → $8000
    ram[0xFFFC] = 0x00; ram[0xFFFD] = 0x80;

    let mut cpu = Cpu::new();

    // Run until memory reflects the expected store or a safety limit is hit
    for _ in 0..2000 {
        if ram[0x0200] == 0x08 { break; }
        cpu.run(&mut ram);
    }

    assert_eq!(ram[0x0200], 0x08); // Expected: 0x05 + 0x03 = 0x08
}
```

**Key points:**
- `Cpu::run(&mut MemorySpace)` advances a **single cycle**, not a complete instruction
- The program counter starts at the Reset instruction; it reads the reset vector from `$FFFC/$FFFD` before fetching the first opcode
- Different addressing modes take different numbers of cycles (see [Addressing Modes](#addressing-modes))
- Test should verify final register/memory state, not cycle counts


## Generators (optional)

Two code generators live in `src/bin/` and are gated by features. They regenerate lookup tables used by the CPU:

### gen_decode_switch
Maps each opcode (0x00-0xFF) to its corresponding instruction operation and addressing mode.

```bash
cargo run --bin gen_decode_switch --features "decode_logic"
```
- Regenerates: `src/instr/opcodes/decode_switch.rs`
- Requires `rustfmt` on `PATH` to format the output
- Use when: adding new opcodes or modifying the instruction set

### gen_write_cycle_query
Generates a lookup table answering: "Does instruction X at execution step S write to memory?"

```bash
cargo run --bin gen_write_cycle_query --features "gen_write_cycle_query"
```
- Regenerates: `src/cpu/write_cycle_query.rs`
- How it works: Simulates each instruction step using `execute(...)` with a custom `MemorySpace` that records writes
- Requires `rustfmt` on `PATH`
- Use when: instruction execution logic changes or new instructions are added


## Development notes
- Module layout
  - `cpu` (cpu_impl.rs): core CPU state machine; public API includes `Cpu::new`, `Cpu::run`, counters, IRQ/NMI control, and logging setup (when `logging` is enabled).
  - `instr`: instruction enum, decode logic, and per-cycle microcode. Decode switch may be generated (`decode_switch.rs`) or implemented via logic depending on features.
  - `alu`: flag-correct arithmetic/logic helpers for instruction executors.
  - `memory`: `MemorySpace` trait, `BasicRam` implementation, and `new_basic_ram()` factory.
  - `registers`: register file and status flag helpers.

- Execution model
  - Prefer testing on instruction completion or externally observable state (e.g., memory/registers) rather than fixed cycle counts.

## No-std Considerations

The library is `no_std` by default (`#![no_std]`). Features requiring `std` (like file I/O logging or test helpers) depend on the `std` feature flag.

When adding new code:
- **Avoid allocations** when possible (use `arrayvec` for small fixed-size buffers)
- **Wrap `std` dependencies** behind `#[cfg(feature = "std")]` or `#[cfg(any(feature = "std", test))]`
- **Test CPU core logic** with `cargo test --no-default-features` to ensure no_std compatibility
- Keep the core execution engine in `no_std` for embedded/constrained environments


## License

Licensed under either of

- Apache License, Version 2.0 (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

