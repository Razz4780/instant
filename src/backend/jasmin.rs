use super::Backend;
use crate::{
    ast::{Exp, Op, Stmt},
    UndeclaredVariableError,
};
use std::{
    cmp::{self, Ordering},
    collections::{hash_map::Entry, HashMap},
    fmt::{self, Display, Formatter},
};

impl Op {
    fn commutative(self) -> bool {
        matches!(self, Self::Add | Self::Mul)
    }
}

/// Backend for generating [Jasmin](https://jasmin.sourceforge.net/) from an Instant program.
pub struct JasminBackend {
    class_name: String,
}

impl JasminBackend {
    /// Creates a new instance of this struct.
    /// The given class name will be used to create the class encapsulating the main function.
    pub fn new(class_name: String) -> Self {
        Self { class_name }
    }
}

impl Backend for JasminBackend {
    type Representation = Jasmin;

    fn process<'a>(&self, program: &[Stmt<'a>]) -> Result<Jasmin, UndeclaredVariableError<'a>> {
        let mut builder = JasminBuilder::new(self.class_name.clone());

        for stmt in program {
            builder.add_stmt(stmt)?;
        }

        Ok(builder.build())
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Instruction {
    GetPrintStream,
    Swap,
    Println,
    IStore(usize),
    Push(i32),
    ILoad(usize),
    BinOp(Op),
}

impl Display for Instruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::GetPrintStream => {
                f.write_str("getstatic java/lang/System/out Ljava/io/PrintStream;")
            }
            Self::Swap => f.write_str("swap"),
            Self::Println => f.write_str("invokevirtual java/io/PrintStream/println(I)V"),
            Self::IStore(i @ 0..=3) => write!(f, "istore_{}", i),
            Self::IStore(i) => write!(f, "istore {}", i),
            Self::Push(i @ 0..=5) => write!(f, "iconst_{}", i),
            Self::Push(i @ 6..=127) => write!(f, "bipush {}", i),
            Self::Push(i @ 128..=32767) => {
                write!(f, "sipush {} {}", i.to_le_bytes()[1], i.to_le_bytes()[0])
            }
            Self::Push(i) => write!(f, "ldc {}", i),
            Self::ILoad(i @ 0..=3) => write!(f, "iload_{}", i),
            Self::ILoad(i) => write!(f, "iload {}", i),
            Self::BinOp(Op::Add) => f.write_str("iadd"),
            Self::BinOp(Op::Div) => f.write_str("idiv"),
            Self::BinOp(Op::Mul) => f.write_str("imul"),
            Self::BinOp(Op::Sub) => f.write_str("isub"),
        }
    }
}

#[derive(Debug)]
struct ProcessedExp {
    instructions: Vec<Instruction>,
    depth: usize,
}

struct JasminBuilder<'a> {
    class_name: String,
    stack_depth: usize,
    locals: HashMap<&'a str, usize>,
    instructions: Vec<Instruction>,
}

impl<'a> JasminBuilder<'a> {
    fn new(class_name: String) -> Self {
        Self {
            class_name,
            stack_depth: 0,
            locals: Default::default(),
            instructions: Default::default(),
        }
    }

    fn process_exp(&self, exp: &Exp<'a>) -> Result<ProcessedExp, UndeclaredVariableError<'a>> {
        match exp {
            Exp::Lit(val) => Ok(ProcessedExp {
                instructions: vec![Instruction::Push(*val)],
                depth: 1,
            }),
            Exp::Var { name, position } => {
                let slot = self
                    .locals
                    .get(name)
                    .copied()
                    .ok_or(UndeclaredVariableError {
                        name: *name,
                        byte_offset: *position,
                    })?;

                Ok(ProcessedExp {
                    instructions: vec![Instruction::ILoad(slot)],
                    depth: 1,
                })
            }
            Exp::Bi { lhs, op, rhs } => {
                let mut lhs = self.process_exp(lhs)?;
                let mut rhs = self.process_exp(rhs)?;

                let (instructions, depth) = match rhs.depth.cmp(&lhs.depth) {
                    Ordering::Equal => {
                        lhs.instructions.reserve(rhs.instructions.len() + 1);
                        lhs.instructions.extend(rhs.instructions);
                        lhs.instructions.push(Instruction::BinOp(*op));

                        (lhs.instructions, rhs.depth + 1)
                    }
                    Ordering::Greater => {
                        rhs.instructions.reserve(lhs.instructions.len() + 2);
                        rhs.instructions.extend(lhs.instructions);
                        if !op.commutative() {
                            rhs.instructions.push(Instruction::Swap);
                        }
                        rhs.instructions.push(Instruction::BinOp(*op));

                        (rhs.instructions, rhs.depth)
                    }
                    Ordering::Less => {
                        lhs.instructions.reserve(rhs.instructions.len() + 1);
                        lhs.instructions.extend(rhs.instructions);
                        lhs.instructions.push(Instruction::BinOp(*op));

                        (lhs.instructions, lhs.depth)
                    }
                };

                Ok(ProcessedExp {
                    instructions,
                    depth,
                })
            }
        }
    }

    fn add_stmt(&mut self, stmt: &Stmt<'a>) -> Result<(), UndeclaredVariableError<'a>> {
        let depth = match stmt {
            Stmt::Exp(exp) => {
                let exp = self.process_exp(exp)?;
                if exp.depth > 1 {
                    self.instructions.reserve(exp.instructions.len() + 3);
                    self.instructions.extend(exp.instructions);
                    self.instructions.push(Instruction::GetPrintStream);
                    self.instructions.push(Instruction::Swap);
                    self.instructions.push(Instruction::Println);

                    exp.depth
                } else {
                    self.instructions.reserve(exp.instructions.len() + 2);
                    self.instructions.push(Instruction::GetPrintStream);
                    self.instructions.extend(exp.instructions);
                    self.instructions.push(Instruction::Println);

                    2
                }
            }
            Stmt::Ass { var, exp } => {
                let exp = self.process_exp(exp)?;

                let next_local = self.locals.len() + 1;
                let slot = match self.locals.entry(*var) {
                    Entry::Occupied(e) => *e.get(),
                    Entry::Vacant(e) => {
                        e.insert(next_local);
                        next_local
                    }
                };

                self.instructions.reserve(exp.instructions.len() + 1);
                self.instructions.extend(exp.instructions);
                self.instructions.push(Instruction::IStore(slot));

                exp.depth
            }
        };

        self.stack_depth = cmp::max(self.stack_depth, depth);

        Ok(())
    }

    fn build(self) -> Jasmin {
        Jasmin {
            class_name: self.class_name,
            stack_limit: self.stack_depth,
            locals: self.locals.len() + 1,
            instructions: self.instructions,
        }
    }
}

/// [Jasmin](https://jasmin.sourceforge.net/) representation of an Instant program.
pub struct Jasmin {
    class_name: String,
    stack_limit: usize,
    locals: usize,
    instructions: Vec<Instruction>,
}

impl Display for Jasmin {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, ".source {}.j\n", self.class_name)?;
        writeln!(f, ".class public {}\n", self.class_name)?;
        writeln!(f, ".super java/lang/Object\n")?;

        writeln!(f, ".method public <init>()V")?;
        writeln!(f, ".limit stack 1")?;
        writeln!(f, ".limit locals 1")?;
        writeln!(f, "aload_0")?;
        writeln!(f, "invokenonvirtual java/lang/Object/<init>()V")?;
        writeln!(f, "return")?;
        writeln!(f, ".end method\n")?;

        writeln!(f, ".method public static main([Ljava/lang/String;)V")?;
        writeln!(f, ".limit stack {}", self.stack_limit)?;
        writeln!(f, ".limit locals {}", self.locals)?;
        for instruction in &self.instructions {
            writeln!(f, "{}", instruction)?;
        }
        writeln!(f, "return")?;
        writeln!(f, ".end method")?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn instructions_representation() {
        assert_eq!(Instruction::IStore(0).to_string(), "istore_0");
        assert_eq!(Instruction::IStore(3).to_string(), "istore_3");
        assert_eq!(Instruction::IStore(4).to_string(), "istore 4");

        assert_eq!(Instruction::Push(0).to_string(), "iconst_0");
        assert_eq!(Instruction::Push(5).to_string(), "iconst_5");
        assert_eq!(Instruction::Push(6).to_string(), "bipush 6");
        assert_eq!(Instruction::Push(127).to_string(), "bipush 127");
        assert_eq!(Instruction::Push(128).to_string(), "sipush 0 128");
        assert_eq!(Instruction::Push(32767).to_string(), "sipush 127 255");
        assert_eq!(Instruction::Push(32768).to_string(), "ldc 32768");

        assert_eq!(Instruction::ILoad(0).to_string(), "iload_0");
        assert_eq!(Instruction::ILoad(3).to_string(), "iload_3");
        assert_eq!(Instruction::ILoad(4).to_string(), "iload 4");
    }

    #[test]
    fn expression_optimization() {
        let builder = JasminBuilder::new("dummy".into());

        let processed = builder.process_exp(&Exp::Lit(0)).unwrap();
        assert_eq!(processed.instructions, [Instruction::Push(0)]);
        assert_eq!(processed.depth, 1);

        let error = builder
            .process_exp(&Exp::Var {
                name: "name",
                position: 0,
            })
            .expect_err("undeclared variable access should result in an error");
        assert_eq!(error.name, "name");
        assert_eq!(error.byte_offset, 0);

        let processed = builder
            .process_exp(&Exp::Bi {
                lhs: Exp::Lit(0).into(),
                op: Op::Add,
                rhs: Exp::Lit(1).into(),
            })
            .unwrap();
        assert_eq!(
            processed.instructions,
            [
                Instruction::Push(0),
                Instruction::Push(1),
                Instruction::BinOp(Op::Add)
            ]
        );
        assert_eq!(processed.depth, 2);

        let processed = builder
            .process_exp(&Exp::Bi {
                lhs: Exp::Lit(0).into(),
                op: Op::Mul,
                rhs: Exp::Bi {
                    lhs: Exp::Lit(2).into(),
                    op: Op::Sub,
                    rhs: Exp::Lit(5).into(),
                }
                .into(),
            })
            .unwrap();
        assert_eq!(
            processed.instructions,
            [
                Instruction::Push(2),
                Instruction::Push(5),
                Instruction::BinOp(Op::Sub),
                Instruction::Push(0),
                Instruction::BinOp(Op::Mul)
            ]
        );
        assert_eq!(processed.depth, 2);

        let processed = builder
            .process_exp(&Exp::Bi {
                lhs: Exp::Lit(0).into(),
                op: Op::Div,
                rhs: Exp::Bi {
                    lhs: Exp::Lit(2).into(),
                    op: Op::Sub,
                    rhs: Exp::Lit(5).into(),
                }
                .into(),
            })
            .unwrap();
        assert_eq!(
            processed.instructions,
            [
                Instruction::Push(2),
                Instruction::Push(5),
                Instruction::BinOp(Op::Sub),
                Instruction::Push(0),
                Instruction::Swap,
                Instruction::BinOp(Op::Div)
            ]
        );
        assert_eq!(processed.depth, 2);
    }
}
