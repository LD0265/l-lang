use crate::statement::Statement;


#[derive(Debug, Clone)]
pub struct Program {
    pub body: Vec<Statement>,
}
