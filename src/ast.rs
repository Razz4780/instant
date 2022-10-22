pub struct LiteralOutOfBoundsError<'a> {
    pub literal: &'a str,
    pub position: usize,
}

/// Binary operation on i32 values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
}

/// i32 expression.
#[derive(Debug)]
pub enum Exp<'a> {
    /// i32 literal.
    Lit(i32),
    /// Variable access.
    Var {
        /// Name of the variable.
        name: &'a str,
        /// Byte offset into the parsed input.
        position: usize,
    },
    /// Binary expression.
    Bi {
        lhs: Box<Exp<'a>>,
        op: Op,
        rhs: Box<Exp<'a>>,
    },
}

/// Statement.
#[derive(Debug)]
pub enum Stmt<'a> {
    /// Assignment to a variable.
    Ass { var: &'a str, exp: Exp<'a> },
    /// Expression.
    Exp(Exp<'a>),
}
