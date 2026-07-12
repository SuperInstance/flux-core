# ⚡ FLUX — Rust Bytecode VM

[![crates.io](https://img.shields.io/crates/v/fluxvm)](https://crates.io/crates/fluxvm)
[![CI](https://github.com/SuperInstance/flux-core/actions/workflows/ci.yml/badge.svg)](https://github.com/SuperInstance/flux-core/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](https://opensource.org/licenses/MIT)

> **FLUX — Fluid Language Universal eXecution**
> A register-based bytecode VM for deterministic agent computation. Published on crates.io as `fluxvm`.

---

## Quick Start

```bash
cargo add fluxvm
```

```rust
use flux_core::bytecode::assembler::Assembler;
use flux_core::vm::Interpreter;

let bytecode = Assembler::assemble("MOVI R0, 42\nHALT").unwrap();
let mut vm = Interpreter::new(&bytecode);
vm.execute().unwrap();
assert_eq!(vm.read_gp(0), 42);
```

Natural language → bytecode:

```rust
use flux_core::vocabulary::Interpreter;

let interp = Interpreter::with_builtins();
assert_eq!(interp.execute("compute 6 * 7").unwrap(), 42);
assert_eq!(interp.execute("factorial of 5").unwrap(), 120);
```

---

## What It Does

FLUX is the bytecode VM that runs **inside** the shell. Not the shell itself — the mechanism. It executes deterministic programs so agents don't need to agree on what "add R1, R2" means. When an agent needs to compute something, it doesn't reach for Python or Bash — it reaches for FLUX. A compact, auditable, register-based VM that does exactly one thing: execute bytecode predictably, every time, on every agent.

Most agent frameworks interpret high-level language directly. FLUX takes a different route: it compiles natural-language intents into **bytecode**, executes them on a deterministic VM, and lets agents share results through a structured A2A protocol. This matters because:

- **Determinism** — Same bytecode, same result. Every node can verify.
- **Auditability** — Disassemble any agent's program. See exactly what it would do.
- **Sandboxing** — Cycle budgets prevent runaway execution.
- **Swarm coordination** — Built-in majority voting and message passing.

---

## Architecture

FLUX embodies **γ + η = C**: the fixed, deterministic bytecode contract (γ) plus the adaptive vocabulary and coordination layer (η) equals the complete agent execution system (C).

### Register File

```
RegisterFile {
    gp: [i32; 16],   // general purpose registers (R0–R15)
    fp: [f64; 16],   // floating point registers
    pc: u32,         // program counter
    sp: u32,         // stack pointer
    flag_zero: bool, // set by CMP
    flag_sign: bool, // set by CMP
}
```

16 general-purpose registers, 16 floating-point registers, a program counter, stack pointer, and two condition flags. Simple. Familiar. Predictable.

### Instruction Set (0x00–0x81)

| Category | Opcodes | Description |
|----------|---------|-------------|
| **Arithmetic** | 0x08–0x0F | IADD, ISUB, IMUL, IDIV, IMOD, INEG, INC, DEC |
| **Logic** | 0x10–0x15 | IAND, IOR, IXOR, INOT, ISHL, ISHR |
| **Control Flow** | 0x04–0x07 | JMP, JZ, JNZ, CALL |
| **Stack** | 0x20–0x22, 0x28 | PUSH, POP, DUP, RET |
| **Memory** | 0x01, 0x2B | MOV, MOVI (immediate) |
| **Comparison** | 0x2D | CMP (sets zero/sign flags) |
| **A2A** | 0x60–0x66 | TELL, ASK, DELEGATE, BROADCAST |
| **System** | 0x80, 0x81 | HALT, YIELD |

Single-byte opcodes. 1–4 byte instructions. O(1) per instruction fetch-decode-execute.

### A2A Protocol

Messages carry: `| sender 16B | receiver 16B | conv_id 16B | type 1B | len 2B | payload | trust 4B |`. UUID-paired agents with floating-point trust scores [0, 1].

### Swarm Consensus

Run N agents for one tick each. Majority vote on a register value — O(N) per tick, verifiable, shell-agnostic.

FLUX is the **Rust implementation** in the SuperInstance ecosystem — the fastest FLUX VM, used where determinism and performance matter most. Same bytecode ISA runs across [Python](https://github.com/SuperInstance/flux-runtime), [JavaScript](https://github.com/SuperInstance/flux-js), [C](https://github.com/SuperInstance/flux-runtime-c), [Zig](https://github.com/SuperInstance/flux-zig), [Go](https://github.com/SuperInstance/flux-swarm), and more.

---

## API / Usage

### Module Overview

| Module | Key Types | Purpose |
|--------|-----------|---------|
| `vm` | `Interpreter`, `RegisterFile` | Bytecode execution |
| `bytecode` | `Op`, `Assembler`, `Disassembler` | Encode/decode instructions |
| `vocabulary` | `VocabEntry`, `Vocabulary`, `Interpreter` | NL pattern → assembly |
| `a2a` | `A2AMessage`, `Agent`, `Swarm` | Agent protocol |
| `error` | `FluxError` | All error variants |

### Assembly Syntax

```
loop:
    CMP R1, 0       ; compare R1 to zero
    JZ end           ; jump if zero
    IMUL R0, R1     ; R0 *= R1
    DEC R1           ; R1 -= 1
    JMP loop         ; back to top
end:
    HALT
```

---

## Testing

```bash
cargo test
# 51 tests covering: VM execution, assembler, disassembler,
# vocabulary patterns, A2A messaging, swarm consensus
```

---

## Contributing

Contributions are welcome! See the [SuperInstance Contributing Guide](https://github.com/SuperInstance/SuperInstance/blob/main/CONTRIBUTING.md).

1. Fork the repo
2. Create a feature branch
3. Add tests for new functionality
4. Ensure `cargo test` passes
5. Submit a PR

---

## 📦 Related Packages

| Package | Language | Registry | Install |
|---------|----------|----------|---------|
| **[flux-vm](https://pypi.org/project/flux-vm/)** | Python | PyPI | `pip install flux-vm` |
| **[fluxvm](https://crates.io/crates/fluxvm)** | Rust | crates.io | `cargo add fluxvm` |
| **[flux-js](https://www.npmjs.com/package/flux-js)** | JavaScript | npm | `npm install flux-js` |

Additional implementations: [C](https://github.com/SuperInstance/flux-runtime-c) · [Zig](https://github.com/SuperInstance/flux-zig) · [Go](https://github.com/SuperInstance/flux-swarm) · [Java](https://github.com/SuperInstance/flux-java) · [WASM](https://github.com/SuperInstance/flux-wasm) · [CUDA](https://github.com/SuperInstance/flux-cuda)

---

## Ecosystem

This repo is part of the **SuperInstance** flagship ecosystem — agent-first computation, constraint theory, and self-improving runtimes.

### FLUX Runtime Family

| Repo | Language | Description |
|------|----------|-------------|
| [flux-runtime](https://github.com/SuperInstance/flux-runtime) | Python | Full FLUX runtime: markdown→bytecode, 2037 tests, zero deps |
| [flux-core](https://github.com/SuperInstance/flux-core) | Rust | Register-based bytecode VM, deterministic agent computation |
| [flux-js](https://github.com/SuperInstance/flux-js) | JavaScript | FLUX VM for Node.js and browsers, ~400ns/iter |
| [flux-compiler](https://github.com/SuperInstance/flux-compiler) | Rust/Python | Formal-methods compiler for safety-critical codegen |
| [flux-vm](https://github.com/SuperInstance/flux-vm) | Rust | Stack-based constraint-checking VM, 50 opcodes, Turing-incomplete |

### PLATO Engine Family

| Repo | Language | Description |
|------|----------|-------------|
| [plato-server](https://github.com/SuperInstance/plato-server) | Python | Knowledge tiles, fleet sync via Matrix, HTTP API |
| [plato-engine-block](https://github.com/SuperInstance/plato-engine-block) | Rust | Original room runtime: no_std + alloc, builder pattern |
| [plato-engine-block-c](https://github.com/SuperInstance/plato-engine-block-c) | C99 | Embedded reference: zero heap alloc, bare-metal portable |
| [plato-engine-block-elixir](https://github.com/SuperInstance/plato-engine-block-elixir) | Elixir | BEAM supervision trees, fault tolerance, hot reload |
| [plato-runtime-kernel](https://github.com/SuperInstance/plato-runtime-kernel) | Rust | Spatial model: tensor grid, batons, assertion traps |

### Constraint / Theory Family

| Repo | Language | Description |
|------|----------|-------------|
| [categorical-agents](https://github.com/SuperInstance/categorical-agents) | Rust | Category theory for agent composition (functors, naturality) |
| [cuda-constraint-engine](https://github.com/SuperInstance/cuda-constraint-engine) | CUDA/C | GPU constraint checking at 1B+ constraints/sec |
| [grand-pattern-rs](https://github.com/SuperInstance/grand-pattern-rs) | Rust | Fibonacci dual-direction cellular graph architecture |
| [lau-hodge-theory](https://github.com/SuperInstance/lau-hodge-theory) | Rust | Hodge decomposition, Betti numbers, spectral sequences |
| [ternary-science](https://github.com/SuperInstance/ternary-science) | Rust | Experimental evidence for ternary intelligence, 5 conservation laws |

### Agent / Infrastructure Family

| Repo | Language | Description |
|------|----------|-------------|
| [construct-core](https://github.com/SuperInstance/construct-core) | Rust | Layered trait system: bare-metal → alloc → async agent runtime |
| [crab](https://github.com/SuperInstance/crab) | Bash | Agent shell for repo entry/leave (MUD-room metaphor) |
| [exocortex](https://github.com/SuperInstance/exocortex) | Rust | Persistent cognitive substrate, S3-compatible memory |
| [git-agent](https://github.com/SuperInstance/git-agent) | Python | The repo IS the agent — autonomous lifecycle via Git |
| [capitaine-1](https://github.com/SuperInstance/capitaine-1) | TypeScript | Git-native repo-agent, Cloudflare Workers heartbeat |
| [codespace-edge-rd](https://github.com/SuperInstance/codespace-edge-rd) | Research | Codespace→Edge agent lifecycle and yoke transfer protocols |
| [git-agent-codespace](https://github.com/SuperInstance/git-agent-codespace) | DevContainer | One-click Codespace template for Git-Agent runtimes |

### Registries

| Registry | Package | Install |
|----------|---------|---------|
| **PyPI** | `flux-vm` | `pip install flux-vm` |
| **crates.io** | `fluxvm` | `cargo add fluxvm` |
| **npm** | `flux-js` | `npm install flux-js` |

### Philosophy & Architecture

- 📖 [AI-Writings](https://github.com/SuperInstance/AI-Writings) — Philosophy, essays, and design rationale
- 📦 [PACKAGES.md](https://github.com/SuperInstance/SuperInstance/blob/main/PACKAGES.md) — Full package index

---

## License

MIT

---

> *The crab inherits the shell.* 🦀
