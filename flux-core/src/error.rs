//! Error types for the FLUX VM.

use core::fmt;

/// Result type alias for FLUX VM operations.
pub type Result<T> = core::result::Result<T, Error>;

/// Errors that can occur during VM execution or bytecode operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// Invalid opcode encountered
    InvalidOpcode(u8),

    /// Invalid register identifier
    InvalidRegister(u8),

    /// Invalid memory address
    InvalidAddress(u32),

    /// Stack overflow
    StackOverflow,

    /// Stack underflow
    StackUnderflow,

    /// Division by zero
    DivisionByZero,

    /// Invalid instruction format
    InvalidInstruction,

    /// Unexpected end of bytecode
    UnexpectedEndOfBytecode,

    /// Parse error in assembler
    ParseError,

    /// Unknown mnemonic in assembler
    UnknownMnemonic,

    /// Invalid immediate value
    InvalidImmediate,

    /// Halt instruction executed
    Halted,

    /// Invalid A2A message type
    InvalidMessageType(u8),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidOpcode(op) => write!(f, "Invalid opcode: 0x{:02X}", op),
            Error::InvalidRegister(reg) => write!(f, "Invalid register: {}", reg),
            Error::InvalidAddress(addr) => write!(f, "Invalid memory address: 0x{:08X}", addr),
            Error::StackOverflow => write!(f, "Stack overflow"),
            Error::StackUnderflow => write!(f, "Stack underflow"),
            Error::DivisionByZero => write!(f, "Division by zero"),
            Error::InvalidInstruction => write!(f, "Invalid instruction format"),
            Error::UnexpectedEndOfBytecode => write!(f, "Unexpected end of bytecode"),
            Error::ParseError => write!(f, "Parse error"),
            Error::UnknownMemonic => write!(f, "Unknown mnemonic"),
            Error::InvalidImmediate => write!(f, "Invalid immediate value"),
            Error::Halted => write!(f, "VM halted"),
            Error::InvalidMessageType(ty) => write!(f, "Invalid message type: {}", ty),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}
