use std::io::{self, Read, Write};

/// An intepreter that can execute Brainfuck code.
pub struct Interpreter<'a, R, W> {
    /// Code to execute.
    code: &'a [u8],

    /// Instruction pointer into `code`.
    ip: usize,

    /// Zero initialized, available memory for `code`.
    data: [u8; 30_000],

    /// Data pointer into `data`.
    dp: usize,

    /// Reader to read a byte from when the input instruction is encountered.
    reader: R,

    /// Writer to write a byte to when the output instruction is encountered.
    writer: W,
}

impl<'a, R, W> Interpreter<'a, R, W>
where
    R: Read,
    W: Write,
{
    /// Creates a new intepreter to execute Brainfuck code.
    pub fn new(code: &'a str, reader: R, writer: W) -> Self {
        Self {
            code: code.as_bytes(),
            ip: 0,
            data: [0; 30_000],
            dp: 0,
            reader,
            writer,
        }
    }

    /// Executes the program, returning an error if reading from the reader
    /// or writing to the writer fails.
    pub fn execute(mut self) -> io::Result<()> {
        while self.ip < self.code.len() {
            let instruction = self.code[self.ip];
            match instruction {
                b'>' => self.dp += 1,
                b'<' => self.dp -= 1,
                b'+' => self.data[self.dp] += 1,
                b'-' => self.data[self.dp] -= 1,
                b',' => self
                    .reader
                    .read_exact(&mut self.data[self.dp..self.dp + 1])?,
                b'.' => {
                    self.writer.write_all(&self.data[self.dp..self.dp + 1])?;
                    self.writer.flush()?;
                }
                b'[' if self.data[self.dp] == 0 => {
                    let mut brackets = 0;
                    loop {
                        match self.code[self.ip] {
                            b'[' => brackets += 1,
                            b']' => brackets -= 1,
                            _ => {}
                        };
                        if brackets == 0 {
                            break;
                        }
                        self.ip += 1;
                    }
                }
                b']' if self.data[self.dp] != 0 => {
                    let mut brackets = 0;
                    loop {
                        match self.code[self.ip] {
                            b'[' => brackets -= 1,
                            b']' => brackets += 1,
                            _ => {}
                        };
                        if brackets == 0 {
                            break;
                        }
                        self.ip -= 1;
                    }
                }
                _ => {}
            }

            self.ip += 1;
        }

        Ok(())
    }
}
