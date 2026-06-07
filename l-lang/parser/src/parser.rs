use lexer::token::Token;
use util::error::{CompileError, Result};

use crate::{
    expression::Expression,
    program::Program,
    statement::{Parameter, Statement},
    types::Type,
};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    line: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            current: 0,
            line: 1,
        }
    }

    pub fn parse(&mut self) -> Result<Program> {
        let mut statements = Vec::new();

        // Maybe check if main exists in scope during semantic pass
        // then insert thish

        // text_section.scope.body.push(Statement::Label {
        //     name: "_start".to_string(),
        //     body: {
        //         vec![
        //             Statement::Instruction {
        //                 opcode: "jal".to_string(),
        //                 operands: vec!["main".to_string()],
        //             },
        //             Statement::Instruction {
        //                 opcode: "li".to_string(),
        //                 operands: vec!["$v0, 10".to_string()],
        //             },
        //             Statement::Instruction {
        //                 opcode: "syscall\n".to_string(),
        //                 operands: vec![],
        //             },
        //         ]
        //     },
        // });

        while !self.is_at_end() {
            statements.push(self.parse_statement()?);
        }

        Ok(Program { body: statements })
    }

    fn parse_statement(&mut self) -> Result<Statement> {
        match self.peek() {
            Token::Void | Token::Bool | Token::I32 | Token::I16 | Token::I8 => {
                let is_function = self.peek_ahead(2) == Some(&Token::LeftParen);

                if is_function {
                    self.parse_function()
                } else {
                    self.parse_variable_decleration()
                }
            }

            Token::Identifier(_) => {
                let name = self.parse_identifier()?;
                self.expect(Token::Equal)?;
                let value = self.parse_expresion()?;
                self.expect(Token::Semicolon)?;
                Ok(Statement::Assign {
                    var_name: name,
                    value,
                    line: self.line,
                })
            }

            Token::Return => {
                self.advance();

                // if the next non-newline token is a semicolon, it's a bare return
                let return_value = if self.peek() == &Token::Semicolon {
                    None
                } else {
                    Some(self.parse_expresion()?)
                };

                self.expect(Token::Semicolon)?;

                Ok(Statement::Return { return_value })
            }

            Token::Newline => {
                self.line += 1;
                self.advance();
                Ok(Statement::NewLine)
            }

            _ => Err(CompileError::CompilerError {
                message: format!("{:?} not implemented in parse_statement", self.peek()),
                line: self.line,
            }),
        }
    }

    fn parse_type(&mut self) -> Result<Type> {
        let t = match self.peek() {
            Token::Void => Type::Void,
            Token::I8 => Type::Int8,
            Token::I16 => Type::Int16,
            Token::I32 => Type::Int32,
            Token::Bool => Type::Bool,

            _ => {
                return Err(CompileError::ParseError {
                    message: format!("Expected type, found {:?}", self.peek()),
                    line: self.line,
                });
            }
        };

        self.advance();
        Ok(t)
    }

    fn parse_identifier(&mut self) -> Result<String> {
        match self.peek() {
            Token::Identifier(name) => {
                let name = name.clone();
                self.advance();
                Ok(name)
            }

            _ => Err(CompileError::ParseError {
                message: format!("Expected identifier, found {:?}", self.peek()),
                line: self.line,
            }),
        }
    }

    fn parse_parameters(&mut self) -> Result<Vec<Parameter>> {
        let mut params = Vec::new();

        if matches!(self.peek(), Token::RightParen) {
            return Ok(params);
        }

        loop {
            let typ = self.parse_type()?;
            let name = self.parse_identifier()?;
            params.push(Parameter {
                name,
                param_type: typ,
            });

            if !matches!(self.peek(), Token::Comma) {
                break;
            }
            self.advance();
        }

        Ok(params)
    }

    fn parse_block(&mut self) -> Result<Vec<Statement>> {
        let mut statements = Vec::new();

        while !matches!(self.peek(), Token::RightBrace | Token::Eof) {
            statements.push(self.parse_statement()?);
        }

        Ok(statements)
    }

    fn parse_function(&mut self) -> Result<Statement> {
        let return_type = self.parse_type()?;
        let name = self.parse_identifier()?;

        self.expect(Token::LeftParen)?;

        let params = self.parse_parameters()?;

        self.expect(Token::RightParen)?;
        self.expect(Token::LeftBrace)?;

        let body = self.parse_block()?;

        self.expect(Token::RightBrace)?;

        Ok(Statement::FunctionDecleration {
            return_type,
            name,
            params,
            body,
            line: self.line,
        })
    }

    fn parse_expresion(&mut self) -> Result<Expression> {
        // Its a single token expression
        if self.peek_ahead(1) == Some(&Token::Semicolon) {
            match self.peek() {
                Token::IntegerLiteral(n) => {
                    let n = *n;
                    self.advance();
                    return Ok(Expression::Integer(n));
                }

                Token::BoolLiteral(b) => {
                    let b = *b;
                    self.advance();
                    return Ok(Expression::Bool(b));
                }

                Token::Identifier(name) => {
                    let name = name.clone();
                    self.advance();
                    return Ok(Expression::Identifier(name));
                }

                _ => Err(CompileError::CompilerError {
                    message: format!("{:?} not implemented in parse_expresion", self.peek()),
                    line: self.line,
                }),
            }
        } else {
            Err(CompileError::CompilerError {
                message: format!("Multi variable expressions not implemented in parse_expresion"),
                line: self.line,
            })
        }
    }

    fn parse_variable_decleration(&mut self) -> Result<Statement> {
        let var_type = self.parse_type()?;
        let var_name = self.parse_identifier()?;

        let mut operation: Option<Expression> = None;

        if self.peek() == &Token::Equal {
            self.advance();
            operation = Some(self.parse_expresion()?);
        }

        self.expect(Token::Semicolon)?;

        Ok(Statement::VariableDeclaration {
            var_name,
            var_type,
            operation,
            line: self.line,
        })
    }

    fn advance(&mut self) {
        if self.current < self.tokens.len() - 1 {
            self.current += 1;
        }
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn peek_ahead(&self, n: i32) -> Option<&Token> {
        let change = self.current as i32 + n;
        self.tokens.get(change as usize)
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek(), Token::Eof)
    }

    fn expect(&mut self, expected: Token) -> Result<()> {
        if self.peek() == &expected {
            self.advance();
            Ok(())
        } else if self.peek() == &Token::Newline {
            self.line += 1;
            self.advance();
            self.expect(expected)?;
            Ok(())
        } else {
            Err(CompileError::ParseError {
                message: format!("Expected {:?}, found {:?}", expected, self.peek()),
                line: self.line,
            })
        }
    }
}
