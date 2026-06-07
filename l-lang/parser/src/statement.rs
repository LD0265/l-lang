use crate::{expression::Expression, types::Type};

#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub param_type: Type,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Instruction {
        opcode: String,
        operands: Vec<String>,
    },

    Label {
        name: String,
        body: Vec<Statement>,
    },

    FunctionDecleration {
        return_type: Type,
        name: String,
        params: Vec<Parameter>,
        body: Vec<Statement>,
        line: usize,
    },

    VariableDeclaration {
        var_name: String,
        var_type: Type,
        operation: Option<Expression>,
        line: usize,
    },

    Assign {
        var_name: String,
        value: Expression,
        line: usize,
    },

    Return {
        return_value: Option<Expression>,
    },

    NewLine,
}
