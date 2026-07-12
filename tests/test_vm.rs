use fluxvm::vm::Interpreter;

#[test]
fn test_movi_halt() {
    let bc = [0x2B, 0x00, 0x2A, 0x00, 0x80]; // MOVI R0, 42; HALT
    let mut vm = Interpreter::new(&bc);
    vm.execute().unwrap();
    assert_eq!(vm.read_gp(0), 42);
}

#[test]
fn test_addition() {
    let bc = [0x2B, 0x00, 0x0A, 0x00,  // MOVI R0, 10
              0x2B, 0x01, 0x14, 0x00,  // MOVI R1, 20
              0x08, 0x02, 0x00, 0x01,  // IADD R2, R0, R1
              0x80];                   // HALT
    let mut vm = Interpreter::new(&bc);
    vm.execute().unwrap();
    assert_eq!(vm.read_gp(2), 30);
}

#[test]
fn test_multiplication() {
    let bc = [0x2B, 0x00, 0x06, 0x00,  // MOVI R0, 6
              0x2B, 0x01, 0x07, 0x00,  // MOVI R1, 7
              0x0A, 0x02, 0x00, 0x01,  // IMUL R2, R0, R1
              0x80];
    let mut vm = Interpreter::new(&bc);
    vm.execute().unwrap();
    assert_eq!(vm.read_gp(2), 42);
}

#[test]
fn test_loop_sum() {
    // Sum 1+2+...+10 = 55
    // Uses 2-operand style: IADD R0, R0, R1 (R0 += R1)
    let bc = [0x2B, 0x00, 0x00, 0x00,  // MOVI R0, 0
              0x2B, 0x01, 0x0A, 0x00,  // MOVI R1, 10
              0x08, 0x00, 0x00, 0x01,  // IADD R0, R0, R1
              0x0F, 0x01,              // DEC R1
              0x06, 0x01, 0xF6, 0xFF,  // JNZ R1, -10
              0x80];
    let mut vm = Interpreter::new(&bc);
    vm.execute().unwrap();
    assert_eq!(vm.read_gp(0), 55);
}

#[test]
fn test_factorial() {
    // 5! = 120
    let bc = [0x2B, 0x03, 0x05, 0x00,  // MOVI R3, 5
              0x2B, 0x04, 0x01, 0x00,  // MOVI R4, 1
              0x0A, 0x04, 0x04, 0x03,  // IMUL R4, R4, R3
              0x0F, 0x03,              // DEC R3
              0x06, 0x03, 0xF6, 0xFF,  // JNZ R3, -10
              0x80];
    let mut vm = Interpreter::new(&bc);
    vm.execute().unwrap();
    assert_eq!(vm.read_gp(4), 120);
}

#[test]
fn test_division_by_zero() {
    let bc = [0x2B, 0x00, 0x0A, 0x00,  // MOVI R0, 10
              0x2B, 0x01, 0x00, 0x00,  // MOVI R1, 0
              0x0B, 0x02, 0x00, 0x01,  // IDIV R2, R0, R1
              0x80];
    let mut vm = Interpreter::new(&bc);
    let result = vm.execute();
    assert!(result.is_err());
}

#[test]
fn test_ishl() {
    let bc = [0x2B, 0x00, 0x01, 0x00,  // MOVI R0, 1
              0x2B, 0x01, 0x04, 0x00,  // MOVI R1, 4
              0x14, 0x02, 0x00, 0x01,  // ISHL R2, R0, R1
              0x80];
    let mut vm = Interpreter::new(&bc);
    vm.execute().unwrap();
    assert_eq!(vm.read_gp(2), 16);
}

#[test]
fn test_ishr() {
    let bc = [0x2B, 0x00, 0x00, 0x01,  // MOVI R0, 256
              0x2B, 0x01, 0x03, 0x00,  // MOVI R1, 3
              0x15, 0x02, 0x00, 0x01,  // ISHR R2, R0, R1
              0x80];
    let mut vm = Interpreter::new(&bc);
    vm.execute().unwrap();
    assert_eq!(vm.read_gp(2), 32);
}
