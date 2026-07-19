// FLUX VM CLI — runs bytecode files and dumps register state.
// Used by the cross-implementation conformance test.
//
// Usage:
//   fluxvm --dump-regs program.bin
//   fluxvm program.bin

use std::env;
use std::fs;
use std::process;

use flux_core::vm::Interpreter;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: fluxvm [--dump-regs] <bytecode-file>");
        process::exit(1);
    }

    let mut dump_regs = false;
    let mut path: Option<&str> = None;

    for arg in &args[1..] {
        if arg == "--dump-regs" {
            dump_regs = true;
        } else {
            path = Some(arg);
        }
    }

    let path = match path {
        Some(p) => p,
        None => {
            eprintln!("Error: no bytecode file specified");
            process::exit(1);
        }
    };

    let bytecode = match fs::read(path) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Error reading {}: {}", path, e);
            process::exit(1);
        }
    };

    let mut vm = Interpreter::new(&bytecode);

    match vm.execute() {
        Ok(cycles) => {
            if dump_regs {
                for i in 0..16 {
                    println!("R{}: {}", i, vm.read_gp(i as u8));
                }
                println!("FLAG_ZERO: {}", vm.regs.flag_zero);
                println!("FLAG_SIGN: {}", vm.regs.flag_sign);
                println!("CYCLES: {}", cycles);
            } else {
                println!("Executed {} cycles", cycles);
            }
        }
        Err(e) => {
            eprintln!("VM error: {}", e);
            process::exit(1);
        }
    }
}
