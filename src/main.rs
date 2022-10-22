#[macro_use]
extern crate lalrpop_util;

lalrpop_mod!(#[allow(clippy::all)] pub grammar);

use grammar::ProgParser;
use instant::{
    backend::{jasmin::JasminBackend, llvm::LLVMBackend, Backend},
    lines::Lines,
};
use lalrpop_util::ParseError;
use std::{
    env,
    io::{self, Read},
    process::ExitCode,
};

fn run(class_name: Option<String>) -> Result<(), String> {
    let mut input = String::new();
    io::stdin()
        .read_to_string(&mut input)
        .map_err(|e| format!("failed to read STDIN: {}", e))?;

    let lines = Lines::new(&input);

    let stmts = ProgParser::new().parse(&input).map_err(|e| match e {
        ParseError::ExtraToken { token } | ParseError::UnrecognizedToken { token, .. } => {
            format!("unexpected token at {}", lines.position(token.0))
        }
        ParseError::InvalidToken { location } => {
            format!("invalid token at {}", lines.position(location))
        }
        ParseError::UnrecognizedEOF { location, .. } => {
            format!("unexpected EOF at {}", lines.position(location))
        }
        ParseError::User { error } => format!(
            "literal {} out of bounds at {}",
            error.literal,
            lines.position(error.position)
        ),
    })?;

    match class_name {
        Some(class_name) => {
            let backend = JasminBackend::new(class_name);
            let representation = backend.process(&stmts).map_err(|e| {
                format!(
                    "undeclared variable {} at {}",
                    e.name,
                    lines.position(e.byte_offset)
                )
            })?;
            println!("{}", representation);
        }
        None => {
            let backend = LLVMBackend::default();
            let representation = backend.process(&stmts).map_err(|e| {
                format!(
                    "undeclared variable {} at {}",
                    e.name,
                    lines.position(e.byte_offset)
                )
            })?;
            println!("{}", representation);
        }
    }

    Ok(())
}

fn main() -> ExitCode {
    let args = env::args().collect::<Vec<_>>();

    let class_name = match &args[..] {
        [_, mode, class] if mode == "--jasmin" => Some(class.to_string()),
        [_, mode] if mode == "--llvm" => None,
        other => {
            let prog = other
                .first()
                .map(String::as_ref)
                .unwrap_or("<program name>");
            eprintln!(
                "USAGE:\n\t{} --llvm\n\t{} --jasmin <class name>\n\t{} --help",
                prog, prog, prog
            );

            return if other.len() == 2 && other[1] == "--help" {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            };
        }
    };

    match run(class_name) {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("ERROR: {}", e);
            ExitCode::FAILURE
        }
    }
}
