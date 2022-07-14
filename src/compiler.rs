use crate::syntax::{
    IDENTS, IDENT_DEC_DATA, IDENT_DEC_DP, IDENT_INC_DATA, IDENT_INC_DP, IDENT_JUMP_NOT_ZERO,
    IDENT_JUMP_ZERO, IDENT_READ_BYTE, IDENT_WRITE_BYTE,
};

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
        let mut instructions = Vec::new();
        let mut i = 0;

        while i < self.code.len() {
            let prev_i = i;

            for ident in IDENTS.iter() {
                self.push_instruction(&mut i, *ident, &mut instructions);
            }

            if prev_i == i {
                i += 1;
            }
        }

        i = 0;
        while i < instructions.len() {
            if let Instruction::JumpZeroPlaceholder = instructions[i] {
                let mut jumps = 0;
                let mut j = i;
                loop {
                    if j == instructions.len() {
                        break;
                    }
                    match instructions[j] {
                        Instruction::JumpZeroPlaceholder => jumps += 1,
                        Instruction::JumpNotZeroPlaceholder => jumps -= 1,
                        _ => {}
                    };
                    if jumps == 0 {
                        break;
                    }
                    j += 1;
                }
                // Jump target ist the instruction after the matching backward jump.
                instructions[i] = Instruction::JumpZero(j - i + 1);
            }
            i += 1;
        }

        i = 0;
        while i < instructions.len() {
            if let Instruction::JumpZero(offset) = instructions[i] {
                // Jump target is the instruction after the matching backward jump.
                let target = i + offset;
                let matching_jump = target - 1;
                assert_eq!(
                    instructions[matching_jump],
                    Instruction::JumpNotZeroPlaceholder
                );
                // Jump target ist the instruction after the matching forward jump.
                instructions[matching_jump] = Instruction::JumpNotZero(matching_jump - i - 1);
            }
            i += 1;
        }

        instructions
    }

    fn push_instruction(
        &self,
        i: &mut usize,
        instruction: u8,
        instructions: &mut Vec<Instruction>,
    ) {
        let mut args = 0;

        while *i < self.code.len() {
            if self.code[*i] != instruction && !IDENTS.contains(&self.code[*i]) {
                *i += 1;
                continue;
            } else if self.code[*i] != instruction && IDENTS.contains(&self.code[*i]) {
                break;
            }

            args += 1;
            *i += 1;

            // These instructions can not be repeated.
            // TODO: Read and write instructions could be repeated but take no argument at the
            // moment.
            match instruction {
                IDENT_JUMP_ZERO | IDENT_JUMP_NOT_ZERO | IDENT_WRITE_BYTE | IDENT_READ_BYTE => break,
                _ => {}
            }
        }

        if args > 0 {
            instructions.push(match instruction {
                IDENT_INC_DP => Instruction::IncDP(args),
                IDENT_DEC_DP => Instruction::DecDP(args),
                IDENT_INC_DATA => Instruction::IncByteAtDP(args),
                IDENT_DEC_DATA => Instruction::DecByteAtDP(args),
                IDENT_WRITE_BYTE => Instruction::WriteByte,
                IDENT_READ_BYTE => Instruction::ReadByte,
                IDENT_JUMP_ZERO => Instruction::JumpZeroPlaceholder,
                IDENT_JUMP_NOT_ZERO => Instruction::JumpNotZeroPlaceholder,
                _ => unreachable!(),
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
    JumpZero(usize),
    JumpZeroPlaceholder,
    JumpNotZero(usize),
    JumpNotZeroPlaceholder,
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
                Instruction::JumpZero(31),
                Instruction::IncDP(1),
                Instruction::JumpZero(26),
                Instruction::DecDP(1),
                Instruction::DecByteAtDP(1),
                Instruction::JumpZero(2),
                Instruction::JumpNotZero(0),
                Instruction::IncDP(1),
                Instruction::IncByteAtDP(1),
                Instruction::JumpZero(18),
                Instruction::IncDP(1),
                Instruction::IncByteAtDP(3),
                Instruction::IncDP(1),
                Instruction::JumpZero(4),
                Instruction::IncByteAtDP(11),
                Instruction::IncDP(1),
                Instruction::JumpNotZero(2),
                Instruction::JumpZero(3),
                Instruction::IncDP(1),
                Instruction::JumpNotZero(1),
                Instruction::DecByteAtDP(1),
                Instruction::JumpZero(3),
                Instruction::DecDP(1),
                Instruction::JumpNotZero(1),
                Instruction::IncDP(1),
                Instruction::DecByteAtDP(1),
                Instruction::JumpNotZero(16),
                Instruction::JumpNotZero(24),
                Instruction::IncByteAtDP(10),
                Instruction::DecDP(1),
                Instruction::JumpNotZero(29),
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
