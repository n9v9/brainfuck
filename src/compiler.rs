use crate::syntax::{
    IDENTS, IDENT_DEC_DATA, IDENT_DEC_DP, IDENT_INC_DATA, IDENT_INC_DP, IDENT_JUMP_NOT_ZERO,
    IDENT_JUMP_ZERO, IDENT_READ_BYTE, IDENT_WRITE_BYTE,
};

/// A compiler that turns a Brainfuck program into a list of instructions.
pub struct Compiler<'a> {
    code: &'a [u8],
}

impl<'a> Compiler<'a> {
    /// Create a new Compiler.
    pub fn new(code: &'a str) -> Self {
        Self {
            code: code.as_bytes(),
        }
    }

    /// Analyze the given program and return a list of instructions to execute.
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

            // Jump instructions can not be folded.
            match instruction {
                IDENT_JUMP_ZERO | IDENT_JUMP_NOT_ZERO => break,
                _ => {}
            }
        }

        if args > 0 {
            instructions.push(match instruction {
                IDENT_INC_DP => Instruction::IncDP(args),
                IDENT_DEC_DP => Instruction::DecDP(args),
                IDENT_INC_DATA => Instruction::IncByteAtDP(args),
                IDENT_DEC_DATA => Instruction::DecByteAtDP(args),
                IDENT_WRITE_BYTE => Instruction::WriteByte(args),
                IDENT_READ_BYTE => Instruction::ReadByte,
                IDENT_JUMP_ZERO => Instruction::JumpZeroPlaceholder,
                IDENT_JUMP_NOT_ZERO => Instruction::JumpNotZeroPlaceholder,
                _ => unreachable!(),
            });
        }
    }
}

/// Represents an instruction to execute.
/// The same instruction repeated multiple times is folded into one instruction
/// with the number of repetitions as its argument.
#[derive(Debug, PartialEq)]
pub enum Instruction {
    /// Increase the data pointer.
    IncDP(usize),

    /// Decrease the data pointer.
    DecDP(usize),

    /// Increase the byte at the data pointer.
    IncByteAtDP(usize),

    /// Decrease the byte at the data pointer.
    DecByteAtDP(usize),

    /// Write the byte at the data pointer to the writer.
    WriteByte(usize),

    /// Read a byte from the reader into the byte at the data pointer.
    ReadByte,

    /// If the byte at the data pointer is zero, jump to the instruction after the matching
    /// `JumpNotZero` instruction.
    JumpZero(usize),

    /// Used to determine the relative offset to `JumpNotZero` in the compilation step.
    JumpZeroPlaceholder,

    /// If the byte at the data pointer is not zero, jump to the instruction after the
    /// matching `JumpZero` instruction.
    JumpNotZero(usize),

    /// Used to determine the relative offset to `JumpZero` in the compilation step.
    JumpNotZeroPlaceholder,
}

#[cfg(test)]
mod tests {
    use super::{Compiler, Instruction};

    #[test]
    fn test_remove_repeating_reads() {
        let instructions = Compiler::new(",,,,,.,,,.,").compile();

        assert_eq!(
            instructions,
            vec![
                Instruction::ReadByte,
                Instruction::WriteByte(1),
                Instruction::ReadByte,
                Instruction::WriteByte(1),
                Instruction::ReadByte
            ]
        );
    }

    #[test]
    fn test_program_hello_world() {
        let instructions = Compiler::new(include_str!("../programs/hello_world.b")).compile();

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
                Instruction::WriteByte(1),
                Instruction::DecDP(2),
                Instruction::IncByteAtDP(3),
                Instruction::WriteByte(1),
                Instruction::DecDP(1),
                Instruction::DecByteAtDP(1),
                Instruction::WriteByte(2),
                Instruction::IncByteAtDP(3),
                Instruction::WriteByte(1),
                Instruction::DecDP(1),
                Instruction::DecByteAtDP(1),
                Instruction::WriteByte(1),
                Instruction::IncDP(3),
                Instruction::WriteByte(1),
                Instruction::DecDP(2),
                Instruction::WriteByte(1),
                Instruction::IncByteAtDP(3),
                Instruction::WriteByte(1),
                Instruction::DecByteAtDP(6),
                Instruction::WriteByte(1),
                Instruction::IncDP(1),
                Instruction::DecByteAtDP(1),
                Instruction::WriteByte(1),
                Instruction::DecDP(2),
                Instruction::IncByteAtDP(1),
                Instruction::WriteByte(1),
                Instruction::DecDP(1),
                Instruction::WriteByte(1)
            ]
        );
    }
}
