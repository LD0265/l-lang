#[derive(Debug, Clone)]
pub struct SemanticWarning {
    pub warning_type: WarningType,
    pub name: String,
    pub message: String,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub enum WarningType {
    UninitializedVariable,
    LiteralOverflow,
    TypeMismatch,
    UndeclaredFunction,
}

impl WarningType {
    pub fn get_message(&self, msg: String) -> String {
        match self {
            WarningType::UninitializedVariable => {
                format!("variable `{}` is declared but never initialized", msg)
            }
            WarningType::LiteralOverflow => {
                format!("literal value overflows the declared type of `{}`", msg)
            }
            WarningType::TypeMismatch => format!("{}", msg),
            WarningType::UndeclaredFunction => format!("{}", msg),
        }
    }
}
