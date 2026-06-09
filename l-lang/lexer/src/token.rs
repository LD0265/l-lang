#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Void,
    I32,
    I16,
    I8,
    Bool,
    If,
    While,
    Return,

    IntegerLiteral(i32),
    BoolLiteral(bool),

    Identifier(String),

    Equal,
    Plus,
    Minus,
    Star,
    Slash,

    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Semicolon,
    Comma,

    Eof,
    Newline,
}

impl Token {
    pub fn keyword(s: &str) -> Option<Token> {
        match s {
            "void" => Some(Token::Void),
            "i32" => Some(Token::I32),
            "i16" => Some(Token::I16),
            "i8" => Some(Token::I8),
            "bool" => Some(Token::Bool),
            "if" => Some(Token::If),
            "while" => Some(Token::While),
            "return" => Some(Token::Return),
            "true" => Some(Token::BoolLiteral(true)),
            "false" => Some(Token::BoolLiteral(false)),
            _ => None,
        }
    }
}
