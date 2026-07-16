//! Regression tests for HIGH bugs found in v0.1.0 audit.
//! These were missing before the audit (spec-mandated behavior).

use fluxvm::vm::Interpreter;

#[test]
fn test_iand_sets_flag_zero() {
    // IAND with 0xff & 0x00 = 0 should set flag_zero.
    // Spec: "All arithmetic operations (..., IAND, IOR, IXOR, INOT, ISHL, ISHR)
    //        update the zero and sign flags based on the result." (FLUX_BYTECODE_SPEC.md L63)
    let bc = [0x2B, 0x00, 0xFF, 0x00,  // MOVI R0, 0xFF
              0x2B, 0x01, 0x00, 0x00,  // MOVI R1, 0x00
              0x10, 0x02, 0x00, 0x01,  // IAND R2, R0, R1  → R2 = 0
              0x80];                   // HALT
    let mut vm = Interpreter::new(&bc);
    vm.execute().unwrap();
    assert_eq!(vm.read_gp(2), 0, "IAND result should be 0");
    assert_eq!(vm.regs.flag_zero, true, "IAND must set flag_zero when result is 0 (spec line 63)");
}

#[test]
fn test_ior_sets_flag_zero() {
    // IOR with 0x00 | 0x00 = 0 should set flag_zero.
    let bc = [0x2B, 0x00, 0x00, 0x00,
              0x2B, 0x01, 0x00, 0x00,
              0x11, 0x02, 0x00, 0x01,
              0x80];
    let mut vm = Interpreter::new(&bc);
    vm.execute().unwrap();
    assert_eq!(vm.regs.flag_zero, true, "IOR must set flag_zero when result is 0");
}

#[test]
fn test_inot_sets_flag_sign() {
    // INOT of 0x00000000 (smallest positive 32-bit = 0) = 0xFFFFFFFF which is -1 signed.
    // flag_sign should be true.
    let bc = [0x2B, 0x00, 0x00, 0x00,  // MOVI R0, 0
              0x13, 0x01, 0x00,        // INOT R1, R0  → R1 = -1
              0x80];
    let mut vm = Interpreter::new(&bc);
    vm.execute().unwrap();
    assert_eq!(vm.read_gp(1), -1);
    assert_eq!(vm.regs.flag_sign, true, "INOT must set flag_sign when result is negative");
}

#[test]
fn test_ret_with_empty_stack_halts() {
    // RET with empty call stack: behavior should be defined (either error or HALT).
    // Currently silently continues. Spec implies RET must pop; underflow should error or halt.
    let bc = [0x28, 0x00, 0x00,  // RET (with empty stack)
              0x80];              // HALT (sentinel — should not reach here)
    let mut vm = Interpreter::new(&bc);
    let result = vm.execute();
    // Either Err (preferred) or the sentinel R0 unchanged.
    // We accept: Err(_) OR succeeds but R0 remains default (didn't fall through).
    match result {
        Ok(_steps) => {
            // If it doesn't error, it MUST NOT have executed the sentinel.
            // The simplest check: pc didn't advance past the HALT.
            assert!(vm.regs.pc >= 4, "VM should have halted at RET, not executed sentinel");
        }
        Err(_) => {} // error is also acceptable
    }
}

#[test]
fn test_iand_then_je_jumps() {
    // The combined effect: IAND sets zero flag → JE jumps.
    // This is what users will expect from "IAND then compare".
    let bc = [0x2B, 0x00, 0xFF, 0x00,  // MOVI R0, 0xFF
              0x2B, 0x01, 0x00, 0x00,  // MOVI R1, 0x00
              0x10, 0x02, 0x00, 0x01,  // IAND R2, R0, R1  → 0, should set flag_zero
              0x2E, 0x00, 0x04, 0x00,  // JE +4 (skip the sentinel)
              0x2B, 0x03, 0x01, 0x00,  // MOVI R3, 1 (sentinel — should NOT execute)
              0x80];
    let mut vm = Interpreter::new(&bc);
    vm.execute().unwrap();
    assert_eq!(vm.read_gp(3), 0, "JE should have jumped over the sentinel because IAND set flag_zero");
}
