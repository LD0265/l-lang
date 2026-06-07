use parser::expression::Expression;
use parser::types::Type;

use crate::symbol::SymbolId;

#[derive(Debug, Clone)]
pub enum SemanticStatement {
    SemanticFunction {
        symbol: SymbolId,
        return_type: Type,
        body: Vec<SemanticStatement>,
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

    SemanticReturn {
        value: Option<Expression>,
    },
}
