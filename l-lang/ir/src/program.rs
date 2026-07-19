use crate::instruction::IrFunction;

#[derive(Debug, Clone)]
pub struct IrProgram {
    pub functions: Vec<IrFunction>,
    pub strings: Vec<(String, String)>,
}
