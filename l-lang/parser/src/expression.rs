#[derive(Debug, Clone)]
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
}
