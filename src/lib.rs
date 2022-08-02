use syntax::IDENTS;

pub mod compiler;
pub mod interpreter;
#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
pub mod jit;
pub mod virtual_machine;

mod mmap;
mod syntax;

/// Describes when the [writer](std::io::Write) where bytes are written to is flushed.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum FlushBehavior {
    /// No call to [flush](std::io::Write::flush) will be made.
    Disabled,
    /// Call [flush](std::io::Write::flush) after every write instruction.
    OnWrite,
    /// Call [flush](std::io::Write::flush) once at the end, after all instructions have been
    /// executed.
    OnEnd,
}

/// Returns the source as a vector containing only identifiers.
///
/// This way, UTF-8 comments for example are filtered out.
fn remove_non_idents(code: &str) -> Vec<u8> {
    code.chars()
        .filter(|c| c.is_ascii() && IDENTS.contains(&(*c as u8)))
        .map(|c| c as u8)
        .collect()
}
