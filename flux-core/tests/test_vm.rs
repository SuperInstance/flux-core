//! Integration tests for the FLUX VM.

use flux_core::*;

/// Test MOVI and HALT - load 42 into R0, halt, check R0=42
#[test]
fn test_movi_halt() {
    let mut vm = Interpreter::new();
    let mut asm = Assembler::new();

    let bytecode = asm.assemble("MOVI R0, 42\nHALT").unwrap();
    vm.load_bytecode(&bytecode).unwrap();
    vm.run().unwrap();

    assert_eq!(vm.registers().get_gp(0).unwrap(), 42);
    assert_eq!(vm.state(), VmState::Halted);
}

/// Test arithmetic - MOVI R0,10; MOVI R1,20; IADD R0,R1; HALT → R0=30
#[test]
fn test_arithmetic() {
    let mut vm = Interpreter::new();
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

    vm.load_bytecode(&bytecode).unwrap();
    vm.run().unwrap();

    assert_eq!(vm.registers().get_gp(0).unwrap(), 30);
}

/// Test loop - sum 1..10 = 55 using DEC+JNZ loop
#[test]
fn test_loop() {
    let mut vm = Interpreter::new();
    let mut asm = Assembler::new();

    // Loop: R0 = 0, R1 = 10, loop: INC R0, DEC R1, JNZ loop
    let bytecode = asm
        .assemble(
            "
        MOVI R0, 0      ; sum = 0
        MOVI R1, 10     ; counter = 10
        MOVI R2, 1      ; increment
    loop:
        IADD R0, R0, R2 ; sum += 1
        DEC R1          ; counter--
        JNZ -3          ; jump to loop if counter != 0
        HALT
    ",
        )
        .unwrap();

    vm.load_bytecode(&bytecode).unwrap();
    vm.run().unwrap();

    assert_eq!(vm.registers().get_gp(0).unwrap(), 55); // Sum of 1..10
}

/// Test factorial - 5! = 120
#[test]
fn test_factorial() {
    let mut vm = Interpreter::new();
    let mut asm = Assembler::new();

    // Compute 5! = 120
    // R0 = n, R1 = result, loop: result *= n, n--
    let bytecode = asm
        .assemble(
            "
        MOVI R0, 5      ; n = 5
        MOVI R1, 1      ; result = 1
    loop:
        IMUL R1, R1, R0 ; result *= n
        DEC R0          ; n--
        CMP R0, 0
        JNZ -4          ; jump to loop if n != 0
        HALT
    ",
        )
        .unwrap();

    vm.load_bytecode(&bytecode).unwrap();
    vm.run().unwrap();

    assert_eq!(vm.registers().get_gp(1).unwrap(), 120); // 5! = 120
}

/// Test fibonacci - compute F(10)=55
#[test]
fn test_fibonacci() {
    let mut vm = Interpreter::new();
    let mut asm = Assembler::new();

    // Compute fibonacci(10) = 55
    // F(0) = 0, F(1) = 1, F(n) = F(n-1) + F(n-2)
    // Iterative approach
    let bytecode = asm
        .assemble(
            "
        MOVI R0, 0      ; prev = 0
        MOVI R1, 1      ; curr = 1
        MOVI R2, 9      ; iterations = 9 (we already have F(1))
        MOVI R3, 0      ; temp
    loop:
        MOV R3, R1      ; temp = curr
        IADD R1, R1, R0 ; curr = curr + prev
        MOV R0, R3      ; prev = temp
        DEC R2          ; iterations--
        JNZ -6          ; jump to loop if iterations != 0
        HALT
    ",
        )
        .unwrap();

    vm.load_bytecode(&bytecode).unwrap();
    vm.run().unwrap();

    assert_eq!(vm.registers().get_gp(1).unwrap(), 55); // F(10) = 55
}

/// Test push/pop - push R0, modify, pop into R1
#[test]
fn test_push_pop() {
    let mut vm = Interpreter::new();
    let mut asm = Assembler::new();

    let bytecode = asm
        .assemble(
            "
        MOVI R0, 42
        PUSH R0
        MOVI R0, 0
        POP R1
        HALT
    ",
        )
        .unwrap();

    vm.load_bytecode(&bytecode).unwrap();
    vm.run().unwrap();

    assert_eq!(vm.registers().get_gp(0).unwrap(), 0);
    assert_eq!(vm.registers().get_gp(1).unwrap(), 42);
    assert_eq!(vm.stack_depth(), 0); // Stack should be empty
}

/// Test nested calls using CALL and RET
#[test]
fn test_call_ret() {
    let mut vm = Interpreter::new();
    let mut asm = Assembler::new();

    let bytecode = asm
        .assemble(
            "
        MOVI R0, 5
        CALL func
        HALT

    func:
        INC R0
        INC R0
        RET
    ",
        )
        .unwrap();

    vm.load_bytecode(&bytecode).unwrap();
    vm.run().unwrap();

    assert_eq!(vm.registers().get_gp(0).unwrap(), 7); // 5 + 2 = 7
}

/// Test conditional jump with JZ and JNZ
#[test]
fn test_conditional_jump() {
    let mut vm = Interpreter::new();
    let mut asm = Assembler::new();

    let bytecode = asm
        .assemble(
            "
        MOVI R0, 10
        MOVI R1, 10
        CMP R0, R1
        JZ skip
        INC R0          ; Should not execute
    skip:
        HALT
    ",
        )
        .unwrap();

    vm.load_bytecode(&bytecode).unwrap();
    vm.run().unwrap();

    assert_eq!(vm.registers().get_gp(0).unwrap(), 10);
}

/// Test subtraction and negative numbers
#[test]
fn test_subtraction() {
    let mut vm = Interpreter::new();
    let mut asm = Assembler::new();

    let bytecode = asm
        .assemble(
            "
        MOVI R0, 10
        MOVI R1, 3
        ISUB R0, R1, R2
        HALT
    ",
        )
        .unwrap();

    vm.load_bytecode(&bytecode).unwrap();
    vm.run().unwrap();

    assert_eq!(vm.registers().get_gp(0).unwrap(), 7);
}

/// Test multiplication
#[test]
fn test_multiplication() {
    let mut vm = Interpreter::new();
    let mut asm = Assembler::new();

    let bytecode = asm
        .assemble(
            "
        MOVI R0, 7
        MOVI R1, 6
        IMUL R0, R1, R2
        HALT
    ",
        )
        .unwrap();

    vm.load_bytecode(&bytecode).unwrap();
    vm.run().unwrap();

    assert_eq!(vm.registers().get_gp(0).unwrap(), 42);
}

/// Test division
#[test]
fn test_division() {
    let mut vm = Interpreter::new();
    let mut asm = Assembler::new();

    let bytecode = asm
        .assemble(
            "
        MOVI R0, 100
        MOVI R1, 5
        IDIV R0, R1, R2
        HALT
    ",
        )
        .unwrap();

    vm.load_bytecode(&bytecode).unwrap();
    vm.run().unwrap();

    assert_eq!(vm.registers().get_gp(0).unwrap(), 20);
}

/// Test modulo
#[test]
fn test_modulo() {
    let mut vm = Interpreter::new();
    let mut asm = Assembler::new();

    let bytecode = asm
        .assemble(
            "
        MOVI R0, 17
        MOVI R1, 5
        IMOD R0, R1, R2
        HALT
    ",
        )
        .unwrap();

    vm.load_bytecode(&bytecode).unwrap();
    vm.run().unwrap();

    assert_eq!(vm.registers().get_gp(0).unwrap(), 2);
}

/// Test bitwise operations
#[test]
fn test_bitwise_ops() {
    let mut vm = Interpreter::new();
    let mut asm = Assembler::new();

    let bytecode = asm
        .assemble(
            "
        MOVI R0, 12     ; 1100
        MOVI R1, 10     ; 1010
        IAND R2, R0, R1 ; 1000 = 8
        IOR R3, R0, R1  ; 1110 = 14
        IXOR R4, R0, R1 ; 0110 = 6
        HALT
    ",
        )
        .unwrap();

    vm.load_bytecode(&bytecode).unwrap();
    vm.run().unwrap();

    assert_eq!(vm.registers().get_gp(2).unwrap(), 8);
    assert_eq!(vm.registers().get_gp(3).unwrap(), 14);
    assert_eq!(vm.registers().get_gp(4).unwrap(), 6);
}

/// Test shift operations
#[test]
fn test_shift_ops() {
    let mut vm = Interpreter::new();
    let mut asm = Assembler::new();

    let bytecode = asm
        .assemble(
            "
        MOVI R0, 8      ; 1000
        MOVI R1, 1
        ISHL R2, R0, R1 ; 10000 = 16
        ISHR R3, R0, R1 ; 100 = 4
        HALT
    ",
        )
        .unwrap();

    vm.load_bytecode(&bytecode).unwrap();
    vm.run().unwrap();

    assert_eq!(vm.registers().get_gp(2).unwrap(), 16);
    assert_eq!(vm.registers().get_gp(3).unwrap(), 4);
}

/// Test negation
#[test]
fn test_negation() {
    let mut vm = Interpreter::new();
    let mut asm = Assembler::new();

    let bytecode = asm
        .assemble(
            "
        MOVI R0, 42
        INEG R0
        HALT
    ",
        )
        .unwrap();

    vm.load_bytecode(&bytecode).unwrap();
    vm.run().unwrap();

    assert_eq!(vm.registers().get_gp(0).unwrap(), -42);
}

/// Test bitwise NOT
#[test]
fn test_bitwise_not() {
    let mut vm = Interpreter::new();
    let mut asm = Assembler::new();

    let bytecode = asm
        .assemble(
            "
        MOVI R0, 0      ; 0x00000000
        INOT R0         ; 0xFFFFFFFF
        HALT
    ",
        )
        .unwrap();

    vm.load_bytecode(&bytecode).unwrap();
    vm.run().unwrap();

    assert_eq!(vm.registers().get_gp(0).unwrap(), -1); // 0xFFFFFFFF = -1 in two's complement
}

/// Test flags after operations
#[test]
fn test_flags() {
    let mut vm = Interpreter::new();
    let mut asm = Assembler::new();

    let bytecode = asm
        .assemble(
            "
        MOVI R0, 0
        CMP R0, R0      ; Zero flag should be set
        HALT
    ",
        )
        .unwrap();

    vm.load_bytecode(&bytecode).unwrap();
    vm.run().unwrap();

    assert!(vm.registers().flags.zero);
}

/// Test load and store from memory
#[test]
fn test_load_store() {
    let mut vm = Interpreter::new();
    let mut asm = Assembler::new();

    let bytecode = asm
        .assemble(
            "
        MOVI R0, 42
        MOVI R1, 1000   ; Address in data segment
        STORE R1, R0    ; Store 42 at address 1000
        MOVI R0, 0
        LOAD R0, R1     ; Load from address 1000
        HALT
    ",
        )
        .unwrap();

    vm.load_bytecode(&bytecode).unwrap();
    vm.run().unwrap();

    assert_eq!(vm.registers().get_gp(0).unwrap(), 42);
}

/// Test that stack operations properly handle overflow
#[test]
fn test_stack_overflow_protection() {
    let mut vm = Interpreter::new();

    // Manually fill the stack
    vm.stack = vec![0u8; 8 * 1024 - 4];

    let bytecode = vec![0x20, 0x00]; // PUSH R0
    vm.load_bytecode(&bytecode).unwrap();

    // This should work
    vm.step().unwrap();

    // This should overflow
    assert!(matches!(vm.step(), Err(Error::StackOverflow)));
}

/// Test that stack operations properly handle underflow
#[test]
fn test_stack_underflow_protection() {
    let mut vm = Interpreter::new();

    let bytecode = vec![0x21, 0x00]; // POP R0
    vm.load_bytecode(&bytecode).unwrap();

    assert!(matches!(vm.step(), Err(Error::StackUnderflow)));
}

/// Test YIELD instruction
#[test]
fn test_yield() {
    let mut vm = Interpreter::new();
    let mut asm = Assembler::new();

    let bytecode = asm.assemble("YIELD").unwrap();
    vm.load_bytecode(&bytecode).unwrap();
    vm.run().unwrap();

    assert_eq!(vm.state(), VmState::Yielded);
}

/// Test DUP instruction
#[test]
fn test_dup() {
    let mut vm = Interpreter::new();
    let mut asm = Assembler::new();

    let bytecode = asm
        .assemble(
            "
        MOVI R0, 42
        PUSH R0
        DUP
        POP R1
        POP R2
        HALT
    ",
        )
        .unwrap();

    vm.load_bytecode(&bytecode).unwrap();
    vm.run().unwrap();

    assert_eq!(vm.registers().get_gp(1).unwrap(), 42);
    assert_eq!(vm.registers().get_gp(2).unwrap(), 42);
}

/// Test execution count
#[test]
fn test_instruction_count() {
    let mut vm = Interpreter::new();

    let bytecode = vec![
        0x00, // NOP
        0x00, // NOP
        0x00, // NOP
        0x80, // HALT
    ];

    vm.load_bytecode(&bytecode).unwrap();
    vm.run().unwrap();

    assert_eq!(vm.instructions_executed(), 4);
}

/// Test VM reset
#[test]
fn test_vm_reset() {
    let mut vm = Interpreter::new();

    let bytecode = vec![
        0x2B, 0x00, 0x2A, 0x00, // MOVI R0, 42
        0x80,                   // HALT
    ];

    vm.load_bytecode(&bytecode).unwrap();
    vm.run().unwrap();

    assert_eq!(vm.registers().get_gp(0).unwrap(), 42);

    vm.reset();

    assert_eq!(vm.registers().get_gp(0).unwrap(), 0);
    assert_eq!(vm.state(), VmState::Ready);
}

/// Test step-by-step execution
#[test]
fn test_step_execution() {
    let mut vm = Interpreter::new();

    let bytecode = vec![
        0x2B, 0x00, 0x0A, 0x00, // MOVI R0, 10
        0x2B, 0x00, 0x05, 0x00, // MOVI R0, 5
        0x80,                   // HALT
    ];

    vm.load_bytecode(&bytecode).unwrap();

    vm.step().unwrap();
    assert_eq!(vm.registers().get_gp(0).unwrap(), 10);

    vm.step().unwrap();
    assert_eq!(vm.registers().get_gp(0).unwrap(), 5);

    vm.step().unwrap();
    assert_eq!(vm.state(), VmState::Halted);
}

/// Test complex nested loop (calculate sum of sums)
#[test]
fn test_nested_loop() {
    let mut vm = Interpreter::new();
    let mut asm = Assembler::new();

    // Calculate 1 + 2 + 3 + 4 + 5 = 15
    let bytecode = asm
        .assemble(
            "
        MOVI R0, 0      ; sum = 0
        MOVI R1, 0      ; outer counter
    outer:
        INC R1
        MOV R2, R1      ; inner counter = outer counter
    inner:
        IADD R0, R0, R3 ; sum += 1
        DEC R2
        JNZ -2          ; jump to inner
        CMP R1, 5
        JNZ -10         ; jump to outer
        HALT
    ",
        )
        .unwrap();

    vm.load_bytecode(&bytecode).unwrap();
    vm.run().unwrap();

    // Sum: 1 + (1+1) + (1+1+1) + (1+1+1+1) + (1+1+1+1+1) = 1+2+3+4+5 = 15
    assert_eq!(vm.registers().get_gp(0).unwrap(), 15);
}
