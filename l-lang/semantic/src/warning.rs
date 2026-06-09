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
    UndeclaredFunction
}

impl WarningType {
    pub fn get_message(&self, name: String) -> String {
        match self {
            WarningType::UninitializedVariable => {
                format!("variable `{}` is declared but never initialized", name)
            }
            WarningType::LiteralOverflow => {
                format!("literal value overflows the declared type of `{}`", name)
            }
            WarningType::TypeMismatch => format!("{}", name),
            WarningType::UndeclaredFunction => format!("{}", name),
        }
    }
}
