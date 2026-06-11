use codegen::mips::Mips;
use ir::ir_generator::IrGenerator;
use ir::program::IrProgram;
use lexer::lexer::Lexer;
use parser::parser::Parser;
use parser::program::Program;
use semantic::analyzer::Analyzer;
use semantic::program::SemanticProgram;
use util::error::Result;

pub struct Compiler {
    semantic_program: SemanticProgram,
    ir: IrProgram,
}

impl Compiler {
    pub fn new(
        source: &String,
        print_tokens: bool,
        print_ast: bool,
        print_sat: bool,
        print_ir: bool,
    ) -> Result<Self> {
        let stdlib = include_str!("../../std/io.l");
        let mut stdlib_lexer = Lexer::new(&stdlib.to_string());
        let stdlib_tokens = stdlib_lexer.tokenize()?;
        let mut user_lexer = Lexer::new(&source);
        let user_tokens = user_lexer.tokenize()?;

        if print_tokens {
            println!("STD: {:#?}", stdlib_tokens);
            println!("USER: {:#?}", user_tokens);
        }

        let mut stdlib_parser = Parser::new(stdlib_tokens.clone());
        let stdlib_ast = stdlib_parser.parse()?;
        let mut user_parser = Parser::new(user_tokens.clone());
        let user_ast = user_parser.parse()?;

        if print_ast {
            println!("STD: {:#?}", stdlib_ast);
            println!("USER: {:#?}", user_ast);
        }

        // prepend stdlib functions to user program
        let combined = Program {
            body: stdlib_ast.clone().body.into_iter().chain(user_ast.clone().body).collect(),
        };

        let mut s = Analyzer::new(combined.clone());
        let semantic_program = s.analyze()?;

        if print_sat {
            println!("{:#?}", semantic_program);
        }

        let mut i = IrGenerator::new(semantic_program.clone());
        let ir = i.generate();

        if print_ir {
            println!("{:#?}", ir);
        }

        Ok(Compiler {
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
}
