# flux-core

*The FLUX bytecode runtime — Fluid Language Universal eXecution. A zero-dependency Rust VM, assembler, and A2A agent protocol. The beating heart of the five-layer architecture.*

## Why This Exists

The five-layer stack (open-parallel → pincher → flux-core → cuda-oxide → cudaclaw) needs a portable intermediate representation. FLUX is it. Instead of compiling agent logic directly to GPU kernels, we compile to FLUX bytecode — a compact, deterministic, cross-platform format that can be interpreted on CPU, JIT-compiled to GPU, or serialized for network transfer.

FLUX isn't trying to be WebAssembly. It's designed for one specific use case: agent coordination logic that needs to run everywhere from an ESP32 (279 bytes of ternary lookup) to an RTX 4050 (20 SMs of parallel ternary matmul).

## Architecture

```
Agent Logic (Rust/Python/any)
         ↓ flux-core assembler
FLUX Bytecode (compact binary)
         ↓
    ┌────┴────┐
    │         │
Interpreter   JIT → PTX → GPU
(CPU debug)   (cuda-oxide)
    │         │
    └────┬────┘
         ↓
    A2A Messages (agent-to-agent)
```

### Modules

- **`vm/`** — Register-based virtual machine with 16 GP + 16 FP registers
  - `Interpreter` — Execute bytecode with cycle limits and debug hooks
  - `RegisterFile` — 16 integer + 16 float registers, zero-cost abstraction
- **`bytecode/`** — Assembly and disassembly
  - `Op` — 40+ opcodes (MOVI, ADD, MUL, CMP, JMP, CALL, RET, HALT, etc.)
  - `Assembler` — Human-readable assembly → bytecode
  - `Disassembler` — Bytecode → human-readable with offsets
- **`a2a/`** — Agent-to-agent communication protocol
  - `A2AMessage` — Tell/Ask/Offer/Ack/Fail message types
  - `Agent` — Named agent with mailbox and capability tracking
  - `Swarm` — Manage N agents with message routing
- **`vocabulary/`** — Standard library of FLUX words
- **`error`** — Unified error type

## Usage

### Interpreting Bytecode

```rust
use flux_core::vm::Interpreter;
use flux_core::bytecode::opcodes::Op;

// Compute 6 * 7 = 42
let bytecode = vec![
    Op::MOVI as u8, 0, 6, 0,  // R0 = 6
    Op::MOVI as u8, 1, 7, 0,  // R1 = 7
    Op::MUL  as u8, 0, 1, 0,  // R0 = R0 * R1
    Op::HALT as u8,
];

let mut vm = Interpreter::new(&bytecode);
vm.execute();
assert_eq!(vm.read_gp(0), 42);
```

### Assembling from Text

```rust
use flux_core::bytecode::assembler::Assembler;

let mut asm = Assembler::new();
asm.label("start");
asm.emit_mov_i(0, 42);  // MOVI R0, 42
asm.emit_halt();

let bytecode = asm.assemble().unwrap();
```

### A2A Agent Protocol

```rust
use flux_core::a2a::{Agent, Swarm, A2AMessage, MessageType};

let mut swarm = Swarm::new();
swarm.register(Agent::new("alice"));
swarm.register(Agent::new("bob"));

// Alice tells Bob something
let msg = A2AMessage::tell("alice", "bob", b"ready");
swarm.route(&msg);
```

## The Deeper Idea

FLUX is the Rosetta Stone of the SuperInstance architecture. Every layer speaks it:
- **pincher** compiles .nail files to FLUX bytecode
- **flux-core** interprets or JIT-compiles that bytecode
- **cuda-oxide** lowers FLUX to PTX for GPU execution
- **cudaclaw** wraps the GPU execution in a safe Rust API

The bytecode format is designed so that every instruction fits in 4 bytes or less. This means FLUX bytecode is cache-friendly, network-efficient, and deterministic across platforms. The same bytecode that runs on your laptop will produce the same result on a GPU cluster.

## Related Crates

- `pincher` — Compiles .nail files to FLUX bytecode
- `pincher-flux-bridge` — Bidirectional bridge between .nail format and FLUX IR
- `cuda-oxide` — FLUX → PTX lowering for GPU
- `cudaclaw` — Safe GPU execution API
- `flux-vm-dispatch` — Async dispatch for FLUX VMs
- `flux-autoscale` — Auto-scale FLUX execution across resources
