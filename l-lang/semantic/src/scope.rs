use crate::symbol::{Symbol, SymbolId};

#[derive(Debug, Clone)]
pub enum ScopeType {
    Global,

    FunctionBody {
        name: String, // for debugging
        parent: SymbolId
    },

    IfBody {
        parent: SymbolId,
    }
}

#[derive(Debug, Clone)]
pub struct ScopeId(pub i32);

impl ScopeId {
    pub fn value(&self) -> i32 {
        self.0
    }
}

#[derive(Debug, Clone)]
pub struct Scope {
    pub scope_id: ScopeId,
    pub kind: ScopeType,
    pub parent: Option<ScopeId>,
    pub symbols: Vec<Symbol>,
}