//! Memory access logging wrapper for debugging purposes
//!
//! This module provides a wrapper around any `MemorySpace` implementation that
//! logs all memory read and write operations. This is useful for debugging and
//! tracing program execution in the 6502 emulator.
//!
//! The logging is enabled when the "logging" feature is activated.

use crate::MemorySpace;
use log::trace;

/// A wrapper around any memory implementation that logs all memory accesses
///
/// This struct takes ownership of a reference to any `MemorySpace` implementation
/// and forwards all memory operations to it while logging them. It's commonly used
/// during debugging to track memory reads and writes during CPU operation.
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
        trace!("\t\t\tRead Memory[{:04X}] = {:02X}", addr, data);
        data
    }

    fn write(&mut self, data: u8, addr: u16) {
        trace!("\t\t\tWrite Memory[{:04X}] = {:02X}", addr, data,);
        self.inner.write(data, addr);
    }

    fn read_array(&mut self, addr: u16, out: &mut [u8]) {
        self.inner.read_array(addr, out);
        for (i, &data) in out.iter().enumerate() {
            trace!("\t\t\tRead Array Memory[{:04X}] = {:02X}", addr.wrapping_add(i as u16), data);
        }
    }
}
