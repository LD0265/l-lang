use std::path::Path;

use crate::{cli::Args, compiler::Compiler};
use clap::{CommandFactory, Parser, error::ErrorKind};

mod cli;
mod compiler;

fn main() {
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

    let compiler = match Compiler::new(&source, args.tokens, args.ast, args.sat, args.ir) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("\x1b[1;31m{}", e);
            std::process::exit(1);
        }
    };

    let output_path = args.output.as_str();
    let assembly = compiler.compile();

    let write_res = std::fs::write(output_path, assembly);

    match write_res {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Failed to write to output file: {}", e);
        }
    };
}
