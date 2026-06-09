use std::fmt;

pub type Result<T> = std::result::Result<T, CompileError>;

#[derive(Debug, Clone)]
pub enum CompileError {
    LexError { message: String, line: usize },
    ParseError { message: String, line: usize },
    SemanticError { message: String, line: usize },
    CodeGenError { message: String, line: usize },
    CompilerError { message: String, line: usize },
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CompileError::LexError { message, line } => {
                write!(f, "Lexical error at line {}: {}", line, message)
            }

            CompileError::ParseError { message, line } => {
                write!(f, "Parse error at line {}: {}", line, message)
            }

            CompileError::SemanticError { message, line } => {
                write!(f, "Semantic error at line {}: {}", line, message)
            }

            CompileError::CodeGenError { message, line } => {
                write!(f, "Code generation error at line {}: {}", line, message)
            }

            CompileError::CompilerError { message, line } => {
                panic!("Critical Compiler Error at line {}: {}", line, message);
            }
        }
    }
}
