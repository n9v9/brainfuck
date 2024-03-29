use std::io;

use crate::compiler::Instruction;
use crate::jit::machine_code::MachineCode;
use crate::mmap::MemoryMap;

/// A JIT compiler takes instructions and turns them into machine code which can be
/// run on x64 Linux machines.
pub struct JitCompiler<'a> {
    instructions: &'a [Instruction],
    machine_code: MachineCode,
}

impl<'a> JitCompiler<'a> {
    /// Create a new JIT Compiler that executes the given instructions.
    pub fn new(instructions: &'a [Instruction]) -> Self {
        Self {
            instructions,
            machine_code: MachineCode::default(),
        }
    }

    /// Emit machine code which will then execute the given instructions.
    pub fn execute(mut self) -> io::Result<()> {
        let data = vec![0; 30_000];

        self.machine_code.emit_stack_setup(&data[0]);

        for (i, instruction) in self.instructions.iter().enumerate() {
            match instruction {
                Instruction::IncDP(n) => self.machine_code.emit_inc_dp(*n),
                Instruction::DecDP(n) => self.machine_code.emit_dec_dp(*n),
                Instruction::IncByteAtDP(n) => self.machine_code.emit_inc_byte_at_dp(*n),
                Instruction::DecByteAtDP(n) => self.machine_code.emit_dec_byte_at_dp(*n),
                Instruction::WriteByte(n) => self.machine_code.emit_write_byte_at_dp(*n),
                Instruction::ReadByte => self.machine_code.emit_read_byte_at_dp(),
                Instruction::JumpZero(n) => {
                    assert_eq!(
                        self.instructions[i + n - 1],
                        Instruction::JumpNotZero(n - 2)
                    );

                    let offset: usize = self.instructions[i + 1..i + n]
                        .iter()
                        .map(|instruction| self.get_instruction_bytes(instruction))
                        .sum();

                    self.machine_code.emit_jump_zero(offset as i32)
                }
                Instruction::JumpNotZero(n) => {
                    assert_eq!(self.instructions[i - n - 1], Instruction::JumpZero(n + 2));

                    let offset: usize = self.instructions[i - n..i]
                        .iter()
                        .map(|instruction| self.get_instruction_bytes(instruction))
                        .sum();

                    self.machine_code.emit_jump_not_zero(offset)
                }
                _ => unreachable!(),
            };
        }

        self.machine_code.emit_stack_teardown();

        let mut mmap = MemoryMap::new(self.machine_code.get_buf().len())?;
        mmap.get_mut().copy_from_slice(self.machine_code.get_buf());
        let mmap = mmap.set_executable()?;

        // SAFETY: We wrote the machine code to the memory mapped region;
        // and the machine code is valid.
        unsafe { mmap.execute() }

        Ok(())
    }

    fn get_instruction_bytes(&mut self, instruction: &Instruction) -> usize {
        self.machine_code.get_only_len(|mc| match instruction {
            Instruction::IncDP(n) => mc.emit_inc_dp(*n),
            Instruction::DecDP(n) => mc.emit_dec_dp(*n),
            Instruction::IncByteAtDP(n) => mc.emit_inc_byte_at_dp(*n),
            Instruction::DecByteAtDP(n) => mc.emit_dec_byte_at_dp(*n),
            Instruction::WriteByte(n) => mc.emit_write_byte_at_dp(*n),
            Instruction::ReadByte => mc.emit_read_byte_at_dp(),
            Instruction::JumpZero(_) => mc.emit_jump_zero(0),
            Instruction::JumpNotZero(_) => mc.emit_jump_not_zero(0),
            _ => unreachable!(),
        })
    }
}

mod machine_code {

    /// Encapsulates machine code instructions.
    #[derive(Debug, Default)]
    pub struct MachineCode {
        buf: Vec<u8>,
        suspend_write: bool,
    }

    impl MachineCode {
        pub fn emit_stack_setup(&mut self, data_start: *const u8) -> usize {
            // push rbp
            // push r12
            // mov  r12,<data_start>
            // mov  rbp,rsp
            let data_start = (data_start as usize).to_le_bytes();
            self.write(&[
                0x55,
                0x41,
                0x54,
                0x49,
                0xbc,
                data_start[0],
                data_start[1],
                data_start[2],
                data_start[3],
                data_start[4],
                data_start[5],
                data_start[6],
                data_start[7],
                0x48,
                0x89,
                0xe5,
            ])
        }

        pub fn emit_stack_teardown(&mut self) -> usize {
            // mov rsp,rbp
            // pop r12
            // pop rbp
            // ret
            self.write(&[0x48, 0x89, 0xec, 0x41, 0x5c, 0x5d, 0xc3])
        }

        pub fn emit_inc_dp(&mut self, n: usize) -> usize {
            let n = n as u8;
            match n {
                1 => {
                    // inc r12
                    self.write(&[0x49, 0xff, 0xc4])
                }
                2..=127 => {
                    // add r12,<n>
                    self.write(&[0x49, 0x83, 0xc4, n])
                }
                128..=255 => {
                    // add r12,<n>
                    self.write(&[0x49, 0x81, 0xc4, n, 0x00, 0x00, 0x00])
                }
                _ => 0,
            }
        }

        pub fn emit_dec_dp(&mut self, n: usize) -> usize {
            let n = n as u8;
            match n {
                1 => {
                    // dec r12
                    self.write(&[0x49, 0xff, 0xcc])
                }
                2..=127 => {
                    // sub r12,<n>
                    self.write(&[0x49, 0x83, 0xec, n])
                }
                128..=255 => {
                    // sub r12,<n>
                    self.write(&[0x49, 0x81, 0xec, n, 0x00, 0x00, 0x00])
                }
                _ => 0,
            }
        }

        pub fn emit_inc_byte_at_dp(&mut self, n: usize) -> usize {
            let n = n as u8;
            match n {
                1 => {
                    // inc BYTE PTR [r12]
                    self.write(&[0x41, 0xfe, 0x04, 0x24])
                }
                2..=255 => {
                    // add BYTE PTR [r12],<n>
                    self.write(&[0x41, 0x80, 0x04, 0x24, n])
                }
                _ => 0,
            }
        }

        pub fn emit_dec_byte_at_dp(&mut self, n: usize) -> usize {
            let n = n as u8;
            match n {
                1 => {
                    // dec BYTE PTR [r12]
                    self.write(&[0x41, 0xfe, 0x0c, 0x24])
                }
                2..=255 => {
                    // sub BYTE PTR [r12],<n>
                    self.write(&[0x41, 0x80, 0x2c, 0x24, n])
                }
                _ => 0,
            }
        }

        pub fn emit_write_byte_at_dp(&mut self, n: usize) -> usize {
            (0..n)
                .map(|_| {
                    // mov     eax,0x1
                    // mov     edi,0x1
                    // mov     rsi,r12
                    // mov     edx,0x1
                    // syscall
                    self.write(&[
                        0xb8, 0x01, 0x00, 0x00, 0x00, 0xbf, 0x01, 0x00, 0x00, 0x00, 0x4c, 0x89,
                        0xe6, 0xba, 0x01, 0x00, 0x00, 0x00, 0x0f, 0x05,
                    ])
                })
                .sum()
        }

        pub fn emit_read_byte_at_dp(&mut self) -> usize {
            // mov     eax,0x0
            // mov     edi,0x0
            // mov     rsi,r12
            // mov     edx,0x1
            // syscall
            self.write(&[
                0xb8, 0x00, 0x00, 0x00, 0x00, 0xbf, 0x00, 0x00, 0x00, 0x00, 0x4c, 0x89, 0xe6, 0xba,
                0x01, 0x00, 0x00, 0x00, 0x0f, 0x05,
            ])
        }

        pub fn emit_jump_zero(&mut self, skip_bytes: i32) -> usize {
            // cmp BYTE PTR [r12],0x0
            // je  <skip_bytes>
            let jump = skip_bytes.to_le_bytes();
            self.write(&[
                0x41, 0x80, 0x3c, 0x24, 0x00, 0x0f, 0x84, jump[0], jump[1], jump[2], jump[3],
            ])
        }

        pub fn emit_jump_not_zero(&mut self, skip_bytes: usize) -> usize {
            // cmp BYTE PTR [r12],0x0
            // jne <skip_bytes>

            // The current instruction is 11 bytes long.
            let jump = ((skip_bytes + 11) as i32).wrapping_neg().to_le_bytes();
            self.write(&[
                0x41, 0x80, 0x3c, 0x24, 0x00, 0x0f, 0x85, jump[0], jump[1], jump[2], jump[3],
            ])
        }

        pub fn get_only_len(&mut self, f: impl Fn(&mut Self) -> usize) -> usize {
            self.suspend_write = true;
            let len = f(self);
            self.suspend_write = false;
            len
        }

        pub fn get_buf(&self) -> &[u8] {
            &self.buf
        }

        fn write(&mut self, code: &[u8]) -> usize {
            if !self.suspend_write {
                self.buf.extend_from_slice(code);
            }
            code.len()
        }
    }
}

#[cfg(test)]
mod tests {
    // TODO: Test output must be manually checked as the generated machine code writes directly
    // to stdout.

    use crate::compiler::Compiler;
    use crate::jit::JitCompiler;

    #[test]
    fn test_program_hello_world() {
        let instructions = Compiler::new(include_str!("../programs/hello_world.b")).compile();
        JitCompiler::new(&instructions).execute().unwrap();
        // Output must be `Hello World!`.
    }

    #[test]
    fn test_program_bitwidth() {
        let instructions = Compiler::new(include_str!("../programs/bitwidth.b")).compile();
        JitCompiler::new(&instructions).execute().unwrap();
        // Output must be `Hello World! 255`.
    }
}
