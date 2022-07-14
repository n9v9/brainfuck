use std::collections::HashSet;

pub struct Compiler<'a> {
    code: &'a [u8],
}

impl<'a> Compiler<'a> {
    pub fn new(code: &'a str) -> Self {
        Self {
            code: code.as_bytes(),
        }
    }

    pub fn compile(&mut self) -> Vec<Instruction> {
        let mut res = Vec::new();
        let mut i = 0;
        let mut changed = false;

        while i < self.code.len() {
            self.push_instruction(&mut i, b'>', &mut res, &mut changed);
            self.push_instruction(&mut i, b'<', &mut res, &mut changed);
            self.push_instruction(&mut i, b'+', &mut res, &mut changed);
            self.push_instruction(&mut i, b'-', &mut res, &mut changed);
            self.push_instruction(&mut i, b'.', &mut res, &mut changed);
            self.push_instruction(&mut i, b',', &mut res, &mut changed);
            self.push_instruction(&mut i, b'[', &mut res, &mut changed);
            self.push_instruction(&mut i, b']', &mut res, &mut changed);

            if !changed {
                i += 1;
            }

            changed = false;
        }

        i = 0;
        while i < res.len() {
            if let Instruction::JumpIfZeroPlaceholder = res[i] {
                let mut jumps = 0;
                let mut j = i;
                loop {
                    if j == res.len() {
                        break;
                    }
                    match res[j] {
                        Instruction::JumpIfZeroPlaceholder => jumps += 1,
                        Instruction::JumpIfNotZeroPlaceholder => jumps -= 1,
                        _ => {}
                    };
                    if jumps == 0 {
                        break;
                    }
                    j += 1;
                }
                // Jump target ist the instruction after the matching backward jump.
                res[i] = Instruction::JumpIfZero(j - i + 1);
            }
            i += 1;
        }

        i = 0;
        while i < res.len() {
            if let Instruction::JumpIfZero(offset) = res[i] {
                // Jump target is the instruction after the matching backward jump.
                let target = i + offset;
                let matching_jump = target - 1;
                assert_eq!(res[matching_jump], Instruction::JumpIfNotZeroPlaceholder);
                // Jump target ist the instruction after the matching forward jump.
                res[matching_jump] = Instruction::JumpIfNotZero(matching_jump - i - 1);
            }
            i += 1;
        }

        res
    }

    fn push_instruction(
        &self,
        i: &mut usize,
        instruction: u8,
        instructions: &mut Vec<Instruction>,
        changed: &mut bool,
    ) {
        let mut args = 0;

        let valid: HashSet<_> = [b'>', b'<', b'+', b'-', b'.', b',', b'[', b']']
            .into_iter()
            .collect();

        while *i < self.code.len() {
            if self.code[*i] != instruction && !valid.contains(&self.code[*i]) {
                *i += 1;
                continue;
            } else if self.code[*i] != instruction && valid.contains(&self.code[*i]) {
                break;
            }

            args += 1;
            *i += 1;

            // These instructions can not be repeated.
            // TODO: Read and write instructions could be repeated but take no argument at the
            // moment.
            match instruction {
                b'[' | b']' | b'.' | b',' => break,
                _ => {}
            }
        }

        if args > 0 {
            *changed = true;
            instructions.push(match instruction {
                b'>' => Instruction::IncDP(args),
                b'<' => Instruction::DecDP(args),
                b'+' => Instruction::IncByteAtDP(args),
                b'-' => Instruction::DecByteAtDP(args),
                b'.' => Instruction::WriteByte,
                b',' => Instruction::ReadByte,
                b'[' => Instruction::JumpIfZeroPlaceholder,
                b']' => Instruction::JumpIfNotZeroPlaceholder,
                _ => todo!(),
            });
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Instruction {
    IncDP(usize),
    DecDP(usize),
    IncByteAtDP(usize),
    DecByteAtDP(usize),
    WriteByte,
    ReadByte,
    JumpIfZero(usize),
    JumpIfZeroPlaceholder,
    JumpIfNotZero(usize),
    JumpIfNotZeroPlaceholder,
}

#[cfg(test)]
mod tests {
    use super::{Compiler, Instruction};

    #[test]
    fn test_program_hello_world() {
        let mut compiler = Compiler::new(include_str!("../programs/hello_world.b"));
        let instructions = compiler.compile();

        // Special case regarding jumps:
        // The same consecutive instructions are compiled into one instruction with
        // an argument specifying how often to repeat the instruction. So `>>` counts
        // as one instruction instead of two.

        assert_eq!(
            instructions,
            vec![
                Instruction::IncByteAtDP(1),
                Instruction::JumpIfZero(31),
                Instruction::IncDP(1),
                Instruction::JumpIfZero(26),
                Instruction::DecDP(1),
                Instruction::DecByteAtDP(1),
                Instruction::JumpIfZero(2),
                Instruction::JumpIfNotZero(0),
                Instruction::IncDP(1),
                Instruction::IncByteAtDP(1),
                Instruction::JumpIfZero(18),
                Instruction::IncDP(1),
                Instruction::IncByteAtDP(3),
                Instruction::IncDP(1),
                Instruction::JumpIfZero(4),
                Instruction::IncByteAtDP(11),
                Instruction::IncDP(1),
                Instruction::JumpIfNotZero(2),
                Instruction::JumpIfZero(3),
                Instruction::IncDP(1),
                Instruction::JumpIfNotZero(1),
                Instruction::DecByteAtDP(1),
                Instruction::JumpIfZero(3),
                Instruction::DecDP(1),
                Instruction::JumpIfNotZero(1),
                Instruction::IncDP(1),
                Instruction::DecByteAtDP(1),
                Instruction::JumpIfNotZero(16),
                Instruction::JumpIfNotZero(24),
                Instruction::IncByteAtDP(10),
                Instruction::DecDP(1),
                Instruction::JumpIfNotZero(29),
                Instruction::IncDP(6),
                Instruction::DecByteAtDP(4),
                Instruction::WriteByte,
                Instruction::DecDP(2),
                Instruction::IncByteAtDP(3),
                Instruction::WriteByte,
                Instruction::DecDP(1),
                Instruction::DecByteAtDP(1),
                Instruction::WriteByte,
                Instruction::WriteByte,
                Instruction::IncByteAtDP(3),
                Instruction::WriteByte,
                Instruction::DecDP(1),
                Instruction::DecByteAtDP(1),
                Instruction::WriteByte,
                Instruction::IncDP(3),
                Instruction::WriteByte,
                Instruction::DecDP(2),
                Instruction::WriteByte,
                Instruction::IncByteAtDP(3),
                Instruction::WriteByte,
                Instruction::DecByteAtDP(6),
                Instruction::WriteByte,
                Instruction::IncDP(1),
                Instruction::DecByteAtDP(1),
                Instruction::WriteByte,
                Instruction::DecDP(2),
                Instruction::IncByteAtDP(1),
                Instruction::WriteByte,
                Instruction::DecDP(1),
                Instruction::WriteByte
            ]
        );
    }
}
