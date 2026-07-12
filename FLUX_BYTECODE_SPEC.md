# FLUX Bytecode Specification

> Cross-implementation reference for Python (`flux-vm`), Rust (`flux-core`/`fluxvm`), and JS (`flux-js`).

## Instruction Formats

| Format | Layout | Description |
|--------|--------|-------------|
| A | `[opcode]` | No operands (1 byte) |
| B | `[opcode][reg:u8]` | One register (2 bytes) |
| C | `[opcode][rd:u8][rs:u8]` | Two registers (3 bytes) |
| D | `[opcode][reg:u8][off_lo:u8][off_hi:u8]` | Register + signed i16 offset (4 bytes) |
| E | `[opcode][rd:u8][rs1:u8][rs2:u8]` | Three registers (4 bytes) |
| G | `[opcode][len_lo:u8][len_hi:u8][data:len bytes]` | Variable-length (A2A protocol) |

## Opcode Table

| Hex | Mnemonic | Format | Flags? | Description |
|-----|----------|--------|--------|-------------|
| 0x00 | NOP | A | — | No operation |
| 0x01 | MOV | C | — | Copy register |
| 0x02 | LOAD | C | — | Load i32 from memory[rs1] → rd |
| 0x03 | STORE | C | — | Store rd → memory[rs1] |
| 0x04 | JMP | D | — | Unconditional relative jump |
| 0x05 | JZ | D | — | Jump if register == 0 |
| 0x06 | JNZ | D | — | Jump if register != 0 |
| 0x07 | CALL | D | — | Push return addr, relative jump |
| 0x08 | IADD | E | ✅ | rd = rs1 + rs2 |
| 0x09 | ISUB | E | ✅ | rd = rs1 − rs2 |
| 0x0A | IMUL | E | ✅ | rd = rs1 × rs2 |
| 0x0B | IDIV | E | ✅ | rd = rs1 ÷ rs2 |
| 0x0C | IMOD | E | ✅ | rd = rs1 % rs2 |
| 0x0D | INEG | C | ✅ | rd = −rs1 |
| 0x0E | INC | B | ✅ | rd = rd + 1 |
| 0x0F | DEC | B | ✅ | rd = rd − 1 |
| 0x10 | IAND | E | ✅ | rd = rs1 & rs2 |
| 0x11 | IOR | E | ✅ | rd = rs1 \| rs2 |
| 0x12 | IXOR | E | ✅ | rd = rs1 ^ rs2 |
| 0x13 | INOT | C | ✅ | rd = ~rs1 |
| 0x14 | ISHL | E | ✅ | rd = rs1 << rs2 |
| 0x15 | ISHR | E | ✅ | rd = rs1 >> rs2 |
| 0x20 | PUSH | B | — | Push register to stack |
| 0x21 | POP | B | — | Pop stack to register |
| 0x22 | DUP | A | — | Duplicate stack top |
| 0x28 | RET | A* | — | Return (2 padding bytes in JS/Rust) |
| 0x2B | MOVI | D* | — | Load i16 immediate to register |
| 0x2D | CMP | C | ✅ | Set flags from rd − rs1 |
| 0x2E | JE | D | reads | Jump if flag_zero (equal) |
| 0x2F | JNE | D | reads | Jump if !flag_zero (not equal) |
| 0x40 | FADD | E | — | Float add |
| 0x41 | FSUB | E | — | Float sub |
| 0x42 | FMUL | E | — | Float mul |
| 0x43 | FDIV | E | — | Float div |
| 0x60 | TELL | G | — | A2A: send message (**experimental**) |
| 0x61 | ASK | G | — | A2A: query agent (**experimental**) |
| 0x62 | DELEGATE | G | — | A2A: delegate task (**experimental**) |
| 0x66 | BROADCAST | G | — | A2A: broadcast to swarm (**experimental**) |
| 0x80 | HALT | A | — | Stop execution |
| 0x81 | YIELD | A | — | Cooperative yield |

## Condition Flags

All arithmetic operations (IADD, ISUB, IMUL, IDIV, IMOD, INEG, INC, DEC, IAND, IOR, IXOR, INOT, ISHL, ISHR) update the zero and sign flags based on the result.

CMP sets flags from a subtraction (rd − rs1) without storing the result.

JE/JNE check `flag_zero` set by CMP or arithmetic ops.

| Flag | Set when |
|------|----------|
| `flag_zero` | Result == 0 |
| `flag_sign` | Result < 0 |

## Memory Model (LOAD/STORE)

All implementations provide a linear byte-addressed memory for LOAD/STORE:

- **Python**: `MemoryManager` with named regions (stack, heap). Stack grows downward.
- **Rust**: 64 KB `Vec<u8>` linear memory. LOAD reads 4 bytes (i32 LE), STORE writes 4 bytes.
- **JS**: Not yet implemented (stub).

Addresses are byte offsets. LOAD reads a 32-bit little-endian integer from `memory[addr..addr+4]`.

## Stack Model — Implementation Detail

The internal stack growth direction differs between implementations. This is an **implementation detail**, not a portability concern:

| Implementation | Stack mechanism | Growth direction |
|----------------|-----------------|------------------|
| Python | Memory-backed (`MemoryManager`) | Downward (high → low addresses) |
| Rust | `Vec<i32>` | Upward (push/pop) |
| JS | `Array.push/pop` | Upward (push/pop) |

**Observable behavior is identical**: PUSH/POP order is the same regardless of internal growth direction. Bytecode that runs on one implementation will produce the same results on all three.

## A2A Protocol — Experimental

The A2A (Agent-to-Agent) opcodes (0x60–0x66) use **Format G** (variable-length with u16 length prefix). All three implementations parse Format G consistently:

```
[opcode][length_lo:u8][length_hi:u8][data:length bytes]
```

A2A behavior is **stubbed** in all implementations — the data is parsed and skipped unless a handler/callback is registered. A2A is considered experimental and may change in future versions.

## Cross-Implementation Compatibility

As of 2026-07-12, all three implementations handle the following opcodes identically:

- Core arithmetic (0x08–0x0F) with flag updates
- Bitwise ops (0x10–0x15) with flag updates
- Control flow (0x04–0x07, 0x2E–0x2F) including JE/JNE
- Stack ops (0x20–0x22)
- LOAD/STORE (0x02–0x03) with linear memory
- A2A parsing (0x60–0x66) with Format G

**Bytecode portability guarantee**: Any bytecode using the opcodes listed above will produce identical results across Python, Rust, and JS implementations.
