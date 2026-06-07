use codegen::mips::Mips;
use ir::generator::IrGenerator;
use ir::program::IrProgram;
use lexer::{lexer::Lexer, token::Token};
use parser::parser::Parser;
use parser::program::Program;
use semantic::analyzer::Analyzer;
use semantic::program::SemanticProgram;
use util::error::{CompileError, Result};

pub struct Compiler {
    source: String,
}

impl Compiler {
    pub fn new(source: &String) -> Self {
        Compiler {
            source: source.to_string(),
        }
    }

    pub fn compile(&mut self, output_path: String) -> Result<()> {
        let mut l = Lexer::new(&self.source);
        let tokens = l.tokenize()?;
        let mut p = Parser::new(tokens);
        let ast = p.parse()?;
        let mut s = Analyzer::new(ast);
        let semantic_program = s.analyze();

        for warning in &semantic_program.diagnostics {
            eprintln!(
                "\x1b[1;31mWarning\x1b[0m [line {}]: {}",
                warning.line, warning.message
            );
        }

        let mut i = IrGenerator::new(semantic_program);
        let ir = i.generate();
        let mut codegen = Mips::new(ir);
        let mips_code = codegen.generate();

        let res = std::fs::write(output_path, mips_code.to_string());

        match res {
            Ok(_) => Ok(()),
            Err(e) => {
                return Err(CompileError::CompilerError {
                    message: format!("Failed to write to output file: {}", e),
                    line: 0,
                });
            }
        }
    }

    pub fn get_ast(&mut self) -> Result<Program> {
        let mut l = Lexer::new(&self.source);
        let tokens = l.tokenize()?;
        let mut p = Parser::new(tokens);
        let program = p.parse()?;

        Ok(program)
    }

    pub fn get_sat(&mut self) -> Result<SemanticProgram> {
        let mut l = Lexer::new(&self.source);
        let tokens = l.tokenize()?;
        let mut p = Parser::new(tokens);
        let ast = p.parse()?;
        let mut s = Analyzer::new(ast);
        let semantic_program = s.analyze();

        Ok(semantic_program)
    }

    pub fn get_ir(&mut self) -> Result<IrProgram> {
        let mut l = Lexer::new(&self.source);
        let tokens = l.tokenize()?;
        let mut p = Parser::new(tokens);
        let ast = p.parse()?;
        let mut s = Analyzer::new(ast);
        let semantic_program = s.analyze();
        let mut i = IrGenerator::new(semantic_program);
        let ir = i.generate();

        Ok(ir)
    }

    pub fn get_tokens(&mut self) -> Result<Vec<Token>> {
        let mut l = Lexer::new(&self.source);
        let tokens = l.tokenize()?;

        Ok(tokens)
    }
}
