pub mod compiler;
pub mod interpreter;
pub mod virtual_machine;

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
