use crate::{scope::Scope, statement::SemanticStatement, warning::SemanticWarning};

#[derive(Debug, Clone)]
pub struct SemanticProgram {
    pub scope_table: Vec<Scope>,
    pub body: Vec<SemanticStatement>,
    pub diagnostics: Vec<SemanticWarning>,
}