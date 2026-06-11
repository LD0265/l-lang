use parser::expression::Expression;
use parser::types::Type;

use crate::symbol::SymbolId;

#[derive(Debug, Clone)]
pub struct SemanticParam {
    pub name: String,
    pub param_type: Type,
    pub symbol: SymbolId,
}

#[derive(Debug, Clone)]
pub enum SemanticStatement {
    SemanticFunction {
        name: String, // for
        symbol: SymbolId,
        return_type: Type,
        params: Vec<SemanticParam>,
        body: Vec<SemanticStatement>,
        line: usize,
    },

    SemanticFunctionCall {
        name: String,
        args: Vec<Expression>,
        line: usize,
    },

    SemanticVarDecl {
        symbol: SymbolId,
        initializer: Option<Expression>,
    },

    SemanticAssign {
        symbol: SymbolId,
        value: Expression,
    },

    SemanticIf {
        label: String,
        condition: Expression,
        body: Vec<SemanticStatement>,
        else_label: Option<String>,
        else_body: Option<Vec<SemanticStatement>>,
    },

    SemanticAssembly {
        body: Vec<String>,
    },

    SemanticReturn {
        value: Option<Expression>,
    },
}
