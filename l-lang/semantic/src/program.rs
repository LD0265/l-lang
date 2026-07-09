use std::collections::HashMap;

use crate::{scope::Scope, statement::SemanticStatement, structs::StructDef, warning::SemanticWarning};

#[derive(Debug, Clone)]
pub struct SemanticProgram {
    pub scope_table: Vec<Scope>,
    pub body: Vec<SemanticStatement>,
    pub diagnostics: Vec<SemanticWarning>,
    pub struct_table: HashMap<String, StructDef>,
}