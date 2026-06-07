use parser::expression::Expression;
use semantic::{
    program::SemanticProgram,
    statement::SemanticStatement,
    symbol::{Symbol, SymbolId, SymbolKind},
};

use crate::{
    instruction::{IrFunction, IrInstruction, IrReg, IrType, IrValue},
    program::IrProgram,
};

pub struct IrGenerator {
    program: SemanticProgram,
}

impl IrGenerator {
    pub fn new(program: SemanticProgram) -> Self {
        Self { program }
    }

    pub fn generate(&mut self) -> IrProgram {
        let mut functions = Vec::new();

        for stmt in &self.program.body.clone() {
            if let SemanticStatement::SemanticFunction {
                symbol,
                return_type,
                body,
                ..
            } = stmt
            {
                let name = self.lookup(symbol).name.clone();
                let ir_return_type = IrType::from(return_type);

                let mut allocs = Vec::new();
                let mut rest = Vec::new();
                for s in body {
                    self.emit_statement(s, &mut allocs, &mut rest);
                }

                let needs_ra = rest.iter().any(|i| matches!(i, IrInstruction::Call { .. }));

                let mut instructions = allocs;
                if needs_ra {
                    instructions.push(IrInstruction::SaveRa);
                }

                instructions.extend(rest);

                functions.push(IrFunction {
                    name,
                    return_type: ir_return_type,
                    instructions,
                });
            }
        }

        IrProgram { functions }
    }

    fn emit_statement(
        &mut self,
        stmt: &SemanticStatement,
        allocs: &mut Vec<IrInstruction>,
        rest: &mut Vec<IrInstruction>,
    ) {
        match stmt {
            SemanticStatement::SemanticVarDecl {
                symbol,
                initializer,
            } => {
                let sym = self.lookup(symbol).clone();
                if let SymbolKind::Variable { ref var_type, .. } = sym.kind {
                    allocs.push(IrInstruction::Alloc {
                        ir_type: IrType::from(var_type),
                        symbol: symbol.clone(),
                    });

                    if let Some(expr) = initializer {
                        let (reg, store_instrs) = self.emit_expression(expr);
                        rest.extend(store_instrs);
                        rest.push(IrInstruction::StoreStack {
                            ir_type: IrType::from(var_type),
                            symbol: symbol.clone(),
                            src: reg,
                        });
                    }
                }
            }

            SemanticStatement::SemanticAssign { symbol, value } => {
                let sym = self.lookup(symbol).clone();
                if let SymbolKind::Variable { ref var_type, .. } = sym.kind {
                    let (reg, store_instrs) = self.emit_expression(value);
                    rest.extend(store_instrs);
                    rest.push(IrInstruction::StoreStack {
                        ir_type: IrType::from(var_type),
                        symbol: symbol.clone(),
                        src: reg,
                    });
                }
            }

            SemanticStatement::SemanticReturn { value } => {
                if let Some(expr) = value {
                    let (reg, instrs) = self.emit_expression(expr);
                    rest.extend(instrs);
                    rest.push(IrInstruction::StoreImm {
                        dest: IrReg::RetVal,
                        value: IrValue::Reg(reg),
                    });
                }
                rest.push(IrInstruction::Ret);
            }

            SemanticStatement::SemanticFunction { .. } => {
                // nested functions not supported yet
            }
        }
    }

    // returns the register the result lands in, plus any instructions needed
    fn emit_expression(&self, expr: &Expression) -> (IrReg, Vec<IrInstruction>) {
        match expr {
            Expression::Integer(n) => (
                IrReg::Temp(0),
                vec![IrInstruction::StoreImm {
                    dest: IrReg::Temp(0),
                    value: IrValue::Immediate(*n),
                }],
            ),

            Expression::Bool(b) => (
                IrReg::Temp(0),
                vec![IrInstruction::StoreImm {
                    dest: IrReg::Temp(0),
                    value: IrValue::Immediate(if *b { 1 } else { 0 }),
                }],
            ),

            Expression::Identifier(name) => {
                // look up which symbol this name refers to
                let sym = self
                    .program
                    .scope_table
                    .iter()
                    .flat_map(|s| &s.symbols)
                    .find(|s| s.name == *name)
                    .unwrap();
                let var_type = match &sym.kind {
                    SymbolKind::Variable { var_type, .. } => var_type,
                    _ => panic!("expected variable"),
                };
                (
                    IrReg::Temp(0),
                    vec![IrInstruction::LoadStack {
                        ir_type: IrType::from(var_type),
                        dest: IrReg::Temp(0),
                        symbol: sym.id.clone(),
                    }],
                )
            }

            Expression::BinaryOperation { op: _, left, right } => {
                let (_, left_instrs) = self.emit_expression(left);
                let (_, right_instrs) = self.emit_expression(right);
                let mut instrs = left_instrs;
                instrs.push(IrInstruction::StoreImm {
                    dest: IrReg::Temp(1),
                    value: IrValue::Reg(IrReg::Temp(0)),
                });
                instrs.extend(right_instrs);
                (IrReg::Temp(0), instrs)
            }
        }
    }

    fn lookup(&self, id: &SymbolId) -> &Symbol {
        self.program
            .scope_table
            .iter()
            .flat_map(|s| &s.symbols)
            .find(|s| s.id.0 == id.0)
            .unwrap()
    }
}
