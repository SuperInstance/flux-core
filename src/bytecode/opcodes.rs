use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Op {
    NOP = 0x00, MOV = 0x01, LOAD = 0x02, STORE = 0x03,
    JMP = 0x04, JZ = 0x05, JNZ = 0x06, CALL = 0x07,
    IADD = 0x08, ISUB = 0x09, IMUL = 0x0A, IDIV = 0x0B,
    IMOD = 0x0C, INEG = 0x0D, INC = 0x0E, DEC = 0x0F,
    IAND = 0x10, IOR = 0x11, IXOR = 0x12, INOT = 0x13,
    ISHL = 0x14, ISHR = 0x15,
    PUSH = 0x20, POP = 0x21, DUP = 0x22,
    RET = 0x28,
    MOVI = 0x2B,
    CMP = 0x2D,
    FADD = 0x40, FSUB = 0x41, FMUL = 0x42, FDIV = 0x43,
    TELL = 0x60, ASK = 0x61, DELEGATE = 0x62, BROADCAST = 0x66,
    HALT = 0x80, YIELD = 0x81,
}

impl fmt::Display for Op {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Op {
    pub fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            0x00 => Some(Self::NOP), 0x01 => Some(Self::MOV),
            0x04 => Some(Self::JMP), 0x05 => Some(Self::JZ), 0x06 => Some(Self::JNZ),
            0x07 => Some(Self::CALL), 0x08 => Some(Self::IADD), 0x09 => Some(Self::ISUB),
            0x0A => Some(Self::IMUL), 0x0B => Some(Self::IDIV), 0x0C => Some(Self::IMOD),
            0x0D => Some(Self::INEG), 0x0E => Some(Self::INC), 0x0F => Some(Self::DEC),
            0x10 => Some(Self::IAND), 0x11 => Some(Self::IOR), 0x12 => Some(Self::IXOR),
            0x13 => Some(Self::INOT), 0x14 => Some(Self::ISHL), 0x15 => Some(Self::ISHR),
            0x20 => Some(Self::PUSH), 0x21 => Some(Self::POP), 0x22 => Some(Self::DUP),
            0x28 => Some(Self::RET), 0x2B => Some(Self::MOVI), 0x2D => Some(Self::CMP),
            0x40 => Some(Self::FADD), 0x41 => Some(Self::FSUB),
            0x42 => Some(Self::FMUL), 0x43 => Some(Self::FDIV),
            0x60 => Some(Self::TELL), 0x61 => Some(Self::ASK),
            0x62 => Some(Self::DELEGATE), 0x66 => Some(Self::BROADCAST),
            0x80 => Some(Self::HALT), 0x81 => Some(Self::YIELD),
            _ => None,
        }
    }
}
