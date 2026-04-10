//! # flux-core
//!
//! FLUX bytecode runtime — Fluid Language Universal eXecution.
//!
//! A zero-dependency Rust implementation of the FLUX virtual machine,
//! bytecode assembler/disassembler, and A2A agent protocol.
//!
//! ## Quick Start
//!
//! ```
//! use flux_core::vm::Interpreter;
//! use flux_core::bytecode::opcodes::Op;
//!
//! // Build bytecode: MOVI R0, 42; HALT
//! let bytecode = vec![Op::MOVI as u8, 0, 42, 0, Op::HALT as u8];
//!
//! let mut vm = Interpreter::new(&bytecode);
//! vm.execute();
//! assert_eq!(vm.read_gp(0), 42);
//! ```

pub mod vm;
pub mod bytecode;
pub mod a2a;
pub mod error;
pub mod vocabulary;

pub use error::FluxError;
