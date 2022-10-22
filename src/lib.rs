pub mod ast;
pub mod backend;
pub mod lines;

/// Undeclared variable access error.
#[derive(Debug)]
pub struct UndeclaredVariableError<'a> {
    /// Name of the variable.
    pub name: &'a str,
    /// Byte offset into the input Instant program.
    pub byte_offset: usize,
}
