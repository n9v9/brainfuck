use std::fs::File;
use std::io::{self, Read};
use std::str::FromStr;

use anyhow::{Context, Result};
use argh::FromArgs;
use brainfuck::compiler::Compiler;
use brainfuck::interpreter::Interpreter;
use brainfuck::jit::JitCompiler;
use brainfuck::virtual_machine::VirtualMachine;
use brainfuck::FlushBehavior;

/// Execute Brainfuck programs and choose the execution environment to run them in.
#[derive(FromArgs, Debug)]
struct Args {
    /// execution environment to run the brainfuck program in (`interpreter`, `vm` or `jit`)
    #[argh(option, default = "Environment::JitCompiler")]
    env: Environment,

    /// the brainfuck program to execute
    #[argh(positional)]
    file: String,
}

#[derive(Debug)]
enum Environment {
    Interpreter,
    VirtualMachine,
    JitCompiler,
}

impl FromStr for Environment {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "interpreter" => Ok(Environment::Interpreter),
            "vm" => Ok(Environment::VirtualMachine),
            "jit" => Ok(Environment::JitCompiler),
            _ => Err(r#"

    valid values:
    - `interpreter` to use the interpreter     (slow)
    - `vm`          to use the virtual machine (faster)
    - `jit`         to use the jit compiler    (fastest but fallbacks to `vm` on non x64 Linux systems)"#
                .to_string()),
        }
    }
}

fn main() -> Result<()> {
    let args: Args = argh::from_env();

    let mut program = String::new();

    File::open(&args.file)
        .with_context(|| format!("failed to open file {}", args.file))?
        .read_to_string(&mut program)
        .with_context(|| format!("failed to read file {}", args.file))?;

    match args.env {
        Environment::Interpreter => run_interpreter(&program),
        Environment::VirtualMachine => run_virtual_machine(&program),
        Environment::JitCompiler => run_jit_compiler(&program),
    }
}

fn run_interpreter(program: &str) -> Result<()> {
    Interpreter::new(program, &mut io::stdin().lock(), &mut io::stdout().lock())
        .execute(FlushBehavior::OnWrite)
        .context("failed to execute the program with the interpreter")
}

fn run_virtual_machine(program: &str) -> Result<()> {
    VirtualMachine::new(
        &Compiler::new(program).compile(),
        &mut io::stdin().lock(),
        &mut io::stdout().lock(),
    )
    .execute(FlushBehavior::OnWrite)
    .context("failed to execute the program on the virtual machine")
}

fn run_jit_compiler(program: &str) -> Result<()> {
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    return JitCompiler::new(&Compiler::new(program).compile())
        .execute()
        .context("failed to execute the program with the jit compiler");

    #[cfg(not(all(target_os = "linux", target_arch = "x86_64")))]
    run_virtual_machine(program)
}
