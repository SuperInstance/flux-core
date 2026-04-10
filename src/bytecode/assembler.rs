use crate::bytecode::opcodes::Op;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Assembler {
    output: Vec<u8>,
    labels: HashMap<String, usize>,
    fixups: Vec<(usize, usize, String)>, // (patch_pos, instr_end, label)
}

impl Assembler {
    pub fn new() -> Self {
        Self {
            output: Vec::new(),
            labels: HashMap::new(),
            fixups: Vec::new(),
        }
    }

    pub fn assemble(source: &str) -> Result<Vec<u8>, String> {
        let mut asm = Self::new();
        let lines: Vec<&str> = source.lines().collect();

        // Pass 1: calculate sizes and find labels
        let mut pos = 0;
        for line in &lines {
            let trimmed = line.split(';').next().unwrap().trim();
            if trimmed.is_empty() { continue; }
            if let Some(label) = trimmed.strip_suffix(':') {
                asm.labels.insert(label.trim().to_string(), pos);
                continue;
            }
            // Check for inline label
            if let Some(colon_pos) = trimmed.find(':') {
                let label = trimmed[..colon_pos].trim();
                asm.labels.insert(label.to_string(), pos);
                let rest = trimmed[colon_pos+1..].trim();
                if rest.is_empty() { continue; }
            }
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.is_empty() { continue; }
            pos += asm.instr_size(parts[0]);
        }

        // Pass 2: emit bytecode
        for line in &lines {
            let trimmed = line.split(';').next().unwrap().trim();
            if trimmed.is_empty() { continue; }
            if trimmed.ends_with(':') { continue; }
            let line = if let Some(colon_pos) = trimmed.find(':') {
                trimmed[colon_pos+1..].trim()
            } else { trimmed };
            if line.is_empty() { continue; }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.is_empty() { continue; }
            asm.emit_instruction(&parts)?;
        }

        // Apply fixups
        for (patch_pos, instr_end, label) in &asm.fixups {
            let target = asm.labels.get(label)
                .ok_or_else(|| format!("Undefined label: {}", label))?;
            let offset = *target as i64 - *instr_end as i64;
            let offset_i16 = offset as i16;
            asm.output[*patch_pos] = (offset_i16 & 0xFF) as u8;
            asm.output[*patch_pos + 1] = ((offset_i16 >> 8) & 0xFF) as u8;
        }

        Ok(asm.output)
    }

    fn instr_size(&self, op: &str) -> usize {
        match op.to_uppercase().as_str() {
            "HALT" | "NOP" | "DUP" | "YIELD" => 1,
            "INC" | "DEC" | "PUSH" | "POP" | "INEG" | "INOT" => 2,
            "CMP" | "MOV" | "LOAD" | "STORE" => 3,
            "MOVI" | "JMP" | "JZ" | "JNZ" | "CALL" => 4,
            "IADD" | "ISUB" | "IMUL" | "IDIV" | "IMOD" |
            "IAND" | "IOR" | "IXOR" | "ISHL" | "ISHR" => 3,
            _ => 4,
        }
    }

    fn emit_instruction(&mut self, parts: &[&str]) -> Result<(), String> {
        let op = parts[0].to_uppercase();
        let pos = self.output.len();

        match op.as_str() {
            "HALT" => self.output.push(Op::HALT as u8),
            "NOP" => self.output.push(Op::NOP as u8),
            "DUP" => self.output.push(Op::DUP as u8),
            "INC" => { self.output.push(Op::INC as u8); self.output.push(parse_reg(parts[1])); }
            "DEC" => { self.output.push(Op::DEC as u8); self.output.push(parse_reg(parts[1])); }
            "MOVI" => {
                self.output.push(Op::MOVI as u8);
                self.output.push(parse_reg(parts[1]));
                let imm = parse_int(parts[2]);
                self.output.extend_from_slice(&(imm as i16).to_le_bytes());
            }
            "IADD" | "ISUB" | "IMUL" | "IDIV" | "IMOD" |
            "IAND" | "IOR" | "IXOR" | "ISHL" | "ISHR" => {
                let opc = match op.as_str() {
                    "IADD" => Op::IADD, "ISUB" => Op::ISUB, "IMUL" => Op::IMUL,
                    "IDIV" => Op::IDIV, "IMOD" => Op::IMOD, "IAND" => Op::IAND,
                    "IOR" => Op::IOR, "IXOR" => Op::IXOR, "ISHL" => Op::ISHL,
                    "ISHR" => Op::ISHR, _ => Op::NOP,
                };
                self.output.push(opc as u8);
                self.output.push(parse_reg(parts[1]));
                self.output.push(parse_reg(parts[2]));
            }
            "CMP" => {
                self.output.push(Op::CMP as u8);
                self.output.push(parse_reg(parts[1]));
                self.output.push(parse_reg(parts[2]));
            }
            "JMP" => {
                self.output.push(Op::JMP as u8);
                self.output.push(0);
                let label = parts[1].to_string();
                self.fixups.push((pos + 2, pos + 4, label));
                self.output.extend_from_slice(&[0, 0]);
            }
            "JZ" | "JNZ" => {
                let opc = if op == "JZ" { Op::JZ } else { Op::JNZ };
                self.output.push(opc as u8);
                self.output.push(parse_reg(parts[1]));
                let label = parts[2].to_string();
                self.fixups.push((pos + 2, pos + 4, label));
                self.output.extend_from_slice(&[0, 0]);
            }
            "PUSH" => { self.output.push(Op::PUSH as u8); self.output.push(parse_reg(parts[1])); }
            "POP" => { self.output.push(Op::POP as u8); self.output.push(parse_reg(parts[1])); }
            "RET" => { self.output.push(Op::RET as u8); self.output.push(0); self.output.push(0); }
            _ => return Err(format!("Unknown instruction: {}", op)),
        }
        Ok(())
    }
}

fn parse_reg(s: &str) -> u8 {
    let s = s.trim_end_matches(',');
    if s.starts_with('R') || s.starts_with('r') {
        s[1..].parse().unwrap_or(0)
    } else { 0 }
}

fn parse_int(s: &str) -> i32 {
    s.trim_end_matches(',').parse().unwrap_or(0)
}
