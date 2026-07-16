use crate::bytecode::opcodes::Op;
use crate::error::FluxError;
use super::registers::RegisterFile;

const DEFAULT_MAX_CYCLES: u64 = 10_000_000;
const MEMORY_SIZE: usize = 65536; // 64 KB linear memory for LOAD/STORE

#[derive(Debug)]
pub struct Interpreter<'a> {
    bytecode: &'a [u8],
    pub regs: RegisterFile,
    pub halted: bool,
    pub cycle_count: u64,
    max_cycles: u64,
    stack: Vec<i32>,
    memory: Vec<u8>, // linear memory for LOAD/STORE (matches Python's memory regions)
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
            memory: vec![0u8; MEMORY_SIZE],
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

    #[inline]
    fn read_u16(&mut self) -> u16 {
        let pc = self.regs.pc as usize;
        if pc + 1 < self.bytecode.len() {
            let lo = self.bytecode[pc] as u16;
            let hi = self.bytecode[pc + 1] as u16;
            self.regs.pc += 2;
            lo | (hi << 8)
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

            match op_byte {
                0x00 => {} // NOP
                0x01 => { let d = self.read_u8(); let s = self.read_u8(); self.regs.write_gp(d, self.regs.read_gp(s)); } // MOV rd, rs
                0x02 => { // LOAD rd, rs(addr) — read i32 from linear memory
                    let d = self.read_u8(); let s = self.read_u8();
                    let addr = self.regs.read_gp(s) as usize;
                    if addr + 4 <= self.memory.len() {
                        let val = i32::from_le_bytes([
                            self.memory[addr], self.memory[addr+1],
                            self.memory[addr+2], self.memory[addr+3],
                        ]);
                        self.regs.write_gp(d, val);
                    } else {
                        self.regs.write_gp(d, 0);
                    }
                } // LOAD
                0x03 => { // STORE rd(val), rs(addr) — write i32 to linear memory
                    let d = self.read_u8(); let s = self.read_u8();
                    let addr = self.regs.read_gp(s) as usize;
                    let val = self.regs.read_gp(d);
                    if addr + 4 <= self.memory.len() {
                        self.memory[addr..addr+4].copy_from_slice(&val.to_le_bytes());
                    }
                } // STORE
                0x04 => { let _r = self.read_u8(); let off = self.read_i16(); self.regs.pc = (self.regs.pc as i64 + off as i64) as u32; } // JMP
                0x05 => { let r = self.read_u8(); let off = self.read_i16(); if self.regs.read_gp(r) == 0 { self.regs.pc = (self.regs.pc as i64 + off as i64) as u32; } } // JZ
                0x06 => { let r = self.read_u8(); let off = self.read_i16(); if self.regs.read_gp(r) != 0 { self.regs.pc = (self.regs.pc as i64 + off as i64) as u32; } } // JNZ
                0x07 => { let _r = self.read_u8(); let off = self.read_i16(); self.stack.push(self.regs.pc as i32); self.regs.pc = (self.regs.pc as i64 + off as i64) as u32; } // CALL
                // 3-operand format: [op][rd][rs1][rs2] — matches Python/JS
                0x08 => { let d = self.read_u8(); let a = self.read_u8(); let b = self.read_u8(); let r = self.regs.read_gp(a).wrapping_add(self.regs.read_gp(b)); self.regs.write_gp(d, r); self.regs.set_flags(r); } // IADD rd, rs1, rs2
                0x09 => { let d = self.read_u8(); let a = self.read_u8(); let b = self.read_u8(); let r = self.regs.read_gp(a).wrapping_sub(self.regs.read_gp(b)); self.regs.write_gp(d, r); self.regs.set_flags(r); } // ISUB rd, rs1, rs2
                0x0A => { let d = self.read_u8(); let a = self.read_u8(); let b = self.read_u8(); let r = self.regs.read_gp(a).wrapping_mul(self.regs.read_gp(b)); self.regs.write_gp(d, r); self.regs.set_flags(r); } // IMUL rd, rs1, rs2
                0x0B => { let d = self.read_u8(); let a = self.read_u8(); let b = self.read_u8(); if self.regs.read_gp(b) == 0 { return Err(FluxError::DivisionByZero); } let r = self.regs.read_gp(a) / self.regs.read_gp(b); self.regs.write_gp(d, r); self.regs.set_flags(r); } // IDIV rd, rs1, rs2
                0x0C => { let d = self.read_u8(); let a = self.read_u8(); let b = self.read_u8(); if self.regs.read_gp(b) == 0 { return Err(FluxError::DivisionByZero); } let r = self.regs.read_gp(a) % self.regs.read_gp(b); self.regs.write_gp(d, r); self.regs.set_flags(r); } // IMOD rd, rs1, rs2
                0x0D => { let d = self.read_u8(); let s = self.read_u8(); let r = -self.regs.read_gp(s); self.regs.write_gp(d, r); self.regs.set_flags(r); } // INEG rd, rs
                0x0E => { let d = self.read_u8(); let r = self.regs.read_gp(d).wrapping_add(1); self.regs.write_gp(d, r); self.regs.set_flags(r); } // INC rd
                0x0F => { let d = self.read_u8(); let r = self.regs.read_gp(d).wrapping_sub(1); self.regs.write_gp(d, r); self.regs.set_flags(r); } // DEC rd
                0x10 => { let d = self.read_u8(); let a = self.read_u8(); let b = self.read_u8(); let r = self.regs.read_gp(a) & self.regs.read_gp(b); self.regs.write_gp(d, r); self.regs.set_flags(r); } // IAND rd, rs1, rs2 (sets flags per FLUX_BYTECODE_SPEC.md L63)
                0x11 => { let d = self.read_u8(); let a = self.read_u8(); let b = self.read_u8(); let r = self.regs.read_gp(a) | self.regs.read_gp(b); self.regs.write_gp(d, r); self.regs.set_flags(r); } // IOR rd, rs1, rs2 (sets flags per spec L63)
                0x12 => { let d = self.read_u8(); let a = self.read_u8(); let b = self.read_u8(); let r = self.regs.read_gp(a) ^ self.regs.read_gp(b); self.regs.write_gp(d, r); self.regs.set_flags(r); } // IXOR rd, rs1, rs2 (sets flags per spec L63)
                0x13 => { let d = self.read_u8(); let s = self.read_u8(); let r = !self.regs.read_gp(s); self.regs.write_gp(d, r); self.regs.set_flags(r); } // INOT rd, rs (sets flags per spec L63)
                0x14 => { let d = self.read_u8(); let a = self.read_u8(); let b = self.read_u8(); let shift = (self.regs.read_gp(b) & 0x3F) as u32; let r = self.regs.read_gp(a).wrapping_shl(shift); self.regs.write_gp(d, r); self.regs.set_flags(r); } // ISHL rd, rs1, rs2
                0x15 => { let d = self.read_u8(); let a = self.read_u8(); let b = self.read_u8(); let shift = (self.regs.read_gp(b) & 0x3F) as u32; let r = self.regs.read_gp(a).wrapping_shr(shift); self.regs.write_gp(d, r); self.regs.set_flags(r); } // ISHR rd, rs1, rs2
                0x20 => { let r = self.read_u8(); self.stack.push(self.regs.read_gp(r)); } // PUSH
                0x21 => { let r = self.read_u8(); match self.stack.pop() { Some(v) => self.regs.write_gp(r, v), None => return Err(FluxError::StackUnderflow), } } // POP (errors on empty stack — v0.1.1 audit fix)
                0x22 => { match self.stack.last() { Some(&v) => self.stack.push(v), None => return Err(FluxError::StackUnderflow), } } // DUP (errors on empty stack — v0.1.1 audit fix)
                0x28 => { let _r = self.read_u8(); let _p = self.read_u8(); match self.stack.pop() { Some(ret_pc) => self.regs.pc = ret_pc as u32, None => return Err(FluxError::StackUnderflow), } } // RET (errors on empty stack — v0.1.1 audit fix)
                0x2B => { let d = self.read_u8(); let imm = self.read_i16(); self.regs.write_gp(d, imm as i32); } // MOVI
                0x2D => { let a = self.read_u8(); let b = self.read_u8(); let va = self.regs.read_gp(a); let vb = self.regs.read_gp(b); self.regs.flag_zero = va == vb; self.regs.flag_sign = va < vb; } // CMP
                0x2E => { let _r = self.read_u8(); let off = self.read_i16(); if self.regs.flag_zero { self.regs.pc = (self.regs.pc as i64 + off as i64) as u32; } } // JE
                0x2F => { let _r = self.read_u8(); let off = self.read_i16(); if !self.regs.flag_zero { self.regs.pc = (self.regs.pc as i64 + off as i64) as u32; } } // JNE
                0x40 => { let d = self.read_u8(); let a = self.read_u8(); let b = self.read_u8(); let r = self.regs.read_fp(a) + self.regs.read_fp(b); self.regs.write_fp(d, r); } // FADD
                0x41 => { let d = self.read_u8(); let a = self.read_u8(); let b = self.read_u8(); let r = self.regs.read_fp(a) - self.regs.read_fp(b); self.regs.write_fp(d, r); } // FSUB
                0x42 => { let d = self.read_u8(); let a = self.read_u8(); let b = self.read_u8(); let r = self.regs.read_fp(a) * self.regs.read_fp(b); self.regs.write_fp(d, r); } // FMUL
                0x43 => { let d = self.read_u8(); let a = self.read_u8(); let b = self.read_u8(); let divisor = self.regs.read_fp(b); if divisor == 0.0 { return Err(FluxError::DivisionByZero); } let r = self.regs.read_fp(a) / divisor; self.regs.write_fp(d, r); } // FDIV
                0x60 => { /* TELL: A2A stub — Format G [len:u16][data] */ let _len = self.read_u16(); for _ in 0.._len { let _ = self.read_u8(); } } // TELL (stub)
                0x61 => { /* ASK: A2A stub — Format G */ let _len = self.read_u16(); for _ in 0.._len { let _ = self.read_u8(); } } // ASK (stub)
                0x62 => { /* DELEGATE: A2A stub — Format G */ let _len = self.read_u16(); for _ in 0.._len { let _ = self.read_u8(); } } // DELEGATE (stub)
                0x66 => { /* BROADCAST: A2A stub — Format G */ let _len = self.read_u16(); for _ in 0.._len { let _ = self.read_u8(); } } // BROADCAST (stub)
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
