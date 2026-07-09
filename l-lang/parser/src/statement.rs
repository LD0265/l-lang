use crate::{expression::Expression, types::Type};

#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub param_type: Type,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FunctionFlag {
    NoStack,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    FunctionDecleration {
        return_type: Type,
        name: String,
        params: Vec<Parameter>,
        body: Vec<Statement>,
        flags: Vec<FunctionFlag>,
        line: usize,
    },

    FunctionCall {
        name: String,
        args: Vec<Expression>,
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

    DerefAssign {
        target: Expression,
        value: Expression,
        line: usize,
    },

    StructDef {
        name: String,
        fields: Vec<(String, Type, Option<usize>)>,
    },

    LValueAssign {
        target: Expression,
        value: Expression,
        line: usize,
    },

    If {
        label: String,
        condition: Expression,
        body: Vec<Statement>,
        else_label: Option<String>,
        else_body: Option<Vec<Statement>>,
    },

    While {
        body_label: String,
        body: Vec<Statement>,
        cond_label: String,
        condition: Expression,
    },

    Assembly {
        body: Vec<String>,
    },

    Return {
        return_value: Option<Expression>,
    },

    NewLine,
}
