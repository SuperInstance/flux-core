use crate::bytecode::opcodes::Op;

#[derive(Debug, Clone)]
pub struct DisassembledInstruction {
    pub offset: usize,
    pub opcode: Op,
    pub text: String,
    pub size: usize,
}

pub struct Disassembler;

impl Disassembler {
    pub fn disassemble(bytecode: &[u8]) -> Vec<DisassembledInstruction> {
        let mut instructions = Vec::new();
        let mut pc = 0;

        while pc < bytecode.len() {
            let offset = pc;
            let op_byte = bytecode[pc];
            let opcode = Op::from_byte(op_byte).unwrap_or(Op::NOP);
            pc += 1;

            let (text, size) = match opcode {
                Op::HALT | Op::NOP | Op::DUP | Op::YIELD => {
                    (format!("{}", opcode), 1)
                }
                Op::INC | Op::DEC | Op::PUSH | Op::POP | Op::INEG | Op::INOT => {
                    if pc < bytecode.len() {
                        let r = bytecode[pc]; pc += 1;
                        (format!("{} R{}", opcode, r), 2)
                    } else { (format!("{} (truncated)", opcode), 1) }
                }
                Op::MOVI => {
                    if pc + 2 < bytecode.len() {
                        let r = bytecode[pc]; pc += 1;
                        let imm = i16::from_le_bytes([bytecode[pc], bytecode[pc+1]]);
                        pc += 2;
                        (format!("MOVI R{}, {}", r, imm), 4)
                    } else { (format!("MOVI (truncated)"), 1) }
                }
                Op::IADD | Op::ISUB | Op::IMUL | Op::IDIV | Op::IMOD |
                Op::IAND | Op::IOR | Op::IXOR | Op::ISHL | Op::ISHR => {
                    if pc + 1 < bytecode.len() {
                        let d = bytecode[pc]; let s = bytecode[pc+1]; pc += 2;
                        (format!("{} R{}, R{}", opcode, d, s), 3)
                    } else { (format!("{} (truncated)", opcode), 1) }
                }
                Op::CMP | Op::MOV => {
                    if pc + 1 < bytecode.len() {
                        let d = bytecode[pc]; let s = bytecode[pc+1]; pc += 2;
                        (format!("{} R{}, R{}", opcode, d, s), 3)
                    } else { (format!("{} (truncated)", opcode), 1) }
                }
                Op::JZ | Op::JNZ | Op::JMP | Op::CALL => {
                    if pc + 2 < bytecode.len() {
                        let r = bytecode[pc]; pc += 1;
                        let off = i16::from_le_bytes([bytecode[pc], bytecode[pc+1]]);
                        pc += 2;
                        (format!("{} R{}, {}", opcode, r, off), 4)
                    } else { (format!("{} (truncated)", opcode), 1) }
                }
                _ => (format!("??? (0x{:02X})", op_byte), 1),
            };

            instructions.push(DisassembledInstruction { offset, opcode, text, size });
        }

        instructions
    }
}
