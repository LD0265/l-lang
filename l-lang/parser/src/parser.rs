use lexer::token::Token;
use util::error::{CompileError, Result};

use crate::{
    expression::{BinaryOperator, Expression},
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

                if self.peek() == &Token::LeftParen {
                    self.advance();
                    let mut args = Vec::new();
                    if self.peek() != &Token::RightParen {
                        loop {
                            args.push(self.parse_expresion()?);
                            if !matches!(self.peek(), Token::Comma) {
                                break;
                            }
                            self.advance();
                        }
                    }
                    self.expect(Token::RightParen)?;
                    self.expect(Token::Semicolon)?;
                    Ok(Statement::FunctionCall {
                        name,
                        args,
                        line: self.line,
                    })
                } else {
                    self.expect(Token::Equal)?;
                    let value = self.parse_expresion()?;
                    self.expect(Token::Semicolon)?;
                    Ok(Statement::Assign {
                        var_name: name,
                        value,
                        line: self.line,
                    })
                }
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

        if params.len() > 4 {
            return Err(CompileError::ParseError {
                message: String::from("Function has more than 4 params"),
                line: self.line,
            });
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
        self.parse_additive()
    }

    fn parse_additive(&mut self) -> Result<Expression> {
        let mut left = self.parse_multiplicative()?;

        while matches!(self.peek(), Token::Plus | Token::Minus) {
            let op = match self.peek() {
                Token::Plus => BinaryOperator::Add,
                Token::Minus => BinaryOperator::Sub,
                _ => unreachable!(),
            };
            self.advance();
            let right = self.parse_multiplicative()?;
            left = Expression::BinaryOperation {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_multiplicative(&mut self) -> Result<Expression> {
        let mut left = self.parse_primary()?;

        while matches!(self.peek(), Token::Star | Token::Slash) {
            let op = match self.peek() {
                Token::Star => BinaryOperator::Mul,
                Token::Slash => BinaryOperator::Div,
                _ => unreachable!(),
            };
            self.advance();
            let right = self.parse_primary()?;
            left = Expression::BinaryOperation {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_primary(&mut self) -> Result<Expression> {
        match self.peek() {
            Token::IntegerLiteral(n) => {
                let n = *n;
                self.advance();
                Ok(Expression::Integer(n))
            }

            Token::BoolLiteral(b) => {
                let b = *b;
                self.advance();
                Ok(Expression::Bool(b))
            }

            Token::Identifier(name) => {
                let name = name.clone();
                self.advance();

                if self.peek() == &Token::LeftParen {
                    self.advance(); // consume (
                    let mut args = Vec::new();

                    if self.peek() != &Token::RightParen {
                        loop {
                            args.push(self.parse_expresion()?);
                            if !matches!(self.peek(), Token::Comma) {
                                break;
                            }
                            self.advance(); // consume ,
                        }
                    }

                    self.expect(Token::RightParen)?;
                    Ok(Expression::FunctionCall {
                        name,
                        args,
                        return_type: None,
                    })
                } else {
                    Ok(Expression::Identifier(name))
                }
            }

            Token::LeftParen => {
                self.advance();
                let expr = self.parse_expresion()?;
                self.expect(Token::RightParen)?;
                Ok(expr)
            }

            _ => Err(CompileError::ParseError {
                message: format!("{:?} not implemented in parse_primary", self.peek()),
                line: self.line,
            }),
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
