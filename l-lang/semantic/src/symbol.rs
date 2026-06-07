use parser::{statement::Parameter, types::Type};

use crate::scope::ScopeId;

#[derive(Debug, Clone)]
pub enum State {
    Uninitialized,
    Initialized,
    Used,
}

#[derive(Debug, Clone)]
pub enum SymbolKind {
    Function {
        return_type: Type,
        params: Vec<Parameter>,
    },

    Variable {
        var_type: Type,
        state: State,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SymbolId(pub usize);

impl SymbolId {
    pub fn value(&self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub id: SymbolId,
    pub kind: SymbolKind,
    pub declared_scope: ScopeId,
    pub line: usize,
}
