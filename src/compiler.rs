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

        while i < self.code.len() {
            self.push_instruction(&mut i, b'>', &mut res);
            self.push_instruction(&mut i, b'<', &mut res);
            self.push_instruction(&mut i, b'+', &mut res);
            self.push_instruction(&mut i, b'-', &mut res);
            self.push_instruction(&mut i, b'.', &mut res);
            self.push_instruction(&mut i, b',', &mut res);
            self.push_instruction(&mut i, b'[', &mut res);
            self.push_instruction(&mut i, b']', &mut res);
        }

        i = 0;
        while i < res.len() {
            if let Instruction::JumpIfZeroPlaceholder = res[i] {
                let mut jumps = 0;
                let mut j = i;
                loop {
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
                res[i] = Instruction::JumpIfZero(j - i + 1);
            }
            i += 1;
        }

        i = 0;
        while i < res.len() {
            if let Instruction::JumpIfNotZeroPlaceholder = res[i] {
                let mut jumps = 0;
                let mut j = i;
                loop {
                    match res[j] {
                        Instruction::JumpIfZero(_) => jumps -= 1,
                        Instruction::JumpIfNotZeroPlaceholder => jumps += 1,
                        _ => {}
                    };
                    if jumps == 0 {
                        break;
                    }
                    j -= 1;
                }
                res[i] = Instruction::JumpIfNotZero(i - j - 1);
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
    ) {
        let mut args = 0;

        while *i < self.code.len() && self.code[*i] == instruction {
            args += 1;
            *i += 1;
        }

        if args > 0 {
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
    fn test_compile() {
        let code = "[]+[>>][,.--++][]";
        let mut compiler = Compiler::new(code);
        let instructions = compiler.compile();

        // Special case regarding jumps:
        // The same consecutive instructions are compiled into one instruction with
        // an argument specifying how often to repeat the instruction. So `>>` counts
        // as one instruction instead of two.

        assert_eq!(
            instructions,
            vec![
                Instruction::JumpIfZero(2),
                Instruction::JumpIfNotZero(0),
                Instruction::IncByteAtDP(1),
                Instruction::JumpIfZero(3),
                Instruction::IncDP(2),
                Instruction::JumpIfNotZero(1),
                Instruction::JumpIfZero(6),
                Instruction::ReadByte,
                Instruction::WriteByte,
                Instruction::DecByteAtDP(2),
                Instruction::IncByteAtDP(2),
                Instruction::JumpIfNotZero(4),
                Instruction::JumpIfZero(2),
                Instruction::JumpIfNotZero(0),
            ]
        );
    }
}
