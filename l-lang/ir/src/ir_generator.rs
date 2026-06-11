use std::collections::HashSet;

use parser::expression::{Expression, UnaryOperator};
use semantic::{
    program::SemanticProgram,
    scope::ScopeId,
    statement::{SemanticParam, SemanticStatement},
    symbol::{Symbol, SymbolId, SymbolKind},
};

use crate::{
    instruction::{BranchType, IrFunction, IrInstruction, IrReg, IrType, IrValue},
    program::IrProgram,
};

pub struct IrGenerator {
    program: SemanticProgram,
    reg_counter: usize,
    free_regs: Vec<usize>,
    current_scope_id: usize,
    spill_counter: usize,
}

impl IrGenerator {
    pub fn new(program: SemanticProgram) -> Self {
        Self {
            program,
            reg_counter: 0,
            free_regs: Vec::new(),
            current_scope_id: 0,
            spill_counter: 0,
        }
    }

    pub fn generate(&mut self) -> IrProgram {
        let mut functions = Vec::new();
        let called = self.collect_called_functions(&self.program.body);

        for stmt in &self.program.body.clone() {
            if let SemanticStatement::SemanticFunction {
                symbol,
                return_type,
                body,
                params,
                name,
                ..
            } = stmt
            {
                if name != "main" && !called.contains(name) {
                    continue; // skip unused functions
                }

                self.current_scope_id = self
                    .program
                    .scope_table
                    .iter()
                    .position(|s| match &s.kind {
                        semantic::scope::ScopeType::FunctionBody { parent, .. } => parent == symbol,
                        _ => false,
                    })
                    .unwrap_or(0);

                self.reg_counter = 0;
                self.spill_counter = 0;

                let name = self.lookup(symbol).name.clone();
                let ir_return_type = IrType::from(return_type);

                let mut allocs = Vec::new();
                let mut rest = Vec::new();

                self.emit_store_params(params, &mut allocs, &mut rest);

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

    fn fresh_reg(&mut self) -> IrReg {
        if let Some(r) = self.free_regs.pop() {
            IrReg::Temp(r)
        } else {
            let r = self.reg_counter;
            self.reg_counter += 1;
            IrReg::Temp(r)
        }
    }

    fn free_reg(&mut self, reg: IrReg) {
        if let IrReg::Temp(n) = reg {
            self.free_regs.push(n);
        }
    }

    fn collect_called_functions(&self, body: &[SemanticStatement]) -> HashSet<String> {
        let mut called = HashSet::new();

        for stmt in body {
            match stmt {
                SemanticStatement::SemanticFunctionCall { name, .. } => {
                    called.insert(name.clone());
                }
                SemanticStatement::SemanticFunction { body, .. } => {
                    called.extend(self.collect_called_functions(body));
                }
                SemanticStatement::SemanticVarDecl { initializer, .. } => {
                    if let Some(expr) = initializer {
                        self.collect_called_in_expr(expr, &mut called);
                    }
                }
                SemanticStatement::SemanticAssign { value, .. } => {
                    self.collect_called_in_expr(value, &mut called);
                }
                SemanticStatement::SemanticReturn { value, .. } => {
                    if let Some(expr) = value {
                        self.collect_called_in_expr(expr, &mut called);
                    }
                }
                SemanticStatement::SemanticIf {
                    condition,
                    body,
                    else_body,
                    ..
                } => {
                    self.collect_called_in_expr(condition, &mut called);

                    called.extend(self.collect_called_functions(body));

                    if let Some(else_b) = else_body {
                        called.extend(self.collect_called_functions(else_b));
                    }
                }
                _ => {}
            }
        }

        called
    }

    fn collect_called_in_expr(&self, expr: &Expression, called: &mut HashSet<String>) {
        match expr {
            Expression::FunctionCall { name, args, .. } => {
                called.insert(name.clone());
                for arg in args {
                    self.collect_called_in_expr(arg, called);
                }
            }
            Expression::BinaryOperation { left, right, .. } => {
                self.collect_called_in_expr(left, called);
                self.collect_called_in_expr(right, called);
            }
            _ => {}
        }
    }

    fn contains_call(expr: &Expression) -> bool {
        match expr {
            Expression::FunctionCall { .. } => true,
            Expression::BinaryOperation { left, right, .. } => {
                Self::contains_call(left) || Self::contains_call(right)
            }
            Expression::UnaryOperation { operand, .. } => Self::contains_call(operand),
            _ => false,
        }
    }

    fn emit_store_params(
        &mut self,
        params: &Vec<SemanticParam>,
        allocs: &mut Vec<IrInstruction>,
        rest: &mut Vec<IrInstruction>,
    ) {
        let mut i = 0;
        for param in params {
            let ir_type = IrType::from(&param.param_type);
            let symbol = param.symbol;

            allocs.push(IrInstruction::Alloc {
                ir_type: ir_type.clone(),
                symbol,
            });

            rest.push(IrInstruction::StoreStack {
                ir_type,
                symbol,
                src: IrReg::Arg(i),
            });

            i += 1;
        }
    }

    fn emit_statement(
        &mut self,
        stmt: &SemanticStatement,
        allocs: &mut Vec<IrInstruction>,
        rest: &mut Vec<IrInstruction>,
    ) {
        self.free_regs.clear();
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

            SemanticStatement::SemanticFunctionCall { name, args, .. } => {
                // evaluate each arg and move into $a0-$a3
                for (i, arg) in args.iter().enumerate() {
                    let (reg, instrs) = self.emit_expression(arg);
                    rest.extend(instrs);
                    rest.push(IrInstruction::StoreImm {
                        dest: IrReg::Arg(i),
                        value: IrValue::Reg(reg),
                    });
                }

                rest.push(IrInstruction::Call {
                    function_name: name.clone(),
                });
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

            SemanticStatement::SemanticIf {
                label,
                condition,
                body,
                else_label,
                else_body,
            } => {
                let (reg, instrs) = self.emit_expression(condition);
                rest.extend(instrs);

                if else_label.is_some() {
                    rest.push(IrInstruction::Branch {
                        reg,
                        label: else_label.clone().unwrap(), // unwrap is ok cuz else_label is some
                        branch_type: BranchType::EqZero,
                    });
                } else {
                    rest.push(IrInstruction::Branch {
                        reg,
                        label: label.clone(), // unwrap is ok cuz else_label is some
                        branch_type: BranchType::EqZero,
                    });
                }

                for stmt in body {
                    self.emit_statement(stmt, allocs, rest);
                }

                let body_returns =
                    matches!(body.last(), Some(SemanticStatement::SemanticReturn { .. }));
                if !body_returns {
                    rest.push(IrInstruction::Jump {
                        label: label.clone(),
                    });
                }

                if else_label.is_some() {
                    rest.push(IrInstruction::Label {
                        label_name: else_label.clone().unwrap(),
                    });

                    for stmt in else_body.clone().unwrap() {
                        self.emit_statement(&stmt, allocs, rest);
                    }

                    let body_returns = matches!(
                        else_body.as_ref().and_then(|v| v.last()),
                        Some(SemanticStatement::SemanticReturn { .. })
                    );
                    if !body_returns {
                        rest.push(IrInstruction::Jump {
                            label: label.clone(),
                        });
                    }
                }

                rest.push(IrInstruction::Label {
                    label_name: label.clone(),
                });
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

            SemanticStatement::SemanticAssembly { body } => {
                for s in body {
                    rest.push(IrInstruction::Assembly { line: s.clone() });
                }
            }

            SemanticStatement::SemanticFunction { .. } => {
                panic!("Nested functions not supported");
            }
        }
    }

    // returns the register the result lands in, plus any instructions needed
    fn emit_expression(&mut self, expr: &Expression) -> (IrReg, Vec<IrInstruction>) {
        match expr {
            Expression::Integer(n) => {
                let dest = self.fresh_reg();
                (
                    dest,
                    vec![IrInstruction::StoreImm {
                        dest,
                        value: IrValue::Immediate(*n),
                    }],
                )
            }

            Expression::Bool(b) => {
                let dest = self.fresh_reg();
                (
                    dest,
                    vec![IrInstruction::StoreImm {
                        dest,
                        value: IrValue::Immediate(if *b { 1 } else { 0 }),
                    }],
                )
            }

            Expression::Identifier(name) => {
                let (sym_id, ir_type) = {
                    let mut scope_id = self.current_scope_id;
                    let sym = loop {
                        let scope = &self.program.scope_table[scope_id];
                        if let Some(s) = scope.symbols.iter().find(|s| s.name == *name) {
                            break s;
                        }
                        match scope.parent {
                            Some(ScopeId(id)) => scope_id = id as usize,
                            None => panic!("symbol '{}' not found in scope chain", name),
                        }
                    };
                    let var_type = match &sym.kind {
                        SymbolKind::Variable { var_type, .. } => var_type,
                        _ => panic!("expected variable, got {:?}", sym.kind),
                    };
                    (sym.id.clone(), IrType::from(var_type))
                };

                let dest = self.fresh_reg();
                (
                    dest,
                    vec![IrInstruction::LoadStack {
                        ir_type,
                        dest,
                        symbol: sym_id,
                    }],
                )
            }

            Expression::UnaryOperation { op, operand } => {
                let (operand_reg, operand_instrs) = self.emit_expression(operand);
                let dest = self.fresh_reg();
                let mut instrs = operand_instrs;

                match op {
                    UnaryOperator::Not => {
                        instrs.push(IrInstruction::UnaryOp {
                            op: UnaryOperator::Not,
                            dest,
                            operand: operand_reg,
                        });
                    }
                }

                (dest, instrs)
            }

            Expression::BinaryOperation { op, left, right } => {
                let (left_reg, left_instrs) = self.emit_expression(left);

                // if right side contains a call, left_reg will be clobbered by jal
                // so spill it to the stack before evaluating right
                let (spill_instrs, reload_instrs, left_reg_final) = if Self::contains_call(right) {
                    let slot = self.spill_counter;
                    self.spill_counter += 1;
                    let reloaded = self.fresh_reg();
                    (
                        vec![IrInstruction::SpillTemp {
                            slot,
                            src: left_reg,
                        }],
                        vec![IrInstruction::LoadTemp {
                            slot,
                            dest: reloaded,
                        }],
                        reloaded,
                    )
                } else {
                    (vec![], vec![], left_reg)
                };

                let (right_reg, right_instrs) = self.emit_expression(right);
                let dest = self.fresh_reg();

                let mut instrs = left_instrs;
                instrs.extend(spill_instrs);
                instrs.extend(right_instrs);
                instrs.extend(reload_instrs);
                instrs.push(IrInstruction::BinaryOp {
                    op: op.clone(),
                    dest,
                    left: left_reg_final,
                    right: right_reg,
                });

                self.free_reg(left_reg);
                self.free_reg(right_reg);

                (dest, instrs)
            }

            Expression::FunctionCall { name, args, .. } => {
                let mut instrs = Vec::new();

                for (i, arg) in args.iter().enumerate() {
                    let (reg, arg_instrs) = self.emit_expression(arg);
                    instrs.extend(arg_instrs);
                    instrs.push(IrInstruction::StoreImm {
                        dest: IrReg::Arg(i),
                        value: IrValue::Reg(reg),
                    });
                }

                instrs.push(IrInstruction::Call {
                    function_name: name.clone(),
                });

                let dest = self.fresh_reg();
                instrs.push(IrInstruction::StoreImm {
                    dest,
                    value: IrValue::Reg(IrReg::RetVal),
                });

                (dest, instrs)
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
