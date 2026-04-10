//! Integration tests for the assembler.

use flux_core::Assembler;

/// Test assembling a single MOVI instruction
#[test]
fn test_assemble_single_movi() {
    let mut asm = Assembler::new();
    let bytecode = asm.assemble_one("MOVI R0, 42").unwrap();

    assert_eq!(bytecode, vec![0x2B, 0x00, 0x2A, 0x00]);
}

/// Test assembling a complete program
#[test]
fn test_assemble_program() {
    let mut asm = Assembler::new();
    let bytecode = asm
        .assemble(
            "
        MOVI R0, 10
        MOVI R1, 20
        IADD R0, R1, R2
        HALT
    ",
        )
        .unwrap();

    assert_eq!(bytecode.len(), 13); // 4 + 4 + 4 + 1
    assert_eq!(bytecode[0], 0x2B); // MOVI
    assert_eq!(bytecode[4], 0x2B); // MOVI
    assert_eq!(bytecode[8], 0x08); // IADD
    assert_eq!(bytecode[12], 0x80); // HALT
}

/// Test assembling with comments
#[test]
fn test_assemble_with_comments() {
    let mut asm = Assembler::new();
    let bytecode = asm
        .assemble(
            "
        ; This is a comment
        MOVI R0, 42     ; Load 42 into R0
        // This is also a comment
        HALT            ; Stop execution
    ",
        )
        .unwrap();

    assert_eq!(bytecode.len(), 5);
}

/// Test assembling negative immediate values
#[test]
fn test_assemble_negative_immediate() {
    let mut asm = Assembler::new();
    let bytecode = asm.assemble_one("MOVI R0, -100").unwrap();

    assert_eq!(bytecode[0], 0x2B); // MOVI
    assert_eq!(bytecode[1], 0x00); // R0
    // -100 as i16 little-endian: 0x9C 0xFF
    assert_eq!(bytecode[2], 0x9C);
    assert_eq!(bytecode[3], 0xFF);
}

/// Test assembling hexadecimal immediate values
#[test]
fn test_assemble_hex_immediate() {
    let mut asm = Assembler::new();
    let bytecode = asm.assemble_one("MOVI R0, 0xFF").unwrap();

    assert_eq!(bytecode[0], 0x2B); // MOVI
    assert_eq!(bytecode[1], 0x00); // R0
    assert_eq!(bytecode[2], 0xFF);
    assert_eq!(bytecode[3], 0x00);
}

/// Test assembling all arithmetic instructions
#[test]
fn test_assemble_arithmetic() {
    let mut asm = Assembler::new();

    assert_eq!(asm.assemble_one("IADD R0, R1, R2").unwrap()[0], 0x08);
    assert_eq!(asm.assemble_one("ISUB R0, R1, R2").unwrap()[0], 0x09);
    assert_eq!(asm.assemble_one("IMUL R0, R1, R2").unwrap()[0], 0x0A);
    assert_eq!(asm.assemble_one("IDIV R0, R1, R2").unwrap()[0], 0x0B);
    assert_eq!(asm.assemble_one("IMOD R0, R1, R2").unwrap()[0], 0x0C);
}

/// Test assembling bitwise instructions
#[test]
fn test_assemble_bitwise() {
    let mut asm = Assembler::new();

    assert_eq!(asm.assemble_one("IAND R0, R1, R2").unwrap()[0], 0x10);
    assert_eq!(asm.assemble_one("IOR R0, R1, R2").unwrap()[0], 0x11);
    assert_eq!(asm.assemble_one("IXOR R0, R1, R2").unwrap()[0], 0x12);
    assert_eq!(asm.assemble_one("INOT R0").unwrap()[0], 0x13);
    assert_eq!(asm.assemble_one("ISHL R0, R1, R2").unwrap()[0], 0x14);
    assert_eq!(asm.assemble_one("ISHR R0, R1, R2").unwrap()[0], 0x15);
}

/// Test assembling control flow instructions
#[test]
fn test_assemble_control_flow() {
    let mut asm = Assembler::new();

    // Jump instructions
    assert_eq!(asm.assemble_one("JMP 100").unwrap()[0], 0x04);
    assert_eq!(asm.assemble_one("JZ 50").unwrap()[0], 0x05);
    assert_eq!(asm.assemble_one("JNZ -10").unwrap()[0], 0x06);
    assert_eq!(asm.assemble_one("CALL 20").unwrap()[0], 0x07);

    // CMP
    assert_eq!(asm.assemble_one("CMP R0, R1").unwrap()[0], 0x2D);

    // RET
    assert_eq!(asm.assemble_one("RET").unwrap()[0], 0x28);
}

/// Test assembling stack instructions
#[test]
fn test_assemble_stack() {
    let mut asm = Assembler::new();

    assert_eq!(asm.assemble_one("PUSH R0").unwrap(), vec![0x20, 0x00]);
    assert_eq!(asm.assemble_one("POP R1").unwrap(), vec![0x21, 0x01]);
    assert_eq!(asm.assemble_one("DUP").unwrap(), vec![0x22]);
}

/// Test assembling increment/decrement
#[test]
fn test_assemble_inc_dec() {
    let mut asm = Assembler::new();

    assert_eq!(asm.assemble_one("INC R5").unwrap(), vec![0x0E, 0x05]);
    assert_eq!(asm.assemble_one("DEC R10").unwrap(), vec![0x0F, 0x0A]);
}

/// Test assembling negation
#[test]
fn test_assemble_neg() {
    let mut asm = Assembler::new();

    assert_eq!(asm.assemble_one("INEG R0").unwrap(), vec![0x0D, 0x00]);
}

/// Test assembling float instructions
#[test]
fn test_assemble_float_ops() {
    let mut asm = Assembler::new();

    assert_eq!(asm.assemble_one("FADD F0, F1, F2").unwrap()[0], 0x40);
    assert_eq!(asm.assemble_one("FSUB F0, F1, F2").unwrap()[0], 0x41);
    assert_eq!(asm.assemble_one("FMUL F0, F1, F2").unwrap()[0], 0x42);
    assert_eq!(asm.assemble_one("FDIV F0, F1, F2").unwrap()[0], 0x43);
}

/// Test assembling A2A instructions
#[test]
fn test_assemble_a2a() {
    let mut asm = Assembler::new();

    assert_eq!(asm.assemble_one("TELL").unwrap()[0], 0x60);
    assert_eq!(asm.assemble_one("ASK").unwrap()[0], 0x61);
    assert_eq!(asm.assemble_one("DELEGATE").unwrap()[0], 0x62);
    assert_eq!(asm.assemble_one("BROADCAST").unwrap()[0], 0x66);
}

/// Test case insensitivity
#[test]
fn test_case_insensitive() {
    let mut asm = Assembler::new();

    let b1 = asm.assemble_one("MOVI R0, 42").unwrap();
    let b2 = asm.assemble_one("movi r0, 42").unwrap();
    let b3 = asm.assemble_one("Movi R0, 42").unwrap();

    assert_eq!(b1, b2);
    assert_eq!(b2, b3);
}

/// Test various whitespace formats
#[test]
fn test_whitespace_variations() {
    let mut asm = Assembler::new();

    let b1 = asm.assemble_one("MOVI R0,42").unwrap();
    let b2 = asm.assemble_one("MOVI\tR0,\t42").unwrap();
    let b3 = asm.assemble_one("  MOVI  R0  ,  42  ").unwrap();

    assert_eq!(b1, b2);
    assert_eq!(b2, b3);
}

/// Test invalid register
#[test]
fn test_invalid_register() {
    let mut asm = Assembler::new();

    assert!(asm.assemble_one("MOVI R16, 42").is_err());
    assert!(asm.assemble_one("MOVI X0, 42").is_err());
}

/// Test unknown mnemonic
#[test]
fn test_unknown_mnemonic() {
    let mut asm = Assembler::new();

    assert!(asm.assemble_one("INVALID R0, R1").is_err());
    assert!(asm.assemble_one("NOTAREALOP 42").is_err());
}

/// Test malformed instructions
#[test]
fn test_malformed_instructions() {
    let mut asm = Assembler::new();

    // Missing operands
    assert!(asm.assemble_one("MOVI").is_err());
    assert!(asm.assemble_one("IADD R0").is_err());

    // Too many operands
    assert!(asm.assemble_one("MOVI R0, 42, 99").is_err());
}

/// Test empty program
#[test]
fn test_empty_program() {
    let mut asm = Assembler::new();
    let bytecode = asm.assemble("").unwrap();

    assert_eq!(bytecode.len(), 0);
}

/// Test program with only comments
#[test]
fn test_only_comments() {
    let mut asm = Assembler::new();
    let bytecode = asm
        .assemble(
            "
        ; Just a comment
        // Another comment
    ",
        )
        .unwrap();

    assert_eq!(bytecode.len(), 0);
}

/// Test assembling a complex realistic program
#[test]
fn test_complex_program() {
    let mut asm = Assembler::new();
    let program = r#"
        ; Calculate factorial of 5
        MOVI R0, 5       ; n = 5
        MOVI R1, 1       ; result = 1
    loop:
        IMUL R1, R1, R0 ; result *= n
        DEC R0          ; n--
        CMP R0, 0       ; compare n with 0
        JNZ -4          ; jump to loop if n != 0
        HALT
    "#;

    let bytecode = asm.assemble(program).unwrap();

    // Verify we got some bytecode
    assert!(!bytecode.is_empty());

    // Verify it starts with MOVI
    assert_eq!(bytecode[0], 0x2B);

    // Verify it ends with HALT
    assert_eq!(bytecode[bytecode.len() - 1], 0x80);
}

/// Test immediate value bounds
#[test]
fn test_immediate_bounds() {
    let mut asm = Assembler::new();

    // Max i16 value
    let bytecode = asm.assemble_one("MOVI R0, 32767").unwrap();
    assert_eq!(bytecode[2], 0xFF);
    assert_eq!(bytecode[3], 0x7F);

    // Min i16 value
    let bytecode = asm.assemble_one("MOVI R0, -32768").unwrap();
    assert_eq!(bytecode[2], 0x00);
    assert_eq!(bytecode[3], 0x80);
}
