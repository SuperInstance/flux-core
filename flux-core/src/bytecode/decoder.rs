//! Bytecode disassembler (bytecode → text representation).

use crate::bytecode::opcodes::{Format, Opcode};
use crate::error::{Error, Result};
use core::fmt;

/// Disassembler that converts bytecode to human-readable text.
///
/// # Examples
///
/// ```
/// use flux_core::bytecode::Disassembler;
///
/// let disasm = Disassembler::new();
/// let bytecode = vec![0x2B, 0x00, 0x2A, 0x00, 0x80];
/// let text = disasm.disassemble(&bytecode).unwrap();
/// assert!(text.contains("MOVI"));
/// assert!(text.contains("HALT"));
/// ```
#[derive(Debug, Clone)]
pub struct Disassembler {
    /// Show instruction addresses in output
    show_addresses: bool,

    /// Show raw bytes in output
    show_bytes: bool,
}

impl Default for Disassembler {
    fn default() -> Self {
        Self::new()
    }
}

impl Disassembler {
    /// Create a new disassembler with default settings.
    pub fn new() -> Self {
        Self {
            show_addresses: true,
            show_bytes: true,
        }
    }

    /// Create a new disassembler that shows only mnemonics.
    pub fn minimal() -> Self {
        Self {
            show_addresses: false,
            show_bytes: false,
        }
    }

    /// Set whether to show instruction addresses.
    pub fn with_addresses(mut self, show: bool) -> Self {
        self.show_addresses = show;
        self
    }

    /// Set whether to show raw bytes.
    pub fn with_bytes(mut self, show: bool) -> Self {
        self.show_bytes = show;
        self
    }

    /// Disassemble bytecode to text.
    ///
    /// # Errors
    ///
    /// Returns an error if the bytecode is invalid.
    pub fn disassemble(&self, bytecode: &[u8]) -> Result<String> {
        let mut output = String::new();
        let mut pc = 0;

        while pc < bytecode.len() {
            // Add address if requested
            if self.show_addresses {
                output.push_str(&format!("{:04X}: ", pc));
            }

            // Disassemble one instruction
            let (instr, len) = self.disassemble_one(&bytecode[pc..])?;

            // Add bytes if requested
            if self.show_bytes {
                for i in 0..len.max(4) {
                    if i < bytecode.len() - pc {
                        output.push_str(&format!("{:02X} ", bytecode[pc + i]));
                    } else {
                        output.push_str("   ");
                    }
                }
                // Padding for alignment
                if len < 4 {
                    for _ in 0..(4 - len) {
                        output.push_str("   ");
                    }
                }
            }

            output.push_str(&instr);
            output.push('\n');

            pc += len;
        }

        Ok(output)
    }

    /// Disassemble a single instruction.
    ///
    /// Returns (instruction_text, instruction_length).
    pub fn disassemble_one(&self, bytecode: &[u8]) -> Result<(String, usize)> {
        if bytecode.is_empty() {
            return Err(Error::UnexpectedEndOfBytecode);
        }

        let opcode_val = bytecode[0];
        let opcode = Opcode::from_u8(opcode_val)?;
        let format = opcode.format();

        // Ensure we have enough bytes for the instruction format
        if bytecode.len() < format.fixed_length() {
            return Err(Error::UnexpectedEndOfBytecode);
        }

        let instr = match format {
            Format::A => self.disassemble_format_a(opcode),
            Format::B => self.disassemble_format_b(opcode, bytecode)?,
            Format::C => self.disassemble_format_c(opcode, bytecode)?,
            Format::D => self.disassemble_format_d(opcode, bytecode)?,
            Format::E => self.disassemble_format_e(opcode, bytecode)?,
            Format::G => self.disassemble_format_g(opcode, bytecode)?,
        };

        Ok((instr, format.fixed_length()))
    }

    /// Disassemble Format A instruction: [opcode]
    fn disassemble_format_a(&self, opcode: Opcode) -> String {
        format!("{}", opcode)
    }

    /// Disassemble Format B instruction: [opcode][rd]
    fn disassemble_format_b(&self, opcode: Opcode, bytecode: &[u8]) -> Result<String> {
        let rd = bytecode[1];
        Ok(format!("{} R{}", opcode, rd))
    }

    /// Disassemble Format C instruction: [opcode][rd][rs1]
    fn disassemble_format_c(&self, opcode: Opcode, bytecode: &[u8]) -> Result<String> {
        let rd = bytecode[1];
        let rs1 = bytecode[2];
        Ok(format!("{} R{}, R{}", opcode, rd, rs1))
    }

    /// Disassemble Format D instruction: [opcode][rd][imm16]
    fn disassemble_format_d(&self, opcode: Opcode, bytecode: &[u8]) -> Result<String> {
        let rd = bytecode[1];
        let imm = i16::from_le_bytes([bytecode[2], bytecode[3]]);

        match opcode {
            Opcode::MOVI => Ok(format!("{} R{}, {}", opcode, rd, imm)),
            Opcode::JMP | Opcode::JZ | Opcode::JNZ | Opcode::CALL => {
                Ok(format!("{} {}", opcode, imm))
            }
            _ => Ok(format!("{} R{}, {}", opcode, rd, imm)),
        }
    }

    /// Disassemble Format E instruction: [opcode][rd][rs1][rs2]
    fn disassemble_format_e(&self, opcode: Opcode, bytecode: &[u8]) -> Result<String> {
        let rd = bytecode[1];
        let rs1 = bytecode[2];
        let rs2 = bytecode[3];

        // Use appropriate register prefix for float ops
        let reg_prefix = matches!(opcode, Opcode::FADD | Opcode::FSUB | Opcode::FMUL | Opcode::FDIV);

        if reg_prefix {
            Ok(format!("{} F{}, F{}, F{}", opcode, rd, rs1, rs2))
        } else {
            Ok(format!("{} R{}, R{}, R{}", opcode, rd, rs1, rs2))
        }
    }

    /// Disassemble Format G instruction: [opcode][len:u16][data:len bytes]
    fn disassemble_format_g(&self, opcode: Opcode, bytecode: &[u8]) -> Result<String> {
        let len = u16::from_le_bytes([bytecode[1], bytecode[2]]) as usize;
        let total_len = 3 + len;

        if bytecode.len() < total_len {
            return Err(Error::UnexpectedEndOfBytecode);
        }

        let data = &bytecode[3..total_len];

        // Show a preview of the data
        let preview = if data.len() <= 8 {
            format!("{:?}", data)
        } else {
            format!("{} bytes", data.len())
        };

        Ok(format!("{} [{}]", opcode, preview))
    }

    /// Get a list of all instruction addresses in the bytecode.
    ///
    /// Useful for building control flow graphs or analyzing jumps.
    pub fn get_instruction_boundaries(&self, bytecode: &[u8]) -> Result<Vec<usize>> {
        let mut boundaries = Vec::new();
        let mut pc = 0;

        while pc < bytecode.len() {
            let opcode_val = bytecode[pc];
            let opcode = Opcode::from_u8(opcode_val)?;
            let format = opcode.format();

            let fixed_len = format.fixed_length();

            if bytecode.len() < pc + fixed_len {
                return Err(Error::UnexpectedEndOfBytecode);
            }

            let instr_len = if format.is_variable() {
                let len = u16::from_le_bytes([bytecode[pc + 1], bytecode[pc + 2]]) as usize;
                3 + len
            } else {
                fixed_len
            };

            boundaries.push(pc);
            pc += instr_len;
        }

        Ok(boundaries)
    }

    /// Disassemble a single instruction at a specific address.
    pub fn disassemble_at(&self, bytecode: &[u8], addr: usize) -> Result<String> {
        if addr >= bytecode.len() {
            return Err(Error::InvalidAddress(addr as u32));
        }

        let (instr, _) = self.disassemble_one(&bytecode[addr..])?;
        Ok(instr)
    }
}

impl fmt::Display for Disassembler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Disassembler(show_addresses={}, show_bytes={})",
            self.show_addresses, self.show_bytes
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disassemble_nop() {
        let disasm = Disassembler::new();
        let bytecode = vec![0x00];
        let result = disasm.disassemble(&bytecode).unwrap();
        assert!(result.contains("NOP"));
    }

    #[test]
    fn test_disassemble_halt() {
        let disasm = Disassembler::new();
        let bytecode = vec![0x80];
        let (instr, len) = disasm.disassemble_one(&bytecode).unwrap();
        assert_eq!(instr, "HALT");
        assert_eq!(len, 1);
    }

    #[test]
    fn test_disassemble_movi() {
        let disasm = Disassembler::new();
        let bytecode = vec![0x2B, 0x00, 0x2A, 0x00]; // MOVI R0, 42
        let (instr, len) = disasm.disassemble_one(&bytecode).unwrap();
        assert_eq!(instr, "MOVI R0, 42");
        assert_eq!(len, 4);
    }

    #[test]
    fn test_disassemble_movi_negative() {
        let disasm = Disassembler::new();
        let bytecode = vec![0x2B, 0x05, 0x9C, 0xFF]; // MOVI R5, -100
        let (instr, len) = disasm.disassemble_one(&bytecode).unwrap();
        assert_eq!(instr, "MOVI R5, -100");
    }

    #[test]
    fn test_disassemble_iadd() {
        let disasm = Disassembler::new();
        let bytecode = vec![0x08, 0x00, 0x01, 0x02]; // IADD R0, R1, R2
        let (instr, len) = disasm.disassemble_one(&bytecode).unwrap();
        assert_eq!(instr, "IADD R0, R1, R2");
        assert_eq!(len, 4);
    }

    #[test]
    fn test_disassemble_program() {
        let disasm = Disassembler::new();
        let bytecode = vec![
            0x2B, 0x00, 0x0A, 0x00, // MOVI R0, 10
            0x2B, 0x01, 0x14, 0x00, // MOVI R1, 20
            0x08, 0x00, 0x01, 0x02, // IADD R0, R1, R2
            0x80,                   // HALT
        ];

        let result = disasm.disassemble(&bytecode).unwrap();
        assert!(result.contains("MOVI R0, 10"));
        assert!(result.contains("MOVI R1, 20"));
        assert!(result.contains("IADD R0, R1, R2"));
        assert!(result.contains("HALT"));
    }

    #[test]
    fn test_disassemble_push_pop() {
        let disasm = Disassembler::new();
        let bytecode = vec![0x20, 0x00]; // PUSH R0
        let (instr, len) = disasm.disassemble_one(&bytecode).unwrap();
        assert_eq!(instr, "PUSH R0");
        assert_eq!(len, 2);

        let bytecode = vec![0x21, 0x01]; // POP R1
        let (instr, len) = disasm.disassemble_one(&bytecode).unwrap();
        assert_eq!(instr, "POP R1");
        assert_eq!(len, 2);
    }

    #[test]
    fn test_disassemble_inc_dec() {
        let disasm = Disassembler::new();
        let bytecode = vec![0x0E, 0x05]; // INC R5
        let (instr, _) = disasm.disassemble_one(&bytecode).unwrap();
        assert_eq!(instr, "INC R5");

        let bytecode = vec![0x0F, 0x0A]; // DEC R10
        let (instr, _) = disasm.disassemble_one(&bytecode).unwrap();
        assert_eq!(instr, "DEC R10");
    }

    #[test]
    fn test_disassemble_cmp() {
        let disasm = Disassembler::new();
        let bytecode = vec![0x2D, 0x00, 0x01]; // CMP R0, R1
        let (instr, len) = disasm.disassemble_one(&bytecode).unwrap();
        assert_eq!(instr, "CMP R0, R1");
        assert_eq!(len, 3);
    }

    #[test]
    fn test_disassemble_jmp() {
        let disasm = Disassembler::new();
        let bytecode = vec![0x04, 0x00, 0x64, 0x00]; // JMP 100
        let (instr, len) = disasm.disassemble_one(&bytecode).unwrap();
        assert_eq!(instr, "JMP 100");
        assert_eq!(len, 4);
    }

    #[test]
    fn test_disassemble_jz_negative() {
        let disasm = Disassembler::new();
        let bytecode = vec![0x05, 0x00, 0xF6, 0xFF]; // JZ -10
        let (instr, _) = disasm.disassemble_one(&bytecode).unwrap();
        assert_eq!(instr, "JZ -10");
    }

    #[test]
    fn test_disassemble_fadd() {
        let disasm = Disassembler::new();
        let bytecode = vec![0x40, 0x00, 0x01, 0x02]; // FADD F0, F1, F2
        let (instr, len) = disasm.disassemble_one(&bytecode).unwrap();
        assert_eq!(instr, "FADD F0, F1, F2");
        assert_eq!(len, 4);
    }

    #[test]
    fn test_minimal_mode() {
        let disasm = Disassembler::minimal();
        let bytecode = vec![0x2B, 0x00, 0x2A, 0x00];
        let result = disasm.disassemble(&bytecode).unwrap();

        // Should not contain addresses or bytes
        assert!(!result.contains(":"));
        assert!(result.contains("MOVI R0, 42"));
    }

    #[test]
    fn test_get_instruction_boundaries() {
        let disasm = Disassembler::new();
        let bytecode = vec![
            0x00,                   // NOP (1 byte)
            0x2B, 0x00, 0x2A, 0x00, // MOVI (4 bytes)
            0x80,                   // HALT (1 byte)
        ];

        let boundaries = disasm.get_instruction_boundaries(&bytecode).unwrap();
        assert_eq!(boundaries, vec![0, 1, 5, 6]);
    }

    #[test]
    fn test_disassemble_at() {
        let disasm = Disassembler::new();
        let bytecode = vec![
            0x00,                   // NOP
            0x2B, 0x00, 0x2A, 0x00, // MOVI R0, 42
            0x80,                   // HALT
        ];

        let instr = disasm.disassemble_at(&bytecode, 1).unwrap();
        assert_eq!(instr, "MOVI R0, 42");

        let instr = disasm.disassemble_at(&bytecode, 5).unwrap();
        assert_eq!(instr, "HALT");
    }

    #[test]
    fn test_invalid_opcode() {
        let disasm = Disassembler::new();
        let bytecode = vec![0xFF]; // Invalid opcode
        assert!(disasm.disassemble(&bytecode).is_err());
    }

    #[test]
    fn test_incomplete_instruction() {
        let disasm = Disassembler::new();
        let bytecode = vec![0x2B, 0x00]; // Incomplete MOVI
        assert!(disasm.disassemble(&bytecode).is_err());
    }

    #[test]
    fn test_roundtrip_with_assembler() {
        use crate::bytecode::Assembler;

        let original_asm = "MOVI R0, 42
                           MOVI R1, 20
                           IADD R0, R1, R2
                           HALT";

        let mut asm = Assembler::new();
        let bytecode = asm.assemble(original_asm).unwrap();

        let disasm = Disassembler::minimal();
        let recovered_asm = disasm.disassemble(&bytecode).unwrap();

        // Check that the key instructions are present
        assert!(recovered_asm.contains("MOVI R0, 42"));
        assert!(recovered_asm.contains("MOVI R1, 20"));
        assert!(recovered_asm.contains("IADD"));
        assert!(recovered_asm.contains("HALT"));
    }
}
