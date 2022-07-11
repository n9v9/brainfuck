use std::io;

use brainfuck::Interpreter;

fn main() {
    Interpreter::new(
        include_str!("../benches/mandelbrot.b"),
        &mut io::stdin().lock(),
        &mut io::stdout().lock(),
    )
    .execute()
    .unwrap();
}
