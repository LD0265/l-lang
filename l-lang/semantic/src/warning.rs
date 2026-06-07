#[derive(Debug, Clone)]
pub enum WarningType {
    UninitializedVariable,
    LiteralOverflow,
    MissingReturn,
}

#[derive(Debug, Clone)]
pub struct SemanticWarning {
    pub warning_type: WarningType,
    pub name: String,
    pub message: String,
    pub line: usize,
}

impl WarningType {
    pub fn get_message(&self, var_name: String) -> String {
        match self {
            WarningType::UninitializedVariable => {
                format!("variable `{}` is declared but never initialized", var_name)
            }
            WarningType::LiteralOverflow => format!(
                "literal value overflows the declared type of `{}`",
                var_name
            ),
            WarningType::MissingReturn => {
                format!("function `{}` has no return statement", var_name)
            }
        }
    }
}
