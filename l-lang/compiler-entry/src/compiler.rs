use codegen::mips::Mips;
use ir::ir_generator::IrGenerator;
use ir::program::IrProgram;
use lexer::{lexer::Lexer, token::Token};
use parser::program::Program;
use parser::parser::Parser;
use semantic::analyzer::Analyzer;
use semantic::program::SemanticProgram;
use util::error::Result;

pub struct Compiler {
    tokens: Vec<Token>,
    ast: Program,
    semantic_program: SemanticProgram,
    ir: IrProgram,
}

impl Compiler {
    pub fn new(source: &String) -> Result<Self> {
        let mut l = Lexer::new(&source);
        let tokens = l.tokenize()?;
        let mut p = Parser::new(tokens.clone());
        let ast = p.parse()?;
        let mut s = Analyzer::new(ast.clone());
        let semantic_program = s.analyze()?;
        let mut i = IrGenerator::new(semantic_program.clone());
        let ir = i.generate();

        Ok(Compiler {
            tokens: tokens.clone(),
            ast: ast.clone(),
            semantic_program: semantic_program.clone(),
            ir: ir.clone(),
        })
    }

    pub fn compile(&self) -> String {
        for warning in &self.semantic_program.diagnostics {
            eprintln!(
                "\x1b[1;33mWarning\x1b[0m [line {}]: {}",
                warning.line, warning.message
            );
        }

        let mut codegen = Mips::new(self.ir.clone());
        let mips_code = codegen.generate();

        mips_code
    }

    pub fn get_ast(&self) -> Result<Program> {
        Ok(self.ast.clone())
    }

    pub fn get_sat(&self) -> Result<SemanticProgram> {
        Ok(self.semantic_program.clone())
    }

    pub fn get_ir(&self) -> Result<IrProgram> {
        Ok(self.ir.clone())
    }

    pub fn get_tokens(&self) -> Result<Vec<Token>> {
        Ok(self.tokens.clone())
    }
}
