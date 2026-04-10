//! Bytecode encoding, decoding, and instruction formats.

mod decoder;
mod encoder;
mod opcodes;

pub use decoder::Disassembler;
pub use encoder::Assembler;
pub use opcodes::{Format, Opcode};
