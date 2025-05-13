use std::convert::TryFrom;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub enum ImplicitOperation {
    #[default]
    /// No operation (NOP)
    Nop,

    /// Arithmetic shift left accumulator (ASL A)
    ShiftLeftA,

    /// Logical shift right accumulator (LSR A)
    ShiftRightA,

    /// Rotate left accumulator (ROL A)
    RotateLeftA,

    /// Rotate right accumulator (ROR A)
    RotateRightA,

    /// Increment X register (INY)
    IncrementX,

    /// Increment Y register (INY)
    IncrementY,

    /// Decrement X register (INY)
    DecrementX,

    /// Decrement Y register (DEY)
    DecrementY,

    /// Clear carry flag (CLC)
    ClearCarry,

    /// Set carry flag (SEC)
    SetCarry,

    /// Clear decimal mode flag (CLD)
    ClearDecimal,

    /// Set decimal mode flag (SED)
    SetDecimal,

    /// Clear interrupt disable flag (CLI)
    ClearInterruptDisable,

    /// Set interrupt disable flag (SEI)
    SetInterruptDisable,

    /// Clear overflow flag (CLV)
    ClearOverflow,

    /// Set overflow flag (not a native 6502 instruction, mostly for internal use)
    SetOverflow,

    /// Transfer accumulator to X register (TAX)
    TransferAccumulatorToX,

    /// Transfer accumulator to Y register (TAY)
    TransferAccumulatorToY,

    /// Transfer stack pointer to X register (TSX)
    TransferStackPtrToX,

    /// Transfer X register to accumulator (TXA)
    TransferXToAccumulator,

    /// Transfer Y register to accumulator (TYA)
    TransferYToAccumulator,

    /// Transfer X register to stack pointer (TXS)
    TransferXToStackPtr,
}

/// Represents conditional branch operations.
///
/// Branch operations perform conditional jumps based on the state of processor status flags.
/// If the condition is met, the program counter is adjusted by a signed 8-bit offset
/// (relative addressing). If the condition is not met, execution continues with the next
/// instruction.
///
/// Branch instructions have variable cycle timing:
/// - Base: 2 cycles if branch not taken
/// - +1 cycle if branch taken
/// - +1 additional cycle if branch crosses a page boundary
///
/// This variable timing is important for cycle-accurate emulation.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub enum BranchOperation {
    /// Branch if result plus (N=0) (BPL)
    #[default]
    BranchPlus,

    /// Branch if result minus (N=1) (BMI)
    BranchMinus,

    /// Branch if overflow clear (V=0) (BVC)
    BranchOverflowClear,

    /// Branch if overflow set (V=1) (BVS)
    BranchOverflowSet,

    /// Branch if carry clear (C=0) (BCC)
    BranchCarryClear,

    /// Branch if carry set (C=1) (BCS)
    BranchCarrySet,

    /// Branch if not equal (Z=0) (BNE)
    BranchNotEqual,

    /// Branch if equal (Z=1) (BEQ)
    BranchEqual,
}

/// Represents operations that modify memory locations directly.
///
/// These operations read a value from memory, modify it according to the specific
/// operation, and write the result back to the same memory location. This group
/// of operations is collectively known as "Read-Modify-Write" operations in 6502
/// terminology.
///
/// The 6502 implements these operations with specific timing characteristics that
/// affect cycle-accurate emulation and are important for certain hardware effects.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub enum MemoryModifyOperation {
    /// Increment value in memory (INC)
    #[default]
    IncrementMemory,

    /// Decrement value in memory (DEC)
    DecrementMemory,

    /// Arithmetic shift left memory value (ASL)
    ShiftLeftMemory,

    /// Logical shift right memory value (LSR)
    ShiftRightMemory,

    /// Rotate left memory value through carry (ROL)
    RotateLeftMemory,

    /// Rotate right memory value through carry (ROR)
    RotateRightMemory,
}

/// Represents operations that interact between CPU registers and memory.
///
/// This enum includes all operations that either:
/// - Load data from memory into a register
/// - Store data from a register into memory
/// - Perform ALU operations using a memory value as an operand
///
/// These operations form the core of the 6502's data manipulation capabilities,
/// allowing the CPU to move and process data between its internal registers and
/// external memory.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub enum RegisterMemoryOperation {
    /// Store accumulator value in memory (STA)
    #[default]
    StoreA,

    /// Load accumulator from memory (LDA)
    LoadA,

    /// Store X register value in memory (STX)
    StoreX,

    /// Load X register from memory (LDX)
    LoadX,

    /// Store Y register value in memory (STY)
    StoreY,

    /// Load Y register from memory (LDY)
    LoadY,

    /// Test bits in memory with accumulator (BIT)
    Bit,

    /// Test bits with immediate value (BIT #$nn - 65C02 extension)
    BitImmediate,

    /// Logical OR memory with accumulator (ORA)
    Or,

    /// Logical AND memory with accumulator (AND)
    And,

    /// Logical exclusive OR memory with accumulator (EOR)
    Xor,

    /// Add memory to accumulator with carry (ADC)
    Add,

    /// Subtract memory from accumulator with carry (SBC)
    Sub,

    /// Compare memory with accumulator (CMP)
    Cmp,

    /// Compare memory with X register (CPX)
    Cpx,

    /// Compare memory with Y register (CPY)
    Cpy,
}

/// Represents operations that push values onto the stack.
///
/// The 6502 has a hardware stack in page 1 of memory ($0100-$01FF) with the stack
/// pointer initially set to $FF and decrementing as values are pushed. These operations
/// push either the accumulator or the processor status register onto the stack.
///
/// Push operations are commonly used for:
/// - Subroutine calls (JSR pushes return address)
/// - Interrupt handling (pushes return address and processor status)
/// - Explicit stack manipulation by the program
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub enum PushStackOperation {
    /// Push accumulator onto stack (PHA)
    #[default]
    PushA,

    /// Push processor status onto stack (PHP)
    PushStatus,
}

/// Represents operations that pull values from the stack.
///
/// These operations retrieve values previously pushed onto the stack. The 6502 stack
/// pointer increments as values are pulled. These operations pull either the accumulator
/// or the processor status register from the stack.
///
/// Pull operations are commonly used for:
/// - Returning from subroutines (RTS pulls return address)
/// - Returning from interrupts (RTI pulls processor status and return address)
/// - Explicit stack manipulation by the program
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub enum PullStackOperation {
    /// Pull accumulator from stack (PLA)
    #[default]
    PullA,

    /// Pull processor status from stack (PLP)
    PullStatus,
}

/// Represents all possible operations that can be performed by the 6502 CPU.
///
/// This enum contains the complete set of operations used by the 6502 instruction set,
/// encompassing all types of operations:
/// - ALU operations (Add, Sub, And, Or, etc.)
/// - Data transfer operations (Load, Store)
/// - Register operations (Increment, Decrement, Transfer)
/// - Status flag operations (Set, Clear)
/// - Stack operations (Push, Pull)
/// - Shift and rotate operations
/// - Branch operations
///
/// This enum serves as a central point for mapping opcodes to their semantic operations,
/// and is used throughout the emulator to determine what action to perform when
/// processing each instruction.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Default)]
pub enum InstructionOp {
    /// No operation (NOP)
    #[default]
    Nop,

    /// Increment memory location (INC)
    IncrementMemory,

    /// Increment X register (INX)
    IncrementX,

    /// Increment Y register (INY)
    IncrementY,

    /// Decrement memory location (DEC)
    DecrementMemory,

    /// Decrement X register (DEX)
    DecrementX,

    /// Decrement Y register (DEY)
    DecrementY,

    /// Clear carry flag (CLC)
    ClearCarry,

    /// Set carry flag (SEC)
    SetCarry,

    /// Clear decimal mode flag (CLD)
    ClearDecimal,

    /// Set decimal mode flag (SED)
    SetDecimal,

    /// Clear interrupt disable flag (CLI)
    ClearInterruptDisable,

    /// Set interrupt disable flag (SEI)
    SetInterruptDisable,

    /// Clear overflow flag (CLV)
    ClearOverflow,

    /// Set overflow flag (not a native 6502 instruction, mostly for internal use)
    SetOverflow,

    /// Transfer accumulator to X register (TAX)
    TransferAccumulatorToX,

    /// Transfer accumulator to Y register (TAY)
    TransferAccumulatorToY,

    /// Transfer stack pointer to X register (TSX)
    TransferStackPtrToX,

    /// Transfer X register to accumulator (TXA)
    TransferXToAccumulator,

    /// Transfer Y register to accumulator (TYA)
    TransferYToAccumulator,

    /// Transfer X register to stack pointer (TXS)
    TransferXToStackPtr,

    /// Push accumulator onto stack (PHA)
    PushA,

    /// Push processor status onto stack (PHP)
    PushStatus,

    /// Pull accumulator from stack (PLA)
    PullA,

    /// Pull processor status from stack (PLP)
    PullStatus,

    /// Logical OR with accumulator (ORA)
    Or,

    /// Logical AND with accumulator (AND)
    And,

    /// Logical exclusive OR with accumulator (EOR)
    Xor,

    /// Add with carry (ADC)
    Add,

    /// Subtract with carry (SBC)
    Sub,

    /// Compare memory with accumulator (CMP)
    Cmp,

    /// Compare memory with X register (CPX)
    Cpx,

    /// Compare memory with Y register (CPY)
    Cpy,

    /// Test bits in memory with accumulator (BIT)
    Bit,

    /// Test bits with immediate value (BIT #$nn - 65C02 extension)
    BitImmediate,

    /// Arithmetic shift left accumulator (ASL A)
    ShiftLeftA,

    /// Logical shift right accumulator (LSR A)
    ShiftRightA,

    /// Rotate left accumulator (ROL A)
    RotateLeftA,

    /// Rotate right accumulator (ROR A)
    RotateRightA,

    /// Arithmetic shift left memory (ASL)
    ShiftLeftMemory,

    /// Logical shift right memory (LSR)
    ShiftRightMemory,

    /// Rotate left memory (ROL)
    RotateLeftMemory,

    /// Rotate right memory (ROR)
    RotateRightMemory,

    /// Store accumulator in memory (STA)
    StoreA,

    /// Load accumulator from memory (LDA)
    LoadA,

    /// Store X register in memory (STX)
    StoreX,

    /// Load X register from memory (LDX)
    LoadX,

    /// Store Y register in memory (STY)
    StoreY,

    /// Load Y register from memory (LDY)
    LoadY,

    /// Branch on plus (BPL)
    BranchPlus,

    /// Branch on minus (BMI)
    BranchMinus,

    /// Branch on overflow clear (BVC)
    BranchOverflowClear,

    /// Branch on overflow set (BVS)
    BranchOverflowSet,

    /// Branch on carry clear (BCC)
    BranchCarryClear,

    /// Branch on carry set (BCS)
    BranchCarrySet,

    /// Branch on not equal (BNE)
    BranchNotEqual,

    /// Branch on equal (BEQ)
    BranchEqual,
}

impl From<ImplicitOperation> for InstructionOp {
    fn from(op: ImplicitOperation) -> Self {
        match op {
            ImplicitOperation::Nop => InstructionOp::Nop,
            ImplicitOperation::ShiftLeftA => InstructionOp::ShiftLeftA,
            ImplicitOperation::ShiftRightA => InstructionOp::ShiftRightA,
            ImplicitOperation::RotateLeftA => InstructionOp::RotateLeftA,
            ImplicitOperation::RotateRightA => InstructionOp::RotateRightA,
            ImplicitOperation::IncrementX => InstructionOp::IncrementX,
            ImplicitOperation::IncrementY => InstructionOp::IncrementY,
            ImplicitOperation::DecrementX => InstructionOp::DecrementX,
            ImplicitOperation::DecrementY => InstructionOp::DecrementY,
            ImplicitOperation::ClearCarry => InstructionOp::ClearCarry,
            ImplicitOperation::SetCarry => InstructionOp::SetCarry,
            ImplicitOperation::ClearDecimal => InstructionOp::ClearDecimal,
            ImplicitOperation::SetDecimal => InstructionOp::SetDecimal,
            ImplicitOperation::ClearInterruptDisable => InstructionOp::ClearInterruptDisable,
            ImplicitOperation::SetInterruptDisable => InstructionOp::SetInterruptDisable,
            ImplicitOperation::ClearOverflow => InstructionOp::ClearOverflow,
            ImplicitOperation::SetOverflow => InstructionOp::SetOverflow,
            ImplicitOperation::TransferAccumulatorToX => InstructionOp::TransferAccumulatorToX,
            ImplicitOperation::TransferAccumulatorToY => InstructionOp::TransferAccumulatorToY,
            ImplicitOperation::TransferStackPtrToX => InstructionOp::TransferStackPtrToX,
            ImplicitOperation::TransferXToAccumulator => InstructionOp::TransferXToAccumulator,
            ImplicitOperation::TransferYToAccumulator => InstructionOp::TransferYToAccumulator,
            ImplicitOperation::TransferXToStackPtr => InstructionOp::TransferXToStackPtr,
        }
    }
}

impl From<BranchOperation> for InstructionOp {
    fn from(op: BranchOperation) -> Self {
        match op {
            BranchOperation::BranchPlus => InstructionOp::BranchPlus,
            BranchOperation::BranchMinus => InstructionOp::BranchMinus,
            BranchOperation::BranchOverflowClear => InstructionOp::BranchOverflowClear,
            BranchOperation::BranchOverflowSet => InstructionOp::BranchOverflowSet,
            BranchOperation::BranchCarryClear => InstructionOp::BranchCarryClear,
            BranchOperation::BranchCarrySet => InstructionOp::BranchCarrySet,
            BranchOperation::BranchNotEqual => InstructionOp::BranchNotEqual,
            BranchOperation::BranchEqual => InstructionOp::BranchEqual,
        }
    }
}

impl From<MemoryModifyOperation> for InstructionOp {
    fn from(op: MemoryModifyOperation) -> Self {
        match op {
            MemoryModifyOperation::IncrementMemory => InstructionOp::IncrementMemory,
            MemoryModifyOperation::DecrementMemory => InstructionOp::DecrementMemory,
            MemoryModifyOperation::ShiftLeftMemory => InstructionOp::ShiftLeftMemory,
            MemoryModifyOperation::ShiftRightMemory => InstructionOp::ShiftRightMemory,
            MemoryModifyOperation::RotateLeftMemory => InstructionOp::RotateLeftMemory,
            MemoryModifyOperation::RotateRightMemory => InstructionOp::RotateRightMemory,
        }
    }
}

impl From<RegisterMemoryOperation> for InstructionOp {
    fn from(op: RegisterMemoryOperation) -> Self {
        match op {
            RegisterMemoryOperation::StoreA => InstructionOp::StoreA,
            RegisterMemoryOperation::LoadA => InstructionOp::LoadA,
            RegisterMemoryOperation::StoreX => InstructionOp::StoreX,
            RegisterMemoryOperation::LoadX => InstructionOp::LoadX,
            RegisterMemoryOperation::StoreY => InstructionOp::StoreY,
            RegisterMemoryOperation::LoadY => InstructionOp::LoadY,
            RegisterMemoryOperation::Bit => InstructionOp::Bit,
            RegisterMemoryOperation::BitImmediate => InstructionOp::BitImmediate,
            RegisterMemoryOperation::Or => InstructionOp::Or,
            RegisterMemoryOperation::And => InstructionOp::And,
            RegisterMemoryOperation::Xor => InstructionOp::Xor,
            RegisterMemoryOperation::Add => InstructionOp::Add,
            RegisterMemoryOperation::Sub => InstructionOp::Sub,
            RegisterMemoryOperation::Cmp => InstructionOp::Cmp,
            RegisterMemoryOperation::Cpx => InstructionOp::Cpx,
            RegisterMemoryOperation::Cpy => InstructionOp::Cpy,
        }
    }
}

impl From<PushStackOperation> for InstructionOp {
    fn from(op: PushStackOperation) -> Self {
        match op {
            PushStackOperation::PushA => InstructionOp::PushA,
            PushStackOperation::PushStatus => InstructionOp::PushStatus,
        }
    }
}

impl From<PullStackOperation> for InstructionOp {
    fn from(op: PullStackOperation) -> Self {
        match op {
            PullStackOperation::PullA => InstructionOp::PullA,
            PullStackOperation::PullStatus => InstructionOp::PullStatus,
        }
    }
}

impl TryFrom<InstructionOp> for ImplicitOperation {
    type Error = &'static str;
    fn try_from(op: InstructionOp) -> Result<Self, Self::Error> {
        match op {
            InstructionOp::Nop => Ok(ImplicitOperation::Nop),
            InstructionOp::ShiftLeftA => Ok(ImplicitOperation::ShiftLeftA),
            InstructionOp::ShiftRightA => Ok(ImplicitOperation::ShiftRightA),
            InstructionOp::RotateLeftA => Ok(ImplicitOperation::RotateLeftA),
            InstructionOp::RotateRightA => Ok(ImplicitOperation::RotateRightA),
            InstructionOp::IncrementX => Ok(ImplicitOperation::IncrementX),
            InstructionOp::IncrementY => Ok(ImplicitOperation::IncrementY),
            InstructionOp::DecrementX => Ok(ImplicitOperation::DecrementX),
            InstructionOp::DecrementY => Ok(ImplicitOperation::DecrementY),
            InstructionOp::ClearCarry => Ok(ImplicitOperation::ClearCarry),
            InstructionOp::SetCarry => Ok(ImplicitOperation::SetCarry),
            InstructionOp::ClearDecimal => Ok(ImplicitOperation::ClearDecimal),
            InstructionOp::SetDecimal => Ok(ImplicitOperation::SetDecimal),
            InstructionOp::ClearInterruptDisable => Ok(ImplicitOperation::ClearInterruptDisable),
            InstructionOp::SetInterruptDisable => Ok(ImplicitOperation::SetInterruptDisable),
            InstructionOp::ClearOverflow => Ok(ImplicitOperation::ClearOverflow),
            InstructionOp::SetOverflow => Ok(ImplicitOperation::SetOverflow),
            InstructionOp::TransferAccumulatorToX => Ok(ImplicitOperation::TransferAccumulatorToX),
            InstructionOp::TransferAccumulatorToY => Ok(ImplicitOperation::TransferAccumulatorToY),
            InstructionOp::TransferStackPtrToX => Ok(ImplicitOperation::TransferStackPtrToX),
            InstructionOp::TransferXToAccumulator => Ok(ImplicitOperation::TransferXToAccumulator),
            InstructionOp::TransferYToAccumulator => Ok(ImplicitOperation::TransferYToAccumulator),
            InstructionOp::TransferXToStackPtr => Ok(ImplicitOperation::TransferXToStackPtr),
            _ => Err("No matching ImplicitOperation"),
        }
    }
}

impl TryFrom<InstructionOp> for BranchOperation {
    type Error = &'static str;
    fn try_from(op: InstructionOp) -> Result<Self, Self::Error> {
        match op {
            InstructionOp::BranchPlus => Ok(BranchOperation::BranchPlus),
            InstructionOp::BranchMinus => Ok(BranchOperation::BranchMinus),
            InstructionOp::BranchOverflowClear => Ok(BranchOperation::BranchOverflowClear),
            InstructionOp::BranchOverflowSet => Ok(BranchOperation::BranchOverflowSet),
            InstructionOp::BranchCarryClear => Ok(BranchOperation::BranchCarryClear),
            InstructionOp::BranchCarrySet => Ok(BranchOperation::BranchCarrySet),
            InstructionOp::BranchNotEqual => Ok(BranchOperation::BranchNotEqual),
            InstructionOp::BranchEqual => Ok(BranchOperation::BranchEqual),
            _ => Err("No matching BranchOperation"),
        }
    }
}

impl TryFrom<InstructionOp> for MemoryModifyOperation {
    type Error = &'static str;
    fn try_from(op: InstructionOp) -> Result<Self, Self::Error> {
        match op {
            InstructionOp::IncrementMemory => Ok(MemoryModifyOperation::IncrementMemory),
            InstructionOp::DecrementMemory => Ok(MemoryModifyOperation::DecrementMemory),
            InstructionOp::ShiftLeftMemory => Ok(MemoryModifyOperation::ShiftLeftMemory),
            InstructionOp::ShiftRightMemory => Ok(MemoryModifyOperation::ShiftRightMemory),
            InstructionOp::RotateLeftMemory => Ok(MemoryModifyOperation::RotateLeftMemory),
            InstructionOp::RotateRightMemory => Ok(MemoryModifyOperation::RotateRightMemory),
            _ => Err("No matching MemoryModifyOperation"),
        }
    }
}

impl TryFrom<InstructionOp> for RegisterMemoryOperation {
    type Error = &'static str;
    fn try_from(op: InstructionOp) -> Result<Self, Self::Error> {
        match op {
            InstructionOp::StoreA => Ok(RegisterMemoryOperation::StoreA),
            InstructionOp::LoadA => Ok(RegisterMemoryOperation::LoadA),
            InstructionOp::StoreX => Ok(RegisterMemoryOperation::StoreX),
            InstructionOp::LoadX => Ok(RegisterMemoryOperation::LoadX),
            InstructionOp::StoreY => Ok(RegisterMemoryOperation::StoreY),
            InstructionOp::LoadY => Ok(RegisterMemoryOperation::LoadY),
            InstructionOp::Bit => Ok(RegisterMemoryOperation::Bit),
            InstructionOp::BitImmediate => Ok(RegisterMemoryOperation::BitImmediate),
            InstructionOp::Or => Ok(RegisterMemoryOperation::Or),
            InstructionOp::And => Ok(RegisterMemoryOperation::And),
            InstructionOp::Xor => Ok(RegisterMemoryOperation::Xor),
            InstructionOp::Add => Ok(RegisterMemoryOperation::Add),
            InstructionOp::Sub => Ok(RegisterMemoryOperation::Sub),
            InstructionOp::Cmp => Ok(RegisterMemoryOperation::Cmp),
            InstructionOp::Cpx => Ok(RegisterMemoryOperation::Cpx),
            InstructionOp::Cpy => Ok(RegisterMemoryOperation::Cpy),
            _ => Err("No matching RegisterMemoryOperation"),
        }
    }
}

impl TryFrom<InstructionOp> for PushStackOperation {
    type Error = &'static str;
    fn try_from(op: InstructionOp) -> Result<Self, Self::Error> {
        match op {
            InstructionOp::PushA => Ok(PushStackOperation::PushA),
            InstructionOp::PushStatus => Ok(PushStackOperation::PushStatus),
            _ => Err("No matching PushStackOperation"),
        }
    }
}

impl TryFrom<InstructionOp> for PullStackOperation {
    type Error = &'static str;
    fn try_from(op: InstructionOp) -> Result<Self, Self::Error> {
        match op {
            InstructionOp::PullA => Ok(PullStackOperation::PullA),
            InstructionOp::PullStatus => Ok(PullStackOperation::PullStatus),
            _ => Err("No matching PullStackOperation"),
        }
    }
}
