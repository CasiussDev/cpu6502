/// Represents an addressable memory space for the 6502 processor emulation.
///
/// This trait defines the interface for any memory implementation that can be used
/// with the 6502 CPU. It supports the basic read and write operations that the CPU
/// performs during instruction execution, providing a clean abstraction over the
/// actual memory implementation.
///
/// Implementations of this trait can represent:
/// - Basic RAM (simple [u8; 64K] arrays)
/// - Memory-mapped hardware devices
/// - ROM regions
/// - Memory with banking or paging capabilities
/// - Special memory regions with side effects when accessed
///
/// The memory space is accessed using 16-bit addresses (0x0000-0xFFFF),
/// which is the full address range of the 6502 processor.
pub trait MemorySpace {
    /// Reads a single byte from memory at the specified address.
    ///
    /// # Arguments
    ///
    /// * `addr` - A 16-bit memory address to read from
    ///
    /// # Returns
    ///
    /// The byte value at the specified memory address
    fn read(&mut self, addr: u16) -> u8;

    /// Writes a single byte to memory at the specified address.
    ///
    /// # Arguments
    ///
    /// * `data` - The byte value to write
    /// * `addr` - A 16-bit memory address to write to
    fn write(&mut self, data: u8, addr: u16);

    /// Reads multiple consecutive bytes from memory into a buffer.
    ///
    /// This method provides a default implementation that calls `read()`
    /// for each byte, but implementations can override it with more
    /// efficient implementations when bulk reading is supported.
    ///
    /// # Arguments
    ///
    /// * `addr` - The starting memory address to read from
    /// * `out` - The buffer to store the read bytes
    fn read_array(&mut self, addr: u16, out: &mut [u8]) {
        for (i, dst) in out.iter_mut().enumerate() {
            *dst = self.read(addr.saturating_add(i as u16));
        }
    }
}

pub(crate) const MEMORY_64K: usize = (u16::MAX as usize) + 1;
pub(crate) type BasicRam = [u8; MEMORY_64K];

impl MemorySpace for BasicRam {
    fn read(&mut self, addr: u16) -> u8 {
        self[addr as usize]
    }
    fn write(&mut self, data: u8, addr: u16) {
        self[addr as usize] = data;
    }

    fn read_array(&mut self, addr: u16, out: &mut [u8]) {
        let start = addr as usize;
        out.clone_from_slice(&self[start..(start + out.len())]);
    }
}

pub fn new_basic_ram() -> BasicRam {
    [0; MEMORY_64K]
}
