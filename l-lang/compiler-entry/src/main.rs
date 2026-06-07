use std::{path::Path, process::exit};

use clap::{CommandFactory, Parser, error::ErrorKind};
use util::error::CompileError;

use crate::{cli::Args, compiler::Compiler};

mod cli;
mod compiler;

fn main() -> Result<(), CompileError> {
    let args = Args::parse();

    let path = Path::new(args.input_file.as_str());

    if !path.exists() {
        Args::command()
            .error(
                ErrorKind::ArgumentConflict,
                format!("{:?} is not a valid path", path),
            )
            .exit();
    }

    if !path.is_file() {
        Args::command()
            .error(
                ErrorKind::ArgumentConflict,
                format!("{:?} is not a file", path),
            )
            .exit();
    }

    if let Some(extension) = path.extension() {
        if extension != "l" {
            let file_name = path.file_name().unwrap();

            Args::command()
                .error(
                    ErrorKind::ArgumentConflict,
                    format!("{} must be a .l file", file_name.display()),
                )
                .exit();
        }
    }

    let source = std::fs::read_to_string(path).unwrap_or_else(|_| String::from(""));

    let mut compiler = Compiler::new(&source);

    let output = args.output.as_str();
    compiler.compile(output.to_string())?;

    if args.tokens {
        match compiler.get_tokens() {
            Ok(tokens) => {
                println!("{:#?}", tokens);
            }

            Err(e) => {
                eprintln!("{}", e);
                exit(1);
            }
        }
    }

    if args.ast {
        match compiler.get_ast() {
            Ok(ast) => {
                println!("{:#?}", ast);
            }

            Err(e) => {
                eprintln!("{}", e);
            }
        }
    }

    if args.sat {
        match compiler.get_sat() {
            Ok(ast) => {
                println!("{:#?}", ast);
            }

            Err(e) => {
                eprintln!("{}", e);
            }
        }
    }

    if args.ir {
        match compiler.get_ir() {
            Ok(ast) => {
                println!("{:#?}", ast);
            }

            Err(e) => {
                eprintln!("{}", e);
            }
        }
    }

    Ok(())
}
