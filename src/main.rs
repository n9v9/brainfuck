use std::io;
use std::time::{Duration, Instant};

use brainfuck::compiler::{Compiler, Instruction};
use brainfuck::interpreter::Interpreter;
use brainfuck::jit::JitCompiler;
use brainfuck::virtual_machine::VirtualMachine;
use brainfuck::FlushBehavior;

const PROGRAM: &str = include_str!("../programs/mandelbrot.b");

fn main() {
    let interpreter = measure("Interpreter", interpreter);
    let vm = measure("Virtual Machine", virtual_machine);
    let jit = measure("JIT Compiler", jit);

    eprintln!("Interpreter:     {interpreter:?}");
    eprintln!("Virtual Machine: {vm:?}");
    eprintln!("JIT Compiled:    {jit:?}");
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
    JitCompiler::new(&Compiler::new(PROGRAM).compile())
        .execute()
        .unwrap();
}

fn measure(desc: &str, f: impl Fn()) -> Duration {
    eprintln!("{desc}:\n");
    let start = Instant::now();
    f();
    let time = start.elapsed();
    eprintln!();
    time
}
