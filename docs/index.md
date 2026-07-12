# FLUX Core (Rust)

> **FLUX — Fluid Language Universal eXecution**
> A zero-dependency register-based bytecode VM. Published on crates.io as `fluxvm`.

[![crates.io](https://img.shields.io/crates/v/fluxvm.svg)](https://crates.io/crates/fluxvm)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)

## Overview

FLUX Core is the Rust implementation of the FLUX bytecode runtime — a deterministic, register-based VM designed for agent computation. It compiles natural-language intents into bytecode and executes them predictably on every agent.

**FLUX = γ** — the fixed, deterministic, mathematical layer. Same bytecode, same result, every time.

## Install

```bash
cargo add fluxvm
```

## Quick Start

### Bytecode Assembly & Execution

```rust
use flux_core::bytecode::assembler::Assembler;
use flux_core::vm::Interpreter;

let bytecode = Assembler::assemble("MOVI R0, 42\nHALT").unwrap();
let mut vm = Interpreter::new(&bytecode);
vm.execute().unwrap();
assert_eq!(vm.read_gp(0), 42);
```

### Natural Language → Bytecode

```rust
use flux_core::vocabulary::Interpreter;

let interp = Interpreter::with_builtins();
assert_eq!(interp.execute("compute 6 * 7").unwrap(), 42);
assert_eq!(interp.execute("factorial of 5").unwrap(), 120);
```

## Architecture

- **16 general-purpose registers** (R0–R15, i32)
- **16 floating-point registers** (F0–F15, f64)
- **Program counter, stack pointer, condition flags**
- **Cycle budgets** for sandboxed execution

## Key Features

- **Determinism** — Same bytecode, same result. Every node can verify.
- **Auditability** — Disassemble any agent's program.
- **Sandboxing** — Cycle budgets prevent runaway execution.
- **Swarm coordination** — Built-in majority voting and message passing.

## Resources

- [GitHub Repository](https://github.com/SuperInstance/flux-core)
- [crates.io](https://crates.io/crates/fluxvm)
- [FLUX Runtime (Python)](https://github.com/SuperInstance/flux-runtime) — Reference implementation
- [FLUX.js](https://github.com/SuperInstance/flux-js) — JavaScript implementation
- [SuperInstance Ecosystem](https://github.com/SuperInstance/SuperInstance)

---

*Part of the [SuperInstance](https://github.com/SuperInstance) ecosystem.*
