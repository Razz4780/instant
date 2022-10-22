pub mod jasmin;
pub mod llvm;

use crate::{ast::Stmt, UndeclaredVariableError};
use std::fmt::Display;

/// Trait for genereting specific representation from an Instant program.
pub trait Backend {
    type Representation: Display;

    /// This method generated a specific representation of the given Instant program.
    fn process<'a>(
        &self,
        program: &[Stmt<'a>],
    ) -> Result<Self::Representation, UndeclaredVariableError<'a>>;
}
