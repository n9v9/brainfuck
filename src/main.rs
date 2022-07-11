use std::io;

use brainfuck::Interpreter;

fn main() {
    let interpreter = Interpreter::new(include_str!("./program.b"), io::stdin(), io::stdout());

    if let Err(e) = interpreter.execute() {
        panic!("error: {e}");
    }
}
