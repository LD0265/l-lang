pub enum TypeSize {
    Void,
    Byte,
    HalfWord,
    Word,
    DoubleWord,
}

#[derive(Debug, Clone)]
pub enum Type {
    Void,
    Bool,
    Int8,
    Int16,
    Int32,
}

impl Type {
    pub const fn size(&self) -> TypeSize {
        match self {
            Type::Void => TypeSize::Void,
            Type::Bool => TypeSize::Byte,
            Type::Int8 => TypeSize::Byte,
            Type::Int16 => TypeSize::HalfWord,
            Type::Int32 => TypeSize::Word,
        }
    }
}
