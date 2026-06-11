use std::fmt::Display;

use parser::{expression::BinaryOperator, types::Type};
use semantic::symbol::SymbolId;

#[derive(Debug, Clone, PartialEq)]
pub enum IrType {
    I8,
    I16,
    I32,
    Bool,
    Void,
}

impl IrType {
    pub fn from(t: &Type) -> Self {
        match t {
            Type::Int8 => IrType::I8,
            Type::Int16 => IrType::I16,
            Type::Int32 => IrType::I32,
            Type::Bool => IrType::Bool,
            Type::Void => IrType::Void,
        }
    }

    pub fn size_bytes(&self) -> usize {
        match self {
            IrType::I8 => 1,
            IrType::I16 => 2,
            IrType::I32 => 4,
            IrType::Bool => 1,
            IrType::Void => 0,
        }
    }

    pub fn instruction(&self) -> String {
        match self {
            IrType::I8 => String::from("b"),
            IrType::I16 => String::from("h"),
            IrType::I32 => String::from("w"),
            IrType::Bool => String::from("b"),
            IrType::Void => String::from(""),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum IrReg {
    Temp(usize),
    Arg(usize),
    RetVal,
    RetAddr,
}

impl IrReg {
    fn prefix(&self) -> String {
        match self {
            IrReg::Temp(_) => String::from("t"),
            IrReg::Arg(_) => String::from("a"),
            IrReg::RetVal => String::from("v"),
            IrReg::RetAddr => String::from("ra"),
        }
    }

    fn value(&self) -> usize {
        match self {
            IrReg::Temp(n) => *n,
            IrReg::Arg(n) => *n,
            IrReg::RetVal => 0,
            IrReg::RetAddr => 0,
        }
    }
}

impl Display for IrReg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "${}{}", self.prefix(), self.value())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum IrValue {
    Immediate(i32),
    Reg(IrReg),
}

impl IrValue {
    pub fn as_immediate(&self) -> Option<i32> {
        if let IrValue::Immediate(n) = self {
            Some(*n)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum IrInstruction {
    SaveRa,

    Alloc {
        ir_type: IrType,
        symbol: SymbolId,
    },

    StoreImm {
        dest: IrReg,
        value: IrValue,
    },

    StoreStack {
        ir_type: IrType,
        symbol: SymbolId,
        src: IrReg,
    },

    LoadStack {
        ir_type: IrType,
        dest: IrReg,
        symbol: SymbolId,
    },

    BinaryOp {
        op: BinaryOperator,
        dest: IrReg,
        left: IrReg,
        right: IrReg,
    },

    Call {
        function_name: String
    },

    Assembly {
        line: String,
    },

    Ret,
}

#[derive(Debug, Clone)]
pub struct IrFunction {
    pub name: String,
    pub return_type: IrType,
    pub instructions: Vec<IrInstruction>,
}
