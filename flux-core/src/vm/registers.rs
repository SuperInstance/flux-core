//! Register file for the FLUX VM.

use crate::error::{Error, Result};
use core::fmt;

/// Number of general-purpose registers.
pub const NUM_GP_REGS: usize = 16;

/// Number of floating-point registers.
pub const NUM_FP_REGS: usize = 16;

/// Number of SIMD vector registers.
pub const NUM_SIMD_REGS: usize = 16;

/// Size of SIMD registers in bits.
pub const SIMD_BITS: usize = 128;

/// SIMD register element representation.
///
/// SIMD registers store 128-bit vectors, which can be viewed as:
/// - 16 x i8/u8 bytes
/// - 8 x i16/u16 shorts
/// - 4 x i32/u32 ints
/// - 2 x i64/u64 longs
/// - 4 x f32 floats
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SIMDElement {
    /// 8-bit signed integer
    I8(i8),

    /// 8-bit unsigned integer
    U8(u8),

    /// 16-bit signed integer
    I16(i16),

    /// 16-bit unsigned integer
    U16(u16),

    /// 32-bit signed integer
    I32(i32),

    /// 32-bit unsigned integer
    U32(u32),

    /// 64-bit signed integer
    I64(i64),

    /// 64-bit unsigned integer
    U64(u64),

    /// 32-bit float
    F32(f32),
}

impl fmt::Display for SIMDElement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SIMDElement::I8(v) => write!(f, "i8({})", v),
            SIMDElement::U8(v) => write!(f, "u8({})", v),
            SIMDElement::I16(v) => write!(f, "i16({})", v),
            SIMDElement::U16(v) => write!(f, "u16({})", v),
            SIMDElement::I32(v) => write!(f, "i32({})", v),
            SIMDElement::U32(v) => write!(f, "u32({})", v),
            SIMDElement::I64(v) => write!(f, "i64({})", v),
            SIMDElement::U64(v) => write!(f, "u64({})", v),
            SIMDElement::F32(v) => write!(f, "f32({})", v),
        }
    }
}

/// A 128-bit SIMD register.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SIMDRegister {
    bytes: [u8; 16],
}

impl SIMDRegister {
    /// Create a new zero-initialized SIMD register.
    pub fn zero() -> Self {
        Self { bytes: [0; 16] }
    }

    /// Create a SIMD register from raw bytes.
    pub fn from_bytes(bytes: [u8; 16]) -> Self {
        Self { bytes }
    }

    /// Get the raw bytes of this register.
    pub fn to_bytes(self) -> [u8; 16] {
        self.bytes
    }

    /// Extract an i8 value at the given lane (0-15).
    pub fn get_i8(self, lane: usize) -> Result<i8> {
        if lane >= 16 {
            return Err(Error::InvalidAddress(lane as u32));
        }
        Ok(self.bytes[lane] as i8)
    }

    /// Extract a u8 value at the given lane (0-15).
    pub fn get_u8(self, lane: usize) -> Result<u8> {
        if lane >= 16 {
            return Err(Error::InvalidAddress(lane as u32));
        }
        Ok(self.bytes[lane])
    }

    /// Extract an i16 value at the given lane (0-7).
    pub fn get_i16(self, lane: usize) -> Result<i16> {
        if lane >= 8 {
            return Err(Error::InvalidAddress(lane as u32));
        }
        let base = lane * 2;
        Ok(i16::from_le_bytes([self.bytes[base], self.bytes[base + 1]]))
    }

    /// Extract a u16 value at the given lane (0-7).
    pub fn get_u16(self, lane: usize) -> Result<u16> {
        if lane >= 8 {
            return Err(Error::InvalidAddress(lane as u32));
        }
        let base = lane * 2;
        Ok(u16::from_le_bytes([self.bytes[base], self.bytes[base + 1]]))
    }

    /// Extract an i32 value at the given lane (0-3).
    pub fn get_i32(self, lane: usize) -> Result<i32> {
        if lane >= 4 {
            return Err(Error::InvalidAddress(lane as u32));
        }
        let base = lane * 4;
        Ok(i32::from_le_bytes([
            self.bytes[base],
            self.bytes[base + 1],
            self.bytes[base + 2],
            self.bytes[base + 3],
        ]))
    }

    /// Extract a u32 value at the given lane (0-3).
    pub fn get_u32(self, lane: usize) -> Result<u32> {
        if lane >= 4 {
            return Err(Error::InvalidAddress(lane as u32));
        }
        let base = lane * 4;
        Ok(u32::from_le_bytes([
            self.bytes[base],
            self.bytes[base + 1],
            self.bytes[base + 2],
            self.bytes[base + 3],
        ]))
    }

    /// Extract an f32 value at the given lane (0-3).
    pub fn get_f32(self, lane: usize) -> Result<f32> {
        if lane >= 4 {
            return Err(Error::InvalidAddress(lane as u32));
        }
        let base = lane * 4;
        Ok(f32::from_le_bytes([
            self.bytes[base],
            self.bytes[base + 1],
            self.bytes[base + 2],
            self.bytes[base + 3],
        ]))
    }

    /// Set an i8 value at the given lane.
    pub fn set_i8(&mut self, lane: usize, value: i8) -> Result<()> {
        if lane >= 16 {
            return Err(Error::InvalidAddress(lane as u32));
        }
        self.bytes[lane] = value as u8;
        Ok(())
    }

    /// Set a u8 value at the given lane.
    pub fn set_u8(&mut self, lane: usize, value: u8) -> Result<()> {
        if lane >= 16 {
            return Err(Error::InvalidAddress(lane as u32));
        }
        self.bytes[lane] = value;
        Ok(())
    }

    /// Set an i32 value at the given lane.
    pub fn set_i32(&mut self, lane: usize, value: i32) -> Result<()> {
        if lane >= 4 {
            return Err(Error::InvalidAddress(lane as u32));
        }
        let base = lane * 4;
        let bytes = value.to_le_bytes();
        self.bytes[base..base + 4].copy_from_slice(&bytes);
        Ok(())
    }

    /// Set a u32 value at the given lane.
    pub fn set_u32(&mut self, lane: usize, value: u32) -> Result<()> {
        if lane >= 4 {
            return Err(Error::InvalidAddress(lane as u32));
        }
        let base = lane * 4;
        let bytes = value.to_le_bytes();
        self.bytes[base..base + 4].copy_from_slice(&bytes);
        Ok(())
    }

    /// Set an f32 value at the given lane.
    pub fn set_f32(&mut self, lane: usize, value: f32) -> Result<()> {
        if lane >= 4 {
            return Err(Error::InvalidAddress(lane as u32));
        }
        let base = lane * 4;
        let bytes = value.to_le_bytes();
        self.bytes[base..base + 4].copy_from_slice(&bytes);
        Ok(())
    }
}

impl fmt::Display for SIMDRegister {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SIMD[")?;
        for i in 0..4 {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", self.get_i32(i).unwrap_or(0))?;
        }
        write!(f, "]")
    }
}

/// CPU flags register.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Flags {
    /// Zero flag (set when result is zero)
    pub zero: bool,

    /// Sign flag (set when result is negative)
    pub sign: bool,

    /// Carry flag (set on unsigned overflow)
    pub carry: bool,
}

impl Flags {
    /// Create a new flags register with all flags cleared.
    pub fn new() -> Self {
        Self::default()
    }

    /// Update flags based on an integer result.
    pub fn update_from_i32(&mut self, value: i32) {
        self.zero = value == 0;
        self.sign = value < 0;
        self.carry = false; // Carry is set explicitly by instructions
    }

    /// Reset all flags to false.
    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

/// The complete register file for the FLUX VM.
///
/// Contains:
/// - 16 general-purpose 32-bit integer registers (R0-R15)
/// - 16 64-bit floating-point registers (F0-F15)
/// - 16 128-bit SIMD vector registers (V0-V15)
/// - Special registers: PC, SP, FP, LR
/// - Flags register
#[derive(Debug, Clone, PartialEq)]
pub struct RegisterFile {
    /// General-purpose registers (R0-R15)
    pub gp: [i32; NUM_GP_REGS],

    /// Floating-point registers (F0-F15)
    pub fp: [f64; NUM_FP_REGS],

    /// SIMD vector registers (V0-V15)
    pub simd: [SIMDRegister; NUM_SIMD_REGS],

    /// Program counter (instruction address)
    pub pc: u32,

    /// Stack pointer
    pub sp: u32,

    /// Frame pointer
    pub fp_reg: u32,

    /// Link register (return address)
    pub lr: u32,

    /// CPU flags
    pub flags: Flags,
}

impl Default for RegisterFile {
    fn default() -> Self {
        Self::new()
    }
}

impl RegisterFile {
    /// Create a new register file with all registers initialized to zero.
    pub fn new() -> Self {
        Self {
            gp: [0; NUM_GP_REGS],
            fp: [0.0; NUM_FP_REGS],
            simd: [SIMDRegister::zero(); NUM_SIMD_REGS],
            pc: 0,
            sp: 0,
            fp_reg: 0,
            lr: 0,
            flags: Flags::new(),
        }
    }

    /// Validate a register index for general-purpose registers.
    pub fn validate_gp_reg(reg: u8) -> Result<()> {
        if reg < NUM_GP_REGS as u8 {
            Ok(())
        } else {
            Err(Error::InvalidRegister(reg))
        }
    }

    /// Validate a register index for floating-point registers.
    pub fn validate_fp_reg(reg: u8) -> Result<()> {
        if reg < NUM_FP_REGS as u8 {
            Ok(())
        } else {
            Err(Error::InvalidRegister(reg))
        }
    }

    /// Validate a register index for SIMD registers.
    pub fn validate_simd_reg(reg: u8) -> Result<()> {
        if reg < NUM_SIMD_REGS as u8 {
            Ok(())
        } else {
            Err(Error::InvalidRegister(reg))
        }
    }

    /// Get a general-purpose register value.
    pub fn get_gp(&self, reg: u8) -> Result<i32> {
        Self::validate_gp_reg(reg)?;
        Ok(self.gp[reg as usize])
    }

    /// Set a general-purpose register value.
    pub fn set_gp(&mut self, reg: u8, value: i32) -> Result<()> {
        Self::validate_gp_reg(reg)?;
        self.gp[reg as usize] = value;
        Ok(())
    }

    /// Get a floating-point register value.
    pub fn get_fp(&self, reg: u8) -> Result<f64> {
        Self::validate_fp_reg(reg)?;
        Ok(self.fp[reg as usize])
    }

    /// Set a floating-point register value.
    pub fn set_fp(&mut self, reg: u8, value: f64) -> Result<()> {
        Self::validate_fp_reg(reg)?;
        self.fp[reg as usize] = value;
        Ok(())
    }

    /// Get a SIMD register.
    pub fn get_simd(&self, reg: u8) -> Result<SIMDRegister> {
        Self::validate_simd_reg(reg)?;
        Ok(self.simd[reg as usize])
    }

    /// Set a SIMD register.
    pub fn set_simd(&mut self, reg: u8, value: SIMDRegister) -> Result<()> {
        Self::validate_simd_reg(reg)?;
        self.simd[reg as usize] = value;
        Ok(())
    }

    /// Reset all registers to their initial state.
    pub fn reset(&mut self) {
        *self = Self::new();
    }

    /// Display the current state of key registers.
    pub fn display_state(&self) -> String {
        let mut s = String::from("Register State:\n");

        // Show first 8 GP registers
        s.push_str("  GP: ");
        for i in 0..8 {
            s.push_str(&format!("R{}={} ", i, self.gp[i]));
        }
        s.push('\n');

        // Show special registers
        s.push_str(&format!("  PC={} SP={} FP={} LR={} ", self.pc, self.sp, self.fp_reg, self.lr));
        s.push_str(&format!("Z={} S={} C={}\n", self.flags.zero, self.flags.sign, self.flags.carry));

        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_validation() {
        assert!(RegisterFile::validate_gp_reg(0).is_ok());
        assert!(RegisterFile::validate_gp_reg(15).is_ok());
        assert!(RegisterFile::validate_gp_reg(16).is_err());
    }

    #[test]
    fn test_gp_register_access() {
        let mut regs = RegisterFile::new();

        regs.set_gp(0, 42).unwrap();
        assert_eq!(regs.get_gp(0).unwrap(), 42);

        regs.set_gp(15, -100).unwrap();
        assert_eq!(regs.get_gp(15).unwrap(), -100);
    }

    #[test]
    fn test_fp_register_access() {
        let mut regs = RegisterFile::new();

        regs.set_fp(0, 3.14).unwrap();
        assert_eq!(regs.get_fp(0).unwrap(), 3.14);

        regs.set_fp(15, -2.71).unwrap();
        assert_eq!(regs.get_fp(15).unwrap(), -2.71);
    }

    #[test]
    fn test_flags() {
        let mut flags = Flags::new();
        assert!(!flags.zero && !flags.sign && !flags.carry);

        flags.update_from_i32(0);
        assert!(flags.zero);
        assert!(!flags.sign);

        flags.update_from_i32(-5);
        assert!(!flags.zero);
        assert!(flags.sign);

        flags.reset();
        assert!(!flags.zero && !flags.sign && !flags.carry);
    }

    #[test]
    fn test_simd_register() {
        let mut simd = SIMDRegister::zero();

        simd.set_i32(0, 42).unwrap();
        simd.set_i32(1, -10).unwrap();
        simd.set_i32(2, 100).unwrap();
        simd.set_i32(3, 0).unwrap();

        assert_eq!(simd.get_i32(0).unwrap(), 42);
        assert_eq!(simd.get_i32(1).unwrap(), -10);
        assert_eq!(simd.get_i32(2).unwrap(), 100);
        assert_eq!(simd.get_i32(3).unwrap(), 0);

        // Test f32 access
        simd.set_f32(0, 3.14).unwrap();
        assert!((simd.get_f32(0).unwrap() - 3.14).abs() < 0.001);
    }

    #[test]
    fn test_simd_bytes_roundtrip() {
        let original = [1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        let simd = SIMDRegister::from_bytes(original);
        let recovered = simd.to_bytes();
        assert_eq!(original, recovered);
    }

    #[test]
    fn test_simd_lane_access() {
        let mut simd = SIMDRegister::zero();

        // Test u8 lanes
        for i in 0..16 {
            simd.set_u8(i, i as u8).unwrap();
        }
        for i in 0..16 {
            assert_eq!(simd.get_u8(i).unwrap(), i as u8);
        }

        // Test i16 lanes
        for i in 0..8 {
            simd.set_i16(i, (i * 100) as i16).unwrap();
        }
        for i in 0..8 {
            assert_eq!(simd.get_i16(i).unwrap(), (i * 100) as i16);
        }
    }

    #[test]
    fn test_register_reset() {
        let mut regs = RegisterFile::new();
        regs.set_gp(0, 999).unwrap();
        regs.pc = 1000;
        regs.sp = 2000;
        regs.flags.zero = true;

        regs.reset();

        assert_eq!(regs.get_gp(0).unwrap(), 0);
        assert_eq!(regs.pc, 0);
        assert_eq!(regs.sp, 0);
        assert!(!regs.flags.zero);
    }
}
