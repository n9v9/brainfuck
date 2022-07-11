use std::io::{self, Read, Write};

/// The memory size that is available to a Brainfuck program.
const DATA_SIZE: usize = 30_000;

/// An intepreter that can execute Brainfuck code.
pub struct Interpreter<'a, R, W> {
    /// Code to execute.
    code: &'a [u8],

    /// Instruction pointer into `code`.
    ip: usize,

    /// Zero initialized, available memory for `code`.
    data: [u8; DATA_SIZE],

    /// Data pointer into `data`.
    dp: usize,

    /// Reader to read a byte from when the input instruction is encountered.
    reader: &'a mut R,

    /// Writer to write a byte to when the output instruction is encountered.
    writer: &'a mut W,
}

impl<'a, R, W> Interpreter<'a, R, W>
where
    R: Read,
    W: Write,
{
    /// Creates a new intepreter to execute Brainfuck code.
    pub fn new(code: &'a str, reader: &'a mut R, writer: &'a mut W) -> Self {
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
    pub fn execute(&mut self) -> io::Result<()> {
        while self.ip < self.code.len() {
            let instruction = self.code[self.ip];
            match instruction {
                b'>' => {
                    self.dp += 1;
                    assert!(self.dp < DATA_SIZE);
                }
                b'<' => self.dp -= 1,
                b'+' => self.data[self.dp] = self.data[self.dp].wrapping_add(1),
                b'-' => self.data[self.dp] = self.data[self.dp].wrapping_sub(1),
                b',' => self
                    .reader
                    .read_exact(&mut self.data[self.dp..self.dp + 1])?,
                b'.' => self.writer.write_all(&self.data[self.dp..self.dp + 1])?,
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

        self.writer.flush()
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use crate::{Interpreter, DATA_SIZE};

    #[test]
    fn test_increment_dp() {
        let code = ">";
        let mut reader = Cursor::new([0]);
        let mut writer = Cursor::new(Vec::new());

        let mut interpreter = Interpreter::new(code, &mut reader, &mut writer);
        assert_eq!(interpreter.dp, 0);

        interpreter.execute().unwrap();
        assert_eq!(interpreter.dp, 1);
    }

    #[test]
    #[should_panic]
    fn test_increment_dp_overflow() {
        // Incrementing `dp` when `dp` is already the max memory size results in an overflow
        let code = ">".repeat(DATA_SIZE);
        let mut reader = Cursor::new([0]);
        let mut writer = Cursor::new(Vec::new());

        Interpreter::new(&code, &mut reader, &mut writer)
            .execute()
            .unwrap();
    }

    #[test]
    #[should_panic]
    fn test_decrement_dp_overflow() {
        // Decrementing `dp` when `dp` is already 0 results in an overflow.
        let code = "<";
        let mut reader = Cursor::new([0]);
        let mut writer = Cursor::new(Vec::new());

        Interpreter::new(code, &mut reader, &mut writer)
            .execute()
            .unwrap();
    }

    #[test]
    fn test_increment_byte_at_dp() {
        let code = "+>++";
        let mut reader = Cursor::new([0]);
        let mut writer = Cursor::new(Vec::new());

        let mut interpreter = Interpreter::new(code, &mut reader, &mut writer);
        interpreter.execute().unwrap();

        assert_eq!(interpreter.data[0], 1);
        assert_eq!(interpreter.data[1], 2);
    }

    #[test]
    fn test_decrement_byte_at_dp() {
        let code = "->--";
        let mut reader = Cursor::new([0]);
        let mut writer = Cursor::new(Vec::new());

        let mut interpreter = Interpreter::new(code, &mut reader, &mut writer);
        interpreter.execute().unwrap();

        // Wrapping overflow because `data` is with 0 initialized.
        assert_eq!(interpreter.data[0], 255);
        assert_eq!(interpreter.data[1], 254);
    }

    #[test]
    fn test_output_byte_at_dp() {
        let code = ".+.";
        let mut reader = Cursor::new([0]);
        let mut writer = Cursor::new(Vec::new());

        let mut interpreter = Interpreter::new(code, &mut reader, &mut writer);
        interpreter.execute().unwrap();

        assert_eq!(writer.get_ref()[0], 0);
        assert_eq!(writer.get_ref()[1], 1);
    }

    #[test]
    fn test_input_byte_at_dp() {
        let code = ",>,>,";
        let mut reader = Cursor::new([1, 2, 3]);
        let mut writer = Cursor::new(Vec::new());

        let mut interpreter = Interpreter::new(code, &mut reader, &mut writer);
        interpreter.execute().unwrap();

        assert_eq!(interpreter.data[0], 1);
        assert_eq!(interpreter.data[1], 2);
        assert_eq!(interpreter.data[2], 3);
    }

    #[test]
    fn test_loop_skip_to_back() {
        // Execute `+` because `data[0]` is 0 then at `]` do not jump back to `+` because `data[0]`
        // is not 0.
        let code = "[+]";
        let mut reader = Cursor::new([0]);
        let mut writer = Cursor::new(Vec::new());

        let mut interpreter = Interpreter::new(code, &mut reader, &mut writer);
        interpreter.execute().unwrap();

        assert_eq!(interpreter.data[0], 0);
    }

    #[test]
    fn test_loop_skip_to_front() {
        // Increment `data[0]` to 2, `[` executes `.-` because it is not 0,
        // then at `]` jump back to `.` because `data[0]` is not 0.
        let code = "++[.-]";
        let mut reader = Cursor::new([0]);
        let mut writer = Cursor::new(Vec::new());

        let mut interpreter = Interpreter::new(code, &mut reader, &mut writer);
        interpreter.execute().unwrap();

        assert_eq!(writer.get_ref(), &[2, 1]);
    }
}
