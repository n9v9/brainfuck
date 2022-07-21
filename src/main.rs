use std::io;
use std::time::{Duration, Instant};

use brainfuck::compiler::{Compiler, Instruction};
use brainfuck::interpreter::Interpreter;
use brainfuck::jit::Jit;
use brainfuck::virtual_machine::VirtualMachine;
use brainfuck::FlushBehavior;

const PROGRAM: &str = include_str!("../programs/mandelbrot.b");

fn main() {
    measure(interpreter);
    measure(virtual_machine);
    measure(jit);
}

fn interpreter() {
    Interpreter::new(PROGRAM, &mut io::empty(), &mut io::stdout().lock())
        .execute(FlushBehavior::OnWrite)
        .unwrap();
}

fn virtual_machine() {
    VirtualMachine::new(
        &Compiler::new(PROGRAM).compile(),
        &mut io::empty(),
        &mut io::stdout().lock(),
    )
    .execute(FlushBehavior::OnWrite)
    .unwrap();
}

fn jit() {
    Jit::new(&Compiler::new(PROGRAM).compile())
        .execute()
        .unwrap();
}

fn measure(f: impl Fn()) {
    let start = Instant::now();
    f();
    eprintln!("Elapsed: {:?}", start.elapsed());
}
