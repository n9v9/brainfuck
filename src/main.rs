use std::io;
use std::time::{Duration, Instant};

use brainfuck::compiler::Compiler;
use brainfuck::interpreter::Interpreter;
use brainfuck::virtual_machine::VirtualMachine;
use brainfuck::FlushBehavior;

static FLUSH: FlushBehavior = FlushBehavior::OnEnd;

fn main() {
    let program = include_str!("../programs/mandelbrot.b");

    let interpreter = measure(|| run_interpreter(program));
    let virtual_machine = measure(|| run_virtual_machine(program));

    eprintln!("Interpreter:     {:?}", interpreter);
    eprintln!("Virtual machine: {:?}", virtual_machine);
}

fn run_interpreter(program: &str) {
    let mut reader = io::empty();
    let mut writer = io::stdout();

    Interpreter::new(program, &mut reader, &mut writer)
        .execute(FLUSH)
        .unwrap();
}

fn run_virtual_machine(program: &str) {
    let mut reader = io::empty();
    let mut writer = io::stdout();

    let instructions = Compiler::new(program).compile();

    VirtualMachine::new(&instructions, &mut reader, &mut writer)
        .execute(FLUSH)
        .unwrap();
}

fn measure(f: impl FnOnce()) -> Duration {
    let now = Instant::now();
    f();
    now.elapsed()
}
