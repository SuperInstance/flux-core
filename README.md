# ⚡ FLUX — Rust Bytecode VM

> **FLUX — Fluid Language Universal eXecution**
> A register-based bytecode VM for deterministic agent computation.

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![crates.io](https://img.shields.io/crates/v/fluxvm.svg)](https://crates.io/crates/fluxvm)

FLUX is the bytecode VM that runs **inside** the shell. Not the shell itself — the mechanism. It executes deterministic programs so agents don't need to agree on what "add R1, R2" means.

Construct is the shell. FLUX is the muscle that moves within it. When an agent needs to compute something, it doesn't reach for Python or Bash — it reaches for FLUX. A compact, auditable, register-based VM that does exactly one thing: execute bytecode predictably, every time, on every agent.

**FLUX = γ** — the fixed, deterministic, mathematical layer. The instruction set that never surprises. The contract that both sides can verify.

---

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

---

## Why FLUX Exists

Most agent frameworks interpret high-level language directly. FLUX takes a different route: it compiles natural-language intents into **bytecode**, executes them on a deterministic VM, and lets agents share results through a structured A2A protocol.

This matters because:

- **Determinism** — Same bytecode, same result. Every node can verify.
- **Auditability** — Disassemble any agent's program. See exactly what it would do.
- **Sandboxing** — Cycle budgets prevent runaway execution. The shell is safe.
- **Swarm coordination** — Built-in majority voting and message passing.

FLUX doesn't replace the shell. It lives inside it. The shell is where agents persist and communicate. FLUX is where they compute.

---

## Architecture

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

Single-byte opcodes. 1–4 byte instructions. No variable-length decoding. No microcode. The VM fetches, decodes, executes — and it does so at O(1) per instruction.

### Assembler (Two-Pass)

Pass 1 computes instruction sizes and records label positions. Pass 2 emits bytecode with jump fixups. O(n) time, O(n) space. Labels resolve at assembly time — the VM never sees them.

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

### A2A Protocol

The A2A layer is the **η (eta)** — the vocabulary and coordination layer that adapts at runtime. Messages carry:

```
| sender 16B | receiver 16B | conv_id 16B | type 1B | len 2B | payload ... | trust 4B |
```

51+ bytes wire format. UUID-paired agents. Typed messages with floating-point trust scores [0, 1].

### Swarm Consensus

Run N agents for one tick each. Majority vote on a register value:

```
consensus(reg) = argmax_{v} |{agent : agent.result(reg) = v}|
```

O(N) per tick. O(N) for vote counting. Simple, verifiable, shell-agnostic.

---

## γ + η = C

FLUX embodies this equation:

| Component | Layer | Role |
|-----------|-------|------|
| **γ** (gamma) | VM + ISA | Fixed, deterministic, mathematical — the bytecode contract |
| **η** (eta) | Vocabulary + A2A | Adaptive orchestration, NL patterns, swarm coordination |
| **C** | FLUX | Complete agent execution system — auditable AND flexible |

The shell (Construct) holds both. γ provides the floor. η provides the ceiling. The agent lives between them.

---

## API

| Module | Key Types | Purpose |
|--------|-----------|---------|
| `vm` | `Interpreter`, `RegisterFile` | Bytecode execution |
| `bytecode` | `Op`, `Assembler`, `Disassembler` | Encode/decode instructions |
| `vocabulary` | `VocabEntry`, `Vocabulary`, `Interpreter` | NL pattern → assembly |
| `a2a` | `A2AMessage`, `Agent`, `Swarm` | Agent protocol |
| `error` | `FluxError` | All error variants |

---

## 📦 Related Packages

FLUX is implemented across multiple languages — same bytecode, different shells:

| Package | Language | Registry | Install |
|---------|----------|----------|---------|
| **[flux-vm](https://pypi.org/project/flux-vm/)** | Python | PyPI | `pip install flux-vm` |
| **[fluxvm](https://crates.io/crates/fluxvm)** | Rust | crates.io | `cargo add fluxvm` |
| **[flux-js](https://www.npmjs.com/package/flux-js)** | JavaScript | npm | `npm install flux-js` |
| **[flux-compiler](https://github.com/SuperInstance/flux-compiler)** | Rust/Python | GitHub | `cargo install flux-compiler` |

Additional implementations: [C](https://github.com/SuperInstance/flux-runtime-c) · [Zig](https://github.com/SuperInstance/flux-zig) · [Go](https://github.com/SuperInstance/flux-swarm) · [Java](https://github.com/SuperInstance/flux-java) · [WASM](https://github.com/SuperInstance/flux-wasm) · [CUDA](https://github.com/SuperInstance/flux-cuda)

## 🌐 Ecosystem

FLUX is part of a broader research ecosystem exploring agent-first computation:

| Project | Description |
|---------|-------------|
| [PLATO Engine Block](https://github.com/SuperInstance/plato-engine-block) | Constraint engine powering FLUX verification |
| [Constraint-Theory-Core](https://github.com/SuperInstance/Constraint-Theory) | Mathematical foundations for constraint-based computation |
| [AI-Writings](https://github.com/SuperInstance/AI-Writings) | Philosophy, essays, and design rationale behind FLUX |
| [Captain's Log](https://github.com/SuperInstance/captains-log) | Oracle1 growth diary and agent dojo curriculum |
| [Iron-to-Iron](https://github.com/SuperInstance/iron-to-iron) | I2I protocol — agents communicate through git commits |
| [flux-research](https://github.com/SuperInstance/flux-research) | 40K words: compiler taxonomy, ISA v2, agent-first design |

📖 **[Full package index →](https://github.com/SuperInstance/flux/blob/main/PACKAGES.md)**

---

## Design System

FLUX's terminal-based readouts and diagnostic surfaces follow the **Hermit Crab Power Armor** palette:

- **Bioluminescent Green (#00FF88)** — healthy execution, live state
- **Brass (#C9A84C)** — instruction encoding, opcode tables
- **Cyberpunk Magenta (#C84B8E)** — A2A messages, anomalies
- **Deep Teal (#1A4B5C)** — containment, disassembly views

Typography: JetBrains Mono for all bytecode output, Playfair Display for architecture docs.

---

## References

- Tanenbaum, A. S. & Austin, T. (2013). *Structured Computer Organization* (6th ed.).
- Smith, J. E. & Sohi, G. S. (1998). *The Microarchitecture of Superscalar Processors*.
- Hewitt, C. (1977). *Viewing Control Structures as Patterns of Passing Messages*.

---

## License

MIT

---

> *The crab inherits the shell.* 🦀

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
| **npm** | `flux-js` | `npm install flux-js` *(coming soon)* |

### Philosophy & Architecture

- 📖 [AI-Writings](https://github.com/SuperInstance/AI-Writings) — Philosophy, essays, and design rationale
- 📦 [PACKAGES.md](https://github.com/SuperInstance/SuperInstance/blob/main/PACKAGES.md) — Full package index
