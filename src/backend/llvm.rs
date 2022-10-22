use super::Backend;
use crate::{
    ast::{Exp, Op, Stmt},
    UndeclaredVariableError,
};
use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
};

#[derive(Clone, Copy)]
enum Location {
    Register(usize),
    Immediate(i32),
}

impl Display for Location {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Register(num) => write!(f, "%t{}", num),
            Self::Immediate(val) => write!(f, "{}", val),
        }
    }
}

/// Backend for generating LLVM IR from an Instant program.
#[derive(Default)]
pub struct LLVMBackend;

impl Backend for LLVMBackend {
    type Representation = LLVMIr;

    fn process<'a>(&self, program: &[Stmt<'a>]) -> Result<LLVMIr, UndeclaredVariableError<'a>> {
        let mut builder = LLVMIrBuilder::default();

        for stmt in program {
            builder.add_stmt(stmt)?;
        }

        Ok(builder.build())
    }
}

#[derive(Default)]
struct LLVMIrBuilder<'a> {
    variables: HashMap<&'a str, Location>,
    next_register: usize,
    instructions: Vec<Instruction>,
}

impl<'a> LLVMIrBuilder<'a> {
    fn add_exp(&mut self, exp: &Exp<'a>) -> Result<Location, UndeclaredVariableError<'a>> {
        match exp {
            Exp::Lit(val) => Ok(Location::Immediate(*val)),
            Exp::Var { name, position } => {
                self.variables
                    .get(name)
                    .copied()
                    .ok_or(UndeclaredVariableError {
                        name: *name,
                        byte_offset: *position,
                    })
            }
            Exp::Bi { lhs, op, rhs } => {
                let lhs = self.add_exp(lhs)?;
                let rhs = self.add_exp(rhs)?;

                let dst_register = self.next_register;
                self.next_register += 1;
                let dst = Location::Register(dst_register);

                self.instructions.push(Instruction::Bin {
                    lhs,
                    op: *op,
                    rhs,
                    dst,
                });

                Ok(dst)
            }
        }
    }

    fn add_stmt(&mut self, stmt: &Stmt<'a>) -> Result<(), UndeclaredVariableError<'a>> {
        match stmt {
            Stmt::Exp(exp) => {
                let loc = self.add_exp(exp)?;
                self.instructions.push(Instruction::Print(loc));
            }
            Stmt::Ass { var, exp } => {
                let loc = self.add_exp(exp)?;
                self.variables.insert(*var, loc);
            }
        }

        Ok(())
    }

    fn build(self) -> LLVMIr {
        LLVMIr {
            instructions: self.instructions,
        }
    }
}

enum Instruction {
    Print(Location),
    Bin {
        lhs: Location,
        op: Op,
        rhs: Location,
        dst: Location,
    },
}

impl Display for Instruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Print(loc) => write!(f, "call void @printInt(i32 {})", loc),
            Self::Bin { lhs, op, rhs, dst } => write!(
                f,
                "{} = {} i32 {}, {}",
                dst,
                match op {
                    Op::Add => "add",
                    Op::Sub => "sub",
                    Op::Mul => "mul",
                    Op::Div => "sdiv",
                },
                lhs,
                rhs,
            ),
        }
    }
}

/// LLVM Intermediate Representation of an Instant program.
pub struct LLVMIr {
    instructions: Vec<Instruction>,
}

impl Display for LLVMIr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "@d = internal constant [4 x i8] c\"%d\\0A\\00\"\n")?;
        writeln!(f, "declare i32 @printf(i8*, ...)\n")?;

        writeln!(f, "define void @printInt(i32 %x) {{")?;
        writeln!(
            f,
            "\t%t0 = getelementptr [4 x i8], [4 x i8]* @d, i32 0, i32 0"
        )?;
        writeln!(f, "\tcall i32 (i8*, ...) @printf(i8* %t0, i32 %x)")?;
        writeln!(f, "\tret void")?;
        writeln!(f, "}}\n")?;

        writeln!(f, "define i32 @main(i32 %argc, i8** %argv) {{")?;
        for instruction in &self.instructions {
            writeln!(f, "\t{}", instruction)?;
        }
        writeln!(f, "\tret i32 0")?;
        writeln!(f, "}}")
    }
}
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn undeclared_variable() {
        let mut builder = LLVMIrBuilder::default();
        let error = builder
            .add_stmt(&Stmt::Exp(Exp::Var {
                name: "name",
                position: 0,
            }))
            .expect_err("undeclared variable access should result in an error");
        assert_eq!(error.name, "name");
        assert_eq!(error.byte_offset, 0);
    }
}
