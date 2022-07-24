# Brainfuck

This repository contains the following execution environments for executing
Brainfuck programs:

- Interpreter
- Compiler
- Virtual Machine
- JIT-Compiler for x64 Linux

## Execution Environments

### Interpreter

A simple interpreter to execute Brainfuck code. It takes the program as a `&str`
and iterates over all bytes, executing valid Brainfuck instruction along the
way.

### Compiler

The compiler compiles the Brainfuck program into a list of instructions, which
can then be executed by the virtual machine; it takes the program as a `&str`
and produces a `Vec<Instruction>`.

This is more efficient than the interpreter because repeated instructions like
`+++` are represented by one instruction (`Instruction::IncDP(3)`) instead of
three single increment instructions like in the interpreter.

### Virtual Machine

The virtual machine is needed to execute the instructions generated by the
compiler. Its code base is very similar to that of the interpreter, but instead
of working with raw bytes, it uses the `Instruction` enum instead.

### JIT-Compiler

The JIT-Compiler takes instructions generated by the compiler. It then generates
machine code that is specific to x86_64 Linux systems and executes it. This is
even more performant than the virtual machine, because the generated machine
code does not have to check which instruction it has to execute; this also means
that different Brainfuck programs result in different machine code.

The data pointer is kept in the register `r12`.

#### Optimizations

The JIT-Compiler contains a few simple optimizations:

- Single `+`, `-`, `>` and `<` instructions use the `inc` or `dec` assembly
  instructions while repeating instructions use the `add` or `sub` instructions.
  See for example the function that generates machine code for the `>`
  instructions:
  ```rust
  pub fn emit_inc_dp(&mut self, n: usize) -> usize {
      let n = n as u8;
      match n {
          1 => {
              // inc r12
              self.write(&[0x49, 0xff, 0xc4])
          }
          2..=127 => {
              // add r12,<n>
              self.write(&[0x49, 0x83, 0xc4, n])
          }
          128..=255 => {
              // add r12,<n>
              self.write(&[0x49, 0x81, 0xc4, n, 0x00, 0x00, 0x00])
          }
          _ => 0,
      }
  }
  ```

#### Limitations

There are currently a few limitations:

- The generated machine code is architecture and OS specific as it relies on
  Linux system calls and 64-Bit registers.
- Input and output are hard coded to be `stdin` and `stdout` respectively,
  because the generated machine code uses the syscalls `man 2 read` and
  `man 2 write` with hard coded file descriptors.
- Because `stdin` and `stdout` are hard coded, the output of the tests for
  `jit.rs` have to be manually checked.

## Benchmarks

Running the following code that runs the `mandelbrot.b` program on my system
produces the following result:

```
Interpreter:      28.961  s
Virtual Machine:   5.157  s
JIT Compiled:    583.073 ms
```

Code:

```rust
use std::io;
use std::time::{Duration, Instant};

use brainfuck::compiler::Compiler;
use brainfuck::interpreter::Interpreter;
use brainfuck::jit::JitCompiler;
use brainfuck::virtual_machine::VirtualMachine;
use brainfuck::FlushBehavior;

const PROGRAM: &str = include_str!("../programs/mandelbrot.b");

fn main() {
    let interpreter = measure("Interpreter", interpreter);
    let vm = measure("Virtual Machine", virtual_machine);
    let jit = measure("JIT Compiler", jit);

    eprintln!("Interpreter:     {interpreter:.3?}");
    eprintln!("Virtual Machine: {vm:.3?}");
    eprintln!("JIT Compiled:    {jit:.3?}");
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
```
