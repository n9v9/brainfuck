use std::{io, os};

use brainfuck::interpreter::Interpreter;

fn main() {
    let mut reader = io::empty();
    let mut writer = io::stdout();

    Interpreter::new(
        include_str!("../programs/hello_world.b"),
        &mut reader,
        &mut writer,
    )
    .execute()
    .unwrap();
}
