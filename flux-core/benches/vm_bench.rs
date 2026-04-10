//! Benchmarks for the FLUX VM.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use flux_core::*;

fn benchmark_arithmetic(c: &mut Criterion) {
    let mut group = c.benchmark_group("arithmetic");

    // Benchmark simple arithmetic operations
    let bytecode = vec![
        0x2B, 0x00, 0x0A, 0x00, // MOVI R0, 10
        0x2B, 0x01, 0x14, 0x00, // MOVI R1, 20
        0x08, 0x00, 0x01, 0x02, // IADD R0, R1, R2
        0x80,                   // HALT
    ];

    group.bench_function("add", |b| {
        b.iter(|| {
            let mut vm = Interpreter::new();
            vm.load_bytecode(black_box(&bytecode)).unwrap();
            vm.run().unwrap();
            black_box(vm.registers().get_gp(0).unwrap());
        });
    });

    // Benchmark multiplication
    let bytecode = vec![
        0x2B, 0x00, 0x0A, 0x00, // MOVI R0, 10
        0x2B, 0x01, 0x14, 0x00, // MOVI R1, 20
        0x0A, 0x00, 0x01, 0x02, // IMUL R0, R1, R2
        0x80,                   // HALT
    ];

    group.bench_function("mul", |b| {
        b.iter(|| {
            let mut vm = Interpreter::new();
            vm.load_bytecode(black_box(&bytecode)).unwrap();
            vm.run().unwrap();
            black_box(vm.registers().get_gp(0).unwrap());
        });
    });

    group.finish();
}

fn benchmark_loops(c: &mut Criterion) {
    let mut group = c.benchmark_group("loops");

    // Benchmark small loop (10 iterations)
    let bytecode = vec![
        0x2B, 0x00, 0x00, 0x00, // MOVI R0, 0
        0x2B, 0x01, 0x0A, 0x00, // MOVI R1, 10
        0x2B, 0x02, 0x01, 0x00, // MOVI R2, 1
    loop_add: // 0x0C
        0x08, 0x00, 0x00, 0x02, // IADD R0, R0, R2
        0x0F, 0x01,             // DEC R1
        0x06, 0x00, 0xF8, 0xFF, // JNZ -8
        0x80,                   // HALT
    ];

    group.bench_function("loop_10", |b| {
        b.iter(|| {
            let mut vm = Interpreter::new();
            vm.load_bytecode(black_box(&bytecode)).unwrap();
            vm.run().unwrap();
            black_box(vm.registers().get_gp(0).unwrap());
        });
    });

    // Benchmark medium loop (100 iterations)
    let bytecode = vec![
        0x2B, 0x00, 0x00, 0x00, // MOVI R0, 0
        0x2B, 0x01, 0x64, 0x00, // MOVI R1, 100
        0x2B, 0x02, 0x01, 0x00, // MOVI R2, 1
        0x08, 0x00, 0x00, 0x02, // IADD R0, R0, R2
        0x0F, 0x01,             // DEC R1
        0x06, 0x00, 0xF9, 0xFF, // JNZ -7
        0x80,                   // HALT
    ];

    group.bench_function("loop_100", |b| {
        b.iter(|| {
            let mut vm = Interpreter::new();
            vm.load_bytecode(black_box(&bytecode)).unwrap();
            vm.run().unwrap();
            black_box(vm.registers().get_gp(0).unwrap());
        });
    });

    group.finish();
}

fn benchmark_stack_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("stack");

    // Benchmark push/pop
    let bytecode = vec![
        0x2B, 0x00, 0x2A, 0x00, // MOVI R0, 42
        0x20, 0x00,             // PUSH R0
        0x2B, 0x00, 0x00, 0x00, // MOVI R0, 0
        0x21, 0x01,             // POP R1
        0x80,                   // HALT
    ];

    group.bench_function("push_pop", |b| {
        b.iter(|| {
            let mut vm = Interpreter::new();
            vm.load_bytecode(black_box(&bytecode)).unwrap();
            vm.run().unwrap();
            black_box(vm.registers().get_gp(1).unwrap());
        });
    });

    group.finish();
}

fn benchmark_factorial(c: &mut Criterion) {
    let mut group = c.benchmark_group("algorithms");

    // Benchmark factorial(10)
    let bytecode = vec![
        0x2B, 0x00, 0x0A, 0x00, // MOVI R0, 10 (n = 10)
        0x2B, 0x01, 0x01, 0x00, // MOVI R1, 1 (result = 1)
        0x0A, 0x01, 0x01, 0x00, // IMUL R1, R1, R0 (loop start)
        0x0F, 0x00,             // DEC R0
        0x2D, 0x00, 0x00,       // CMP R0, 0
        0x06, 0x00, 0xF8, 0xFF, // JNZ -8 (jump to loop)
        0x80,                   // HALT
    ];

    group.bench_function("factorial_10", |b| {
        b.iter(|| {
            let mut vm = Interpreter::new();
            vm.load_bytecode(black_box(&bytecode)).unwrap();
            vm.run().unwrap();
            black_box(vm.registers().get_gp(1).unwrap());
        });
    });

    // Benchmark fibonacci(15)
    let bytecode = vec![
        0x2B, 0x00, 0x00, 0x00, // MOVI R0, 0 (prev)
        0x2B, 0x01, 0x01, 0x00, // MOVI R1, 1 (curr)
        0x2B, 0x02, 0x0E, 0x00, // MOVI R2, 14 (iterations)
        0x2B, 0x03, 0x00, 0x00, // MOVI R3, 0 (temp)
        0x01, 0x03, 0x01,       // MOV R3, R1 (loop start)
        0x08, 0x01, 0x01, 0x00, // IADD R1, R1, R0
        0x01, 0x00, 0x03,       // MOV R0, R3
        0x0F, 0x02,             // DEC R2
        0x06, 0x00, 0xF6, 0xFF, // JNZ -10
        0x80,                   // HALT
    ];

    group.bench_function("fibonacci_15", |b| {
        b.iter(|| {
            let mut vm = Interpreter::new();
            vm.load_bytecode(black_box(&bytecode)).unwrap();
            vm.run().unwrap();
            black_box(vm.registers().get_gp(1).unwrap());
        });
    });

    group.finish();
}

fn benchmark_assembler(c: &mut Criterion) {
    let mut group = c.benchmark_group("assembler");

    let program = r#"
        ; Calculate factorial
        MOVI R0, 10
        MOVI R1, 1
    loop:
        IMUL R1, R1, R0
        DEC R0
        CMP R0, 0
        JNZ -4
        HALT
    "#;

    group.bench_function("assemble_program", |b| {
        b.iter(|| {
            let mut asm = Assembler::new();
            let bytecode = asm.assemble(black_box(program)).unwrap();
            black_box(bytecode);
        });
    });

    group.finish();
}

fn benchmark_disassembler(c: &mut Criterion) {
    let mut group = c.benchmark_group("disassembler");

    let bytecode = vec![
        0x2B, 0x00, 0x0A, 0x00, // MOVI R0, 10
        0x2B, 0x01, 0x14, 0x00, // MOVI R1, 20
        0x08, 0x00, 0x01, 0x02, // IADD R0, R1, R2
        0x80,                   // HALT
    ];

    group.bench_function("disassemble_program", |b| {
        b.iter(|| {
            let disasm = Disassembler::minimal();
            let text = disasm.disassemble(black_box(&bytecode)).unwrap();
            black_box(text);
        });
    });

    group.finish();
}

fn benchmark_a2a(c: &mut Criterion) {
    let mut group = c.benchmark_group("a2a");

    let msg = A2AMessage::new(
        [1u8; 16],
        [2u8; 16],
        [3u8; 16],
        MessageType::Tell,
        b"hello world".to_vec(),
        0.9,
        1234567890,
    );

    group.bench_function("serialize", |b| {
        b.iter(|| {
            let bytes = black_box(&msg).serialize();
            black_box(bytes);
        });
    });

    let bytes = msg.serialize();

    group.bench_function("deserialize", |b| {
        b.iter(|| {
            let decoded = A2AMessage::deserialize(black_box(&bytes)).unwrap();
            black_box(decoded);
        });
    });

    group.finish();
}

fn benchmark_ops_per_second(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput");

    // Create a program with 1000 NOPs followed by HALT
    let mut bytecode = vec![0x00u8; 1000]; // 1000 NOPs
    bytecode.push(0x80); // HALT

    group.bench_function("ops_per_second", |b| {
        b.iter(|| {
            let mut vm = Interpreter::new();
            vm.load_bytecode(black_box(&bytecode)).unwrap();
            vm.run().unwrap();
            black_box(vm.instructions_executed());
        });
    });

    group.throughput(criterion::Throughput::Elements(
        bytecode.len() as u64,
    ));

    group.finish();
}

fn benchmark_memory_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory");

    // Benchmark LOAD
    let bytecode = vec![
        0x2B, 0x01, 0x00, 0x10, // MOVI R1, 4096 (address)
        0x02, 0x00, 0x01,       // LOAD R0, R1
        0x80,                   // HALT
    ];

    group.bench_function("load", |b| {
        b.iter(|| {
            let mut vm = Interpreter::new();
            // Write some data at address 4096
            vm.memory_mut().write_i32(4096, 42).unwrap();
            vm.load_bytecode(black_box(&bytecode)).unwrap();
            vm.run().unwrap();
            black_box(vm.registers().get_gp(0).unwrap());
        });
    });

    // Benchmark STORE
    let bytecode = vec![
        0x2B, 0x00, 0x2A, 0x00, // MOVI R0, 42
        0x2B, 0x01, 0x00, 0x10, // MOVI R1, 4096
        0x03, 0x01, 0x00,       // STORE R1, R0
        0x80,                   // HALT
    ];

    group.bench_function("store", |b| {
        b.iter(|| {
            let mut vm = Interpreter::new();
            vm.load_bytecode(black_box(&bytecode)).unwrap();
            vm.run().unwrap();
            black_box(vm.memory().read_i32(4096).unwrap());
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_arithmetic,
    benchmark_loops,
    benchmark_stack_operations,
    benchmark_factorial,
    benchmark_assembler,
    benchmark_disassembler,
    benchmark_a2a,
    benchmark_ops_per_second,
    benchmark_memory_operations
);

criterion_main!(benches);
