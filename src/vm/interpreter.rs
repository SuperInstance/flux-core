use crate::bytecode::opcodes::Op;
use crate::error::FluxError;
use super::registers::RegisterFile;

const DEFAULT_MAX_CYCLES: u64 = 10_000_000;

#[derive(Debug)]
pub struct Interpreter<'a> {
    bytecode: &'a [u8],
    pub regs: RegisterFile,
    pub halted: bool,
    pub cycle_count: u64,
    max_cycles: u64,
    stack: Vec<i32>,
}

impl<'a> Interpreter<'a> {
    pub fn new(bytecode: &'a [u8]) -> Self {
        Self {
            bytecode,
            regs: RegisterFile::new(),
            halted: false,
            cycle_count: 0,
            max_cycles: DEFAULT_MAX_CYCLES,
            stack: Vec::with_capacity(1024),
        }
    }

    pub fn with_max_cycles(mut self, max: u64) -> Self {
        self.max_cycles = max;
        self
    }

    #[inline]
    fn read_u8(&mut self) -> u8 {
        let pc = self.regs.pc as usize;
        if pc < self.bytecode.len() {
            self.regs.pc += 1;
            self.bytecode[pc]
        } else {
            Op::HALT as u8
        }
    }

    #[inline]
    fn read_i16(&mut self) -> i16 {
        let pc = self.regs.pc as usize;
        if pc + 1 < self.bytecode.len() {
            let lo = self.bytecode[pc] as u16;
            let hi = self.bytecode[pc + 1] as u16;
            self.regs.pc += 2;
            (lo | (hi << 8)) as i16
        } else {
            self.regs.pc = self.bytecode.len() as u32;
            0
        }
    }

    pub fn execute(&mut self) -> Result<u64, FluxError> {
        self.halted = false;
        self.cycle_count = 0;

        while !self.halted && self.cycle_count < self.max_cycles {
            let pc = self.regs.pc as usize;
            if pc >= self.bytecode.len() {
                break;
            }
            let op_byte = self.read_u8();
            self.cycle_count += 1;

            // SAFETY: We cast u8 to Op via FromPrimitive-style match
            match op_byte {
                0x00 => {} // NOP
                0x01 => { let d = self.read_u8(); let s = self.read_u8(); self.regs.write_gp(d, self.regs.read_gp(s)); } // MOV
                0x04 => { let _r = self.read_u8(); let off = self.read_i16(); self.regs.pc = (self.regs.pc as i64 + off as i64) as u32; } // JMP
                0x05 => { let r = self.read_u8(); let off = self.read_i16(); if self.regs.read_gp(r) == 0 { self.regs.pc = (self.regs.pc as i64 + off as i64) as u32; } } // JZ
                0x06 => { let r = self.read_u8(); let off = self.read_i16(); if self.regs.read_gp(r) != 0 { self.regs.pc = (self.regs.pc as i64 + off as i64) as u32; } } // JNZ
                0x07 => { let _r = self.read_u8(); let off = self.read_i16(); self.stack.push(self.regs.pc as i32); self.regs.pc = (self.regs.pc as i64 + off as i64) as u32; } // CALL
                0x08 => { let d = self.read_u8(); let s = self.read_u8(); let r = self.regs.read_gp(d).wrapping_add(self.regs.read_gp(s)); self.regs.write_gp(d, r); self.regs.set_flags(r); } // IADD
                0x09 => { let d = self.read_u8(); let s = self.read_u8(); let r = self.regs.read_gp(d).wrapping_sub(self.regs.read_gp(s)); self.regs.write_gp(d, r); self.regs.set_flags(r); } // ISUB
                0x0A => { let d = self.read_u8(); let s = self.read_u8(); let r = self.regs.read_gp(d).wrapping_mul(self.regs.read_gp(s)); self.regs.write_gp(d, r); self.regs.set_flags(r); } // IMUL
                0x0B => { let d = self.read_u8(); let s = self.read_u8(); if self.regs.read_gp(s) == 0 { return Err(FluxError::DivisionByZero); } let r = self.regs.read_gp(d) / self.regs.read_gp(s); self.regs.write_gp(d, r); self.regs.set_flags(r); } // IDIV
                0x0C => { let d = self.read_u8(); let s = self.read_u8(); if self.regs.read_gp(s) == 0 { return Err(FluxError::DivisionByZero); } let r = self.regs.read_gp(d) % self.regs.read_gp(s); self.regs.write_gp(d, r); self.regs.set_flags(r); } // IMOD
                0x0D => { let d = self.read_u8(); let r = -self.regs.read_gp(d); self.regs.write_gp(d, r); self.regs.set_flags(r); } // INEG
                0x0E => { let d = self.read_u8(); let r = self.regs.read_gp(d).wrapping_add(1); self.regs.write_gp(d, r); self.regs.set_flags(r); } // INC
                0x0F => { let d = self.read_u8(); let r = self.regs.read_gp(d).wrapping_sub(1); self.regs.write_gp(d, r); self.regs.set_flags(r); } // DEC
                0x10 => { let d = self.read_u8(); let s = self.read_u8(); let r = self.regs.read_gp(d) & self.regs.read_gp(s); self.regs.write_gp(d, r); } // IAND
                0x11 => { let d = self.read_u8(); let s = self.read_u8(); let r = self.regs.read_gp(d) | self.regs.read_gp(s); self.regs.write_gp(d, r); } // IOR
                0x12 => { let d = self.read_u8(); let s = self.read_u8(); let r = self.regs.read_gp(d) ^ self.regs.read_gp(s); self.regs.write_gp(d, r); } // IXOR
                0x13 => { let d = self.read_u8(); let r = !self.regs.read_gp(d); self.regs.write_gp(d, r); } // INOT
                0x20 => { let r = self.read_u8(); self.stack.push(self.regs.read_gp(r)); } // PUSH
                0x21 => { let r = self.read_u8(); if let Some(v) = self.stack.pop() { self.regs.write_gp(r, v); } } // POP
                0x22 => { if let Some(&v) = self.stack.last() { self.stack.push(v); } } // DUP
                0x28 => { let _r = self.read_u8(); let _p = self.read_u8(); if let Some(ret_pc) = self.stack.pop() { self.regs.pc = ret_pc as u32; } } // RET
                0x2B => { let d = self.read_u8(); let imm = self.read_i16(); self.regs.write_gp(d, imm as i32); } // MOVI
                0x2D => { let a = self.read_u8(); let b = self.read_u8(); let va = self.regs.read_gp(a); let vb = self.regs.read_gp(b); self.regs.flag_zero = va == vb; self.regs.flag_sign = va < vb; } // CMP
                0x80 => { self.halted = true; } // HALT
                0x81 => {} // YIELD
                _ => return Err(FluxError::InvalidOpcode(op_byte)),
            }
        }

        if self.cycle_count >= self.max_cycles {
            return Err(FluxError::CycleBudgetExceeded(self.max_cycles));
        }

        Ok(self.cycle_count)
    }

    #[inline]
    pub fn read_gp(&self, idx: u8) -> i32 { self.regs.read_gp(idx) }
    #[inline]
    pub fn write_gp(&mut self, idx: u8, val: i32) { self.regs.write_gp(idx, val) }
}
