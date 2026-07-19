use lexer::token::Token;
use util::error::{CompileError, Result};

use crate::{
    expression::{BinaryOperator, Expression, UnaryOperator},
    program::Program,
    statement::{FunctionFlag, Parameter, Statement},
    types::Type,
};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    line: usize,
    label_count: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>, label_count: usize) -> Self {
        Parser {
            tokens,
            current: 0,
            line: 1,
            label_count,
        }
    }

    pub fn parse(&mut self) -> Result<Program> {
        let mut statements = Vec::new();

        while !self.is_at_end() {
            statements.push(self.parse_statement()?);
        }

        Ok(Program { body: statements })
    }

    pub fn get_label_count(&self) -> usize {
        self.label_count
    }

    fn parse_statement(&mut self) -> Result<Statement> {
        match self.peek() {
            Token::Void | Token::Bool | Token::I32 | Token::I16 | Token::I8 => {
                let mut offset = 1;
                while self.peek_ahead(offset) == Some(&Token::Percent) {
                    offset += 1;
                }

                let is_function = self.peek_ahead(offset + 1) == Some(&Token::LeftParen);

                if is_function {
                    self.parse_function()
                } else {
                    self.parse_variable_decleration()
                }
            }

            Token::If => self.parse_if(),
            Token::While => self.parse_while(),
            Token::For => self.parse_for(),

            Token::Identifier(_) => {
                let mut offset = 1;
                while self.peek_ahead(offset) == Some(&Token::Percent) {
                    offset += 1;
                }

                if self.peek_ahead(offset + 1) == Some(&Token::LeftParen) {
                    return self.parse_function();
                }

                if matches!(self.peek_ahead(offset), Some(Token::Identifier(_))) {
                    return self.parse_variable_decleration();
                }

                let name = self.parse_identifier()?;

                // postfix chain: field access or index before assignment
                if matches!(self.peek(), Token::Period | Token::LeftBracket) {
                    let mut expr = Expression::Identifier(name);
                    loop {
                        if matches!(self.peek(), Token::Period) {
                            self.advance();
                            let field = self.parse_identifier()?;
                            expr = Expression::FieldAccess {
                                base: Box::new(expr),
                                field,
                            };
                        } else if matches!(self.peek(), Token::LeftBracket) {
                            self.advance();
                            let index = self.parse_expresion()?;
                            self.expect(Token::RightBracket)?;
                            expr = Expression::Index {
                                base: Box::new(expr),
                                index: Box::new(index),
                            };
                        } else {
                            break;
                        }
                    }
                    self.expect(Token::Equal)?;
                    let value = self.parse_expresion()?;
                    self.expect(Token::Semicolon)?;
                    return Ok(Statement::LValueAssign {
                        target: expr,
                        value,
                        line: self.line,
                    });
                }

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
                } else if matches!(self.peek(), Token::PlusEqual | Token::MinusEqual) {
                    let op = match self.peek() {
                        Token::PlusEqual => BinaryOperator::Add,
                        Token::MinusEqual => BinaryOperator::Sub,
                        _ => unreachable!(),
                    };
                    self.advance();
                    let rhs = self.parse_expresion()?;
                    self.expect(Token::Semicolon)?;
                    Ok(Statement::Assign {
                        var_name: name.clone(),
                        value: Expression::BinaryOperation {
                            op,
                            left: Box::new(Expression::Identifier(name)),
                            right: Box::new(rhs),
                        },
                        line: self.line,
                    })
                } else if matches!(self.peek(), Token::PlusPlus | Token::MinusMinus) {
                    let op = match self.peek() {
                        Token::PlusPlus => BinaryOperator::Add,
                        Token::MinusMinus => BinaryOperator::Sub,
                        _ => unreachable!(),
                    };
                    self.advance();
                    self.expect(Token::Semicolon)?;
                    Ok(Statement::Assign {
                        var_name: name.clone(),
                        value: Expression::BinaryOperation {
                            op,
                            left: Box::new(Expression::Identifier(name)),
                            right: Box::new(Expression::Integer(1)),
                        },
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

            Token::Percent => {
                let target = self.parse_unary()?; // parses %ptr, %%pptr, etc. as an expression
                self.expect(Token::Equal)?;
                let value = self.parse_expresion()?;
                self.expect(Token::Semicolon)?;
                Ok(Statement::DerefAssign {
                    target,
                    value,
                    line: self.line,
                })
            }

            Token::Struct => {
                self.advance();
                let name = self.parse_identifier()?;
                self.expect(Token::LeftBrace)?;
                let mut fields = Vec::new();
                while !matches!(self.peek(), Token::RightBrace | Token::Eof) {
                    if matches!(self.peek(), Token::Newline) {
                        self.advance();
                        continue;
                    }
                    let decl = self.parse_variable_decleration()?;
                    if let Statement::VariableDeclaration {
                        var_name,
                        var_type,
                        operation,
                        ..
                    } = decl
                    {
                        let arr_size = match &operation {
                            Some(Expression::Array { size, values }) if values.is_empty() => {
                                Some(*size)
                            }
                            _ => None,
                        };
                        fields.push((var_name, var_type, arr_size));
                    }
                }
                self.expect(Token::RightBrace)?;
                self.expect(Token::Semicolon)?;
                Ok(Statement::StructDef { name, fields })
            }

            Token::AsmBlock(code) => {
                let body = code.clone();
                self.advance();
                Ok(Statement::Assembly { body })
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

            _ => Err(CompileError::ParseError {
                message: format!("{:?} is not a statement", self.peek()),
                line: self.line,
            }),
        }
    }

    fn parse_type(&mut self) -> Result<Type> {
        let mut t = match self.peek() {
            Token::Void => Type::Void,
            Token::I8 => Type::Int8,
            Token::I16 => Type::Int16,
            Token::I32 => Type::Int32,
            Token::Bool => Type::Bool,
            Token::Identifier(name) => Type::Struct(name.clone()),
            _ => {
                return Err(CompileError::ParseError {
                    message: format!("Expected type, found {:?}", self.peek()),
                    line: self.line,
                });
            }
        };

        self.advance();

        while matches!(self.peek(), Token::Percent) {
            self.advance();
            t = Type::Pointer(Box::new(t));
        }

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
        let line = self.line; // capture here, before parsing anything

        let return_type = self.parse_type()?;
        let name = self.parse_identifier()?;
        let mut function_flags: Vec<FunctionFlag> = Vec::new();

        self.expect(Token::LeftParen)?;
        let params = self.parse_parameters()?;
        self.expect(Token::RightParen)?;

        // function flags
        if self.peek() == &Token::Pound {
            self.advance();
            self.expect(Token::LeftBracket)?;

            while self.peek() != &Token::RightBracket {
                let str = self.parse_identifier()?;
                let flag = match str.as_str() {
                    "no_stack" => FunctionFlag::NoStack,
                    _ => {
                        return Err(CompileError::ParseError {
                            message: format!("{} is not a valid function flag", str),
                            line,
                        });
                    }
                };

                if !function_flags.contains(&flag) {
                    function_flags.push(flag);
                }

                if self.peek() == &Token::Comma {
                    self.advance();
                }
            }

            self.expect(Token::RightBracket)?;
        }

        self.expect(Token::LeftBrace)?;

        let body = self.parse_block()?;

        self.expect(Token::RightBrace)?;

        Ok(Statement::FunctionDecleration {
            return_type,
            name,
            params,
            body,
            flags: function_flags,
            line,
        })
    }

    fn parse_if(&mut self) -> Result<Statement> {
        self.advance();

        self.expect(Token::LeftParen)?;
        let condition = self.parse_expresion()?;
        self.expect(Token::RightParen)?;

        self.expect(Token::LeftBrace)?;
        let body = self.parse_block()?;
        self.expect(Token::RightBrace)?;

        let mut else_label = None;
        let mut else_body = None;

        if self.peek() == &Token::Else {
            self.advance();
            self.expect(Token::LeftBrace)?;
            else_body = Some(self.parse_block()?);
            self.expect(Token::RightBrace)?;
            else_label = Some(self.new_label());
        }

        Ok(Statement::If {
            label: self.new_label(),
            condition,
            body,
            else_label,
            else_body,
        })
    }

    fn parse_while(&mut self) -> Result<Statement> {
        self.advance();

        self.expect(Token::LeftParen)?;
        let condition = self.parse_expresion()?;
        self.expect(Token::RightParen)?;

        self.expect(Token::LeftBrace)?;
        let body = self.parse_block()?;
        self.expect(Token::RightBrace)?;

        Ok(Statement::While {
            body_label: self.new_label(),
            body,
            cond_label: self.new_label(),
            condition,
        })
    }

    fn parse_for(&mut self) -> Result<Statement> {
        self.advance(); // 'for'
        self.expect(Token::LeftParen)?;

        let init = self.parse_statement()?;

        let condition = self.parse_expresion()?;
        self.expect(Token::Semicolon)?;

        let increment = self.parse_for_increment_stmt()?;

        self.expect(Token::RightParen)?;
        self.expect(Token::LeftBrace)?;
        let mut body = self.parse_block()?;
        self.expect(Token::RightBrace)?;
        body.push(increment);

        Ok(Statement::Block(vec![
            init,
            Statement::While {
                body_label: self.new_label(),
                cond_label: self.new_label(),
                condition,
                body,
            },
        ]))
    }

    fn parse_for_increment_stmt(&mut self) -> Result<Statement> {
        let line = self.line;
        let name = self.parse_identifier()?;
        if matches!(self.peek(), Token::PlusEqual | Token::MinusEqual) {
            let op = match self.peek() {
                Token::PlusEqual => BinaryOperator::Add,
                Token::MinusEqual => BinaryOperator::Sub,
                _ => unreachable!(),
            };
            self.advance();
            let rhs = self.parse_expresion()?;
            Ok(Statement::Assign {
                var_name: name.clone(),
                value: Expression::BinaryOperation {
                    op,
                    left: Box::new(Expression::Identifier(name)),
                    right: Box::new(rhs),
                },
                line,
            })
        } else if matches!(self.peek(), Token::PlusPlus | Token::MinusMinus) {
            let op = match self.peek() {
                Token::PlusPlus => BinaryOperator::Add,
                Token::MinusMinus => BinaryOperator::Sub,
                _ => unreachable!(),
            };
            self.advance();
            Ok(Statement::Assign {
                var_name: name.clone(),
                value: Expression::BinaryOperation {
                    op,
                    left: Box::new(Expression::Identifier(name)),
                    right: Box::new(Expression::Integer(1)),
                },
                line: self.line,
            })
        } else {
            Err(CompileError::ParseError {
                message: "expected += / ++ or -= / -- in for-loop increment".to_string(),
                line,
            })
        }
    }

    fn parse_expresion(&mut self) -> Result<Expression> {
        self.parse_logical()
    }

    fn parse_logical(&mut self) -> Result<Expression> {
        let mut left = self.parse_comparison()?;

        while matches!(self.peek(), Token::AndAnd | Token::OrOr) {
            let op = match self.peek() {
                Token::AndAnd => BinaryOperator::And,
                Token::OrOr => BinaryOperator::Or,
                _ => unreachable!(),
            };
            self.advance();
            let right = self.parse_comparison()?;
            left = Expression::BinaryOperation {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expression> {
        let mut left = self.parse_additive()?;

        while matches!(
            self.peek(),
            Token::EqualEqual
                | Token::NotEqual
                | Token::LessThan
                | Token::GreaterThan
                | Token::LessEqual
                | Token::GreaterEqual
        ) {
            let op = match self.peek() {
                Token::EqualEqual => BinaryOperator::Eq,
                Token::NotEqual => BinaryOperator::NotEq,
                Token::LessThan => BinaryOperator::Lt,
                Token::GreaterThan => BinaryOperator::Gt,
                Token::LessEqual => BinaryOperator::LtEq,
                Token::GreaterEqual => BinaryOperator::GtEq,
                _ => unreachable!(),
            };
            self.advance();
            let right = self.parse_additive()?;
            left = Expression::BinaryOperation {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
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
        let mut left = self.parse_unary()?;

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

    fn parse_unary(&mut self) -> Result<Expression> {
        if matches!(self.peek(), Token::Not) {
            self.advance();
            let operand = self.parse_unary()?;
            return Ok(Expression::UnaryOperation {
                op: UnaryOperator::Not,
                operand: Box::new(operand),
            });
        }

        if matches!(self.peek(), Token::Percent) {
            self.advance();
            let operand = self.parse_unary()?;
            return Ok(Expression::UnaryOperation {
                op: UnaryOperator::Deref,
                operand: Box::new(operand),
            });
        }

        if matches!(self.peek(), Token::And) {
            self.advance();
            let operand = self.parse_unary()?;
            return Ok(Expression::UnaryOperation {
                op: UnaryOperator::AddressOf,
                operand: Box::new(operand),
            });
        }

        if matches!(self.peek(), Token::Minus) {
            self.advance();
            let operand = self.parse_unary()?;

            return Ok(Expression::UnaryOperation {
                op: UnaryOperator::Neg,
                operand: Box::new(operand),
            });
        }

        let mut expr = self.parse_primary()?;
        while matches!(self.peek(), Token::LeftBracket | Token::Period) {
            if matches!(self.peek(), Token::LeftBracket) {
                self.advance();
                let index = self.parse_expresion()?;
                self.expect(Token::RightBracket)?;
                expr = Expression::Index {
                    base: Box::new(expr),
                    index: Box::new(index),
                };
            } else {
                self.advance();
                let field = self.parse_identifier()?;
                expr = Expression::FieldAccess {
                    base: Box::new(expr),
                    field,
                };
            }
        }

        Ok(expr)
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

            Token::StringLiteral(s) => {
                let str = s.to_string();
                self.advance();
                Ok(Expression::String(str))
            }

            Token::SizeOf => {
                self.advance();
                let t = self.parse_type()?;
                Ok(Expression::SizeOf(t))
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

            Token::LeftBrace => {
                self.advance();
                let mut exprs: Vec<Box<Expression>> = Vec::new();
                let mut i = 0;
                while self.peek() != &Token::RightBrace {
                    exprs.push(Box::new(self.parse_expresion()?));
                    if self.peek() == &Token::Comma {
                        self.advance();
                    }
                    i += 1;
                }
                self.expect(Token::RightBrace)?;
                Ok(Expression::Array {
                    values: exprs,
                    size: i,
                })
            }

            _ => Err(CompileError::ParseError {
                message: format!("{:?} not implemented in parse_primary", self.peek()),
                line: self.line,
            }),
        }
    }

    fn parse_variable_decleration(&mut self) -> Result<Statement> {
        let line = self.line;
        let var_type = self.parse_type()?;
        let var_name = self.parse_identifier()?;

        let mut operation: Option<Expression> = None;
        let arr_size;

        if self.peek() == &Token::Equal {
            self.advance();
            operation = Some(self.parse_expresion()?);
        } else if self.peek() == &Token::LeftBracket {
            self.advance();
            match self.peek() {
                Token::IntegerLiteral(n) => {
                    arr_size = *n as usize;
                    self.advance();
                }
                _ => {
                    return Err(CompileError::ParseError {
                        message: String::from("Expected integer literal in array size initializer"),
                        line: self.line,
                    });
                }
            }
            self.expect(Token::RightBracket)?;

            operation = Some(Expression::Array {
                values: Vec::new(),
                size: arr_size,
            })
        }

        self.expect(Token::Semicolon)?;

        Ok(Statement::VariableDeclaration {
            var_name,
            var_type,
            operation,
            line,
        })
    }

    fn new_label(&mut self) -> String {
        let s = format!("L{}", self.label_count);
        self.label_count += 1;
        s
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
