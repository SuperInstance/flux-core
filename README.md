# flux-core: FLUX Bytecode Runtime — Fluid Language Universal eXecution

A zero-dependency Rust implementation of a register-based virtual machine, bytecode assembler/disassembler, natural-language vocabulary layer, and agent-to-agent (A2A) communication protocol. FLUX is designed as the execution substrate for AI agent swarms: agents define behavior as bytecode, communicate via typed messages, and coordinate through consensus.

## Why It Matters

Most agent frameworks interpret high-level language directly. FLUX takes a different approach: it compiles natural-language intents into **bytecode**, executes them on a deterministic VM, and lets agents share results through a structured A2A protocol. This gives you:

- **Determinism**: Same bytecode always produces the same result
- **Auditability**: Disassemble any agent's program to inspect behavior
- **Sandboxing**: Cycle budgets prevent runaway execution
- **Swarm coordination**: Built-in majority voting and message passing

## How It Works

### Register File

The VM uses 16 general-purpose registers (R0–R15) and 16 floating-point registers, plus PC (program counter), SP (stack pointer), and zero/sign flags:

```
RegisterFile {
    gp: [i32; 16],   // general purpose
    fp: [f64; 16],   // floating point
    pc: u32,         // program counter
    sp: u32,         // stack pointer
    flag_zero: bool,
    flag_sign: bool,
}
```

### Instruction Set

The ISA uses single-byte opcodes (0x00–0x81) with 1–4 byte instructions:

| Category | Opcodes | Examples |
|----------|---------|---------|
| Arithmetic | 0x08–0x0F | `IADD`, `ISUB`, `IMUL`, `IDIV`, `IMOD`, `INEG`, `INC`, `DEC` |
| Logic | 0x10–0x15 | `IAND`, `IOR`, `IXOR`, `INOT`, `ISHL`, `ISHR` |
| Control Flow | 0x04–0x07 | `JMP`, `JZ`, `JNZ`, `CALL` |
| Stack | 0x20–0x22, 0x28 | `PUSH`, `POP`, `DUP`, `RET` |
| Memory | 0x01, 0x2B | `MOV`, `MOVI` (immediate) |
| Comparison | 0x2D | `CMP` (sets zero/sign flags) |
| A2A | 0x60–0x66 | `TELL`, `ASK`, `DELEGATE`, `BROADCAST` |
| System | 0x80, 0x81 | `HALT`, `YIELD` |

### Assembler (Two-Pass with Label Resolution)

Pass 1 computes instruction sizes and records label positions. Pass 2 emits bytecode. Jump fixups are applied after all labels are known:

```
loop:
    CMP R1, 0
    JZ R1, end       ; fixup: resolve 'end' after Pass 1
    IMUL R0, R1
    DEC R1
    JMP loop         ; fixup: resolve 'loop'
end:
    HALT
```

**Complexity**: O(n) time, O(n) space where n = source lines.

### Interpreter (Fetch-Decode-Execute)

Each cycle: fetch opcode byte → decode → execute. A cycle budget (default 10M) prevents infinite loops:

```
while !halted && cycle_count < max_cycles:
    op = fetch()
    cycle_count += 1
    match op { ... }
```

**Complexity**: O(1) per instruction, O(n) total where n = bytecode length (excluding loops).

### A2A Protocol

Messages carry sender/receiver UUIDs (16 bytes each), a conversation ID, type tag, payload, and a trust score [0, 1]. Wire format is 51+ bytes:

```
| sender 16B | receiver 16B | conv_id 16B | type 1B | len 2B | payload ... | trust 4B |
```

### Swarm Consensus

The `Swarm` runs all agents for one step, then takes a majority vote on a register value:

```
consensus(reg) = argmax_{v} |{agent : agent.result(reg) = v}|
```

**Complexity**: O(N) for N agents per tick, O(N) for vote counting.

## Quick Start

```rust
use flux_core::bytecode::assembler::Assembler;
use flux_core::vm::Interpreter;

// Write assembly
let bytecode = Assembler::assemble("MOVI R0, 42\nHALT").unwrap();

// Execute
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

## API

| Module | Key Types | Purpose |
|--------|-----------|---------|
| `vm` | `Interpreter`, `RegisterFile` | Bytecode execution |
| `bytecode` | `Op`, `Assembler`, `Disassembler` | Encode/decode instructions |
| `vocabulary` | `VocabEntry`, `Vocabulary`, `Interpreter` | NL pattern → assembly |
| `a2a` | `A2AMessage`, `Agent`, `Swarm` | Agent protocol |
| `error` | `FluxError` | All error variants |

## Architecture Notes

FLUX embodies the **γ + η = C** principle. The VM and bytecode ISA are the **γ (gamma)** — fixed, deterministic, mathematical. The vocabulary layer and A2A swarm are the **η (eta)** — the orchestration and coordination layer that adapts at runtime. Together they form **C** — a complete agent execution system where behavior is auditable (γ) and coordination is flexible (η).

The design mirrors real CPU architectures: the register file and fetch-execute loop would be familiar to anyone who has implemented a CHIP-8 or RISC-V emulator, but extended with agent-specific opcodes (`TELL`, `ASK`, `DELEGATE`, `BROADCAST`).

## References

- Tanenbaum, A. S. & Austin, T. (2013). *Structured Computer Organization* (6th ed.). Pearson.
- Smith, J. E. & Sohi, G. S. (1998). *The Microarchitecture of Superscalar Processors*. Proc. IEEE 83(12).
- Hewitt, C. (1977). *Viewing Control Structures as Patterns of Passing Messages*. AI Memo 410, MIT.

## License

MIT
