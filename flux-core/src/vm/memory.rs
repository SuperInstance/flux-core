//! Linear memory management for the FLUX VM.

use crate::error::{Error, Result};

/// Default memory size in bytes (64KB).
pub const DEFAULT_MEMORY_SIZE: usize = 64 * 1024;

/// Page size in bytes (4KB).
pub const PAGE_SIZE: usize = 4 * 1024;

/// Maximum memory size in bytes (16MB).
pub const MAX_MEMORY_SIZE: usize = 16 * 1024 * 1024;

/// Linear memory for the FLUX VM.
///
/// Memory is organized as a contiguous byte array with separate regions:
/// - Code segment: Read-only, stores bytecode
/// - Data segment: Read-write, stores global data
/// - Stack segment: Grows downward from high memory
/// - Heap segment: Grows upward from end of data segment
#[derive(Debug, Clone, PartialEq)]
pub struct Memory {
    /// The actual memory contents
    data: Vec<u8>,

    /// Size of memory in bytes
    size: usize,

    /// Start of code segment (usually 0)
    code_start: u32,

    /// End of code segment / start of data segment
    data_start: u32,

    /// Stack pointer (top of stack, grows downward)
    stack_bottom: u32,
}

impl Memory {
    /// Create new memory with the default size (64KB).
    pub fn new() -> Self {
        Self::with_size(DEFAULT_MEMORY_SIZE)
    }

    /// Create new memory with a custom size.
    ///
    /// # Panics
    ///
    /// Panics if size is larger than MAX_MEMORY_SIZE or not page-aligned.
    pub fn with_size(size: usize) -> Self {
        assert!(size <= MAX_MEMORY_SIZE, "Memory size exceeds maximum");
        assert!(size % PAGE_SIZE == 0, "Memory size must be page-aligned");

        let mut mem = Self {
            data: vec![0u8; size],
            size,
            code_start: 0,
            data_start: 0,
            stack_bottom: size as u32,
        };

        // Default layout: bottom 1/4 for code, next 1/4 for data/heap
        mem.data_start = (size / 4) as u32;

        mem
    }

    /// Get the memory size.
    pub fn size(&self) -> usize {
        self.size
    }

    /// Load bytecode into the code segment.
    ///
    /// # Errors
    ///
    /// Returns an error if the bytecode is too large for the code segment.
    pub fn load_code(&mut self, bytecode: &[u8]) -> Result<()> {
        let code_end = self.data_start as usize;
        if bytecode.len() > code_end {
            return Err(Error::InvalidAddress(code_end as u32));
        }

        self.data[0..bytecode.len()].copy_from_slice(bytecode);
        Ok(())
    }

    /// Read a byte from memory.
    pub fn read_u8(&self, addr: u32) -> Result<u8> {
        self.validate_address(addr, 1)?;
        Ok(self.data[addr as usize])
    }

    /// Read a 16-bit value from memory (little-endian).
    pub fn read_u16(&self, addr: u32) -> Result<u16> {
        self.validate_address(addr, 2)?;
        let bytes = [self.data[addr as usize], self.data[addr as usize + 1]];
        Ok(u16::from_le_bytes(bytes))
    }

    /// Read a 32-bit value from memory (little-endian).
    pub fn read_u32(&self, addr: u32) -> Result<u32> {
        self.validate_address(addr, 4)?;
        let bytes = [
            self.data[addr as usize],
            self.data[addr as usize + 1],
            self.data[addr as usize + 2],
            self.data[addr as usize + 3],
        ];
        Ok(u32::from_le_bytes(bytes))
    }

    /// Read a 64-bit value from memory (little-endian).
    pub fn read_u64(&self, addr: u32) -> Result<u64> {
        self.validate_address(addr, 8)?;
        let bytes = [
            self.data[addr as usize],
            self.data[addr as usize + 1],
            self.data[addr as usize + 2],
            self.data[addr as usize + 3],
            self.data[addr as usize + 4],
            self.data[addr as usize + 5],
            self.data[addr as usize + 6],
            self.data[addr as usize + 7],
        ];
        Ok(u64::from_le_bytes(bytes))
    }

    /// Read a signed 32-bit value from memory (little-endian).
    pub fn read_i32(&self, addr: u32) -> Result<i32> {
        self.validate_address(addr, 4)?;
        let bytes = [
            self.data[addr as usize],
            self.data[addr as usize + 1],
            self.data[addr as usize + 2],
            self.data[addr as usize + 3],
        ];
        Ok(i32::from_le_bytes(bytes))
    }

    /// Read a 64-bit float from memory (little-endian).
    pub fn read_f64(&self, addr: u32) -> Result<f64> {
        self.validate_address(addr, 8)?;
        let bytes = [
            self.data[addr as usize],
            self.data[addr as usize + 1],
            self.data[addr as usize + 2],
            self.data[addr as usize + 3],
            self.data[addr as usize + 4],
            self.data[addr as usize + 5],
            self.data[addr as usize + 6],
            self.data[addr as usize + 7],
        ];
        Ok(f64::from_le_bytes(bytes))
    }

    /// Read a slice of bytes from memory.
    pub fn read_slice(&self, addr: u32, len: usize) -> Result<Vec<u8>> {
        self.validate_address(addr, len)?;
        Ok(self.data[addr as usize..addr as usize + len].to_vec())
    }

    /// Write a byte to memory.
    pub fn write_u8(&mut self, addr: u32, value: u8) -> Result<()> {
        self.validate_address(addr, 1)?;
        self.data[addr as usize] = value;
        Ok(())
    }

    /// Write a 16-bit value to memory (little-endian).
    pub fn write_u16(&mut self, addr: u32, value: u16) -> Result<()> {
        self.validate_address(addr, 2)?;
        let bytes = value.to_le_bytes();
        self.data[addr as usize..addr as usize + 2].copy_from_slice(&bytes);
        Ok(())
    }

    /// Write a 32-bit value to memory (little-endian).
    pub fn write_u32(&mut self, addr: u32, value: u32) -> Result<()> {
        self.validate_address(addr, 4)?;
        let bytes = value.to_le_bytes();
        self.data[addr as usize..addr as usize + 4].copy_from_slice(&bytes);
        Ok(())
    }

    /// Write a 64-bit value to memory (little-endian).
    pub fn write_u64(&mut self, addr: u32, value: u64) -> Result<()> {
        self.validate_address(addr, 8)?;
        let bytes = value.to_le_bytes();
        self.data[addr as usize..addr as usize + 8].copy_from_slice(&bytes);
        Ok(())
    }

    /// Write a signed 32-bit value to memory (little-endian).
    pub fn write_i32(&mut self, addr: u32, value: i32) -> Result<()> {
        self.validate_address(addr, 4)?;
        let bytes = value.to_le_bytes();
        self.data[addr as usize..addr as usize + 4].copy_from_slice(&bytes);
        Ok(())
    }

    /// Write a 64-bit float to memory (little-endian).
    pub fn write_f64(&mut self, addr: u32, value: f64) -> Result<()> {
        self.validate_address(addr, 8)?;
        let bytes = value.to_le_bytes();
        self.data[addr as usize..addr as usize + 8].copy_from_slice(&bytes);
        Ok(())
    }

    /// Write a slice of bytes to memory.
    pub fn write_slice(&mut self, addr: u32, data: &[u8]) -> Result<()> {
        self.validate_address(addr, data.len())?;
        self.data[addr as usize..addr as usize + data.len()].copy_from_slice(data);
        Ok(())
    }

    /// Get a reference to the raw memory contents.
    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }

    /// Get a mutable reference to the raw memory contents.
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.data
    }

    /// Validate that an address range is within memory bounds.
    fn validate_address(&self, addr: u32, size: usize) -> Result<()> {
        let end = addr as usize + size;
        if end > self.size {
            Err(Error::InvalidAddress(addr))
        } else {
            Ok(())
        }
    }

    /// Zero out all memory.
    pub fn clear(&mut self) {
        self.data.fill(0);
    }

    /// Get the stack bottom (highest address in memory).
    pub fn stack_bottom(&self) -> u32 {
        self.stack_bottom
    }

    /// Get the data segment start address.
    pub fn data_start(&self) -> u32 {
        self.data_start
    }

    /// Get the code segment end address.
    pub fn code_end(&self) -> u32 {
        self.data_start
    }

    /// Create a memory dump for debugging.
    pub fn dump(&self, start: u32, len: usize) -> String {
        let mut output = String::new();

        for i in 0..len {
            if i % 16 == 0 {
                if i > 0 {
                    output.push('\n');
                }
                output.push_str(&format!("{:08X}: ", start + i as u32));
            }

            if let Ok(byte) = self.read_u8(start + i as u32) {
                output.push_str(&format!("{:02X} ", byte));
            } else {
                output.push_str("?? ");
            }
        }

        output
    }
}

impl Default for Memory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_creation() {
        let mem = Memory::new();
        assert_eq!(mem.size(), DEFAULT_MEMORY_SIZE);
        assert_eq!(mem.data_start(), (DEFAULT_MEMORY_SIZE / 4) as u32);
    }

    #[test]
    fn test_read_write_u8() {
        let mut mem = Memory::new();
        mem.write_u8(100, 0xAB).unwrap();
        assert_eq!(mem.read_u8(100).unwrap(), 0xAB);
    }

    #[test]
    fn test_read_write_u32() {
        let mut mem = Memory::new();
        mem.write_u32(1000, 0xDEADBEEF).unwrap();
        assert_eq!(mem.read_u32(1000).unwrap(), 0xDEADBEEF);
    }

    #[test]
    fn test_read_write_i32() {
        let mut mem = Memory::new();
        mem.write_i32(500, -12345).unwrap();
        assert_eq!(mem.read_i32(500).unwrap(), -12345);
    }

    #[test]
    fn test_read_write_u64() {
        let mut mem = Memory::new();
        mem.write_u64(2000, 0x123456789ABCDEF0).unwrap();
        assert_eq!(mem.read_u64(2000).unwrap(), 0x123456789ABCDEF0);
    }

    #[test]
    fn test_read_write_f64() {
        let mut mem = Memory::new();
        mem.write_f64(3000, 3.14159265359).unwrap();
        let val = mem.read_f64(3000).unwrap();
        assert!((val - 3.14159265359).abs() < 0.0000001);
    }

    #[test]
    fn test_read_write_slice() {
        let mut mem = Memory::new();
        let data = vec![1, 2, 3, 4, 5];
        mem.write_slice(1000, &data).unwrap();
        assert_eq!(mem.read_slice(1000, 5).unwrap(), data);
    }

    #[test]
    fn test_invalid_address() {
        let mem = Memory::new();
        assert!(mem.read_u8(mem.size() as u32).is_err());
        assert!(mem.write_u32(mem.size() as u32 - 1, 0).is_err());
    }

    #[test]
    fn test_load_code() {
        let mut mem = Memory::new();
        let code = vec![0x00, 0x01, 0x02, 0x03];
        mem.load_code(&code).unwrap();

        assert_eq!(mem.read_u8(0).unwrap(), 0x00);
        assert_eq!(mem.read_u8(1).unwrap(), 0x01);
        assert_eq!(mem.read_u8(2).unwrap(), 0x02);
        assert_eq!(mem.read_u8(3).unwrap(), 0x03);
    }

    #[test]
    fn test_memory_clear() {
        let mut mem = Memory::new();
        mem.write_u32(100, 0x12345678).unwrap();
        mem.write_u32(200, 0xABCDEF00).unwrap();

        mem.clear();

        assert_eq!(mem.read_u32(100).unwrap(), 0);
        assert_eq!(mem.read_u32(200).unwrap(), 0);
    }

    #[test]
    fn test_dump() {
        let mut mem = Memory::new();
        mem.write_u8(0x1000, 0xAA).unwrap();
        mem.write_u8(0x1001, 0xBB).unwrap();
        mem.write_u8(0x1002, 0xCC).unwrap();

        let dump = mem.dump(0x1000, 16);
        assert!(dump.contains("AA"));
        assert!(dump.contains("BB"));
        assert!(dump.contains("CC"));
    }

    #[test]
    fn test_endianness() {
        let mut mem = Memory::new();
        mem.write_u32(100, 0x12345678).unwrap();

        // Little-endian: 78 56 34 12
        assert_eq!(mem.read_u8(100).unwrap(), 0x78);
        assert_eq!(mem.read_u8(101).unwrap(), 0x56);
        assert_eq!(mem.read_u8(102).unwrap(), 0x34);
        assert_eq!(mem.read_u8(103).unwrap(), 0x12);
    }
}
