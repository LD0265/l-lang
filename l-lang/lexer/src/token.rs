#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Void,
    I32,
    I16,
    I8,
    Bool,
    If,
    Else,
    While,
    Return,
    Struct,
    Size,
    ASMKeyword,

    IntegerLiteral(i32),
    StringLiteral(String),
    BoolLiteral(bool),
    AsmBlock(Vec<String>),

    Identifier(String),

    Equal,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    PlusEqual,
    MinusEqual,
    Not,
    And,
    Or,
    EqualEqual,
    NotEqual,
    LessThan,
    GreaterThan,
    LessEqual,
    GreaterEqual,
    AndAnd,
    OrOr,


    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Semicolon,
    Comma,
    Period,
    Quote,
    Pound,

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
            "else" => Some(Token::Else),
            "while" => Some(Token::While),
            "return" => Some(Token::Return),
            "struct" => Some(Token::Struct),
            "size" => Some(Token::Size),
            "true" => Some(Token::BoolLiteral(true)),
            "false" => Some(Token::BoolLiteral(false)),
            "__asm__" => Some(Token::ASMKeyword),
            _ => None,
        }
    }
}
