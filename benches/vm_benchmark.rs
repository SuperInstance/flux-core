use flux_core::vm::Interpreter;

fn factorial_bytecode() -> Vec<u8> {
    vec![
        0x2B, 0x03, 0x0A, 0x00,  // MOVI R3, 10
        0x2B, 0x04, 0x01, 0x00,  // MOVI R4, 1
        0x0A, 0x04, 0x03,        // IMUL R4, R3
        0x0F, 0x03,              // DEC R3
        0x06, 0x03, 0xF7, 0xFF,  // JNZ R3, -9
        0x80,                    // HALT
    ]
}

fn criterion_benchmark(c: &mut criterion::Criterion) {
    let bc = factorial_bytecode();
    c.bench_function("factorial_10", |b| {
        b.iter(|| {
            let mut vm = Interpreter::new(&bc);
            vm.execute().unwrap()
        })
    });
}

criterion::criterion_group!(benches, criterion_benchmark);
criterion::criterion_main!(benches);
