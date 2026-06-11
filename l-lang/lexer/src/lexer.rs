use crate::token::Token;
use util::error::{CompileError, Result};

pub struct Lexer {
    source: Vec<char>,
    line: usize,
    current: usize,
}

impl Lexer {
    pub fn new(source: &String) -> Self {
        Lexer {
            source: source.chars().collect(),
            line: 1,
            current: 0,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();

        while !self.is_at_end() {
            self.skip_comments();
            self.skip_whitespace(&mut tokens);

            if self.is_at_end() {
                break;
            }

            tokens.push(self.next_token()?);
        }

        tokens.push(Token::Eof);
        Ok(tokens)
    }

    fn next_token(&mut self) -> Result<Token> {
        let ch = self.peek();

        match ch {
            '(' => {
                self.advance();
                Ok(Token::LeftParen)
            }

            ')' => {
                self.advance();
                Ok(Token::RightParen)
            }

            '{' => {
                self.advance();
                Ok(Token::LeftBrace)
            }

            '}' => {
                self.advance();
                Ok(Token::RightBrace)
            }

            ';' => {
                self.advance();
                Ok(Token::Semicolon)
            }

            ',' => {
                self.advance();
                Ok(Token::Comma)
            }

            '+' => {
                self.advance();
                Ok(Token::Plus)
            }

            '-' => {
                self.advance();
                Ok(Token::Minus)
            }

            '*' => {
                self.advance();
                Ok(Token::Star)
            }

            '/' => {
                self.advance();
                Ok(Token::Slash)
            }

            '=' => self.scan_equal(),

            '0'..='9' => self.scan_number(false),
            'a'..='z' | 'A'..='Z' | '_' => self.scan_identifier(),

            _ => Err(CompileError::LexError {
                message: format!("Unexpected character '{}'", ch),
                line: self.line,
            }),
        }
    }

    fn scan_identifier(&mut self) -> Result<Token> {
        let start = self.current;
        self.advance();

        while !self.is_at_end() {
            let ch = self.peek();
            if ch.is_alphanumeric() || ch == '_' {
                self.advance();
            } else {
                break;
            }
        }

        let text: String = self.source[start..self.current].iter().collect();

        if text == "__asm__" {
            return self.scan_asm_block();
        }

        let token = Token::keyword(&text).unwrap_or(Token::Identifier(text));
        Ok(token)
    }

    fn scan_asm_block(&mut self) -> Result<Token> {
        while !self.is_at_end()
            && (self.peek() == ' '
                || self.peek() == '\t'
                || self.peek() == '\n'
                || self.peek() == '\r')
        {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.peek() != '{' {
            return Err(CompileError::LexError {
                message: String::from("expected '{' after __asm__"),
                line: self.line,
            });
        }
        self.advance();

        let mut lines: Vec<String> = Vec::new();
        let mut current_line = String::new();

        while !self.is_at_end() {
            let ch = self.peek();

            if ch == '}' {
                self.advance();
                if !current_line.trim().is_empty() {
                    lines.push(current_line.trim().to_string());
                }
                return Ok(Token::AsmBlock(lines));
            }

            if ch == '\n' {
                self.line += 1;
                if !current_line.trim().is_empty() {
                    lines.push(current_line.trim().to_string());
                }
                current_line = String::new();
            } else {
                current_line.push(ch);
            }

            self.advance();
        }

        Err(CompileError::LexError {
            message: String::from("unterminated __asm__ block"),
            line: self.line,
        })
    }

    fn scan_number(&mut self, is_negative: bool) -> Result<Token> {
        let start = self.current;

        while !self.is_at_end() && self.peek().is_ascii_digit() {
            self.advance();
        }

        let is_hex = self.source[start..self.current].iter().collect::<String>() == "0"
            && !self.is_at_end()
            && (self.peek() == 'x' || self.peek() == 'X');

        if is_hex {
            self.advance();

            while !self.is_at_end() && self.peek().is_ascii_hexdigit() {
                self.advance();
            }
        }

        let text: String = self.source[start..self.current].iter().collect();

        let mut value: i32 = if is_hex {
            i32::from_str_radix(&text[2..], 16).map_err(|_| CompileError::LexError {
                message: format!("Invalid hex number: {}", text),
                line: self.line,
            })?
        } else {
            text.parse::<i32>().map_err(|_| CompileError::LexError {
                message: format!("Invalid number: {}", text),
                line: self.line,
            })?
        };

        if is_negative {
            value *= -1;
        }

        Ok(Token::IntegerLiteral(value))
    }

    fn scan_equal(&mut self) -> Result<Token> {
        self.advance();

        let ch = self.peek();

        match ch {
            // '=' => {
            //     self.advance();
            //     Ok(Token::EqualEqual)
            // }
            _ => Ok(Token::Equal),
        }
    }

    fn skip_whitespace(&mut self, tokens: &mut Vec<Token>) {
        while !self.is_at_end() {
            match self.peek() {
                ' ' | '\t' | '\r' => {
                    self.advance();
                }

                '\n' => {
                    self.line += 1;
                    tokens.push(Token::Newline);
                    self.advance();
                }

                _ => break,
            }
        }
    }

    fn skip_comments(&mut self) {
        let mut new_source: Vec<char> = Vec::with_capacity(self.source.len());
        let mut i = 0;
        while i < self.source.len() {
            if i + 1 < self.source.len() && self.source[i] == '/' && self.source[i + 1] == '/' {
                let mut j = i + 2;
                while j < self.source.len() && self.source[j] != '\n' {
                    j += 1;
                }
                i = j;
            } else {
                new_source.push(self.source[i]);
                i += 1;
            }
        }
        self.source = new_source;
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.source[self.current]
        }
    }

    fn advance(&mut self) -> char {
        let ch = self.peek();
        self.current += 1;
        ch
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }
}
