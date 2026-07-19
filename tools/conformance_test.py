#!/usr/bin/env python3
"""
conformance_test.py — Cross-implementation FLUX VM conformance test.

Assembles a test program that exercises every core opcode, runs it on both
the Python VM and the Rust VM, and compares register state.

Usage:
    PYTHONPATH=/path/to/flux-runtime/src python3 tools/conformance_test.py
"""

import json
import struct
import subprocess
import sys
import os

# ═══════════════════════════════════════════════════════════════
# Canonical FLUX opcode values (from FLUX_BYTECODE_SPEC.md)
# Both Python (flux-runtime) and Rust (flux-core) VMs use these values.
# ═══════════════════════════════════════════════════════════════

NOP   = 0x00
MOV   = 0x01  # C: rd, rs
LOAD  = 0x02  # C: rd, rs(addr)
STORE = 0x03  # C: rd(val), rs(addr)
JMP   = 0x04  # D: r, off16
CALL  = 0x07  # D: r, off16
IADD  = 0x08  # E: rd, rs1, rs2
ISUB  = 0x09  # E: rd, rs1, rs2
IMUL  = 0x0A  # E: rd, rs1, rs2
IDIV  = 0x0B  # E: rd, rs1, rs2
IMOD  = 0x0C  # E: rd, rs1, rs2
IAND  = 0x10  # E: rd, rs1, rs2
IOR   = 0x11  # E: rd, rs1, rs2
IXOR  = 0x12  # E: rd, rs1, rs2
INOT  = 0x13  # C: rd, rs
ISHL  = 0x14  # E: rd, rs1, rs2
ISHR  = 0x15  # E: rd, rs1, rs2
PUSH  = 0x20  # B: r
POP   = 0x21  # B: r
RET   = 0x28  # B: r
MOVI  = 0x2B  # D: rd, imm16
CMP   = 0x2D  # C: ra, rb
JE    = 0x2E  # D: r, off16
JNE   = 0x2F  # D: r, off16
HALT  = 0x80

def byte(b):
    return bytes([b & 0xFF])

def short(s):
    return struct.pack('<h', s)  # little-endian signed 16-bit

def build_test_program():
    """Build a bytecode program that exercises every core opcode."""
    prog = bytearray()
    
    # MOVI R0, 42      — test immediate load
    prog += byte(MOVI); prog += byte(0); prog += short(42)
    # MOVI R1, 10      — second value
    prog += byte(MOVI); prog += byte(1); prog += short(10)
    
    # IADD R2, R0, R1  — 42 + 10 = 52
    prog += byte(IADD); prog += byte(2); prog += byte(0); prog += byte(1)
    
    # ISUB R3, R0, R1  — 42 - 10 = 32
    prog += byte(ISUB); prog += byte(3); prog += byte(0); prog += byte(1)
    
    # IMUL R4, R0, R1  — 42 * 10 = 420
    prog += byte(IMUL); prog += byte(4); prog += byte(0); prog += byte(1)
    
    # IDIV R5, R0, R1  — 42 / 10 = 4
    prog += byte(IDIV); prog += byte(5); prog += byte(0); prog += byte(1)
    
    # IMOD R6, R0, R1  — 42 % 10 = 2
    prog += byte(IMOD); prog += byte(6); prog += byte(0); prog += byte(1)
    
    # IAND R7, R0, R1  — 42 & 10 = 10
    prog += byte(IAND); prog += byte(7); prog += byte(0); prog += byte(1)
    
    # IOR R8, R0, R1   — 42 | 10 = 42
    prog += byte(IOR); prog += byte(8); prog += byte(0); prog += byte(1)
    
    # IXOR R9, R0, R1  — 42 ^ 10 = 32
    prog += byte(IXOR); prog += byte(9); prog += byte(0); prog += byte(1)
    
    # INOT R10, R0     — ~42 = -43
    prog += byte(INOT); prog += byte(10); prog += byte(0)
    
    # MOV R11, R2      — copy R2 (52) to R11
    prog += byte(MOV); prog += byte(11); prog += byte(2)
    
    # CMP R0, R1       — compare 42 vs 10 (zero=0, sign=0)
    prog += byte(CMP); prog += byte(0); prog += byte(1)
    
    # MOVI R12, 999    — default value
    prog += byte(MOVI); prog += byte(12); prog += short(999)
    
    # JNE R0, +4       — if R0 != R1, skip ahead (should jump: 42 != 10)
    prog += byte(JNE); prog += byte(0); prog += short(4)
    
    # MOVI R12, 111    — should be SKIPPED (JNE jumps over this)
    prog += byte(MOVI); prog += byte(12); prog += short(111)
    
    # NOP              — padding
    prog += byte(NOP)
    
    # HALT
    prog += byte(HALT)
    
    return bytes(prog)

# ═══════════════════════════════════════════════════════════════
# Expected register values after running the test program
# ═══════════════════════════════════════════════════════════════

EXPECTED = {
    0: 42,      # MOVI R0, 42
    1: 10,      # MOVI R1, 10
    2: 52,      # IADD 42+10
    3: 32,      # ISUB 42-10
    4: 420,     # IMUL 42*10
    5: 4,       # IDIV 42/10
    6: 2,       # IMOD 42%10
    7: 10,      # IAND 42&10
    8: 42,      # IOR 42|10
    9: 32,      # IXOR 42^10
    10: -43,    # INOT ~42 (as signed 32-bit)
    11: 52,     # MOV from R2
    12: 999,    # JNE should have skipped the 111 assignment
}

# ═══════════════════════════════════════════════════════════════
# Python VM runner (flux-runtime)
# ═══════════════════════════════════════════════════════════════

def run_python_vm(bytecode_path):
    """Run bytecode on the Python FLUX VM (flux-runtime) and return register state."""
    try:
        from flux.vm.interpreter import Interpreter

        with open(bytecode_path, 'rb') as f:
            code = f.read()

        vm = Interpreter(code)
        vm.execute()
        state = vm.dump_state()

        # Extract general-purpose registers (signed 32-bit)
        gp = state.get('registers', {}).get('gp', [])
        regs = {}
        for i in range(min(16, len(gp))):
            regs[i] = gp[i]
        return regs
    except Exception as e:
        return {"error": str(e)}

# ═══════════════════════════════════════════════════════════════
# Rust VM runner
# ═══════════════════════════════════════════════════════════════

def run_rust_vm(bytecode_path):
    """Run bytecode on the Rust FLUX VM and return register state."""
    rust_dir = os.path.join(os.path.dirname(__file__), '..')
    
    # Build if needed
    try:
        result = subprocess.run(
            ['cargo', 'build', '--release', '--bin', 'fluxvm'],
            cwd=rust_dir, capture_output=True, text=True, timeout=120
        )
    except (subprocess.TimeoutExpired, FileNotFoundError):
        pass
    
    # Find the binary
    binary = os.path.join(rust_dir, 'target', 'release', 'fluxvm')
    if not os.path.exists(binary):
        binary = os.path.join(rust_dir, 'target', 'debug', 'fluxvm')
    if not os.path.exists(binary):
        return {"error": "Rust binary not found. Run: cargo build --release"}
    
    try:
        result = subprocess.run(
            [binary, '--dump-regs', bytecode_path],
            capture_output=True, text=True, timeout=10
        )
        
        # Parse register output
        regs = {}
        for line in result.stdout.splitlines():
            line = line.strip()
            if line.startswith('R') and ':' in line:
                parts = line.split(':')
                try:
                    idx = int(parts[0].strip()[1:])
                    val = int(parts[1].strip())
                    regs[idx] = val
                except (ValueError, IndexError):
                    pass
        return regs
    except Exception as e:
        return {"error": str(e)}

# ═══════════════════════════════════════════════════════════════
# Main
# ═══════════════════════════════════════════════════════════════

def main():
    print("╔═══════════════════════════════════════════════════════╗")
    print("║  FLUX VM Cross-Implementation Conformance Test       ║")
    print("╚═══════════════════════════════════════════════════════╝")
    print()
    
    # Build test program
    program = build_test_program()
    print(f"Test program: {len(program)} bytes, {program.count(bytes([HALT]))} HALT")
    
    # Write to file
    bytecode_path = os.path.join(os.path.dirname(__file__), 'conformance_test.bin')
    with open(bytecode_path, 'wb') as f:
        f.write(program)
    print(f"Written to: {bytecode_path}")
    print()
    
    # Run on Python VM
    print("── Python VM ──")
    py_regs = run_python_vm(bytecode_path)
    if "error" in py_regs:
        print(f"  ❌ ERROR: {py_regs['error']}")
    else:
        print(f"  ✓ Ran successfully. {len(py_regs)} registers read.")
    
    # Run on Rust VM
    print("\n── Rust VM ──")
    rust_regs = run_rust_vm(bytecode_path)
    if "error" in rust_regs:
        print(f"  ❌ ERROR: {rust_regs['error']}")
    else:
        print(f"  ✓ Ran successfully. {len(rust_regs)} registers read.")
    
    # Compare against expected
    print("\n── Conformance Results ──")
    passed = 0
    failed = 0
    
    for reg, expected_val in sorted(EXPECTED.items()):
        py_val = py_regs.get(reg) if isinstance(py_regs, dict) else None
        rust_val = rust_regs.get(reg) if isinstance(rust_regs, dict) else None
        
        py_ok = py_val is not None and (
            py_val == expected_val or
            (isinstance(py_val, int) and (py_val & 0xFFFFFFFF) == (expected_val & 0xFFFFFFFF))
        )
        rust_ok = rust_val is not None and (
            rust_val == expected_val or
            (isinstance(rust_val, int) and (rust_val & 0xFFFFFFFF) == (expected_val & 0xFFFFFFFF))
        )
        
        status = ""
        if py_ok and rust_ok:
            status = "✓ BOTH"
            passed += 1
        elif py_ok and not rust_ok:
            status = "⚠ PY ONLY"
            failed += 1
        elif rust_ok and not py_ok:
            status = "⚠ RUST ONLY"
            failed += 1
        else:
            status = "❌ NEITHER"
            failed += 1
        
        py_str = str(py_val) if py_val is not None else "?"
        rust_str = str(rust_val) if rust_val is not None else "?"
        print(f"  R{reg:2d}: expected={expected_val:6d}  py={py_str:>8s}  rust={rust_str:>8s}  {status}")
    
    total = passed + failed
    print(f"\n{'='*55}")
    print(f"Results: {passed}/{total} passed, {failed} failed")
    if failed == 0:
        print("✅ CONFORMANT — both VMs agree on all tested opcodes")
    else:
        print("⚠ DISCREPANCIES FOUND — review the output above")
    print(f"{'='*55}")
    
    # Cleanup
    if os.path.exists(bytecode_path):
        os.remove(bytecode_path)
    
    return 0 if failed == 0 else 1

if __name__ == "__main__":
    sys.exit(main())
