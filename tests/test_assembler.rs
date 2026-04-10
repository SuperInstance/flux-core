use flux_core::bytecode::assembler::Assembler;
use flux_core::vm::Interpreter;

#[test]
fn test_assemble_movi_halt() {
    let bc = Assembler::assemble("MOVI R0, 42\nHALT").unwrap();
    assert_eq!(bc, vec![0x2B, 0x00, 0x2A, 0x00, 0x80]);
    let mut vm = Interpreter::new(&bc);
    vm.execute().unwrap();
    assert_eq!(vm.read_gp(0), 42);
}

#[test]
fn test_assemble_addition() {
    let source = "MOVI R0, 10\nMOVI R1, 20\nIADD R0, R1\nHALT";
    let bc = Assembler::assemble(source).unwrap();
    let mut vm = Interpreter::new(&bc);
    vm.execute().unwrap();
    assert_eq!(vm.read_gp(0), 30);
}

#[test]
fn test_assemble_loop() {
    let source = r#"
MOVI R0, 0
MOVI R1, 10
loop:
IADD R0, R1
DEC R1
JNZ R1, loop
HALT
"#;
    let bc = Assembler::assemble(source).unwrap();
    let mut vm = Interpreter::new(&bc);
    vm.execute().unwrap();
    assert_eq!(vm.read_gp(0), 55);
}

#[test]
fn test_assemble_factorial() {
    let source = r#"
MOVI R3, 5
MOVI R4, 1
loop:
IMUL R4, R3
DEC R3
JNZ R3, loop
HALT
"#;
    let bc = Assembler::assemble(source).unwrap();
    let mut vm = Interpreter::new(&bc);
    vm.execute().unwrap();
    assert_eq!(vm.read_gp(4), 120);
}
