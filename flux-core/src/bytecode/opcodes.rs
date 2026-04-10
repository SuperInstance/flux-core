//! Opcode definitions and instruction formats.

use crate::error::{Error, Result};
use core::fmt;

/// All FLUX VM opcodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Opcode {
    /// No operation
    NOP = 0x00,

    /// Move register to register
    MOV = 0x01,

    /// Load from memory address
    LOAD = 0x02,

    /// Store to memory address
    STORE = 0x03,

    /// Unconditional jump
    JMP = 0x04,

    /// Jump if zero
    JZ = 0x05,

    /// Jump if not zero
    JNZ = 0x06,

    /// Call subroutine
    CALL = 0x07,

    /// Integer addition
    IADD = 0x08,

    /// Integer subtraction
    ISUB = 0x09,

    /// Integer multiplication
    IMUL = 0x0A,

    /// Integer division
    IDIV = 0x0B,

    /// Integer modulo
    IMOD = 0x0C,

    /// Integer negation
    INEG = 0x0D,

    /// Increment register
    INC = 0x0E,

    /// Decrement register
    DEC = 0x0F,

    /// Integer bitwise AND
    IAND = 0x10,

    /// Integer bitwise OR
    IOR = 0x11,

    /// Integer bitwise XOR
    IXOR = 0x12,

    /// Integer bitwise NOT
    INOT = 0x13,

    /// Integer shift left
    ISHL = 0x14,

    /// Integer shift right
    ISHR = 0x15,

    /// Compare two registers
    CMP = 0x2D,

    /// Move immediate value
    MOVI = 0x2B,

    /// Push to stack
    PUSH = 0x20,

    /// Pop from stack
    POP = 0x21,

    /// Duplicate top of stack
    DUP = 0x22,

    /// Return from subroutine
    RET = 0x28,

    /// Float addition
    FADD = 0x40,

    /// Float subtraction
    FSUB = 0x41,

    /// Float multiplication
    FMUL = 0x42,

    /// Float division
    FDIV = 0x43,

    /// A2A Tell message
    TELL = 0x60,

    /// A2A Ask message
    ASK = 0x61,

    /// A2A Delegate message
    DELEGATE = 0x62,

    /// A2A Broadcast message
    BROADCAST = 0x66,

    /// Halt execution
    HALT = 0x80,

    /// Yield execution
    YIELD = 0x81,
}

impl Opcode {
    /// Create an Opcode from a u8.
    pub fn from_u8(value: u8) -> Result<Opcode> {
        match value {
            0x00 => Ok(Opcode::NOP),
            0x01 => Ok(Opcode::MOV),
            0x02 => Ok(Opcode::LOAD),
            0x03 => Ok(Opcode::STORE),
            0x04 => Ok(Opcode::JMP),
            0x05 => Ok(Opcode::JZ),
            0x06 => Ok(Opcode::JNZ),
            0x07 => Ok(Opcode::CALL),
            0x08 => Ok(Opcode::IADD),
            0x09 => Ok(Opcode::ISUB),
            0x0A => Ok(Opcode::IMUL),
            0x0B => Ok(Opcode::IDIV),
            0x0C => Ok(Opcode::IMOD),
            0x0D => Ok(Opcode::INEG),
            0x0E => Ok(Opcode::INC),
            0x0F => Ok(Opcode::DEC),
            0x10 => Ok(Opcode::IAND),
            0x11 => Ok(Opcode::IOR),
            0x12 => Ok(Opcode::IXOR),
            0x13 => Ok(Opcode::INOT),
            0x14 => Ok(Opcode::ISHL),
            0x15 => Ok(Opcode::ISHR),
            0x2D => Ok(Opcode::CMP),
            0x2B => Ok(Opcode::MOVI),
            0x20 => Ok(Opcode::PUSH),
            0x21 => Ok(Opcode::POP),
            0x22 => Ok(Opcode::DUP),
            0x28 => Ok(Opcode::RET),
            0x40 => Ok(Opcode::FADD),
            0x41 => Ok(Opcode::FSUB),
            0x42 => Ok(Opcode::FMUL),
            0x43 => Ok(Opcode::FDIV),
            0x60 => Ok(Opcode::TELL),
            0x61 => Ok(Opcode::ASK),
            0x62 => Ok(Opcode::DELEGATE),
            0x66 => Ok(Opcode::BROADCAST),
            0x80 => Ok(Opcode::HALT),
            0x81 => Ok(Opcode::YIELD),
            _ => Err(Error::InvalidOpcode(value)),
        }
    }

    /// Get the instruction format for this opcode.
    pub fn format(self) -> Format {
        match self {
            Opcode::NOP | Opcode::HALT | Opcode::DUP | Opcode::YIELD => Format::A,
            Opcode::INC | Opcode::DEC | Opcode::PUSH | Opcode::POP | Opcode::INEG => {
                Format::B
            }
            Opcode::CMP | Opcode::MOV | Opcode::LOAD | Opcode::STORE => Format::C,
            Opcode::MOVI | Opcode::JMP | Opcode::JZ | Opcode::JNZ | Opcode::CALL => Format::D,
            Opcode::IADD
            | Opcode::ISUB
            | Opcode::IMUL
            | Opcode::IDIV
            | Opcode::IMOD
            | Opcode::IAND
            | Opcode::IOR
            | Opcode::IXOR
            | Opcode::ISHL
            | Opcode::ISHR => Format::E,
            Opcode::FADD | Opcode::FSUB | Opcode::FMUL | Opcode::FDIV => Format::E,
            Opcode::TELL | Opcode::ASK | Opcode::DELEGATE | Opcode::BROADCAST => Format::G,
            Opcode::RET => Format::A,
            Opcode::INOT => Format::B,
        }
    }
}

impl fmt::Display for Opcode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Opcode::NOP => "NOP",
            Opcode::MOV => "MOV",
            Opcode::LOAD => "LOAD",
            Opcode::STORE => "STORE",
            Opcode::JMP => "JMP",
            Opcode::JZ => "JZ",
            Opcode::JNZ => "JNZ",
            Opcode::CALL => "CALL",
            Opcode::IADD => "IADD",
            Opcode::ISUB => "ISUB",
            Opcode::IMUL => "IMUL",
            Opcode::IDIV => "IDIV",
            Opcode::IMOD => "IMOD",
            Opcode::INEG => "INEG",
            Opcode::INC => "INC",
            Opcode::DEC => "DEC",
            Opcode::IAND => "IAND",
            Opcode::IOR => "IOR",
            Opcode::IXOR => "IXOR",
            Opcode::INOT => "INOT",
            Opcode::ISHL => "ISHL",
            Opcode::ISHR => "ISHR",
            Opcode::CMP => "CMP",
            Opcode::MOVI => "MOVI",
            Opcode::PUSH => "PUSH",
            Opcode::POP => "POP",
            Opcode::DUP => "DUP",
            Opcode::RET => "RET",
            Opcode::FADD => "FADD",
            Opcode::FSUB => "FSUB",
            Opcode::FMUL => "FMUL",
            Opcode::FDIV => "FDIV",
            Opcode::TELL => "TELL",
            Opcode::ASK => "ASK",
            Opcode::DELEGATE => "DELEGATE",
            Opcode::BROADCAST => "BROADCAST",
            Opcode::HALT => "HALT",
            Opcode::YIELD => "YIELD",
        };
        write!(f, "{}", s)
    }
}

/// Instruction format types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// Format A: [opcode] - 1 byte
    A,

    /// Format B: [opcode][rd] - 2 bytes
    B,

    /// Format C: [opcode][rd][rs1] - 3 bytes
    C,

    /// Format D: [opcode][rd][imm16] - 4 bytes (imm16 is i16)
    D,

    /// Format E: [opcode][rd][rs1][rs2] - 4 bytes
    E,

    /// Format G: [opcode][len:u16][data:len bytes] - variable
    G,
}

impl Format {
    /// Get the length of this format's fixed portion (excluding variable data).
    pub fn fixed_length(self) -> usize {
        match self {
            Format::A => 1,
            Format::B => 2,
            Format::C => 3,
            Format::D => 4,
            Format::E => 4,
            Format::G => 3, // opcode + 2-byte length
        }
    }

    /// Check if this format has a variable length portion.
    pub fn is_variable(self) -> bool {
        matches!(self, Format::G)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opcode_roundtrip() {
        for value in 0u8..=255 {
            let result = Opcode::from_u8(value);
            match value {
                0x00 | 0x01 | 0x02 | 0x03 | 0x04 | 0x05 | 0x06 | 0x07 | 0x08 | 0x09 | 0x0A
                | 0x0B | 0x0C | 0x0D | 0x0E | 0x0F | 0x10 | 0x11 | 0x12 | 0x13 | 0x14
                | 0x15 | 0x2D | 0x2B | 0x20 | 0x21 | 0x22 | 0x28 | 0x40 | 0x41 | 0x42
                | 0x43 | 0x60 | 0x61 | 0x62 | 0x66 | 0x80 | 0x81 => {
                    let op = result.unwrap();
                    assert_eq!(op as u8, value);
                }
                _ => assert!(result.is_err()),
            }
        }
    }

    #[test]
    fn test_format_lengths() {
        assert_eq!(Format::A.fixed_length(), 1);
        assert_eq!(Format::B.fixed_length(), 2);
        assert_eq!(Format::C.fixed_length(), 3);
        assert_eq!(Format::D.fixed_length(), 4);
        assert_eq!(Format::E.fixed_length(), 4);
        assert_eq!(Format::G.fixed_length(), 3);
    }

    #[test]
    fn test_opcode_formats() {
        assert_eq!(Opcode::NOP.format(), Format::A);
        assert_eq!(Opcode::MOVI.format(), Format::D);
        assert_eq!(Opcode::IADD.format(), Format::E);
        assert_eq!(Opcode::TELL.format(), Format::G);
    }

    #[test]
    fn test_opcode_display() {
        assert_eq!(format!("{}", Opcode::NOP), "NOP");
        assert_eq!(format!("{}", Opcode::MOVI), "MOVI");
        assert_eq!(format!("{}", Opcode::IADD), "IADD");
        assert_eq!(format!("{}", Opcode::HALT), "HALT");
    }
}
