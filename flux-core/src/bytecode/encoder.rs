//! Bytecode assembler (text representation → bytecode).

use crate::bytecode::opcodes::Opcode;
use crate::error::{Error, Result};
use core::str::FromStr;

/// Assembler that converts text assembly to bytecode.
///
/// # Syntax
///
/// Each instruction is on its own line with the format:
/// ```text
/// MNEMONIC operand1, operand2, ...
/// ```
///
/// Examples:
/// ```text
/// MOVI R0, 42
/// IADD R0, R1
/// HALT
/// ```
///
/// Registers are specified as R0-R15 for integer registers and F0-F15 for float registers.
/// Immediate values can be decimal (42) or hexadecimal (0x2A).
///
/// # Comments
///
/// Comments start with `;` or `//` and continue to the end of the line.
///
/// # Examples
///
/// ```
/// use flux_core::bytecode::Assembler;
///
/// let asm = Assembler::new();
/// let bytecode = asm.assemble("MOVI R0, 42\nHALT").unwrap();
/// assert_eq!(bytecode, vec![0x2B, 0x00, 0x2A, 0x00, 0x80]);
/// ```
#[derive(Debug, Clone)]
pub struct Assembler {
    /// Current label definitions
    labels: core::collections::HashMap<String, usize>,
}

impl Default for Assembler {
    fn default() -> Self {
        Self::new()
    }
}

impl Assembler {
    /// Create a new assembler.
    pub fn new() -> Self {
        Self {
            labels: core::collections::HashMap::new(),
        }
    }

    /// Assemble text assembly into bytecode.
    ///
    /// # Errors
    ///
    /// Returns an error if the assembly syntax is invalid.
    pub fn assemble(&mut self, input: &str) -> Result<Vec<u8>> {
        let mut bytecode = Vec::new();
        let mut first_pass = true;

        // Two-pass assembly: first pass collects labels, second pass generates code
        for pass in 0..2 {
            bytecode.clear();
            first_pass = pass == 0;

            for (line_num, line) in input.lines().enumerate() {
                let line_num = line_num + 1;

                // Strip comments
                let line = self.strip_comments(line);

                // Strip whitespace
                let line = line.trim();

                // Skip empty lines
                if line.is_empty() {
                    continue;
                }

                // Check for label definition
                if let Some(label) = self.parse_label_definition(line) {
                    if first_pass {
                        let addr = bytecode.len();
                        self.labels.insert(label, addr);
                    }
                    continue;
                }

                // Parse and encode instruction
                let instr_bytes = self.encode_instruction(line, line_num)?;
                bytecode.extend_from_slice(&instr_bytes);
            }

            if first_pass {
                // Prepare for second pass
                bytecode.clear();
            }
        }

        Ok(bytecode)
    }

    /// Strip comments from a line.
    fn strip_comments(&self, line: &str) -> &str {
        if let Some(pos) = line.find(';') {
            &line[..pos]
        } else if let Some(pos) = line.find("//") {
            &line[..pos]
        } else {
            line
        }
    }

    /// Parse a label definition (label:).
    fn parse_label_definition(&self, line: &str) -> Option<String> {
        let line = line.trim();
        if line.ends_with(':') {
            let label = line[..line.len() - 1].trim();
            if !label.is_empty() {
                return Some(label.to_string());
            }
        }
        None
    }

    /// Encode a single instruction line.
    fn encode_instruction(&self, line: &str, line_num: usize) -> Result<Vec<u8>> {
        // Split by comma or whitespace
        let parts: Vec<&str> = line.split([',', ' ', '\t']).filter(|s| !s.is_empty()).collect();

        if parts.is_empty() {
            return Err(Error::ParseError);
        }

        // Parse mnemonic
        let mnemonic = parts[0].to_uppercase();
        let opcode = self.parse_mnemonic(&mnemonic)?;

        // Encode based on instruction format
        match opcode {
            Opcode::NOP | Opcode::HALT | Opcode::DUP | Opcode::YIELD | Opcode::RET => {
                Ok(vec![opcode as u8])
            }

            Opcode::INC | Opcode::DEC | Opcode::PUSH | Opcode::POP | Opcode::INEG => {
                // Format B: [opcode][rd]
                if parts.len() < 2 {
                    return Err(Error::InvalidInstruction);
                }
                let rd = self.parse_register(parts[1])?;
                Ok(vec![opcode as u8, rd])
            }

            Opcode::CMP | Opcode::MOV | Opcode::LOAD | Opcode::STORE => {
                // Format C: [opcode][rd][rs1]
                if parts.len() < 3 {
                    return Err(Error::InvalidInstruction);
                }
                let rd = self.parse_register(parts[1])?;
                let rs1 = self.parse_register(parts[2])?;
                Ok(vec![opcode as u8, rd, rs1])
            }

            Opcode::MOVI | Opcode::JMP | Opcode::JZ | Opcode::JNZ | Opcode::CALL => {
                // Format D: [opcode][rd][imm16]
                if parts.len() < 2 {
                    return Err(Error::InvalidInstruction);
                }

                match opcode {
                    Opcode::MOVI => {
                        // MOVI has a register destination
                        if parts.len() < 3 {
                            return Err(Error::InvalidInstruction);
                        }
                        let rd = self.parse_register(parts[1])?;
                        let imm = self.parse_immediate(parts[2])?;
                        let imm_bytes = (imm as i16).to_le_bytes();
                        Ok(vec![opcode as u8, rd, imm_bytes[0], imm_bytes[1]])
                    }
                    _ => {
                        // Jump instructions have an immediate target
                        let imm = self.parse_immediate_or_label(parts[1])?;
                        let imm_bytes = (imm as i16).to_le_bytes();
                        Ok(vec![opcode as u8, 0, imm_bytes[0], imm_bytes[1]]) // rd is 0 for jumps
                    }
                }
            }

            Opcode::IADD
            | Opcode::ISUB
            | Opcode::IMUL
            | Opcode::IDIV
            | Opcode::IMOD
            | Opcode::IAND
            | Opcode::IOR
            | Opcode::IXOR
            | Opcode::ISHL
            | Opcode::ISHR => {
                // Format E: [opcode][rd][rs1][rs2]
                if parts.len() < 4 {
                    return Err(Error::InvalidInstruction);
                }
                let rd = self.parse_register(parts[1])?;
                let rs1 = self.parse_register(parts[2])?;
                let rs2 = self.parse_register(parts[3])?;
                Ok(vec![opcode as u8, rd, rs1, rs2])
            }

            Opcode::FADD | Opcode::FSUB | Opcode::FMUL | Opcode::FDIV => {
                // Format E for floats: [opcode][rd][rs1][rs2]
                if parts.len() < 4 {
                    return Err(Error::InvalidInstruction);
                }
                let rd = self.parse_register(parts[1])?;
                let rs1 = self.parse_register(parts[2])?;
                let rs2 = self.parse_register(parts[3])?;
                Ok(vec![opcode as u8, rd, rs1, rs2])
            }

            Opcode::TELL | Opcode::ASK | Opcode::DELEGATE | Opcode::BROADCAST => {
                // Format G: variable length (not fully implemented in basic assembler)
                Ok(vec![opcode as u8, 0, 0]) // Placeholder
            }

            Opcode::INOT => {
                // Format B
                if parts.len() < 2 {
                    return Err(Error::InvalidInstruction);
                }
                let rd = self.parse_register(parts[1])?;
                Ok(vec![opcode as u8, rd])
            }
        }
    }

    /// Parse a mnemonic string to an opcode.
    fn parse_mnemonic(&self, mnemonic: &str) -> Result<Opcode> {
        Ok(match mnemonic {
            "NOP" => Opcode::NOP,
            "MOV" => Opcode::MOV,
            "LOAD" => Opcode::LOAD,
            "STORE" => Opcode::STORE,
            "JMP" => Opcode::JMP,
            "JZ" => Opcode::JZ,
            "JNZ" => Opcode::JNZ,
            "CALL" => Opcode::CALL,
            "IADD" => Opcode::IADD,
            "ISUB" => Opcode::ISUB,
            "IMUL" => Opcode::IMUL,
            "IDIV" => Opcode::IDIV,
            "IMOD" => Opcode::IMOD,
            "INEG" => Opcode::INEG,
            "INC" => Opcode::INC,
            "DEC" => Opcode::DEC,
            "IAND" => Opcode::IAND,
            "IOR" => Opcode::IOR,
            "IXOR" => Opcode::IXOR,
            "INOT" => Opcode::INOT,
            "ISHL" => Opcode::ISHL,
            "ISHR" => Opcode::ISHR,
            "CMP" => Opcode::CMP,
            "MOVI" => Opcode::MOVI,
            "PUSH" => Opcode::PUSH,
            "POP" => Opcode::POP,
            "DUP" => Opcode::DUP,
            "RET" => Opcode::RET,
            "FADD" => Opcode::FADD,
            "FSUB" => Opcode::FSUB,
            "FMUL" => Opcode::FMUL,
            "FDIV" => Opcode::FDIV,
            "TELL" => Opcode::TELL,
            "ASK" => Opcode::ASK,
            "DELEGATE" => Opcode::DELEGATE,
            "BROADCAST" => Opcode::BROADCAST,
            "HALT" => Opcode::HALT,
            "YIELD" => Opcode::YIELD,
            _ => return Err(Error::UnknownMnemonic),
        })
    }

    /// Parse a register identifier (R0-R15, F0-F15, V0-V15).
    fn parse_register(&self, reg_str: &str) -> Result<u8> {
        let reg_str = reg_str.trim();

        // Check for register prefix
        let prefix = reg_str.chars().next().ok_or(Error::InvalidRegister(0))?;

        let num_str = &reg_str[1..];
        let num: u8 = num_str.parse().map_err(|_| Error::InvalidRegister(0))?;

        match prefix {
            'R' | 'F' | 'V' => {
                if num < 16 {
                    Ok(num)
                } else {
                    Err(Error::InvalidRegister(num))
                }
            }
            _ => Err(Error::InvalidRegister(0)),
        }
    }

    /// Parse an immediate value (decimal or hex).
    fn parse_immediate(&self, imm_str: &str) -> Result<i32> {
        let imm_str = imm_str.trim();

        if imm_str.starts_with("0x") || imm_str.starts_with("0X") {
            i32::from_str_radix(&imm_str[2..], 16).map_err(|_| Error::InvalidImmediate)
        } else {
            imm_str.parse().map_err(|_| Error::InvalidImmediate)
        }
    }

    /// Parse an immediate value or label reference.
    fn parse_immediate_or_label(&self, imm_str: &str) -> Result<i32> {
        let imm_str = imm_str.trim();

        // Check if it's a label reference
        if imm_str.starts_with('@') {
            let label = &imm_str[1..];
            if let Some(&addr) = self.labels.get(label) {
                return Ok(addr as i32);
            }
            return Err(Error::ParseError);
        }

        self.parse_immediate(imm_str)
    }

    /// Assemble a single instruction (for convenience).
    ///
    /// # Examples
    ///
    /// ```
    /// use flux_core::bytecode::Assembler;
    ///
    /// let asm = Assembler::new();
    /// let bytecode = asm.assemble_one("MOVI R0, 42").unwrap();
    /// assert_eq!(bytecode, vec![0x2B, 0x00, 0x2A, 0x00]);
    /// ```
    pub fn assemble_one(&mut self, instruction: &str) -> Result<Vec<u8>> {
        let input = format!("{}\nHALT", instruction.trim());
        let mut result = self.assemble(&input)?;

        // Remove the trailing HALT
        if result.last() == Some(&(Opcode::HALT as u8)) {
            result.pop();
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assemble_nop() {
        let asm = Assembler::new();
        let bytecode = asm.assemble_one("NOP").unwrap();
        assert_eq!(bytecode, vec![0x00]);
    }

    #[test]
    fn test_assemble_halt() {
        let asm = Assembler::new();
        let bytecode = asm.assemble_one("HALT").unwrap();
        assert_eq!(bytecode, vec![0x80]);
    }

    #[test]
    fn test_assemble_movi() {
        let asm = Assembler::new();
        let bytecode = asm.assemble_one("MOVI R0, 42").unwrap();
        assert_eq!(bytecode, vec![0x2B, 0x00, 0x2A, 0x00]);
    }

    #[test]
    fn test_assemble_mov_negative() {
        let asm = Assembler::new();
        let bytecode = asm.assemble_one("MOVI R5, -100").unwrap();
        assert_eq!(bytecode[0], 0x2B); // MOVI
        assert_eq!(bytecode[1], 0x05); // R5
        // -100 as i16 little-endian
        assert_eq!(bytecode[2], 0x9C);
        assert_eq!(bytecode[3], 0xFF);
    }

    #[test]
    fn test_assemble_mov_hex() {
        let asm = Assembler::new();
        let bytecode = asm.assemble_one("MOVI R1, 0xFF").unwrap();
        assert_eq!(bytecode[0], 0x2B); // MOVI
        assert_eq!(bytecode[1], 0x01); // R1
        assert_eq!(bytecode[2], 0xFF);
        assert_eq!(bytecode[3], 0x00);
    }

    #[test]
    fn test_assemble_iadd() {
        let asm = Assembler::new();
        let bytecode = asm.assemble_one("IADD R0, R1, R2").unwrap();
        assert_eq!(bytecode, vec![0x08, 0x00, 0x01, 0x02]);
    }

    #[test]
    fn test_assemble_program() {
        let asm = Assembler::new();
        let program = "MOVI R0, 10
                       MOVI R1, 20
                       IADD R0, R1, R2
                       HALT";

        let bytecode = asm.assemble(program).unwrap();
        assert_eq!(bytecode.len(), 13); // 4 + 4 + 4 + 1

        assert_eq!(bytecode[0], 0x2B); // MOVI R0, 10
        assert_eq!(bytecode[4], 0x2B); // MOVI R1, 20
        assert_eq!(bytecode[8], 0x08); // IADD
        assert_eq!(bytecode[12], 0x80); // HALT
    }

    #[test]
    fn test_comments() {
        let asm = Assembler::new();
        let bytecode = asm.assemble("MOVI R0, 42 ; This is a comment\nHALT").unwrap();
        assert_eq!(bytecode.len(), 5);
        assert_eq!(bytecode[0], 0x2B);
        assert_eq!(bytecode[4], 0x80);
    }

    #[test]
    fn test_slash_comments() {
        let asm = Assembler::new();
        let bytecode = asm
            .assemble("MOVI R0, 42 // This is a comment\nHALT")
            .unwrap();
        assert_eq!(bytecode.len(), 5);
        assert_eq!(bytecode[0], 0x2B);
        assert_eq!(bytecode[4], 0x80);
    }

    #[test]
    fn test_push_pop() {
        let asm = Assembler::new();
        let bytecode = asm.assemble("PUSH R0\nPOP R1\nHALT").unwrap();
        assert_eq!(bytecode[0], 0x20); // PUSH
        assert_eq!(bytecode[1], 0x00); // R0
        assert_eq!(bytecode[2], 0x21); // POP
        assert_eq!(bytecode[3], 0x01); // R1
    }

    #[test]
    fn test_inc_dec() {
        let asm = Assembler::new();
        let bytecode = asm.assemble_one("INC R5").unwrap();
        assert_eq!(bytecode, vec![0x0E, 0x05]);

        let bytecode = asm.assemble_one("DEC R10").unwrap();
        assert_eq!(bytecode, vec![0x0F, 0x0A]);
    }

    #[test]
    fn test_jump_instructions() {
        let asm = Assembler::new();
        let bytecode = asm.assemble_one("JMP 100").unwrap();
        assert_eq!(bytecode[0], 0x04); // JMP

        let bytecode = asm.assemble_one("JZ 50").unwrap();
        assert_eq!(bytecode[0], 0x05); // JZ

        let bytecode = asm.assemble_one("JNZ -10").unwrap();
        assert_eq!(bytecode[0], 0x06); // JNZ
    }

    #[test]
    fn test_cmp() {
        let asm = Assembler::new();
        let bytecode = asm.assemble_one("CMP R0, R1").unwrap();
        assert_eq!(bytecode, vec![0x2D, 0x00, 0x01]);
    }

    #[test]
    fn test_float_ops() {
        let asm = Assembler::new();
        let bytecode = asm.assemble_one("FADD F0, F1, F2").unwrap();
        assert_eq!(bytecode, vec![0x40, 0x00, 0x01, 0x02]);
    }

    #[test]
    fn test_invalid_register() {
        let asm = Assembler::new();
        assert!(asm.assemble_one("MOVI R16, 42").is_err());
        assert!(asm.assemble_one("MOVI X0, 42").is_err());
    }

    #[test]
    fn test_unknown_mnemonic() {
        let asm = Assembler::new();
        assert!(asm.assemble_one("INVALID R0, R1").is_err());
    }

    #[test]
    fn test_whitespace_variations() {
        let asm = Assembler::new();
        let b1 = asm.assemble_one("MOVI R0,42").unwrap();
        let b2 = asm.assemble_one("MOVI\tR0,\t42").unwrap();
        let b3 = asm.assemble_one("  MOVI  R0  ,  42  ").unwrap();

        assert_eq!(b1, b2);
        assert_eq!(b2, b3);
    }
}
