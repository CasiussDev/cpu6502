//! Memory access logging wrapper for debugging purposes
//!
//! This module provides a wrapper around any `MemorySpace` implementation that
//! logs all memory read and write operations. This is useful for debugging and
//! tracing program execution in the 6502 emulator.
//!
//! The logging is enabled when the "logging" feature is activated.

use crate::MemorySpace;
use log::debug;

/// A wrapper around any memory implementation that logs all memory accesses
///
/// This struct takes ownership of a reference to any `MemorySpace` implementation
/// and forwards all memory operations to it while logging them. It's commonly used
/// during debugging to track memory reads and writes during CPU operation.
///
/// # Example
///
/// ```
/// # use cpu6502::MemorySpace;
/// # use cpu6502::cpu::logging_memory::LoggingMemory;
/// # struct DummyMemory;
/// # impl MemorySpace for DummyMemory {
/// #     fn read(&mut self, address: u16) -> u8 { 0 }
/// #     fn write(&mut self, address: u16, value: u8) {}
/// # }
/// # let mut memory = DummyMemory;
/// // Create a logging wrapper around the memory
/// let mut logging_memory = LoggingMemory::new(&mut memory);
///
/// // Now all reads and writes will be logged
/// let value = logging_memory.read(0x1000);
/// logging_memory.write(0x2000, 0x42);
/// ```
pub struct LoggingMemory<'a, T> {
    /// The wrapped memory implementation
    inner: &'a mut T,
}

impl<'a, T> LoggingMemory<'a, T>
where
    T: MemorySpace,
{
    pub fn new(inner: &'a mut T) -> Self {
        Self { inner }
    }
}

impl<'a, T> MemorySpace for LoggingMemory<'a, T>
where
    T: MemorySpace,
{
    fn read(&mut self, addr: u16) -> u8 {
        let data = self.inner.read(addr);
        debug!("\t\t\tRead Memory[{:04X}] = {:02X}", addr, data);
        data
    }

    fn write(&mut self, data: u8, addr: u16) {
        debug!("\t\t\tWrite Memory[{:04X}] = {:02X}", addr, data,);
        self.inner.write(data, addr);
    }

    fn read_array(&mut self, addr: u16, out: &mut [u8]) {
        self.inner.read_array(addr, out);
    }
}
