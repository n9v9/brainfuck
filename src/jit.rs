use std::arch::asm;
use std::io;

use crate::compiler::Instruction;
use crate::mmap::MemoryMap;

pub struct Jit<'a> {
    instructions: &'a [Instruction],
}

impl<'a> Jit<'a> {
    pub fn new(instructions: &'a [Instruction]) -> Self {
        Self { instructions }
    }

    pub fn execute(self) -> io::Result<()> {
        let mut code = Vec::new();

        copy_from_slice(&mut code, emit_stack_setup(), 1);

        for (i, instruction) in self.instructions.iter().enumerate() {
            match instruction {
                Instruction::IncDP(n) => copy_from_slice(&mut code, emit_inc_dp(), *n),
                Instruction::DecDP(n) => copy_from_slice(&mut code, emit_dec_dp(), *n),
                Instruction::IncByteAtDP(n) => {
                    copy_from_slice(&mut code, emit_inc_byte_at_dp(), *n)
                }
                Instruction::DecByteAtDP(n) => {
                    copy_from_slice(&mut code, emit_dec_byte_at_dp(), *n)
                }
                Instruction::WriteByte(n) => {
                    copy_from_slice(&mut code, emit_write_byte_at_dp(), *n)
                }
                Instruction::ReadByte => copy_from_slice(&mut code, emit_read_byte_at_dp(), 1),
                Instruction::JumpZero(n) => {
                    assert_eq!(
                        self.instructions[i + n - 1],
                        Instruction::JumpNotZero(n - 2)
                    );

                    let offset: usize = self.instructions[i + 1..i + n]
                        .iter()
                        .map(get_instruction_bytes)
                        .sum();

                    copy_from_slice(&mut code, &emit_jump_zero(offset as i32), 1);
                }
                Instruction::JumpNotZero(n) => {
                    assert_eq!(self.instructions[i - n - 1], Instruction::JumpZero(n + 2));

                    let offset: usize = self.instructions[i - n..i]
                        .iter()
                        .map(get_instruction_bytes)
                        .sum();

                    copy_from_slice(&mut code, &emit_jump_not_zero(offset), 1);
                }
                _ => unreachable!(),
            }
        }

        copy_from_slice(&mut code, emit_stack_teardown(), 1);

        let mut mmap = MemoryMap::new(code.len())?;
        mmap.get_mut().copy_from_slice(&code);
        let mmap = mmap.set_executable()?;

        let data = vec![0; 30_000];

        unsafe {
            asm!("mov r12, {}", in(reg) &data[0]);
        }

        mmap.execute();

        Ok(())
    }
}

fn get_instruction_bytes(instruction: &Instruction) -> usize {
    match instruction {
        Instruction::IncDP(n) => (0..*n).map(|_| emit_inc_dp().len()).sum(),
        Instruction::DecDP(n) => (0..*n).map(|_| emit_dec_dp().len()).sum(),
        Instruction::IncByteAtDP(n) => (0..*n).map(|_| emit_inc_byte_at_dp().len()).sum(),
        Instruction::DecByteAtDP(n) => (0..*n).map(|_| emit_dec_byte_at_dp().len()).sum(),
        Instruction::WriteByte(n) => (0..*n).map(|_| emit_write_byte_at_dp().len()).sum(),
        Instruction::ReadByte => emit_read_byte_at_dp().len(),
        Instruction::JumpZero(_) => emit_jump_zero(0).len(),
        Instruction::JumpNotZero(_) => emit_jump_not_zero(0).len(),
        _ => unreachable!(),
    }
}

fn copy_from_slice(dest: &mut Vec<u8>, src: &[u8], n: usize) {
    for _ in 0..n {
        dest.extend_from_slice(src);
    }
}

fn emit_stack_setup() -> &'static [u8] {
    // push rbp
    // mov  rbp,rsp
    &[0x55, 0x48, 0x89, 0xe5]
}

fn emit_stack_teardown() -> &'static [u8] {
    // mov rsp,rbp
    // pop rbp
    // ret
    &[0x48, 0x89, 0xec, 0x5d, 0xc3]
}

fn emit_inc_dp() -> &'static [u8] {
    // inc r12
    &[0x49, 0xff, 0xc4]
}

fn emit_dec_dp() -> &'static [u8] {
    // dec r12
    &[0x49, 0xff, 0xcc]
}

fn emit_inc_byte_at_dp() -> &'static [u8] {
    // inc BYTE PTR [r12]
    &[0x41, 0xfe, 0x04, 0x24]
}

fn emit_dec_byte_at_dp() -> &'static [u8] {
    // dec BYTE PTR [r12]
    &[0x41, 0xfe, 0x0c, 0x24]
}

fn emit_write_byte_at_dp() -> &'static [u8] {
    // mov     eax,0x1
    // mov     edi,0x1
    // mov     rsi,r12
    // mov     edx,0x1
    // syscall
    &[
        0xb8, 0x01, 0x00, 0x00, 0x00, 0xbf, 0x01, 0x00, 0x00, 0x00, 0x4c, 0x89, 0xe6, 0xba, 0x01,
        0x00, 0x00, 0x00, 0x0f, 0x05,
    ]
}

fn emit_read_byte_at_dp() -> &'static [u8] {
    // mov     eax,0x0
    // mov     edi,0x0
    // mov     rsi,r12
    // mov     edx,0x1
    // syscall
    &[
        0xb8, 0x00, 0x00, 0x00, 0x00, 0xbf, 0x00, 0x00, 0x00, 0x00, 0x4c, 0x89, 0xe6, 0xba, 0x01,
        0x00, 0x00, 0x00, 0x0f, 0x05,
    ]
}

fn emit_jump_zero(skip_bytes: i32) -> [u8; 11] {
    // cmp BYTE PTR [r12],0x0
    // je  <offset>
    let jump = skip_bytes.to_le_bytes();
    [
        0x41, 0x80, 0x3c, 0x24, 0x00, 0x0f, 0x84, jump[0], jump[1], jump[2], jump[3],
    ]
}

fn emit_jump_not_zero(skip_bytes: usize) -> [u8; 11] {
    // cmp BYTE PTR [r12],0x0
    // jne  <offset>

    // The current instruction is 11 bytes long.
    let jump = ((skip_bytes + 11) as i32).wrapping_neg().to_le_bytes();
    [
        0x41, 0x80, 0x3c, 0x24, 0x00, 0x0f, 0x85, jump[0], jump[1], jump[2], jump[3],
    ]
}

#[cfg(test)]
mod tests {
    // TODO: Test output must be manually checked as the generated machine code writes directly
    // to stdout.

    use crate::compiler::Compiler;
    use crate::jit::Jit;

    #[test]
    fn test_program_hello_world() {
        let instructions = Compiler::new(include_str!("../programs/hello_world.b")).compile();
        Jit::new(&instructions).execute().unwrap();
        // Output must be `Hello World!`.
    }

    #[test]
    fn test_program_bitwidth() {
        let instructions = Compiler::new(include_str!("../programs/bitwidth.b")).compile();
        Jit::new(&instructions).execute().unwrap();
        // Output must be `Hello World! 255`.
    }
}
