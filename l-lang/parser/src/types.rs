pub enum TypeSize {
    Void,
    Byte,
    HalfWord,
    Word,
    DoubleWord,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Void,
    Bool,
    Int8,
    Int16,
    Int32,
    Pointer(Box<Type>),
    Struct(String),
}

impl Type {
    pub fn size(&self) -> TypeSize {
        match self {
            Type::Void => TypeSize::Void,
            Type::Bool => TypeSize::Byte,
            Type::Int8 => TypeSize::Byte,
            Type::Int16 => TypeSize::HalfWord,
            Type::Int32 => TypeSize::Word,
            Type::Pointer(_) => TypeSize::Word, // 4 bytes on mips, regardless of pointee
            Type::Struct(_) => TypeSize::Word, // structs are always accessed by address (pointer-sized)
        }
    }

    pub fn pointer_to(self) -> Type {
        Type::Pointer(Box::new(self))
    }

    pub fn deref(&self) -> Option<&Type> {
        match self {
            Type::Pointer(inner) => Some(inner),
            _ => None,
        }
    }
}
