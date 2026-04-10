//! # FLUX Core - FLUX Virtual Machine Runtime
//!
//! This crate provides a complete implementation of the FLUX Virtual Machine,
//! including a bytecode interpreter, assembler, disassembler, and A2A protocol support.
//!
//! ## Quick Start
//!
//! ```rust
//! use flux_core::{Interpreter, Assembler};
//!
//! // Create a VM and assemble some bytecode
//! let mut vm = Interpreter::new();
//! let mut asm = Assembler::new();
//!
//! // Assemble a simple program
//! let bytecode = asm.assemble("
//!     MOVI R0, 10
//!     MOVI R1, 20
//!     IADD R0, R1, R2
//!     HALT
//! ").unwrap();
//!
//! // Load and execute
//! vm.load_bytecode(&bytecode).unwrap();
//! vm.run().unwrap();
//!
//! // Check the result
//! assert_eq!(vm.registers().get_gp(0).unwrap(), 30);
//! ```
//!
//! ## Architecture
//!
//! The FLUX VM is a register-based virtual machine with:
//!
//! - 16 general-purpose 32-bit integer registers (R0-R15)
//! - 16 64-bit floating-point registers (F0-F15)
//! - 16 128-bit SIMD vector registers (V0-V15)
//! - Linear memory with code, data, heap, and stack segments
//! - A2A (Agent-to-Agent) protocol for distributed messaging
//!
//! ## Modules
//!
//! - [`vm`] - Virtual machine implementation
//! - [`bytecode`] - Bytecode encoding/decoding
//! - [`a2a`] - A2A protocol message types
//! - [`error`] - Error types
//!
//! ## Features
//!
//! - `std` - Use standard library (enabled by default)
//! - `alloc` - Use alloc library (for no_std environments)

#![no_std]
#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

extern crate alloc;

pub mod a2a;
pub mod bytecode;
pub mod error;
pub mod vm;

// Re-exports for convenience
pub use error::{Error, Result};

// A2A module re-exports
pub use a2a::{A2AMessage, MessageType};

// Bytecode module re-exports
pub use bytecode::{Assembler, Disassembler, Format, Opcode};

// VM module re-exports
pub use vm::{Interpreter, Memory, RegisterFile, SIMDRegister, VmState};

/// FLUX Core version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// FLUX Core crate name
pub const CRATE_NAME: &str = "flux-core";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_crate_name() {
        assert_eq!(CRATE_NAME, "flux-core");
    }
}
