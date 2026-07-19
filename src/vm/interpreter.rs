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
    pub stack: Vec<i32>,
    pub memory: Vec<u8>, // 64 KB linear memory for LOAD/STORE
    #[cfg(feature = "tensor")]
    pub tensor_scratch: [u8; 4096],
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
            #[cfg(feature = "tensor")]
            tensor_scratch: [0u8; 4096],
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
    fn jump_relative(&mut self, off: i16) {
        let new_pc = (self.regs.pc as i64).saturating_add(off as i64);
        let max = self.bytecode.len() as i64;
        self.regs.pc = new_pc.clamp(0, max) as u32;
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
                0x00 => {}
                0x01 => { let d = self.read_u8(); let s = self.read_u8(); self.regs.write_gp(d, self.regs.read_gp(s)); }
                // Format C: LOAD rd, rs(addr) — rd = memory[rs]
                0x02 => { let d = self.read_u8(); let s = self.read_u8(); let addr = self.regs.read_gp(s) as usize; if addr + 4 > self.memory.len() { return Err(FluxError::InvalidOpcode(0x02)); } let val = i32::from_le_bytes([self.memory[addr], self.memory[addr+1], self.memory[addr+2], self.memory[addr+3]]); self.regs.write_gp(d, val); }
                // Format C: STORE rd(val), rs(addr) — memory[rs] = rd
                0x03 => { let d = self.read_u8(); let s = self.read_u8(); let addr = self.regs.read_gp(s) as usize; let val = self.regs.read_gp(d); if addr + 4 > self.memory.len() { return Err(FluxError::InvalidOpcode(0x03)); } let bytes = val.to_le_bytes(); self.memory[addr..addr+4].copy_from_slice(&bytes); }
                0x04 => { let _r = self.read_u8(); let off = self.read_i16(); self.jump_relative(off); }
                0x05 => { let r = self.read_u8(); let off = self.read_i16(); if self.regs.read_gp(r) == 0 { self.jump_relative(off); } }
                0x06 => { let r = self.read_u8(); let off = self.read_i16(); if self.regs.read_gp(r) != 0 { self.jump_relative(off); } }
                0x07 => { let _r = self.read_u8(); let off = self.read_i16(); self.stack.push(self.regs.pc as i32); self.jump_relative(off); }
                // Format E: [opcode][rd][rs1][rs2] — 3-operand
                0x08 => { let d = self.read_u8(); let s1 = self.read_u8(); let s2 = self.read_u8(); let r = self.regs.read_gp(s1).wrapping_add(self.regs.read_gp(s2)); self.regs.write_gp(d, r); self.regs.set_flags(r); }
                0x09 => { let d = self.read_u8(); let s1 = self.read_u8(); let s2 = self.read_u8(); let r = self.regs.read_gp(s1).wrapping_sub(self.regs.read_gp(s2)); self.regs.write_gp(d, r); self.regs.set_flags(r); }
                0x0A => { let d = self.read_u8(); let s1 = self.read_u8(); let s2 = self.read_u8(); let r = self.regs.read_gp(s1).wrapping_mul(self.regs.read_gp(s2)); self.regs.write_gp(d, r); self.regs.set_flags(r); }
                0x0B => { let d = self.read_u8(); let s1 = self.read_u8(); let s2 = self.read_u8(); if self.regs.read_gp(s2) == 0 { return Err(FluxError::DivisionByZero); } let r = self.regs.read_gp(s1) / self.regs.read_gp(s2); self.regs.write_gp(d, r); self.regs.set_flags(r); }
                0x0C => { let d = self.read_u8(); let s1 = self.read_u8(); let s2 = self.read_u8(); if self.regs.read_gp(s2) == 0 { return Err(FluxError::DivisionByZero); } let r = self.regs.read_gp(s1) % self.regs.read_gp(s2); self.regs.write_gp(d, r); self.regs.set_flags(r); }
                0x0D => { let d = self.read_u8(); let r = -self.regs.read_gp(d); self.regs.write_gp(d, r); self.regs.set_flags(r); }
                0x0E => { let d = self.read_u8(); let r = self.regs.read_gp(d).wrapping_add(1); self.regs.write_gp(d, r); self.regs.set_flags(r); }
                0x0F => { let d = self.read_u8(); let r = self.regs.read_gp(d).wrapping_sub(1); self.regs.write_gp(d, r); self.regs.set_flags(r); }
                // Format E: [opcode][rd][rs1][rs2] — 3-operand bitwise
                0x10 => { let d = self.read_u8(); let s1 = self.read_u8(); let s2 = self.read_u8(); let r = self.regs.read_gp(s1) & self.regs.read_gp(s2); self.regs.write_gp(d, r); }
                0x11 => { let d = self.read_u8(); let s1 = self.read_u8(); let s2 = self.read_u8(); let r = self.regs.read_gp(s1) | self.regs.read_gp(s2); self.regs.write_gp(d, r); }
                0x12 => { let d = self.read_u8(); let s1 = self.read_u8(); let s2 = self.read_u8(); let r = self.regs.read_gp(s1) ^ self.regs.read_gp(s2); self.regs.write_gp(d, r); }
                0x13 => { let d = self.read_u8(); let s = self.read_u8(); let r = !self.regs.read_gp(s); self.regs.write_gp(d, r); }
                // Format E: ISHL rd, rs1, rs2 — rd = rs1 << rs2
                0x14 => { let d = self.read_u8(); let s1 = self.read_u8(); let s2 = self.read_u8(); let r = self.regs.read_gp(s1).wrapping_shl(self.regs.read_gp(s2) as u32); self.regs.write_gp(d, r); self.regs.set_flags(r); }
                // Format E: ISHR rd, rs1, rs2 — rd = rs1 >> rs2 (arithmetic)
                0x15 => { let d = self.read_u8(); let s1 = self.read_u8(); let s2 = self.read_u8(); let r = self.regs.read_gp(s1).wrapping_shr(self.regs.read_gp(s2) as u32); self.regs.write_gp(d, r); self.regs.set_flags(r); }
                0x20 => { let r = self.read_u8(); self.stack.push(self.regs.read_gp(r)); }
                0x21 => { let r = self.read_u8(); match self.stack.pop() { Some(v) => self.regs.write_gp(r, v), None => return Err(FluxError::StackUnderflow), } }
                0x22 => { if let Some(&v) = self.stack.last() { self.stack.push(v); } }
                0x28 => { let _r = self.read_u8(); let _p = self.read_u8(); if let Some(ret_pc) = self.stack.pop() { self.regs.pc = ret_pc as u32; } }
                0x2B => { let d = self.read_u8(); let imm = self.read_i16(); self.regs.write_gp(d, imm as i32); }
                0x2D => { let a = self.read_u8(); let b = self.read_u8(); let va = self.regs.read_gp(a); let vb = self.regs.read_gp(b); self.regs.flag_zero = va == vb; self.regs.flag_sign = va < vb; }
                0x2E => { let _r = self.read_u8(); let off = self.read_i16(); if self.regs.flag_zero { self.jump_relative(off); } }
                0x2F => { let _r = self.read_u8(); let off = self.read_i16(); if !self.regs.flag_zero { self.jump_relative(off); } }
                // Format E: FADD/FSUB/FMUL rd, rs1, rs2 (3-operand)
                0x40 => { let d = self.read_u8(); let s1 = self.read_u8(); let s2 = self.read_u8(); let a = f32::from_bits(self.regs.read_gp(s1) as u32); let b = f32::from_bits(self.regs.read_gp(s2) as u32); self.regs.write_gp(d, f32::to_bits(a + b) as i32); }
                0x41 => { let d = self.read_u8(); let s1 = self.read_u8(); let s2 = self.read_u8(); let a = f32::from_bits(self.regs.read_gp(s1) as u32); let b = f32::from_bits(self.regs.read_gp(s2) as u32); self.regs.write_gp(d, f32::to_bits(a - b) as i32); }
                0x42 => { let d = self.read_u8(); let s1 = self.read_u8(); let s2 = self.read_u8(); let a = f32::from_bits(self.regs.read_gp(s1) as u32); let b = f32::from_bits(self.regs.read_gp(s2) as u32); self.regs.write_gp(d, f32::to_bits(a * b) as i32); }
                // Format E: FDIV rd, rs1, rs2 (3-operand, catches -0.0)
                0x43 => { let d = self.read_u8(); let s1 = self.read_u8(); let s2 = self.read_u8(); let a = f32::from_bits(self.regs.read_gp(s1) as u32); let b = f32::from_bits(self.regs.read_gp(s2) as u32); if b == 0.0 || b == -0.0 { return Err(FluxError::DivisionByZero); } self.regs.write_gp(d, f32::to_bits(a / b) as i32); }
                0x80 => { self.halted = true; }
                0x81 => {}
                // Tensor operations (A2 agent protocol, opt-in via feature)
                #[cfg(feature = "tensor")]
                0xA0..=0xA3 => self.exec_tensor(op_byte)?,
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

    /// Execute tensor operations via scratchpad memory.
    /// Uses a 4KB tensor_scratch buffer for packed data.
    /// Data moves between GP registers and scratchpad via TLD/TST ops.
    #[cfg(feature = "tensor")]
    fn exec_tensor(&mut self, op: u8) -> Result<(), FluxError> {
        use neon_kernel::PackedTernary;
        use neon_kernel::attractor_step;

        match op {
            0xA0 => { // TMAT — ternary matmul 16×16 (scratch regions 0-15, 16-31 -> output 32-47)
                let mut rows = [PackedTernary(0); 16];
                let mut cols = [PackedTernary(0); 16];
                for k in 0..16 {
                    let mut bytes = [0u8; 16];
                    let base = k * 2; // 2 bytes packed per row
                    for j in 0..16 {
                        bytes[j] = self.tensor_scratch[base + j];
                    }
                    let mut arr = [0i8; 64];
                    for j in 0..16 { arr[j] = bytes[j] as i8; }
                    rows[k] = PackedTernary::pack(&arr);
                }
                for k in 0..16 {
                    let mut bytes = [0u8; 16];
                    let base = 256 + k * 2;
                    for j in 0..16 {
                        bytes[j] = self.tensor_scratch[base + j];
                    }
                    let mut arr = [0i8; 64];
                    for j in 0..16 { arr[j] = bytes[j] as i8; }
                    cols[k] = PackedTernary::pack(&arr);
                }
                let mut result = [0.0f32; 256];
                neon_kernel::ternary_matmul_tile_16(&rows, &cols, &mut result);
                for k in 0..16 {
                    let val = (result[k].min(127.0).max(-128.0)) as i8;
                    self.tensor_scratch[512 + k] = val as u8;
                }
            }
            0xA1 => { // TATTRACT — attractor step on scratchpad region 0-63 (f32)
                let threshold_bytes = self.read_u8();
                let threshold = if threshold_bytes == 0 { 0.5 } else { threshold_bytes as f32 / 100.0 };
                let mut values = [0.0f32; 64];
                for k in 0..64 {
                    values[k] = f32::from_bits(
                        (self.tensor_scratch[k*4] as u32)
                        | (self.tensor_scratch[k*4+1] as u32) << 8
                        | (self.tensor_scratch[k*4+2] as u32) << 16
                        | (self.tensor_scratch[k*4+3] as u32) << 24
                    );
                }
                let mut output = [0i8; 64];
                attractor_step(&values, threshold, &mut output);
                for k in 0..64 {
                    self.tensor_scratch[256 + k] = output[k] as u8;
                }
            }
            0xA2 => { // TPACK — pack first 64 bytes of scratchpad into u128 at offset 512
                let mut src = [0i8; 64];
                for k in 0..64 {
                    src[k] = self.tensor_scratch[k] as i8;
                }
                let packed = PackedTernary::pack(&src);
                for k in 0..16 {
                    self.tensor_scratch[512 + k] = (packed.0 >> (k * 8)) as u8;
                }
            }
            0xA3 => { // TUNPACK — unpack u128 from offset 512 into first 64 bytes
                let mut packed_val: u128 = 0;
                for k in 0..16 {
                    packed_val |= (self.tensor_scratch[512 + k] as u128) << (k * 8);
                }
                let unpacked = PackedTernary(packed_val).unpack();
                for k in 0..64 {
                    self.tensor_scratch[k] = unpacked[k] as u8;
                }
            }
            _ => {}
        }
        Ok(())
    }
}
