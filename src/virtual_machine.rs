use std::io::{self, Read, Write};

use crate::compiler::Instruction;

const DATA_SIZE: usize = 30_000;

pub struct VirtualMachine<'a, R, W> {
    instructions: &'a [Instruction],
    ip: usize,
    data: [u8; DATA_SIZE],
    dp: usize,
    reader: &'a mut R,
    writer: &'a mut W,
}

impl<'a, R, W> VirtualMachine<'a, R, W>
where
    R: Read,
    W: Write,
{
    pub fn new(instructions: &'a [Instruction], reader: &'a mut R, writer: &'a mut W) -> Self {
        Self {
            instructions,
            ip: 0,
            data: [0; DATA_SIZE],
            dp: 0,
            reader,
            writer,
        }
    }

    pub fn execute(&mut self) -> io::Result<()> {
        while self.ip < self.instructions.len() {
            match self.instructions[self.ip] {
                Instruction::IncDP(n) => {
                    self.dp += n;
                    assert!(self.dp < DATA_SIZE);
                }
                Instruction::DecDP(n) => self.dp -= n,
                Instruction::IncByteAtDP(n) => {
                    self.data[self.dp] = self.data[self.dp].wrapping_add(n as u8)
                }
                Instruction::DecByteAtDP(n) => {
                    self.data[self.dp] = self.data[self.dp].wrapping_sub(n as u8)
                }
                Instruction::ReadByte => self
                    .reader
                    .read_exact(&mut self.data[self.dp..self.dp + 1])?,
                Instruction::WriteByte => {
                    self.writer.write_all(&self.data[self.dp..self.dp + 1])?
                }
                Instruction::JumpIfZero(n) if self.data[self.dp] == 0 => {
                    self.ip += n;
                    continue;
                }
                Instruction::JumpIfNotZero(n) if self.data[self.dp] != 0 => {
                    self.ip -= n;
                    continue;
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
    use std::io;

    use crate::compiler::Compiler;

    use super::VirtualMachine;

    #[test]
    fn test_program_hello_world() {
        let mut reader = io::empty();
        let mut writer = Vec::new();

        let instructions = Compiler::new(include_str!("../programs/hello_world.b")).compile();

        VirtualMachine::new(&instructions, &mut reader, &mut writer)
            .execute()
            .unwrap();

        assert_eq!(String::from_utf8(writer), Ok("Hello World!\n".into()));
    }

    #[test]
    fn test_program_bitwidth() {
        let mut reader = io::empty();
        let mut writer = Vec::new();

        let instructions = Compiler::new(include_str!("../programs/bitwidth.b")).compile();

        VirtualMachine::new(&instructions, &mut reader, &mut writer)
            .execute()
            .unwrap();

        assert_eq!(String::from_utf8(writer), Ok("Hello World! 255\n".into()));
    }
}