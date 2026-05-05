# flux-core

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![crates.io](https://img.shields.io/crates/v/flux-core.svg)](https://crates.io/crates/flux-core)

**FLUX Runtime** — Fluid Language Universal eXecution. A pure Rust crate implementing the FLUX bytecode virtual machine, assembler, disassembler, and A2A agent protocol. One dependency: `regex`.

## Quick Start

```rust
use flux_core::vm::Interpreter;
use flux_core::bytecode::opcodes::Op;

// Build bytecode: MOVI R0, 42; HALT
let bytecode = vec![Op::MOVI as u8, 0, 42, 0, Op::HALT as u8];

let mut vm = Interpreter::new(&bytecode);
vm.execute().unwrap();
assert_eq!(vm.read_gp(0), 42);
```

## Assembler

```rust
use flux_core::bytecode::assembler::Assembler;

let source = r#"
    MOVI R0, 0
    MOVI R1, 10
    loop:
    IADD R0, R1
    DEC R1
    JNZ R1, loop
    HALT
"#;

let bytecode = Assembler::assemble(source).unwrap();
let mut vm = Interpreter::new(&bytecode);
vm.execute().unwrap();
assert_eq!(vm.read_gp(0), 55); // sum 1..10
```

## A2A Protocol

```rust
use flux_core::a2a::{A2AMessage, MessageType};

let msg = A2AMessage::new(
    [1u8; 16],  // sender UUID
    [2u8; 16],  // receiver UUID
    MessageType::Tell,
    b"hello agent".to_vec(),
);

let bytes = msg.to_bytes();
let decoded = A2AMessage::from_bytes(&bytes).unwrap();
```

## Features

- **One dependency** — `regex` crate only, pure safe Rust
- **VM Interpreter** — 30+ opcodes, 16 GP + 16 FP registers
- **Assembler** — text to bytecode with label resolution
- **Disassembler** — bytecode to human-readable text
- **A2A Protocol** — Agent-to-Agent messaging (TELL, ASK, DELEGATE, BROADCAST)
- **Comprehensive tests** — VM, assembler, A2A roundtrip

## Instruction Formats

| Format | Size | Layout | Examples |
|--------|------|--------|----------|
| A | 1B | `[op]` | NOP, HALT, DUP |
| B | 2B | `[op][rd]` | INC, DEC, PUSH, POP |
| C | 3B | `[op][rd][rs1]` | IADD, IMUL, CMP, MOV |
| D | 4B | `[op][rd][imm16]` | MOVI, JMP, JZ, JNZ |
| G | var | `[op][len][data...]` | A2A messages |

## Performance

Factorial(10) benchmark: **~100ns per execution** on ARM64.

## License

MIT — SuperInstance (DiGennaro et al.)
