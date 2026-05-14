# Changelog

## [0.1.0] - 2026-05-14

### Added

#### Core CPU Emulation
- Fully cycle-accurate 6502 CPU emulator with complete instruction set support
- All standard 6502 addressing modes (Implied, Immediate, Absolute, Zero Page, Indexed, Indirect, Relative)
- Multi-cycle instruction execution (2-7 cycles per instruction depending on addressing mode and page boundary crossing)
- Page boundary crossing detection and cycle penalty handling
- Interrupt support (IRQ and NMI) with proper edge triggering
- Reset vector initialization via `$FFFC/$FFFD`

#### Instruction Set
- Complete 6502 instruction set implementation with cycle-accurate timing
- ALU operations: ADD, ADC, AND, OR, EOR, ASL, LSR, ROL, ROR, BIT operations
- Load/Store operations: LDA, LDX, LDY, STA, STX, STY with all addressing modes
- Increment/Decrement: INC, DEC, INX, INY, DEX, DEY
- Shift and Rotate: ASL, LSR, ROL, ROR (both accumulator and memory variants)
- Branch instructions: BEQ, BNE, BCS, BCC, BMI, BPL, BVS, BVC with signed relative addressing
- Jump and Subroutine: JMP, JSR, RTS with proper return address handling
- Stack operations: PHA, PLA, PHP, PLP with stack page isolation
- Flag operations: CLC, SEC, CLD, SED, CLI, SEI, CLV
- Comparison instructions: CMP, CPX, CPY with flag-correct behavior
- **Note**: Decimal (BCD) mode and undocumented opcodes are not yet implemented

#### Memory System
- `MemorySpace` trait for custom memory implementations
- `BasicRam` 64KB default RAM implementation
- Customizable memory access patterns for logging, breakpoints, and emulator extensions
- Support for memory-mapped I/O via trait implementation

#### Register File
- 8-bit Accumulator (A), X Index, Y Index registers
- 16-bit Program Counter (PC) and Stack Pointer (SP)
- 8-bit Status register with flags: Carry, Zero, Interrupt Disable, Decimal, Break, Overflow, Negative

#### Features
- **`std` (default)**: Standard library support with full functionality
- **`logging`**: Instruction tracing and disassembly for debugging with `LoggingMemory` wrapper
- **`decimal`**: Decimal (BCD) mode support for ADC/SBC instructions *(not yet implemented)*
- **`undoc_opcodes`**: Undocumented and illegal 6502 opcodes *(not yet implemented)*
- **`decode_logic`**: Code generator for canonical opcode decoding
- **`gen_write_cycle_query`**: Code generator for write-cycle detection
- **`no_std`**: Core emulation compatible with no_std environments (alloc-free)

#### Code Generators
- `gen_decode_switch`: Generates opcode decode lookup table (`src/instr/opcodes/decode_switch.rs`)
- `gen_write_cycle_query`: Generates write-cycle detection table (`src/cpu/write_cycle_query.rs`)

#### Performance & Optimization
- Efficient cycle counting with `u64` for high instruction counts
- Inlined hot-path functions for minimal overhead
- LTO (Link Time Optimization) in release builds
- Code generation utilities for lookup table creation

#### Testing & Validation
- Unit tests for individual instruction execution across multiple cycles
- Integration tests with NES ROM (nestest) validation
- Comprehensive test suite covering addressing modes and edge cases
- Test helpers for reproducible CPU state setup

#### Documentation
- Comprehensive README with architecture overview
- Module-level documentation for public API
- Inline documentation for complex instruction sequences and addressing modes
- Quick-start example with minimal 6502 program
- Development notes for maintainers and contributors

#### Dual Licensing
- Licensed under MIT or Apache-2.0

### Technical Details

- Written in Rust 2021 edition
- No external runtime dependencies for core functionality
- Optional dependencies for logging, disassembly, and code generation
- Modular architecture separating concerns:
  - `cpu`: Core CPU state machine and public API
  - `instr`: Instruction decoding and execution
  - `alu`: Arithmetic and logic operations
  - `memory`: Memory abstraction layer
  - `registers`: Register file and status flags
  - `disassemble`: Optional disassembly support (when `logging` feature enabled)

### Known Limitations

- **`decimal` feature**: BCD mode not yet implemented
- **`undoc_opcodes` feature**: Undocumented opcodes not yet implemented

[0.1.0]: https://github.com/CasiussDev/cpu6502/releases/tag/v0.1.0
