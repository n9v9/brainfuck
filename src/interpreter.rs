use std::io::{self, Read, Write};

use crate::syntax::{
    IDENT_DEC_DATA, IDENT_DEC_DP, IDENT_INC_DATA, IDENT_INC_DP, IDENT_JUMP_NOT_ZERO,
    IDENT_JUMP_ZERO, IDENT_READ_BYTE, IDENT_WRITE_BYTE,
};
use crate::FlushBehavior;

/// The memory size that is available to a Brainfuck program.
const DATA_SIZE: usize = 30_000;

/// An interpreter that can execute Brainfuck code.
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
    /// Creates a new interpreter to execute Brainfuck code.
    pub fn new(code: &'a str, reader: &'a mut R, writer: &'a mut W) -> Self {
        Self {
            code: code.as_bytes(),
            ip: 0,
            data: [0; DATA_SIZE],
            dp: 0,
            reader,
            writer,
        }
    }

    /// Executes the program, returning an error if reading from the reader
    /// or writing to the writer fails.
    pub fn execute(&mut self, flush: FlushBehavior) -> io::Result<()> {
        while self.ip < self.code.len() {
            let instruction = self.code[self.ip];
            match instruction {
                IDENT_INC_DP => {
                    self.dp += 1;
                    assert!(self.dp < DATA_SIZE);
                }
                IDENT_DEC_DP => self.dp -= 1,
                IDENT_INC_DATA => self.data[self.dp] = self.data[self.dp].wrapping_add(1),
                IDENT_DEC_DATA => self.data[self.dp] = self.data[self.dp].wrapping_sub(1),
                IDENT_READ_BYTE => self
                    .reader
                    .read_exact(&mut self.data[self.dp..self.dp + 1])?,
                IDENT_WRITE_BYTE => {
                    self.writer.write_all(&self.data[self.dp..self.dp + 1])?;
                    if flush == FlushBehavior::OnWrite {
                        self.writer.flush()?;
                    }
                }
                IDENT_JUMP_ZERO if self.data[self.dp] == 0 => {
                    let mut brackets = 0;
                    loop {
                        match self.code[self.ip] {
                            IDENT_JUMP_ZERO => brackets += 1,
                            IDENT_JUMP_NOT_ZERO => brackets -= 1,
                            _ => {}
                        };
                        if brackets == 0 {
                            break;
                        }
                        self.ip += 1;
                    }
                }
                IDENT_JUMP_NOT_ZERO if self.data[self.dp] != 0 => {
                    let mut brackets = 0;
                    loop {
                        match self.code[self.ip] {
                            IDENT_JUMP_ZERO => brackets -= 1,
                            IDENT_JUMP_NOT_ZERO => brackets += 1,
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

        if flush == FlushBehavior::OnEnd {
            self.writer.flush()
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::{self, Cursor};

    use crate::FlushBehavior;

    use super::{Interpreter, DATA_SIZE};

    #[test]
    fn test_increment_dp() {
        let code = ">";
        let mut reader = io::empty();
        let mut writer = Vec::new();

        let mut interpreter = Interpreter::new(code, &mut reader, &mut writer);
        assert_eq!(interpreter.dp, 0);

        interpreter.execute(FlushBehavior::OnEnd).unwrap();
        assert_eq!(interpreter.dp, 1);
    }

    #[test]
    #[should_panic]
    fn test_increment_dp_overflow() {
        // Incrementing `dp` when `dp` is already the max memory size results in an overflow
        let code = ">".repeat(DATA_SIZE);
        let mut reader = io::empty();
        let mut writer = Vec::new();

        Interpreter::new(&code, &mut reader, &mut writer)
            .execute(FlushBehavior::OnEnd)
            .unwrap();
    }

    #[test]
    #[should_panic]
    fn test_decrement_dp_overflow() {
        // Decrementing `dp` when `dp` is already 0 results in an overflow.
        let code = "<";
        let mut reader = io::empty();
        let mut writer = Vec::new();

        Interpreter::new(code, &mut reader, &mut writer)
            .execute(FlushBehavior::OnEnd)
            .unwrap();
    }

    #[test]
    fn test_increment_byte_at_dp() {
        let code = "+>++";
        let mut reader = io::empty();
        let mut writer = Vec::new();

        let mut interpreter = Interpreter::new(code, &mut reader, &mut writer);
        interpreter.execute(FlushBehavior::OnEnd).unwrap();

        assert_eq!(interpreter.data[0], 1);
        assert_eq!(interpreter.data[1], 2);
    }

    #[test]
    fn test_decrement_byte_at_dp() {
        let code = "->--";
        let mut reader = io::empty();
        let mut writer = Vec::new();

        let mut interpreter = Interpreter::new(code, &mut reader, &mut writer);
        interpreter.execute(FlushBehavior::OnEnd).unwrap();

        // Wrapping overflow because `data` is with 0 initialized.
        assert_eq!(interpreter.data[0], 255);
        assert_eq!(interpreter.data[1], 254);
    }

    #[test]
    fn test_output_byte_at_dp() {
        let code = ".+.";
        let mut reader = io::empty();
        let mut writer = Vec::new();

        let mut interpreter = Interpreter::new(code, &mut reader, &mut writer);
        interpreter.execute(FlushBehavior::OnEnd).unwrap();

        assert_eq!(writer[0], 0);
        assert_eq!(writer[1], 1);
    }

    #[test]
    fn test_input_byte_at_dp() {
        let code = ",>,>,";
        let mut reader = Cursor::new([1, 2, 3]);
        let mut writer = Vec::new();

        let mut interpreter = Interpreter::new(code, &mut reader, &mut writer);
        interpreter.execute(FlushBehavior::OnEnd).unwrap();

        assert_eq!(interpreter.data[0], 1);
        assert_eq!(interpreter.data[1], 2);
        assert_eq!(interpreter.data[2], 3);
    }

    #[test]
    fn test_loop_skip_to_back() {
        // Execute `+` because `data[0]` is 0 then at `]` do not jump back to `+` because `data[0]`
        // is not 0.
        let code = "[+]";
        let mut reader = io::empty();
        let mut writer = Vec::new();

        let mut interpreter = Interpreter::new(code, &mut reader, &mut writer);
        interpreter.execute(FlushBehavior::OnEnd).unwrap();

        assert_eq!(interpreter.data[0], 0);
    }

    #[test]
    fn test_loop_skip_to_front() {
        // Increment `data[0]` to 2, `[` executes `.-` because it is not 0,
        // then at `]` jump back to `.` because `data[0]` is not 0.
        let code = "++[.-]";
        let mut reader = io::empty();
        let mut writer = Vec::new();

        let mut interpreter = Interpreter::new(code, &mut reader, &mut writer);
        interpreter.execute(FlushBehavior::OnEnd).unwrap();

        assert_eq!(&writer, &[2, 1]);
    }

    #[test]
    fn test_program_hello_world() {
        let code = include_str!("../programs/hello_world.b");
        let mut reader = io::empty();
        let mut writer = Vec::new();

        let mut interpreter = Interpreter::new(code, &mut reader, &mut writer);
        interpreter.execute(FlushBehavior::OnEnd).unwrap();

        assert_eq!(String::from_utf8(writer), Ok("Hello World!\n".into()));
    }

    #[test]
    fn test_program_bitwidth() {
        let code = include_str!("../programs/bitwidth.b");
        let mut reader = io::empty();
        let mut writer = Vec::new();

        let mut interpreter = Interpreter::new(code, &mut reader, &mut writer);
        interpreter.execute(FlushBehavior::OnEnd).unwrap();

        assert_eq!(String::from_utf8(writer), Ok("Hello World! 255\n".into()));
    }
}
