use crate::types::Type;

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOperator {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug, Clone)]
pub enum Expression {
    Integer(i32),
    Bool(bool),
    Identifier(String),
    BinaryOperation {
        op: BinaryOperator,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    FunctionCall {
        return_type: Option<Type>,
        name: String,
        args: Vec<Expression>,
    },
}
