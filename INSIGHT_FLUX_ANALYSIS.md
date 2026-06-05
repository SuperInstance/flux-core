# Flux-Core Deep Analysis

> Kimi Code deep analysis of flux-core: VM architecture, bytecode format,
> A2A protocol, and integration paths to the Flux-to-PTX compilation pipeline.

---

# Deep Analysis: flux-core & ternary-flux

> Generated from complete source-code audit of both repositories.
> Date: 2026-06-05

---

## Table of Contents

1. [flux-core Architecture Overview](#1-flux-core-architecture-overview)
2. [VM Architecture Diagram](#2-vm-architecture-diagram)
3. [Complete Opcode Table](#3-complete-opcode-table)
4. [Instruction Formats](#4-instruction-formats)
5. [Assembler Deep Dive](#5-assembler-deep-dive)
6. [Disassembler Deep Dive](#6-disassembler-deep-dive)
7. [A2A Protocol Specification](#7-a2a-protocol-specification)
8. [Vocabulary System](#8-vocabulary-system)
9. [ternary-flux State-Flow Engine](#9-ternary-flux-state-flow-engine)
10. [Bytecode → GPU Operation Mapping](#10-bytecode--gpu-operation-mapping)
11. [A2A Distributed Compilation Model](#11-a2a-distributed-compilation-model)
12. [GPU Extensions Required](#12-gpu-extensions-required)

---

## 1. flux-core Architecture Overview

flux-core is implemented in **two parallel forms** found in the repository:

| Variant | Location | Characteristics |
|---------|----------|-----------------|
| **v1 (std)** | `flux-core/src/` | Standard Rust, depends on `regex`, vocabulary support, A2A swarm agents |
| **v2 (no_std)** | `flux-core/flux-core/src/` | `#![no_std]`, zero dependencies, SIMD registers, linear memory, formal instruction formats |

Both implement the same FLUX ISA (Fluid Language Universal eXecution).

### Core Components

```
flux-core
├── vm/
│   ├── interpreter.rs   # Main execution engine
│   ├── registers.rs     # Register file (GP + FP + SIMD)
│   └── memory.rs        # Linear memory (code/data/heap/stack segments)
├── bytecode/
│   ├── opcodes.rs       # Opcode enum + Format enum
│   ├── assembler.rs     # Text → bytecode (two-pass)
│   └── disassembler.rs  # Bytecode → text
├── a2a/
│   ├── messages.rs      # A2AMessage serialization
│   └── swarm.rs         # Agent + Swarm orchestration (v1 only)
├── vocabulary/
│   └── interpreter.rs   # Natural-language → assembly → bytecode
└── error.rs             # FluxError / Error enums
```

### Register Architecture (v2 — canonical)

| Register Bank | Count | Width | Purpose |
|---------------|-------|-------|---------|
| GP (R0-R15) | 16 | 32-bit signed | Integer arithmetic, addresses, general use |
| FP (F0-F15) | 16 | 64-bit IEEE-754 | Floating-point operations |
| SIMD (V0-V15) | 16 | 128-bit | Vector operations (4×i32, 4×f32, 16×i8) |
| PC | 1 | 32-bit | Program counter |
| SP | 1 | 32-bit | Stack pointer |
| FP_reg | 1 | 32-bit | Frame pointer |
| LR | 1 | 32-bit | Link register (return address) |
| Flags | 1 | 3 booleans | Zero, Sign, Carry |

### Memory Layout (v2)

```
+---------------------------------- High Address (64 KB default)
|  Stack Segment  (grows downward)
|         ↓
+---------------------------------- stack_bottom = size
|  Heap Segment   (grows upward)
|         ↑
+---------------------------------- data_start = size / 4
|  Data Segment   (global data)
+---------------------------------- code_start = 0
|  Code Segment   (bytecode, read-only)
+---------------------------------- Low Address (0)
```

- Default size: **64 KB**
- Page size: **4 KB**
- Maximum size: **16 MB**
- Page-aligned allocations required

### Execution Model

- **Register-based** VM (not stack-based)
- Little-endian byte ordering throughout
- Fetch-decode-execute cycle with explicit `step()` and `run()` methods
- Cycle budget: default 10,000,000 (v1), configurable `max_instructions` (v2)
- Stack stored as raw bytes (i32 values serialized to 4 LE bytes each)
- HALT returns `Err(Error::Halted)` internally, mapped to `Ok(())` at `run()` boundary

---

## 2. VM Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           FLUX VIRTUAL MACHINE                               │
├─────────────────────────────────────────────────────────────────────────────┤
│  REGISTER FILE                    │  LINEAR MEMORY (64 KB - 16 MB)          │
│  ┌─────────────┐                  │  ┌─────────────────────────────────┐    │
│  │ GP[0..15]   │ i32              │  │ Code Segment (bottom 1/4)       │    │
│  │ FP[0..15]   │ f64              │  │  • Bytecode loaded at addr 0    │    │
│  │ SIMD[0..15] │ 128-bit          │  └─────────────────────────────────┘    │
│  │ PC          │ u32              │  ┌─────────────────────────────────┐    │
│  │ SP          │ u32              │  │ Data Segment (next 1/4)         │    │
│  │ FP          │ u32              │  │  • Global variables             │    │
│  │ LR          │ u32              │  │  • Heap grows upward            │    │
│  │ Flags(Z,S,C)│ bool×3           │  └─────────────────────────────────┘    │
│  └─────────────┘                  │  ┌─────────────────────────────────┐    │
│                                   │  │ Stack Segment (top half)        │    │
│                                   │  │  • Grows downward from top      │    │
│                                   │  └─────────────────────────────────┘    │
├───────────────────────────────────┼─────────────────────────────────────────┤
│  EXECUTION ENGINE                                                           │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌──────────────┐ │
│  │   FETCH     │───→│   DECODE    │───→│   EXECUTE   │───→│  STATE UPDATE│ │
│  │  memory[PC] │    │ Opcode::    │    │ Format-disp │    │  PC+=len     │ │
│  │  1-4 bytes  │    │ from_u8()   │    │ match fmt   │    │  Flags,Regs  │ │
│  └─────────────┘    └─────────────┘    └─────────────┘    └──────────────┘ │
├─────────────────────────────────────────────────────────────────────────────┤
│  A2A MESSAGE QUEUE         │  STACK (8 KB max)                              │
│  ┌─────────────────────┐   │  ┌─────────────────────────────────────────┐   │
│  │ sent_messages: Vec  │   │  │ Raw bytes (i32 values in LE format)     │   │
│  │ received_messages   │   │  │ PUSH = extend 4 bytes                   │   │
│  └─────────────────────┘   │  │ POP  = truncate last 4 bytes            │   │
│                            │  └─────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 3. Complete Opcode Table

FLUX defines **30 opcodes** across 6 instruction formats. All values are `u8`.

| Hex | Name | Category | Format | Description |
|-----|------|----------|--------|-------------|
| `0x00` | **NOP** | Control | A | No operation |
| `0x01` | **MOV** | Data | C | `Rd = Rs1` (register copy) |
| `0x02` | **LOAD** | Memory | C | `Rd = memory[Rs1]` (i32 load) |
| `0x03` | **STORE** | Memory | C | `memory[Rd] = Rs1` (i32 store) |
| `0x04` | **JMP** | Control | D | `PC += imm16` (unconditional) |
| `0x05` | **JZ** | Control | D | If `Flags.zero`, `PC += imm16` |
| `0x06` | **JNZ** | Control | D | If `!Flags.zero`, `PC += imm16` |
| `0x07` | **CALL** | Control | D | Push PC, `PC += imm16` |
| `0x08` | **IADD** | Integer | E | `Rd = Rs1 + Rs2` (wrapping) |
| `0x09` | **ISUB** | Integer | E | `Rd = Rs1 - Rs2` (wrapping) |
| `0x0A` | **IMUL** | Integer | E | `Rd = Rs1 * Rs2` (wrapping) |
| `0x0B` | **IDIV** | Integer | E | `Rd = Rs1 / Rs2` (trap on div0) |
| `0x0C` | **IMOD** | Integer | E | `Rd = Rs1 % Rs2` (trap on div0) |
| `0x0D` | **INEG** | Integer | B | `Rd = -Rd` |
| `0x0E` | **INC** | Integer | B | `Rd = Rd + 1` (wrapping) |
| `0x0F` | **DEC** | Integer | B | `Rd = Rd - 1` (wrapping) |
| `0x10` | **IAND** | Bitwise | E | `Rd = Rs1 & Rs2` |
| `0x11` | **IOR** | Bitwise | E | `Rd = Rs1 \| Rs2` |
| `0x12` | **IXOR** | Bitwise | E | `Rd = Rs1 ^ Rs2` |
| `0x13` | **INOT** | Bitwise | B | `Rd = !Rd` (bitwise NOT) |
| `0x14` | **ISHL** | Bitwise | E | `Rd = Rs1 << Rs2` |
| `0x15` | **ISHR** | Bitwise | E | `Rd = Rs1 >> Rs2` (logical) |
| `0x20` | **PUSH** | Stack | B | Push `Rd` onto stack |
| `0x21` | **POP** | Stack | B | Pop into `Rd` from stack |
| `0x22` | **DUP** | Stack | A | Duplicate top 4 bytes of stack |
| `0x28` | **RET** | Control | A | Pop return address into PC |
| `0x2B` | **MOVI** | Data | D | `Rd = imm16` (sign-extended to i32) |
| `0x2D` | **CMP** | Control | C | Compare `Rd` and `Rs1`, update Flags |
| `0x40` | **FADD** | Float | E | `Fd = Fs1 + Fs2` |
| `0x41` | **FSUB** | Float | E | `Fd = Fs1 - Fs2` |
| `0x42` | **FMUL** | Float | E | `Fd = Fs1 * Fs2` |
| `0x43` | **FDIV** | Float | E | `Fd = Fs1 / Fs2` (trap on div0) |
| `0x60` | **TELL** | A2A | G | Send Tell message |
| `0x61` | **ASK** | A2A | G | Send Ask (request-response) |
| `0x62` | **DELEGATE** | A2A | G | Send Delegate (task offload) |
| `0x66` | **BROADCAST** | A2A | G | Send Broadcast (one-to-many) |
| `0x80` | **HALT** | Control | A | Stop execution |
| `0x81` | **YIELD** | Control | A | Yield execution (cooperative) |

### Opcode Value Ranges

```
0x00-0x15  : Core integer / memory / control
0x20-0x22  : Stack operations
0x28       : Return
0x2B, 0x2D : Immediate move, compare
0x40-0x43  : Floating-point arithmetic
0x60-0x66  : A2A messaging
0x80-0x81  : Execution control
```

> **Note:** The ISA has large gaps reserved for expansion. The canonical spec (per TASKS.md) targets **247 opcodes**, indicating this is a minimal core.

---

## 4. Instruction Formats

All instructions use **little-endian** encoding.

### Format A — Zero-operand (1 byte)

```
[ opcode ]
```

Instructions: `NOP`, `HALT`, `YIELD`, `DUP`, `RET`

### Format B — Single register (2 bytes)

```
[ opcode ][ rd ]
```

Instructions: `INC`, `DEC`, `PUSH`, `POP`, `INEG`, `INOT`

### Format C — Two registers (3 bytes)

```
[ opcode ][ rd ][ rs1 ]
```

Instructions: `MOV`, `LOAD`, `STORE`, `CMP`

### Format D — Register + Immediate (4 bytes)

```
[ opcode ][ rd ][ imm16_lo ][ imm16_hi ]
```

- `imm16` is a signed 16-bit integer (`i16`), little-endian
- For `JMP`, `JZ`, `JNZ`, `CALL`: `rd` field is present but often ignored / set to 0
- Branch offset is **relative to current PC** after instruction fetch

Instructions: `MOVI`, `JMP`, `JZ`, `JNZ`, `CALL`

### Format E — Three registers (4 bytes)

```
[ opcode ][ rd ][ rs1 ][ rs2 ]
```

Instructions: `IADD`, `ISUB`, `IMUL`, `IDIV`, `IMOD`, `IAND`, `IOR`, `IXOR`, `ISHL`, `ISHR`, `FADD`, `FSUB`, `FMUL`, `FDIV`

### Format G — Variable-length (3+ bytes)

```
[ opcode ][ length: u16_le ][ data... ]
```

- `length` = payload size in bytes
- Total instruction size = `3 + length`

Instructions: `TELL`, `ASK`, `DELEGATE`, `BROADCAST`

---

## 5. Assembler Deep Dive

### v1 Assembler (`src/bytecode/assembler.rs`)

- **Two-pass assembly**: Pass 1 collects labels and computes sizes; Pass 2 emits bytecode
- **Label syntax**: `label:` or inline `label: instruction`
- **Comments**: `;` line comments
- **Registers**: `R0`-`R15` (integer), `F0`-`F15` (float), `V0`-`V15` (SIMD in v2)
- **Fixups**: Branch targets stored as `(patch_pos, instr_end, label)`, resolved at end
- **Branch encoding**: PC-relative offset as `i16`

```rust
// Example assembly
MOVI R0, 0
MOVI R1, 10
loop:
IADD R0, R1      // v1: 2-register form (Rd=dest, Rs1=src, result in Rd)
DEC R1
JNZ R1, loop     // label resolved to relative offset
HALT
```

### v2 Assembler (`flux-core/src/bytecode/encoder.rs`)

- Also two-pass with label resolution
- Supports `//` comments in addition to `;`
- **Three-register** form for arithmetic: `IADD Rd, Rs1, Rs2`
- Hex immediate parsing: `MOVI R0, 0xFF`
- Label references with `@` prefix: `JMP @label`
- Validates register bounds (0-15)

### Assembler Limitations

1. No macro expansion
2. No constant definitions
3. No data segment directives (`.word`, `.byte`)
4. Format G (A2A) emits placeholder `[opcode, 0, 0]` only
5. No floating-point immediate loading (must use integer MOVI + reinterpret)

---

## 6. Disassembler Deep Dive

### v1 Disassembler (`src/bytecode/disassembler.rs`)

- Returns `Vec<DisassembledInstruction>` with `offset`, `opcode`, `text`, `size`
- Handles truncated instructions gracefully (marks as `(truncated)`)
- Register prefix inferred from opcode (all integer ops shown as `R`)

### v2 Disassembler (`flux-core/src/bytecode/decoder.rs`)

- Returns human-readable `String` with configurable output
- **Options**:
  - `show_addresses`: prepend `0000: ` hex offset
  - `show_bytes`: append raw hex bytes
  - `minimal()`: mnemonics only
- Provides `get_instruction_boundaries()` for control-flow analysis
- Provides `disassemble_at()` for single-instruction lookup
- Float operations automatically use `F` prefix in output

### Disassembler Output Example

```
0000: 2B 00 2A 00    MOVI R0, 42
0004: 2B 01 14 00    MOVI R1, 20
0008: 08 00 01 02    IADD R0, R1, R2
000C: 80             HALT
```

---

## 7. A2A Protocol Specification

### Message Types

| Value | Name | Semantics |
|-------|------|-----------|
| `1` | **Tell** | One-way fire-and-forget message |
| `2` | **Ask** | Request-response (synchronous query) |
| `3` | **Delegate** | Task offloading to another agent |
| `4` | **Broadcast** | One-to-many message dispatch |

### Wire Format (v2 — canonical / big-endian)

Total minimum size: **63 bytes** (empty payload)

```
Offset   Size    Field                Encoding
─────────────────────────────────────────────────────
0        16      sender UUID          raw bytes
16       16      receiver UUID        raw bytes
32       16      conversation_id      raw bytes
48       1       message_type         u8 (1-4)
49       2       payload_length       u16 BE
51       N       payload              raw bytes
51+N     4       trust_score          f32 BE
55+N     8       timestamp            u64 BE
─────────────────────────────────────────────────────
Total    63+N
```

### Wire Format (v1 — legacy / little-endian)

```
Offset   Size    Field                Encoding
─────────────────────────────────────────────────────
0        16      sender UUID          raw bytes
16       16      receiver UUID        raw bytes
32       16      conversation_id      raw bytes
48       1       message_type         u8 (1-4)
49       2       payload_length       u16 LE
51       N       payload              raw bytes
51+N     4       trust_score          f32 LE
─────────────────────────────────────────────────────
Total    55+N
```

> ⚠️ **Critical difference**: v2 uses **big-endian** for multi-byte fields; v1 uses **little-endian**. The two are wire-incompatible for payloads > 255 bytes or non-zero trust scores.

### Trust Score

- Range: `0.0` to `1.0`
- Default: `1.0` (fully trusted)
- Used for swarm consensus and delegation decisions

### A2A Opcodes in Bytecode

When the VM encounters `TELL`, `ASK`, `DELEGATE`, or `BROADCAST`:

1. Fetch `length: u16` from bytecode
2. Read `length` bytes of payload from memory
3. Construct `A2AMessage` with placeholder sender/receiver (`[0u8; 16]` / `[1u8; 16]`)
4. Push to `sent_messages` queue
5. Continue execution (non-blocking)

> Current implementation is **placeholder**: real sender/receiver IDs would be read from registers in a full implementation.

### Swarm Model (v1 only)

```rust
pub struct Swarm {
    pub agents: HashMap<String, Agent>,
}

pub struct Agent {
    pub id: String,
    pub role: String,
    pub trust: f32,
    pub inbox: Vec<A2AMessage>,
    pub generation: u32,
    bytecode: Vec<u8>,
}
```

**Swarm Operations:**
- `tick()`: Execute all agents one step, sum cycles
- `vote(reg)`: Count value frequencies across agents at register
- `consensus(reg)`: Return majority value (Byzantine fault tolerance primitive)

---

## 8. Vocabulary System

The vocabulary system (v1 only) bridges **natural language → FLUX assembly → bytecode → execution**.

### Architecture

```
Natural Language Input
        ↓
   Regex Pattern Match (VocabEntry.pattern)
        ↓
   Capture Group Substitution (VocabEntry.assembly_template)
        ↓
   FLUX Assembly Text
        ↓
   Assembler::assemble()
        ↓
   Bytecode
        ↓
   VM Interpreter::execute()
        ↓
   Result (from VocabEntry.result_reg)
```

### Built-in Vocabulary (v1)

| Pattern | Assembly Template | Result Reg |
|---------|-------------------|------------|
| `compute (\d+) \+ (\d+)` | `MOVI R0, {0}\nMOVI R1, {1}\nIADD R0, R1\nHALT` | R0 |
| `compute (\d+) \* (\d+)` | `MOVI R0, {0}\nMOVI R1, {1}\nIMUL R0, R1\nHALT` | R0 |
| `factorial of (\d+)` | Loop with `IMUL`, `DEC`, `JNZ` | R0 |
| `hello` | `MOVI R0, 42\nHALT` | R0 |

### Extensibility

```rust
let mut vocab = Vocabulary::new();
vocab.add_entry(VocabEntry::new(
    r#"square\s+(\d+)"#,
    "MOVI R0, {0}\nMOVI R1, {0}\nIMUL R0, R0, R1\nHALT",
    0,
    "square"
));
```

### Design Philosophy

- Each `VocabEntry` is a **regex-to-assembly** mapping
- Capture groups `{0}`, `{1}`, ... substituted into template
- No type checking — inputs flow directly as immediates
- Result register configurable per entry
- Enables **agent specialization**: different agents load different vocabularies

---

## 9. ternary-flux State-Flow Engine

ternary-flux provides a **dataflow graph** engine using balanced ternary values `{-1, 0, +1}`.

### Core Types

```rust
pub enum Ternary {
    Negative,  // -1
    Zero,      //  0
    Positive,  // +1
}
```

### FluxNode

- **Transform table**: `[Ternary; 3]` mapping `[-1, 0, +1]` input → output
- **Identity**: `[-1, 0, +1]` (passthrough)
- **Inverter**: `[+1, 0, -1]` (negation)
- **Constant**: `[c, c, c]` (ignores input)

### FluxGraph

- Directed graph of `FluxNode`s connected by `FluxEdge`s
- **Weighted edges**: `Ternary` weight modulates flow (ternary multiplication)
- **Topological evaluation**: Kahn's algorithm for DAG ordering
- **Cycle detection**: Returns `None` from `topological_order()`

### Evaluation Semantics

```
For each node in topological order:
    incoming = all edges (from → to=node)
    sum = Σ(ternary_multiply(source_value, edge_weight)).to_i8()
    clamped = sum.clamp(-1, 1)
    node.evaluate(Ternary::from_i8(clamped))
```

### FluxCompiler

Compiles a `FluxGraph` into a flat `CompiledFlux` execution plan:

```rust
pub struct ExecutionStep {
    pub node_id: String,
    pub inputs: Vec<(String, Ternary)>, // (source, weight)
}

pub struct CompiledFlux {
    pub steps: Vec<ExecutionStep>,
    pub input_ids: Vec<String>,
    pub output_ids: Vec<String>,
}
```

This enables **O(n)** sequential execution without graph traversal overhead.

### FluxObserver

- Tracks per-edge flow history
- Computes **dominant value** (mode)
- Detects **anomalies** (values below threshold frequency)
- Enables runtime monitoring and trust scoring

### FluxBalancer

- Enforces conservation: `sum(inputs) ≈ sum(outputs)` in ternary space
- `balance_outputs()`: Distributes input sum equally across outputs
- `conservation_error()`: Clamped difference between input and output sums

### Connection to flux-core

The ternary-flux engine can be **embedded as a compilation target**:

```
FLUX Bytecode (flux-core VM)
        ↓
   ternary-flux compiler
        ↓
   CompiledFlux execution plan
        ↓
   Sequential evaluation (no branching)
```

This is particularly relevant for **GPU batch execution** where control flow divergence is expensive.

---

## 10. Bytecode → GPU Operation Mapping

### Current State

flux-core has **no GPU backend**. The mapping below is an architectural analysis of how the existing bytecode *would* map to GPU concepts.

### VM → GPU Translation Table

| FLUX Concept | GPU / CUDA Equivalent | Notes |
|--------------|----------------------|-------|
| `Interpreter` instance | CUDA thread / OpenCL work-item | One VM per thread for MIMD |
| `Swarm` of agents | CUDA thread block / warp | 32 threads = 1 warp executing same kernel |
| Bytecode program | GPU kernel code (PTX/SASS) | JIT compilation or interpreter loop |
| `gp[0..15]` | Thread-private registers | Mapped to physical GPU registers |
| `fp[0..15]` | Thread-private FP64 registers | Requires compute capability ≥ 1.3 |
| `simd[0..15]` | Vector registers (128-bit) | Maps to CUDA `int4` / `float4` |
| Linear `Memory` | Shared memory / local memory | 64 KB fits in shared memory per block |
| `LOAD` / `STORE` | `ld` / `st` instructions | Need address translation |
| `IADD`, `IMUL` | `IADD`, `IMUL` (SASS) | Native integer ALU ops |
| `FADD`, `FMUL` | `FADD`, `FMUL` (SASS) | Native FP ALU ops |
| `JMP`, `JZ`, `JNZ` | Branch instructions | **Divergence hazard** in SIMT |
| `CALL` / `RET` | Function calls / inlining | Deep call stacks problematic on GPU |
| Stack (`PUSH`/`POP`) | Local memory spills | Very expensive on GPU |
| `HALT` | Thread exit / early return | Divergent HALT is costly |
| `YIELD` | `__syncthreads()` / yield | Cooperative preemption |
| A2A messages | Inter-thread communication | Shared memory or warp shuffle |

### Execution Strategies for GPU

#### Strategy A: Interpreter Loop per Thread (MIMD)

Each CUDA thread runs the full `interpreter.execute()` loop on its own bytecode.

```cuda
__global__ void flux_interpreter_kernel(
    uint8_t* bytecode_batch,   // [num_programs][max_bytecode_size]
    int* results,              // [num_programs]
    int num_programs
) {
    int tid = blockIdx.x * blockDim.x + threadIdx.x;
    if (tid >= num_programs) return;

    uint8_t* my_code = bytecode_batch + tid * MAX_CODE_SIZE;
    FluxVM vm;  // registers, pc, stack in local memory

    while (!vm.halted) {
        uint8_t op = fetch_u8(my_code, &vm.pc);
        switch (op) { ... }  // huge dispatch table
    }
    results[tid] = vm.gp[0];
}
```

- ✅ Simple, direct port
- ❌ Massive branch divergence (each thread at different PC)
- ❌ Dispatch table causes instruction cache thrashing
- ❌ Stack in local memory = terrible performance

#### Strategy B: AOT Compilation to PTX (SIMD)

Translate FLUX bytecode to CUDA C / PTX at compile time.

```
FLUX Bytecode
    ↓ Pattern match on opcode sequences
CUDA C Kernel
    ↓ nvcc
PTX / SASS
```

Example translation:
```
MOVI R0, 10
MOVI R1, 20
IADD R0, R1, R2
HALT

→

int r0 = 10;
int r1 = 20;
int r2 = r0 + r1;
// halt = return
```

- ✅ No interpreter overhead
- ✅ Full compiler optimization
- ✅ No branch divergence within a warp
- ❌ Loses dynamic loading capability
- ❌ A2A messages need explicit CUDA IPC

#### Strategy C: Warp-Uniform Interpreter (SIMT-friendly)

Ensure all threads in a warp execute the **same bytecode program** at the **same PC**.

```cuda
__global__ void flux_uniform_kernel(
    uint8_t* shared_bytecode,  // ONE program per warp
    int* input_data,           // [num_programs] different inputs
    int* results
) {
    int tid = blockIdx.x * blockDim.x + threadIdx.x;
    int lane = threadIdx.x % 32;

    // All 32 lanes share bytecode, but have different register states
    __shared__ uint8_t code[MAX_CODE];
    // Load bytecode once per block...

    FluxVM vm;
    vm.gp[0] = input_data[tid];  // Different data per thread

    while (!vm.halted) {
        uint8_t op = code[vm.pc++];  // All lanes fetch SAME op → no divergence
        // ... execute
    }
    results[tid] = vm.gp[0];
}
```

- ✅ All threads in warp at same PC → zero control-flow divergence
- ✅ Single bytecode fetch per warp (amortized 32×)
- ✅ Needs uniform bytecode; data can vary
- ❌ Cannot support per-thread different programs

### Recommended GPU Mapping

For the **Jetson Super Orin Nano** target (1024 CUDA cores):

| Aspect | Recommendation |
|--------|----------------|
| VM per thread | Use Strategy C (warp-uniform) |
| Batch size | 1024 programs minimum to saturate GPU |
| Register file | Map to physical registers; spill to shared memory if needed |
| Stack | **Eliminate** — replace CALL/RET with inlining or tail recursion |
| A2A messages | Use `__shfl_sync()` for intra-warp, shared memory for inter-warp |
| Memory | Use shared memory (up to 164 KB on Orin) for data segment |
| Branching | Compile `JZ`/`JNZ` to predicated execution (`@P` PTX) where possible |
| SIMD ops | Map V registers to CUDA vector types (`int4`, `float4`) |

---

## 11. A2A Distributed Compilation Model

### How A2A Enables Distributed Compilation

The A2A protocol transforms the VM from a single-node executor into a **distributed compilation cluster**.

```
┌─────────────┐     TELL/ASK      ┌─────────────┐
│  Agent A    │◄─────────────────►│  Agent B    │
│ (Compiler)  │    DELEGATE       │  (Optimizer)│
└──────┬──────┘                   └──────┬──────┘
       │                                 │
       │    BROADCAST (task announce)    │
       └──────────────┬──────────────────┘
                      │
              ┌───────▼───────┐
              │   Agent C     │
              │  (Verifier)   │
              └───────────────┘
```

### Compilation Workflow

```
1. FRONTEND AGENT (has vocabulary)
   Input: "compute 5 + 3"
   Action: Matches vocabulary → produces assembly
   Message: DELEGATE(assembly_bytes) → Backend Agent

2. BACKEND AGENT (has assembler)
   Input: DELEGATE payload = assembly text
   Action: Assembler::assemble() → bytecode
   Message: TELL(bytecode) → Executor Agent

3. EXECUTOR AGENT (has VM)
   Input: TELL payload = bytecode
   Action: Interpreter::execute() → result
   Message: ASK(result) → Verifier Agent

4. VERIFIER AGENT (has reference implementation)
   Input: ASK payload = result
   Action: Compare against known-good result
   Message: BROADCAST(verification_status) → All agents
```

### Consensus for Correctness

The `Swarm::consensus(reg)` function implements **majority voting** across agent outputs:

```rust
pub fn consensus(&self, reg: u8) -> Option<i32> {
    let counts = self.vote(reg);
    counts.into_iter().max_by_key(|(_, c)| *c).map(|(v, _)| v)
}
```

This is a **Byzantine fault tolerance primitive**: if < 50% of agents are malicious/wrong, the consensus is correct.

### Trust Scoring

- Each message carries `trust_score: f32`
- Agents weight incoming messages by sender trust
- Compilation results from low-trust agents can be rejected or re-verified
- Enables **graduated delegation**: high-trust agents get harder tasks

### Distributed Compilation Protocol (Formal)

```
Message: DELEGATE
Payload Structure:
    [1 byte]  task_type: 0x01 = compile, 0x02 = optimize, 0x03 = verify
    [2 bytes] payload_length
    [N bytes] task_data (assembly text / bytecode / source)
    [4 bytes] deadline_ms

Message: TELL
Payload Structure:
    [1 byte]  result_status: 0x00 = success, 0x01 = error
    [2 bytes] payload_length
    [N bytes] result_data (bytecode / error message)

Message: ASK
Payload Structure:
    [1 byte]  query_type: 0x00 = verify, 0x01 = optimize
    [2 bytes] payload_length
    [N bytes] query_data

Message: BROADCAST
Payload Structure:
    [1 byte]  announcement_type
    [N bytes] data (reaches all agents in swarm)
```

### Scaling Model

| Agents | Role Distribution | Throughput |
|--------|-------------------|------------|
| 1 | Local compile + execute | 1 program / cycle |
| 4 | 1 frontend, 1 backend, 1 executor, 1 verifier | Pipeline parallelism |
| 16+ | Multiple backends + executors | Data parallelism |
| 100+ | Swarm with consensus | Fault-tolerant batch |

---

## 12. GPU Extensions Required

To make flux-core a viable GPU computation platform, the following additions are needed:

### A. New Opcodes for GPU Primitives

| Opcode | Hex | Format | Description |
|--------|-----|--------|-------------|
| `GMEM_LD` | `0x30` | C | Load from global GPU memory |
| `GMEM_ST` | `0x31` | C | Store to global GPU memory |
| `SMEM_LD` | `0x32` | C | Load from shared memory |
| `SMEM_ST` | `0x33` | C | Store to shared memory |
| `BARRIER` | `0x34` | A | `__syncthreads()` — block-level barrier |
| `WARP_SHFL` | `0x35` | E | Warp shuffle: `Rd = Rs1[lane=Rs2]` |
| `ATOMIC_ADD` | `0x36` | C | Atomic add to global memory |
| `LANE_ID` | `0x37` | B | `Rd = threadIdx.x % 32` |
| `BLOCK_ID` | `0x38` | B | `Rd = blockIdx.x` |
| `GRID_DIM` | `0x39` | B | `Rd = gridDim.x` |
| `VMOV` | `0x50` | E | SIMD move: `Vd = Vs1` |
| `VIADD` | `0x51` | E | SIMD integer add (4×i32) |
| `VFMUL` | `0x52` | E | SIMD float multiply (4×f32) |
| `VDOT` | `0x53` | E | SIMD dot product accumulate |

### B. Memory Model Extensions

Current memory is a flat 64 KB array. GPU memory is hierarchical:

```
┌─────────────────────────────────────┐
│  GLOBAL MEMORY (GB-scale, slow)     │ ← New: GMEM_LD / GMEM_ST
├─────────────────────────────────────┤
│  SHARED MEMORY (164 KB, fast)       │ ← New: SMEM_LD / SMEM_ST
│  Per block, user-managed            │
├─────────────────────────────────────┤
│  LOCAL MEMORY (spill, slow)         │ ← Stack currently goes here
│  Per thread, compiler-managed       │
├─────────────────────────────────────┤
│  REGISTERS (fastest, limited)       │ ← Current GP/FP/SIMD arrays
│  Per thread, 255 max on modern GPU  │
└─────────────────────────────────────┘
```

Required changes:
1. **Segmented addressing**: Add address space prefix to LOAD/STORE
2. **Shared memory allocator**: Static partition at kernel launch
3. **Coalesced access patterns**: Align memory ops to warp boundaries

### C. Threading & Execution Model

Current VM is strictly single-threaded. GPU needs:

```rust
pub struct GpuInterpreter {
    // Per-thread state (replicated across lanes)
    regs: RegisterFile,

    // Per-block state (shared across warp/block)
    shared_mem: Memory,

    // Grid-level constants (read-only)
    block_id: u32,
    thread_id: u32,
    warp_id: u32,

    // Synchronization state
    barrier_count: u32,
}
```

### D. Synchronization Primitives

| Primitive | Bytecode | GPU Mapping |
|-----------|----------|-------------|
| Block barrier | `BARRIER` | `__syncthreads()` |
| Warp barrier | `WARP_SYNC` | `__syncwarp()` |
| Atomic add | `ATOMIC_ADD` | `atomicAdd()` |
| Vote | `VOTE_ALL` / `VOTE_ANY` | `__all_sync()` / `__any_sync()` |

### E. Divergence Handling

The biggest challenge: FLUX has arbitrary `JMP`/`JZ`/`JNZ`, but GPUs use SIMT execution.

**Solutions:**

1. **Predication**: Convert short branches to predicated instructions
   ```
   JZ R0, skip
   INC R1
   skip:
   
   →
   
   SETP.EQ R0, 0        // P = (R0 == 0)
   @!P INC R1           // Only execute if P is false
   ```

2. **Warp reconvergence**: Insert reconvergence points at join labels
   - CUDA does this automatically for `if/else`
   - FLUX compiler must emit PTX structured control flow or explicit `.sync`

3. **Branch splitting**: Duplicate warps so each takes uniform path
   - Expensive but guarantees no divergence

### F. A2A on GPU

A2A messages between agents need GPU-appropriate transport:

```
Intra-warp:   Warp shuffle instructions (__shfl_sync)  ~ 1 cycle
Intra-block:  Shared memory mailbox                      ~ 10 cycles
Inter-block:  Global memory ring buffer                  ~ 100s cycles
Inter-GPU:    NVLink / PCIe                              ~ μs latency
Inter-node:   Network (RDMA)                             ~ μs-ms latency
```

Recommended A2A encoding for GPU:
- Replace 16-byte UUIDs with 32-bit lane/thread IDs
- Store messages in shared memory ring buffer
- Use warp-level voting for consensus

### G. Complete GPU-Aware Opcode Map

```
0x00-0x15  : Core ALU (unchanged)
0x20-0x22  : Stack (deprecated on GPU — use registers)
0x28       : RET
0x2B-0x2D  : MOV/CMP (unchanged)
0x30-0x39  : GPU Memory & Threading  ← NEW
0x40-0x43  : Float ALU (unchanged)
0x50-0x5F  : SIMD Vector Ops         ← NEW
0x60-0x66  : A2A Messaging (GPU-optimized)
0x70-0x7F  : Synchronization         ← NEW
0x80-0x81  : HALT / YIELD
```

### H. Implementation Priority

| Priority | Feature | Effort | Impact |
|----------|---------|--------|--------|
| P0 | AOT compiler (bytecode → CUDA C) | High | Eliminates interpreter overhead |
| P0 | Warp-uniform execution model | Medium | Removes divergence |
| P1 | GPU memory opcodes (GMEM/SMEM) | Medium | Enables real GPU memory use |
| P1 | Barrier / sync primitives | Low | Required for cooperation |
| P2 | SIMD vector opcodes | Medium | 4× throughput on vector data |
| P2 | A2A shared-memory transport | Medium | Fast inter-agent messaging |
| P3 | Predicated branch conversion | High | Full divergence elimination |
| P3 | Atomic operations | Low | Enables parallel reductions |

---

## Summary

flux-core is a **minimal but well-structured register VM** with:
- Clean 30-opcode ISA across 6 formats
- Two-pass assembler with label resolution
- Full disassembler with configurable output
- A2A messaging primitive for agent swarms
- Natural-language vocabulary bridge

ternary-flux is a **dataflow graph engine** using balanced ternary logic {-1, 0, +1}, with compilation to flat execution plans suitable for parallel execution.

**For GPU deployment**, the recommended path is:
1. **AOT compilation** of FLUX bytecode to CUDA kernels
2. **Warp-uniform execution** to avoid SIMT divergence
3. **Addition of GPU-native opcodes** for memory hierarchy and synchronization
4. **Shared-memory A2A** for fast intra-block agent communication
5. **Integration with ternary-flux compiler** to compile dataflow graphs directly to GPU kernels

The existing architecture provides solid foundations — the register file maps cleanly to GPU registers, the linear memory model can target shared memory, and the A2A protocol provides a natural abstraction for inter-thread communication.
