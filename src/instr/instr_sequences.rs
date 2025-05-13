use crate::registers::IndexRegister;
use strum_macros::EnumDiscriminants;

use crate::instr::{
    BranchOperation, ImplicitOperation, InstructionOp, MemoryModifyOperation, PullStackOperation,
    PushStackOperation, RegisterMemoryOperation,
};
#[cfg(feature = "gen_write_cycle_query")]
use strum_macros::EnumIter;

/// Represents the complete set of instruction execution sequences for the 6502 CPU.
///
/// This enum defines all the instruction types that the 6502 can execute, categorized
/// by their addressing modes and operation types. Each variant encapsulates:
///
/// 1. The instruction sequence mode (how the instruction progresses through cycles)
/// 2. (Optional) The specific operation to perform (e.g., Add, Or, LoadA)
/// 3. (Optional) Index register to use when calculating the memory address
///
/// The CPU's execution logic uses this enum to determine how to process each instruction
/// over multiple clock cycles, following the 6502's cycle-accurate behavior. Each variant
/// corresponds to a specific combination of opcode and addressing mode.
///
/// The enum is used both for execution and for features like cycle-accurate write access
/// detection, which is important for emulating hardware effects precisely.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default, EnumDiscriminants)]
#[cfg_attr(feature = "gen_write_cycle_query", derive(EnumIter))]
#[strum_discriminants(name(InstructionSequenceMode))]
#[strum_discriminants(derive(Default))]
pub enum Instruction {
    /// Fetches the next instruction from memory. This is the starting point
    /// of every instruction execution cycle.
    #[default]
    #[strum_discriminants(default)]
    FetchInstr,

    /// Handles the BRK instruction (software interrupt).
    Break,

    /// Initiates an Interrupt Request (IRQ) sequence.
    StartIrq,

    /// Initiates a Non-Maskable Interrupt (NMI) sequence.
    StartNmi,

    /// Executes a CPU reset sequence.
    Reset,

    /// Returns from an interrupt (RTI instruction).
    ReturnInterrupt,

    /// Jumps to a subroutine (JSR instruction).
    JumpSubroutine,

    /// Returns from a subroutine (RTS instruction).
    ReturnSubroutine,

    /// Pushes a value onto the stack (PHA, PHP instructions).
    Push(PushStackOperation),

    /// Pulls a value from the stack (PLA, PLP instructions).
    Pull(PullStackOperation),

    /// Executes an implied mode instruction (e.g., TAX, INX, CLC).
    Implied(ImplicitOperation),

    /// Executes an immediate mode instruction (e.g., LDA #$00).
    Immediate(RegisterMemoryOperation),

    /// Jumps to an absolute address (JMP instruction).
    AbsoluteJump,

    /// Executes an absolute mode instruction (e.g., LDA $1234).
    Absolute(RegisterMemoryOperation),

    /// Executes a read-modify-write instruction with absolute addressing (e.g., INC $1234).
    AbsoluteReadModifyWrite(MemoryModifyOperation),

    /// Executes a zero-page mode instruction (e.g., LDA $12).
    ZeroPage(RegisterMemoryOperation),

    /// Executes a read-modify-write instruction with zero-page addressing (e.g., INC $12).
    ZeroPageReadModifyWrite(MemoryModifyOperation),

    /// Executes a zero-page indexed mode instruction (e.g., LDA $12,X).
    ZeroPageIdx(RegisterMemoryOperation, IndexRegister),

    /// Executes a read-modify-write instruction with zero-page indexed addressing (e.g., INC $12,X).
    ZeroPageIdxReadModifyWrite(MemoryModifyOperation, IndexRegister),

    /// Executes an absolute indexed read instruction (e.g., LDA $1234,X).
    AbsoluteIdxRead(RegisterMemoryOperation, IndexRegister),

    /// Executes a read-modify-write instruction with absolute indexed addressing (e.g., INC $1234,X).
    AbsoluteIdxReadModifyWrite(MemoryModifyOperation, IndexRegister),

    /// Executes an absolute indexed write instruction (e.g., STA $1234,X).
    AbsoluteIdxWrite(RegisterMemoryOperation, IndexRegister),

    /// Executes a relative branch instruction (e.g., BEQ, BNE).
    Relative(BranchOperation),

    /// Executes a zero-page indexed indirect instruction (pre-indexed, e.g., LDA ($12,X)).
    ZeroPageIdxIndirect(RegisterMemoryOperation, IndexRegister),

    /// Executes a read-modify-write instruction with zero-page indexed indirect addressing.
    ZeroPageIdxIndirectReadModifyWrite(MemoryModifyOperation, IndexRegister),

    /// Executes a zero-page indirect indexed read instruction (post-indexed, e.g., LDA ($12),Y).
    ZeroPageIndirectIdxRead(RegisterMemoryOperation, IndexRegister),

    /// Executes a read-modify-write instruction with zero-page indirect indexed addressing.
    ZeroPageIndirectIdxReadModifyWrite(MemoryModifyOperation, IndexRegister),

    /// Executes a zero-page indirect indexed write instruction (e.g., STA ($12),Y).
    ZeroPageIndirectIdxWrite(RegisterMemoryOperation, IndexRegister),

    /// Jumps to an address stored at the specified location (JMP ($1234)).
    AbsoluteIndirectJump,
}

impl From<Instruction> for (InstructionSequenceMode, InstructionOp, IndexRegister) {
    fn from(sequence: Instruction) -> Self {
        match sequence {
            Instruction::Reset => (
                InstructionSequenceMode::Reset,
                InstructionOp::default(),
                IndexRegister::default(),
            ),
            Instruction::FetchInstr => (
                InstructionSequenceMode::FetchInstr,
                InstructionOp::default(),
                IndexRegister::default(),
            ),
            Instruction::StartNmi => (
                InstructionSequenceMode::StartNmi,
                InstructionOp::default(),
                IndexRegister::default(),
            ),
            Instruction::StartIrq => (
                InstructionSequenceMode::StartIrq,
                InstructionOp::default(),
                IndexRegister::default(),
            ),
            Instruction::Break => (
                InstructionSequenceMode::Break,
                InstructionOp::default(),
                IndexRegister::default(),
            ),
            Instruction::ReturnInterrupt => (
                InstructionSequenceMode::ReturnInterrupt,
                InstructionOp::default(),
                IndexRegister::default(),
            ),
            Instruction::JumpSubroutine => (
                InstructionSequenceMode::JumpSubroutine,
                InstructionOp::default(),
                IndexRegister::default(),
            ),
            Instruction::ReturnSubroutine => (
                InstructionSequenceMode::ReturnSubroutine,
                InstructionOp::default(),
                IndexRegister::default(),
            ),
            Instruction::Push(op) => (
                InstructionSequenceMode::Push,
                op.into(),
                IndexRegister::default(),
            ),
            Instruction::Pull(op) => (
                InstructionSequenceMode::Pull,
                op.into(),
                IndexRegister::default(),
            ),
            Instruction::Implied(op) => (
                InstructionSequenceMode::Implied,
                op.into(),
                IndexRegister::default(),
            ),
            Instruction::Immediate(op) => (
                InstructionSequenceMode::Immediate,
                op.into(),
                IndexRegister::default(),
            ),
            Instruction::AbsoluteJump => (
                InstructionSequenceMode::AbsoluteJump,
                InstructionOp::default(),
                IndexRegister::default(),
            ),
            Instruction::Absolute(op) => (
                InstructionSequenceMode::Absolute,
                op.into(),
                IndexRegister::default(),
            ),
            Instruction::AbsoluteReadModifyWrite(op) => (
                InstructionSequenceMode::AbsoluteReadModifyWrite,
                op.into(),
                IndexRegister::default(),
            ),
            Instruction::ZeroPage(op) => (
                InstructionSequenceMode::ZeroPage,
                op.into(),
                IndexRegister::default(),
            ),
            Instruction::ZeroPageReadModifyWrite(op) => (
                InstructionSequenceMode::ZeroPageReadModifyWrite,
                op.into(),
                IndexRegister::default(),
            ),
            Instruction::ZeroPageIdx(op, idx) => {
                (InstructionSequenceMode::ZeroPageIdx, op.into(), idx)
            }
            Instruction::ZeroPageIdxReadModifyWrite(op, idx) => (
                InstructionSequenceMode::ZeroPageIdxReadModifyWrite,
                op.into(),
                idx,
            ),
            Instruction::AbsoluteIdxRead(op, idx) => {
                (InstructionSequenceMode::AbsoluteIdxRead, op.into(), idx)
            }
            Instruction::AbsoluteIdxReadModifyWrite(op, idx) => (
                InstructionSequenceMode::AbsoluteIdxReadModifyWrite,
                op.into(),
                idx,
            ),
            Instruction::AbsoluteIdxWrite(op, idx) => {
                (InstructionSequenceMode::AbsoluteIdxWrite, op.into(), idx)
            }
            Instruction::Relative(op) => (
                InstructionSequenceMode::Relative,
                op.into(),
                IndexRegister::default(),
            ),
            Instruction::ZeroPageIdxIndirect(op, idx) => {
                (InstructionSequenceMode::ZeroPageIdxIndirect, op.into(), idx)
            }
            Instruction::ZeroPageIdxIndirectReadModifyWrite(op, idx) => (
                InstructionSequenceMode::ZeroPageIdxIndirectReadModifyWrite,
                op.into(),
                idx,
            ),
            Instruction::ZeroPageIndirectIdxRead(op, idx) => (
                InstructionSequenceMode::ZeroPageIndirectIdxRead,
                op.into(),
                idx,
            ),
            Instruction::ZeroPageIndirectIdxReadModifyWrite(op, idx) => (
                InstructionSequenceMode::ZeroPageIndirectIdxReadModifyWrite,
                op.into(),
                idx,
            ),
            Instruction::ZeroPageIndirectIdxWrite(op, idx) => (
                InstructionSequenceMode::ZeroPageIndirectIdxWrite,
                op.into(),
                idx,
            ),
            Instruction::AbsoluteIndirectJump => (
                InstructionSequenceMode::AbsoluteIndirectJump,
                InstructionOp::default(),
                IndexRegister::default(),
            ),
        }
    }
}

impl Instruction {
    pub fn new(
        mode: InstructionSequenceMode,
        operation: InstructionOp,
        index: Option<IndexRegister>,
    ) -> Self {
        match mode {
            InstructionSequenceMode::Immediate => {
                Instruction::Immediate(operation.try_into().unwrap())
            }
            InstructionSequenceMode::ZeroPage => {
                Instruction::ZeroPage(operation.try_into().unwrap())
            }
            InstructionSequenceMode::Absolute => {
                Instruction::Absolute(operation.try_into().unwrap())
            }
            InstructionSequenceMode::ZeroPageIdxIndirect => {
                Instruction::ZeroPageIdxIndirect(operation.try_into().unwrap(), index.unwrap())
            }
            InstructionSequenceMode::ZeroPageIdx => {
                Instruction::ZeroPageIdx(operation.try_into().unwrap(), index.unwrap())
            }
            InstructionSequenceMode::AbsoluteIdxRead => {
                Instruction::AbsoluteIdxRead(operation.try_into().unwrap(), index.unwrap())
            }
            InstructionSequenceMode::AbsoluteIdxWrite => {
                Instruction::AbsoluteIdxWrite(operation.try_into().unwrap(), index.unwrap())
            }
            InstructionSequenceMode::ZeroPageIndirectIdxRead => {
                Instruction::ZeroPageIndirectIdxRead(operation.try_into().unwrap(), index.unwrap())
            }
            InstructionSequenceMode::ZeroPageIndirectIdxWrite => {
                Instruction::ZeroPageIndirectIdxWrite(operation.try_into().unwrap(), index.unwrap())
            }
            InstructionSequenceMode::ZeroPageReadModifyWrite => {
                Instruction::ZeroPageReadModifyWrite(operation.try_into().unwrap())
            }
            InstructionSequenceMode::AbsoluteReadModifyWrite => {
                Instruction::AbsoluteReadModifyWrite(operation.try_into().unwrap())
            }
            InstructionSequenceMode::ZeroPageIdxReadModifyWrite => {
                Instruction::ZeroPageIdxReadModifyWrite(
                    operation.try_into().unwrap(),
                    index.unwrap(),
                )
            }
            InstructionSequenceMode::AbsoluteIdxReadModifyWrite => {
                Instruction::AbsoluteIdxReadModifyWrite(
                    operation.try_into().unwrap(),
                    index.unwrap(),
                )
            }
            InstructionSequenceMode::Implied => Instruction::Implied(operation.try_into().unwrap()),
            InstructionSequenceMode::AbsoluteJump => Instruction::AbsoluteJump,
            InstructionSequenceMode::AbsoluteIndirectJump => Instruction::AbsoluteIndirectJump,
            InstructionSequenceMode::Relative => {
                Instruction::Relative(operation.try_into().unwrap())
            }
            InstructionSequenceMode::Push => Instruction::Push(operation.try_into().unwrap()),
            InstructionSequenceMode::Pull => Instruction::Pull(operation.try_into().unwrap()),
            InstructionSequenceMode::Break => Instruction::Break,
            InstructionSequenceMode::JumpSubroutine => Instruction::JumpSubroutine,
            InstructionSequenceMode::ReturnInterrupt => Instruction::ReturnInterrupt,
            InstructionSequenceMode::ReturnSubroutine => Instruction::ReturnSubroutine,
            InstructionSequenceMode::FetchInstr => Instruction::FetchInstr,
            InstructionSequenceMode::StartIrq => Instruction::StartIrq,
            InstructionSequenceMode::StartNmi => Instruction::StartNmi,
            InstructionSequenceMode::Reset => Instruction::Reset,
            InstructionSequenceMode::ZeroPageIdxIndirectReadModifyWrite => {
                Instruction::ZeroPageIdxIndirectReadModifyWrite(
                    operation.try_into().unwrap(),
                    index.unwrap(),
                )
            }
            InstructionSequenceMode::ZeroPageIndirectIdxReadModifyWrite => {
                Instruction::ZeroPageIndirectIdxReadModifyWrite(
                    operation.try_into().unwrap(),
                    index.unwrap(),
                )
            }
        }
    }
}
