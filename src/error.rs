use std::fmt;

#[derive(Debug, Clone)]
pub enum FluxError {
    InvalidOpcode(u8),
    InvalidRegister(u8),
    TruncatedInstruction { opcode: u8, expected: usize, got: usize },
    StackOverflow,
    StackUnderflow,
    DivisionByZero,
    CycleBudgetExceeded(u64),
    InvalidBytecode(String),
}

impl fmt::Display for FluxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidOpcode(op) => write!(f, "Invalid opcode: 0x{:02X}", op),
            Self::InvalidRegister(r) => write!(f, "Invalid register: R{}", r),
            Self::TruncatedInstruction { opcode, expected, got } =>
                write!(f, "Truncated instruction 0x{:02X}: expected {} bytes, got {}", opcode, expected, got),
            Self::StackOverflow => write!(f, "Stack overflow"),
            Self::StackUnderflow => write!(f, "Stack underflow"),
            Self::DivisionByZero => write!(f, "Division by zero"),
            Self::CycleBudgetExceeded(budget) => write!(f, "Cycle budget exceeded: {}", budget),
            Self::InvalidBytecode(msg) => write!(f, "Invalid bytecode: {}", msg),
        }
    }
}

impl std::error::Error for FluxError {}
