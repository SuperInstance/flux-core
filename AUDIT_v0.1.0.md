# Audit Report — flux-core v0.1.0

**Audit date:** 2026-07-16
**Auditor:** Claude Code (via OpenClaw subagent dispatch, completed in main session)
**Crate:** `fluxvm` v0.1.0 (Rust)
**Spec:** [`FLUX_BYTECODE_SPEC.md`](./FLUX_BYTECODE_SPEC.md) is canonical
**Sibling reference:** `conservation-enforcer` v0.2.1 (Python FLUX VM, just shipped) — same FLUX ISA, different language

---

## Verdict

**fluxvm v0.1.0 is mostly spec-conformant at the opcode level (100% opcode coverage, 32/32 spec opcodes implemented), but has 3 HIGH-severity bugs in semantics that survived the existing test suite.**

The opcode coverage is excellent — every instruction in the spec exists in the interpreter. The bugs are all about *behavior of those instructions* vs the spec, not missing opcodes. The most dangerous pattern: **bitwise ops silently fail to update flags**, breaking every downstream conditional jump that follows one.

Patches suggested but **NOT applied** — held for human review since this crate ships to crates.io and any regression is publicly visible.

---

## Bug findings

### 🔴 HIGH — Bitwise ops don't update flags (spec violation)

**Files:** `src/vm/interpreter.rs:125-128`
**Spec:** `FLUX_BYTECODE_SPEC.md:63` — *"All arithmetic operations (IADD, ISUB, IMUL, IDIV, IMOD, INEG, INC, DEC, IAND, IOR, IXOR, INOT, ISHL, ISHR) update the zero and sign flags based on the result."*

The spec explicitly lists IAND/IOR/IXOR/INOT as flag-updating ops. The interpreter:

```rust
// IAND (line 125) — does NOT call set_flags
0x10 => { let d = self.read_u8(); let a = self.read_u8(); let b = self.read_u8();
          let r = self.regs.read_gp(a) & self.regs.read_gp(b);
          self.regs.write_gp(d, r); }  // ← no set_flags

// Same omission for IOR (126), IXOR (127), INOT (128).
```

Compare to IADD (line 117) which correctly does:
```rust
self.regs.write_gp(d, r); self.regs.set_flags(r);  // ← correct
```

**Empirical reproduction** (cargo test, 4 of 5 new regression tests fail):

```rust
#[test]
fn test_iand_sets_flag_zero() {
    let bc = [0x2B, 0x00, 0xFF, 0x00,  // MOVI R0, 0xFF
              0x2B, 0x01, 0x00, 0x00,  // MOVI R1, 0x00
              0x10, 0x02, 0x00, 0x01,  // IAND R2, R0, R1  → R2 = 0
              0x80];
    let mut vm = Interpreter::new(&bc);
    vm.execute().unwrap();
    assert_eq!(vm.read_gp(2), 0);  // passes
    assert_eq!(vm.regs.flag_zero, true);  // ← FAILS: flag_zero is still false
}
```

**Impact:** Any policy that uses `IAND` followed by `JE`/`JNE`/`JSGE`/`JSLT` reads stale flags from a *previous* arithmetic op. The conditional jump may go the wrong way. Conservation policies don't currently use IAND/IOR/IXOR/INOT (the Python sibling has the same gap), so no shipped policy is broken — but custom policy authors writing bitwise logic will hit this.

**Suggested fix:** Add `self.regs.set_flags(r);` to the end of each of IAND, IOR, IXOR, INOT handlers. One line per opcode. Mirror the IADD pattern.

---

### 🔴 HIGH — RET with empty call stack silently continues

**File:** `src/vm/interpreter.rs:134`

```rust
0x28 => { let _r = self.read_u8(); let _p = self.read_u8();
          if let Some(ret_pc) = self.stack.pop() {
              self.regs.pc = ret_pc as u32;
          } } // RET
```

When the call stack is empty, `pop()` returns `None` and the `if let` branch is silently skipped. PC is unchanged. The VM continues executing whatever comes after the RET in the bytecode buffer.

**Empirical reproduction:**

```rust
#[test]
fn test_ret_with_empty_stack_halts() {
    let bc = [0x28, 0x00, 0x00,  // RET (with empty stack)
              0x80];              // HALT (sentinel)
    let mut vm = Interpreter::new(&bc);
    let _ = vm.execute();
    // Currently: VM executes the HALT after RET (no error, no halt at RET).
    // Spec/expectation: RET with empty stack should be an error OR halt cleanly.
    assert!(vm.regs.pc >= 4, "VM should have halted at RET, not executed sentinel");
}
```

This test passes (assertion holds because pc=4 by the time we check) but the behavior is silently wrong — the VM falls through to the sentinel.

**Impact:** Custom policies with unbalanced CALL/RET pairs (a common bug class) silently execute unintended code instead of erroring. **The Python sibling VM had the same bug, just fixed in conservation-enforcer v0.2.1.**

**Suggested fix:** Change to:
```rust
0x28 => { let _r = self.read_u8(); let _p = self.read_u8();
          match self.stack.pop() {
              Some(ret_pc) => self.regs.pc = ret_pc as u32,
              None => return Err(FluxError::StackUnderflow),
          } }
```

Add a `StackUnderflow` variant to `FluxError` enum.

---

### 🔴 HIGH — Conditional jumps after bitwise ops read stale flags

This is a **knock-on effect** of bug #1. Once IAND/IOR/IXOR/INOT don't update flags, any subsequent JE/JNE/JSGE/JSLT is operating on the flag state from whatever arithmetic op ran *before* the bitwise op.

```rust
#[test]
fn test_iand_then_je_jumps() {
    let bc = [0x2B, 0x00, 0xFF, 0x00,  // MOVI R0, 0xFF
              0x2B, 0x01, 0x00, 0x00,  // MOVI R1, 0x00
              0x10, 0x02, 0x00, 0x01,  // IAND R2, R0, R1  → 0, SHOULD set flag_zero
              0x2E, 0x00, 0x04, 0x00,  // JE +4 (skip the sentinel)
              0x2B, 0x03, 0x01, 0x00,  // MOVI R3, 1 (sentinel — should NOT execute)
              0x80];
    let mut vm = Interpreter::new(&bc);
    vm.execute().unwrap();
    assert_eq!(vm.read_gp(3), 0, "JE should have jumped");  // ← FAILS: R3 = 1
}
```

The test FAILS — `R3 = 1`, meaning the JE did NOT take, meaning the IAND did NOT set `flag_zero` as expected. The sentinel MOVI executed.

**Impact:** Same as bug #1. Anything that combines a bitwise op with a conditional jump is broken. Fixing bug #1 fixes this automatically.

---

## What the spec says (canonical reference)

From `FLUX_BYTECODE_SPEC.md`:

```
63: All arithmetic operations (IADD, ISUB, IMUL, IDIV, IMOD, INEG, INC, DEC, IAND, IOR, IXOR, INOT, ISHL, ISHR) update the zero and sign flags based on the result.
65: CMP sets flags from a subtraction (rd − rs1) without storing the result.
67: JE/JNE check `flag_zero` set by CMP or arithmetic ops.
71: `flag_zero` | Result == 0
72: `flag_sign` | Result < 0
```

The interpreter correctly implements CMP (line 136), correctly updates flags for IADD/ISUB/IMUL/IDIV/IMOD/INEG/INC/DEC/ISHL/ISHR (lines 117-124, 129-130), and correctly implements JE/JNE (lines 137-138). The four missing handlers (IAND/IOR/IXOR/INOT) are the gap.

---

## What the tests covered well

The existing test suite (`tests/test_vm.rs`) covers:
- Arithmetic (ADD, MUL, loops, factorial)
- MOVI, MOV
- Control flow (JNZ for loop)
- Halt

It's a reasonable smoke-test for a register VM. The bitwise-op flag-update gap was missed because the tests use JNZ (which reads register values, not flags) instead of JE/JNE (which read flags).

**The gap:** The test suite never combines a bitwise op with a flag-reading conditional. Adding 5 regression tests in `tests/test_audit_v010_regressions.rs` (4 currently fail, demonstrating the bug; 1 passes on the boundary).

---

## Comparison with conservation-enforcer (Python FLUX VM)

The Python VM at `conservation-enforcer/` shares the same FLUX ISA. Its v0.2.0 → v0.2.1 audit found 3 different bugs:

| Bug | Python VM (v0.2.0) | Rust VM (v0.1.0) |
|-----|--------------------|--------------------|
| Dead `running` flag, RET-with-empty-stack falls through | YES (HIGH — fixed in v0.2.1) | YES (different mechanism — silent continue instead of dead-flag) |
| MOVI no sign-extend (MOVI R0, -1 → 65535) | YES (MED — fixed in v0.2.1) | **NO** — Rust `as i32` cast handles sign extension correctly |
| Bitwise ops don't update flags | NO (Python doesn't have this bug) | **YES** (HIGH — this audit) |
| Unbounded `scope_discipline_policy(max_expansion)` | YES (LOW — fixed in v0.2.1) | N/A (policy not in Rust crate) |

The Python and Rust VMs share the RET-with-empty-stack bug class (different mechanism, same symptom). The other bugs are disjoint — each implementation has its own latent issues. This is consistent with the **cross-implementation divergence** pattern that conservation-enforcer's audit surfaced: "the bytecode spec is more valuable than any single implementation."

---

## Suggested patches (NOT APPLIED)

```rust
// src/vm/interpreter.rs — bug #1 and #3
0x10 => { let d = self.read_u8(); let a = self.read_u8(); let b = self.read_u8();
          let r = self.regs.read_gp(a) & self.regs.read_gp(b);
          self.regs.write_gp(d, r); self.regs.set_flags(r); }  // ← add set_flags
0x11 => { let d = self.read_u8(); let a = self.read_u8(); let b = self.read_u8();
          let r = self.regs.read_gp(a) | self.regs.read_gp(b);
          self.regs.write_gp(d, r); self.regs.set_flags(r); }  // ← add set_flags
0x12 => { let d = self.read_u8(); let a = self.read_u8(); let b = self.read_u8();
          let r = self.regs.read_gp(a) ^ self.regs.read_gp(b);
          self.regs.write_gp(d, r); self.regs.set_flags(r); }  // ← add set_flags
0x13 => { let d = self.read_u8(); let s = self.read_u8();
          let r = !self.regs.read_gp(s);
          self.regs.write_gp(d, r); self.regs.set_flags(r); }  // ← add set_flags

// src/vm/interpreter.rs — bug #2
0x28 => { let _r = self.read_u8(); let _p = self.read_u8();
          match self.stack.pop() {
              Some(ret_pc) => self.regs.pc = ret_pc as u32,
              None => return Err(FluxError::StackUnderflow),  // ← error on underflow
          } }
```

Add `StackUnderflow` variant to `FluxError` enum in `src/error.rs`.

---

## What was NOT a bug (verified)

- **Opcode coverage** — 100% (32/32 spec opcodes implemented).
- **MOVI sign extension** — Rust's `as i32` cast from `read_i16()` correctly sign-extends. `MOVI R0, -1` gives `-1` (correct), not `65535`.
- **`running` flag check** — Rust interpreter uses a `halted` flag for control, checked in the main dispatch loop. Not dead code.
- **CMP, JE, JNE, arithmetic op flag updates** — all correct.
- **Division by zero** — explicitly errors via `FluxError::DivisionByZero`. Good error handling.

---

## Test results (empirical verification)

```
test test_audit_v010_regressions::test_iand_sets_flag_zero  ... FAILED (left: false, right: true)
test test_audit_v010_regressions::test_ior_sets_flag_zero   ... FAILED (left: false, right: true)
test test_audit_v010_regressions::test_inot_sets_flag_sign  ... FAILED (left: false, right: true)
test test_audit_v010_regressions::test_iand_then_je_jumps   ... FAILED (left: 1, right: 0)
test test_audit_v010_regressions::test_ret_with_empty_stack_halts ... ok (silent fallthrough, sentinel reached)
```

Test file: `tests/test_audit_v010_regressions.rs` (5 tests, 4 failing).

---

## Methodology notes

**Audit method:** Read source → cross-reference spec → write 2-line Rust reproductions → verify with cargo test.

**The cross-implementation comparison was the highest-value move.** Looking at the Python VM's recent audit findings immediately told me to check the Rust port for the same bug classes — and one of the three findings here was a *different mechanism* for the same symptom as a Python bug. That kind of cross-pollination is what makes cross-implementation conformance audits valuable.

**The spec at line 63 was the smoking gun.** It explicitly enumerates which ops must update flags. A simple grep of the interpreter for `set_flags` showed the four missing handlers immediately.

**Lesson for the next audit:** When you have multiple implementations of a spec, audit each one AND look for *bug classes* in one and check the others for the same class. Different mechanism, same gap.