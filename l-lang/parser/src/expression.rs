use crate::types::Type;

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOperator {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    NotEq,
    Lt,
    Gt,
    LtEq,
    GtEq,
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOperator {
    Not,
    Deref,
    AddressOf,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Integer(i32),
    Bool(bool),
    Identifier(String),

    BinaryOperation {
        op: BinaryOperator,
        left: Box<Expression>,
        right: Box<Expression>,
    },

    UnaryOperation {
        op: UnaryOperator,
        operand: Box<Expression>,
    },

    Array {
        values: Vec<Box<Expression>>,
        size: usize,
    },

    Index {
        base: Box<Expression>,
        index: Box<Expression>,
    },

    FunctionCall {
        return_type: Option<Type>,
        name: String,
        args: Vec<Expression>,
    },
}
