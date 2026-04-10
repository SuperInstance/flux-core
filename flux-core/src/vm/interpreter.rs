//! FLUX Virtual Machine interpreter.

use crate::a2a::{A2AMessage, MessageType};
use crate::bytecode::opcodes::{Format, Opcode};
use crate::error::{Error, Result};
use crate::vm::memory::Memory;
use crate::vm::registers::{Flags, RegisterFile};
use core::fmt;

/// Maximum stack size in bytes.
const MAX_STACK_SIZE: usize = 8 * 1024;

/// VM execution state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VmState {
    /// VM is ready to execute
    Ready,

    /// VM is currently executing
    Running,

    /// VM has halted (HALT instruction executed)
    Halted,

    /// VM has yielded (YIELD instruction executed)
    Yielded,

    /// VM encountered an error
    Error,
}

/// FLUX Virtual Machine interpreter.
///
/// The interpreter executes FLUX bytecode using a register-based architecture.
/// It supports integer and floating-point operations, stack manipulation,
/// control flow, and A2A protocol messaging.
///
/// # Examples
///
/// ```
/// use flux_core::vm::Interpreter;
///
/// let mut vm = Interpreter::new();
///
/// // Load and execute a simple program
/// let bytecode = vec![
///     0x2B, 0x00, 0x2A, 0x00, // MOVI R0, 42
///     0x80,                   // HALT
/// ];
///
/// vm.load bytecode(&bytecode).unwrap();
/// vm.run().unwrap();
///
/// assert_eq!(vm.registers().get_gp(0).unwrap(), 42);
/// ```
#[derive(Debug, Clone)]
pub struct Interpreter {
    /// Register file
    regs: RegisterFile,

    /// Linear memory
    memory: Memory,

    /// Current VM state
    state: VmState,

    /// Maximum number of instructions to execute (0 = unlimited)
    max_instructions: usize,

    /// Instructions executed so far
    instructions_executed: usize,

    /// Stack for return addresses and temporaries
    stack: Vec<u8>,

    /// A2A messages sent during execution
    sent_messages: Vec<A2AMessage>,

    /// A2A messages received during execution
    received_messages: Vec<A2AMessage>,
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

impl Interpreter {
    /// Create a new VM interpreter with default configuration.
    pub fn new() -> Self {
        Self {
            regs: RegisterFile::new(),
            memory: Memory::new(),
            state: VmState::Ready,
            max_instructions: 0,
            instructions_executed: 0,
            stack: Vec::with_capacity(MAX_STACK_SIZE),
            sent_messages: Vec::new(),
            received_messages: Vec::new(),
        }
    }

    /// Create a new interpreter with custom memory size.
    pub fn with_memory_size(size: usize) -> Self {
        let mut vm = Self::new();
        vm.memory = Memory::with_size(size);
        vm
    }

    /// Load bytecode into memory.
    ///
    /// # Errors
    ///
    /// Returns an error if the bytecode is too large for memory.
    pub fn load_bytecode(&mut self, bytecode: &[u8]) -> Result<()> {
        self.memory.load_code(bytecode)?;
        self.regs.pc = 0;
        self.state = VmState::Ready;
        Ok(())
    }

    /// Reset the VM to its initial state.
    pub fn reset(&mut self) {
        self.regs.reset();
        self.memory.clear();
        self.state = VmState::Ready;
        self.instructions_executed = 0;
        self.stack.clear();
        self.sent_messages.clear();
        self.received_messages.clear();
    }

    /// Get the current VM state.
    pub fn state(&self) -> VmState {
        self.state
    }

    /// Get a reference to the register file.
    pub fn registers(&self) -> &RegisterFile {
        &self.regs
    }

    /// Get a mutable reference to the register file.
    pub fn registers_mut(&mut self) -> &mut RegisterFile {
        &mut self.regs
    }

    /// Get a reference to memory.
    pub fn memory(&self) -> &Memory {
        &self.memory
    }

    /// Get a mutable reference to memory.
    pub fn memory_mut(&mut self) -> &mut Memory {
        &mut self.memory
    }

    /// Set the maximum number of instructions to execute.
    pub fn set_max_instructions(&mut self, max: usize) {
        self.max_instructions = max;
    }

    /// Get the number of instructions executed.
    pub fn instructions_executed(&self) -> usize {
        self.instructions_executed
    }

    /// Execute a single instruction.
    ///
    /// # Errors
    ///
    /// Returns an error if the instruction is invalid or cannot be executed.
    pub fn step(&mut self) -> Result<()> {
        if self.state == VmState::Halted {
            return Err(Error::Halted);
        }

        if self.state == VmState::Error {
            return Err(Error::InvalidInstruction);
        }

        self.state = VmState::Running;

        // Check instruction limit
        if self.max_instructions > 0 && self.instructions_executed >= self.max_instructions {
            self.state = VmState::Halted;
            return Err(Error::Halted);
        }

        // Fetch opcode
        let opcode_byte = self.fetch_u8()?;
        let opcode = Opcode::from_u8(opcode_byte)?;

        // Execute instruction
        self.execute_instruction(opcode)?;

        self.instructions_executed += 1;
        Ok(())
    }

    /// Run the program until HALT or error.
    ///
    /// # Errors
    ///
    /// Returns an error if execution fails.
    pub fn run(&mut self) -> Result<()> {
        loop {
            match self.step() {
                Ok(_) => continue,
                Err(Error::Halted) => {
                    self.state = VmState::Halted;
                    return Ok(());
                }
                Err(e) => {
                    self.state = VmState::Error;
                    return Err(e);
                }
            }
        }
    }

    /// Fetch a single byte from memory at PC.
    fn fetch_u8(&mut self) -> Result<u8> {
        let pc = self.regs.pc;
        let byte = self.memory.read_u8(pc)?;
        self.regs.pc = pc.wrapping_add(1);
        Ok(byte)
    }

    /// Fetch a 16-bit value from memory at PC.
    fn fetch_u16(&mut self) -> Result<u16> {
        let pc = self.regs.pc;
        let value = self.memory.read_u16(pc)?;
        self.regs.pc = pc.wrapping_add(2);
        Ok(value)
    }

    /// Fetch a 16-bit signed immediate from memory at PC.
    fn fetch_i16(&mut self) -> Result<i16> {
        let pc = self.regs.pc;
        let value = i16::from_le_bytes([self.memory.read_u8(pc)?, self.memory.read_u8(pc + 1)?]);
        self.regs.pc = pc.wrapping_add(2);
        Ok(value)
    }

    /// Execute an instruction.
    fn execute_instruction(&mut self, opcode: Opcode) -> Result<()> {
        match opcode {
            Opcode::NOP => {}
            Opcode::HALT => {
                self.state = VmState::Halted;
                return Err(Error::Halted);
            }
            Opcode::YIELD => {
                self.state = VmState::Yielded;
            }
            Opcode::RET => self.execute_ret()?,
            Opcode::DUP => self.execute_dup()?,
            _ => {
                let format = opcode.format();
                match format {
                    Format::A => {
                        // Already handled above
                    }
                    Format::B => self.execute_format_b(opcode)?,
                    Format::C => self.execute_format_c(opcode)?,
                    Format::D => self.execute_format_d(opcode)?,
                    Format::E => self.execute_format_e(opcode)?,
                    Format::G => self.execute_format_g(opcode)?,
                }
            }
        }
        Ok(())
    }

    /// Execute Format B instruction: [opcode][rd]
    fn execute_format_b(&mut self, opcode: Opcode) -> Result<()> {
        let rd = self.fetch_u8()?;

        match opcode {
            Opcode::INC => {
                let val = self.regs.get_gp(rd)?;
                self.regs.set_gp(rd, val.wrapping_add(1))?;
                self.regs.flags.update_from_i32(self.regs.get_gp(rd)?);
            }
            Opcode::DEC => {
                let val = self.regs.get_gp(rd)?;
                self.regs.set_gp(rd, val.wrapping_sub(1))?;
                self.regs.flags.update_from_i32(self.regs.get_gp(rd)?);
            }
            Opcode::PUSH => {
                let val = self.regs.get_gp(rd)?;
                self.stack_push(val)?;
            }
            Opcode::POP => {
                let val = self.stack_pop()?;
                self.regs.set_gp(rd, val)?;
            }
            Opcode::INEG => {
                let val = self.regs.get_gp(rd)?;
                self.regs.set_gp(rd, val.wrapping_neg())?;
                self.regs.flags.update_from_i32(self.regs.get_gp(rd)?);
            }
            Opcode::INOT => {
                let val = self.regs.get_gp(rd)?;
                self.regs.set_gp(rd, !val)?;
                self.regs.flags.update_from_i32(self.regs.get_gp(rd)?);
            }
            _ => return Err(Error::InvalidInstruction),
        }

        Ok(())
    }

    /// Execute Format C instruction: [opcode][rd][rs1]
    fn execute_format_c(&mut self, opcode: Opcode) -> Result<()> {
        let rd = self.fetch_u8()?;
        let rs1 = self.fetch_u8()?;

        match opcode {
            Opcode::MOV => {
                let val = self.regs.get_gp(rs1)?;
                self.regs.set_gp(rd, val)?;
            }
            Opcode::LOAD => {
                let addr = self.regs.get_gp(rs1)? as u32;
                let val = self.memory.read_i32(addr)?;
                self.regs.set_gp(rd, val)?;
            }
            Opcode::STORE => {
                let addr = self.regs.get_gp(rd)? as u32;
                let val = self.regs.get_gp(rs1)?;
                self.memory.write_i32(addr, val)?;
            }
            Opcode::CMP => {
                let a = self.regs.get_gp(rd)?;
                let b = self.regs.get_gp(rs1)?;
                let result = a.wrapping_sub(b);
                self.regs.flags.update_from_i32(result);
                self.regs.flags.carry = a < b; // Unsigned overflow check
            }
            _ => return Err(Error::InvalidInstruction),
        }

        Ok(())
    }

    /// Execute Format D instruction: [opcode][rd][imm16]
    fn execute_format_d(&mut self, opcode: Opcode) -> Result<()> {
        let rd = self.fetch_u8()?;
        let imm = self.fetch_i16()? as i32;

        match opcode {
            Opcode::MOVI => {
                self.regs.set_gp(rd, imm)?;
                self.regs.flags.update_from_i32(imm);
            }
            Opcode::JMP => {
                self.jmp(imm)?;
            }
            Opcode::JZ => {
                if self.regs.flags.zero {
                    self.jmp(imm)?;
                }
            }
            Opcode::JNZ => {
                if !self.regs.flags.zero {
                    self.jmp(imm)?;
                }
            }
            Opcode::CALL => {
                self.call(imm)?;
            }
            _ => return Err(Error::InvalidInstruction),
        }

        Ok(())
    }

    /// Execute Format E instruction: [opcode][rd][rs1][rs2]
    fn execute_format_e(&mut self, opcode: Opcode) -> Result<()> {
        let rd = self.fetch_u8()?;
        let rs1 = self.fetch_u8()?;
        let rs2 = self.fetch_u8()?;

        match opcode {
            // Integer operations
            Opcode::IADD => self.execute_int_op(rd, rs1, rs2, i32::wrapping_add)?,
            Opcode::ISUB => self.execute_int_op(rd, rs1, rs2, i32::wrapping_sub)?,
            Opcode::IMUL => self.execute_int_op(rd, rs1, rs2, i32::wrapping_mul)?,
            Opcode::IDIV => {
                let a = self.regs.get_gp(rs1)?;
                let b = self.regs.get_gp(rs2)?;
                if b == 0 {
                    return Err(Error::DivisionByZero);
                }
                self.regs.set_gp(rd, a.wrapping_div(b))?;
                self.regs.flags.update_from_i32(self.regs.get_gp(rd)?);
            }
            Opcode::IMOD => {
                let a = self.regs.get_gp(rs1)?;
                let b = self.regs.get_gp(rs2)?;
                if b == 0 {
                    return Err(Error::DivisionByZero);
                }
                self.regs.set_gp(rd, a.wrapping_rem(b))?;
                self.regs.flags.update_from_i32(self.regs.get_gp(rd)?);
            }
            Opcode::IAND => {
                let a = self.regs.get_gp(rs1)?;
                let b = self.regs.get_gp(rs2)?;
                self.regs.set_gp(rd, a & b)?;
                self.regs.flags.update_from_i32(self.regs.get_gp(rd)?);
            }
            Opcode::IOR => {
                let a = self.regs.get_gp(rs1)?;
                let b = self.regs.get_gp(rs2)?;
                self.regs.set_gp(rd, a | b)?;
                self.regs.flags.update_from_i32(self.regs.get_gp(rd)?);
            }
            Opcode::IXOR => {
                let a = self.regs.get_gp(rs1)?;
                let b = self.regs.get_gp(rs2)?;
                self.regs.set_gp(rd, a ^ b)?;
                self.regs.flags.update_from_i32(self.regs.get_gp(rd)?);
            }
            Opcode::ISHL => {
                let a = self.regs.get_gp(rs1)?;
                let b = self.regs.get_gp(rs2)?;
                self.regs.set_gp(rd, a.wrapping_shl(b as u32))?;
                self.regs.flags.update_from_i32(self.regs.get_gp(rd)?);
            }
            Opcode::ISHR => {
                let a = self.regs.get_gp(rs1)?;
                let b = self.regs.get_gp(rs2)?;
                self.regs.set_gp(rd, a.wrapping_shr(b as u32))?;
                self.regs.flags.update_from_i32(self.regs.get_gp(rd)?);
            }

            // Float operations
            Opcode::FADD => {
                let a = self.regs.get_fp(rs1)?;
                let b = self.regs.get_fp(rs2)?;
                self.regs.set_fp(rd, a + b)?;
            }
            Opcode::FSUB => {
                let a = self.regs.get_fp(rs1)?;
                let b = self.regs.get_fp(rs2)?;
                self.regs.set_fp(rd, a - b)?;
            }
            Opcode::FMUL => {
                let a = self.regs.get_fp(rs1)?;
                let b = self.regs.get_fp(rs2)?;
                self.regs.set_fp(rd, a * b)?;
            }
            Opcode::FDIV => {
                let a = self.regs.get_fp(rs1)?;
                let b = self.regs.get_fp(rs2)?;
                if b == 0.0 {
                    return Err(Error::DivisionByZero);
                }
                self.regs.set_fp(rd, a / b)?;
            }

            _ => return Err(Error::InvalidInstruction),
        }

        Ok(())
    }

    /// Execute Format G instruction (A2A messages)
    fn execute_format_g(&mut self, opcode: Opcode) -> Result<()> {
        let len = self.fetch_u16()? as usize;

        match opcode {
            Opcode::TELL => self.execute_a2a(MessageType::Tell, len)?,
            Opcode::ASK => self.execute_a2a(MessageType::Ask, len)?,
            Opcode::DELEGATE => self.execute_a2a(MessageType::Delegate, len)?,
            Opcode::BROADCAST => self.execute_a2a(MessageType::Broadcast, len)?,
            _ => return Err(Error::InvalidInstruction),
        }

        Ok(())
    }

    /// Execute an integer operation.
    fn execute_int_op(
        &mut self,
        rd: u8,
        rs1: u8,
        rs2: u8,
        op: fn(i32, i32) -> i32,
    ) -> Result<()> {
        let a = self.regs.get_gp(rs1)?;
        let b = self.regs.get_gp(rs2)?;
        let result = op(a, b);
        self.regs.set_gp(rd, result)?;
        self.regs.flags.update_from_i32(result);
        Ok(())
    }

    /// Execute a jump instruction.
    fn jmp(&mut self, offset: i32) -> Result<()> {
        let pc = self.regs.pc as i32;
        let new_pc = pc.wrapping_add(offset);
        if new_pc < 0 {
            return Err(Error::InvalidAddress(new_pc as u32));
        }
        self.regs.pc = new_pc as u32;
        Ok(())
    }

    /// Execute a call instruction.
    fn call(&mut self, offset: i32) -> Result<()> {
        // Push return address
        self.stack_push(self.regs.pc as i32)?;

        // Jump to target
        self.jmp(offset)?;

        Ok(())
    }

    /// Execute a return instruction.
    fn execute_ret(&mut self) -> Result<()> {
        let ret_addr = self.stack_pop()?;
        self.regs.pc = ret_addr as u32;
        Ok(())
    }

    /// Execute a DUP instruction.
    fn execute_dup(&mut self) -> Result<()> {
        if self.stack.is_empty() {
            return Err(Error::StackUnderflow);
        }

        // Get the top value without popping
        let stack_len = self.stack.len();
        let val_size = 4; // i32 size

        if stack_len < val_size {
            return Err(Error::StackUnderflow);
        }

        // Copy the top 4 bytes
        let start = stack_len - val_size;
        let top_bytes = self.stack[start..].to_vec();
        self.stack.extend_from_slice(&top_bytes);

        Ok(())
    }

    /// Execute an A2A message instruction.
    fn execute_a2a(&mut self, msg_type: MessageType, _len: usize) -> Result<()> {
        // For now, create a placeholder message
        // A full implementation would read sender/receiver IDs from registers
        let msg = A2AMessage::tell([0u8; 16], [1u8; 16], &[]);
        self.sent_messages.push(msg);
        Ok(())
    }

    /// Push a value onto the stack.
    fn stack_push(&mut self, value: i32) -> Result<()> {
        if self.stack.len() + 4 > MAX_STACK_SIZE {
            return Err(Error::StackOverflow);
        }

        let bytes = value.to_le_bytes();
        self.stack.extend_from_slice(&bytes);
        Ok(())
    }

    /// Pop a value from the stack.
    fn stack_pop(&mut self) -> Result<i32> {
        let stack_len = self.stack.len();
        if stack_len < 4 {
            return Err(Error::StackUnderflow);
        }

        let start = stack_len - 4;
        let bytes = [self.stack[start], self.stack[start + 1], self.stack[start + 2], self.stack[start + 3]];
        self.stack.truncate(start);

        Ok(i32::from_le_bytes(bytes))
    }

    /// Get the A2A messages sent during execution.
    pub fn sent_messages(&self) -> &[A2AMessage] {
        &self.sent_messages
    }

    /// Get the A2A messages received during execution.
    pub fn received_messages(&self) -> &[A2AMessage] {
        &self.received_messages
    }

    /// Get the current stack depth (in bytes).
    pub fn stack_depth(&self) -> usize {
        self.stack.len()
    }

    /// Display the current VM state for debugging.
    pub fn dump_state(&self) -> String {
        let mut output = String::new();

        output.push_str(&format!("State: {:?}\n", self.state));
        output.push_str(&format!("Instructions Executed: {}\n", self.instructions_executed));
        output.push_str(&format!("Stack Depth: {} bytes\n", self.stack_depth()));
        output.push_str(&self.regs.display_state());

        output
    }
}

impl fmt::Display for Interpreter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Interpreter(state={:?}, pc={}, instr_executed={})",
            self.state, self.regs.pc, self.instructions_executed
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vm_creation() {
        let vm = Interpreter::new();
        assert_eq!(vm.state(), VmState::Ready);
        assert_eq!(vm.instructions_executed(), 0);
    }

    #[test]
    fn test_vm_reset() {
        let mut vm = Interpreter::new();
        vm.regs.set_gp(0, 42).unwrap();
        vm.instructions_executed = 100;

        vm.reset();

        assert_eq!(vm.instructions_executed, 0);
        assert_eq!(vm.regs.get_gp(0).unwrap(), 0);
    }

    #[test]
    fn test_movi_halt() {
        let mut vm = Interpreter::new();
        let bytecode = vec![0x2B, 0x00, 0x2A, 0x00, 0x80]; // MOVI R0, 42; HALT

        vm.load_bytecode(&bytecode).unwrap();
        vm.run().unwrap();

        assert_eq!(vm.regs.get_gp(0).unwrap(), 42);
        assert_eq!(vm.state(), VmState::Halted);
    }

    #[test]
    fn test_arithmetic() {
        let mut vm = Interpreter::new();
        let bytecode = vec![
            0x2B, 0x00, 0x0A, 0x00, // MOVI R0, 10
            0x2B, 0x01, 0x14, 0x00, // MOVI R1, 20
            0x08, 0x00, 0x01, 0x02, // IADD R0, R1, R2
            0x80,                   // HALT
        ];

        vm.load_bytecode(&bytecode).unwrap();
        vm.run().unwrap();

        assert_eq!(vm.regs.get_gp(0).unwrap(), 30);
    }

    #[test]
    fn test_step() {
        let mut vm = Interpreter::new();
        let bytecode = vec![0x2B, 0x00, 0x2A, 0x00, 0x80];

        vm.load_bytecode(&bytecode).unwrap();

        vm.step().unwrap(); // MOVI
        assert_eq!(vm.regs.get_gp(0).unwrap(), 42);
        assert_eq!(vm.state(), VmState::Running);

        vm.step().unwrap(); // HALT
        assert_eq!(vm.state(), VmState::Halted);
    }

    #[test]
    fn test_stack_push_pop() {
        let mut vm = Interpreter::new();
        let bytecode = vec![
            0x2B, 0x00, 0x2A, 0x00, // MOVI R0, 42
            0x20, 0x00,             // PUSH R0
            0x2B, 0x00, 0x00, 0x00, // MOVI R0, 0
            0x21, 0x01,             // POP R1
            0x80,                   // HALT
        ];

        vm.load_bytecode(&bytecode).unwrap();
        vm.run().unwrap();

        assert_eq!(vm.regs.get_gp(0).unwrap(), 0);
        assert_eq!(vm.regs.get_gp(1).unwrap(), 42);
    }

    #[test]
    fn test_inc_dec() {
        let mut vm = Interpreter::new();
        let bytecode = vec![
            0x2B, 0x00, 0x0A, 0x00, // MOVI R0, 10
            0x0E, 0x00,             // INC R0
            0x0E, 0x00,             // INC R0
            0x0F, 0x00,             // DEC R0
            0x80,                   // HALT
        ];

        vm.load_bytecode(&bytecode).unwrap();
        vm.run().unwrap();

        assert_eq!(vm.regs.get_gp(0).unwrap(), 11);
    }

    #[test]
    fn test_cmp() {
        let mut vm = Interpreter::new();
        let bytecode = vec![
            0x2B, 0x00, 0x0A, 0x00, // MOVI R0, 10
            0x2B, 0x01, 0x0A, 0x00, // MOVI R1, 10
            0x2D, 0x00, 0x01,       // CMP R0, R1
            0x80,                   // HALT
        ];

        vm.load_bytecode(&bytecode).unwrap();
        vm.run().unwrap();

        assert!(vm.regs.flags.zero);
    }

    #[test]
    fn test_jz() {
        let mut vm = Interpreter::new();
        let bytecode = vec![
            0x2B, 0x00, 0x0A, 0x00, // MOVI R0, 10
            0x2B, 0x01, 0x0A, 0x00, // MOVI R1, 10
            0x2D, 0x00, 0x01,       // CMP R0, R1
            0x05, 0x00, 0x02, 0x00, // JZ 2 (skip to HALT)
            0x0E, 0x00,             // INC R0 (should not execute)
            0x80,                   // HALT
        ];

        vm.load_bytecode(&bytecode).unwrap();
        vm.run().unwrap();

        assert_eq!(vm.regs.get_gp(0).unwrap(), 10);
    }

    #[test]
    fn test_division_by_zero() {
        let mut vm = Interpreter::new();
        let bytecode = vec![
            0x2B, 0x00, 0x0A, 0x00, // MOVI R0, 10
            0x2B, 0x01, 0x00, 0x00, // MOVI R1, 0
            0x0B, 0x00, 0x01, 0x02, // IDIV R0, R1, R2
            0x80,                   // HALT
        ];

        vm.load_bytecode(&bytecode).unwrap();
        assert!(matches!(vm.run(), Err(Error::DivisionByZero)));
    }

    #[test]
    fn test_stack_overflow() {
        let mut vm = Interpreter::new();
        vm.stack = vec![0u8; MAX_STACK_SIZE - 4];

        let bytecode = vec![0x20, 0x00]; // PUSH R0
        vm.load_bytecode(&bytecode).unwrap();

        // This should work
        vm.step().unwrap();

        // This should overflow
        vm.step().unwrap_err();
    }

    #[test]
    fn test_float_operations() {
        let mut vm = Interpreter::new();
        let bytecode = vec![
            0x2B, 0x00, 0x00, 0x00, // MOVI R0, 0 (to FP regs - will be 0.0)
            0x80,                   // HALT
        ];

        vm.load_bytecode(&bytecode).unwrap();
        vm.run().unwrap();

        // Just test that we can write to FP registers
        vm.regs.set_fp(0, 3.14).unwrap();
        assert!((vm.regs.get_fp(0).unwrap() - 3.14).abs() < 0.001);
    }

    #[test]
    fn test_bitwise_ops() {
        let mut vm = Interpreter::new();
        let bytecode = vec![
            0x2B, 0x00, 0x0F, 0x00, // MOVI R0, 15
            0x2B, 0x01, 0x0F, 0x00, // MOVI R1, 15
            0x10, 0x02, 0x00, 0x01, // IAND R2, R0, R1
            0x11, 0x03, 0x00, 0x01, // IOR R3, R0, R1
            0x12, 0x04, 0x00, 0x01, // IXOR R4, R0, R1
            0x80,                   // HALT
        ];

        vm.load_bytecode(&bytecode).unwrap();
        vm.run().unwrap();

        assert_eq!(vm.regs.get_gp(2).unwrap(), 15); // 15 & 15
        assert_eq!(vm.regs.get_gp(3).unwrap(), 15); // 15 | 15
        assert_eq!(vm.regs.get_gp(4).unwrap(), 0);  // 15 ^ 15
    }

    #[test]
    fn test_invalid_opcode() {
        let mut vm = Interpreter::new();
        let bytecode = vec![0xFF]; // Invalid opcode

        vm.load_bytecode(&bytecode).unwrap();
        assert!(vm.run().is_err());
    }

    #[test]
    fn test_yield() {
        let mut vm = Interpreter::new();
        let bytecode = vec![0x81]; // YIELD

        vm.load_bytecode(&bytecode).unwrap();
        vm.run().unwrap();

        assert_eq!(vm.state(), VmState::Yielded);
    }

    #[test]
    fn test_max_instructions() {
        let mut vm = Interpreter::new();
        let bytecode = vec![
            0x00, // NOP
            0x00, // NOP
            0x80, // HALT
        ];

        vm.load_bytecode(&bytecode).unwrap();
        vm.set_max_instructions(2);

        vm.run().unwrap();

        assert_eq!(vm.instructions_executed(), 2);
        assert_eq!(vm.state(), VmState::Halted);
    }
}
