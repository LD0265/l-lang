use std::collections::HashSet;

use parser::{
    expression::{BinaryOperator, Expression, UnaryOperator},
    types::Type,
};
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
    scope_cursor: usize,
}

impl IrGenerator {
    pub fn new(program: SemanticProgram) -> Self {
        Self {
            program,
            reg_counter: 0,
            free_regs: Vec::new(),
            current_scope_id: 0,
            spill_counter: 0,
            scope_cursor: 0,
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
                self.scope_cursor = self.current_scope_id + 1;

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
                SemanticStatement::SemanticFunctionCall { name, args, .. } => {
                    for arg in args {
                        self.collect_called_in_expr(arg, &mut called);
                    }

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

                SemanticStatement::SemanticWhile {
                    condition, body, ..
                } => {
                    self.collect_called_in_expr(condition, &mut called);
                    called.extend(self.collect_called_functions(body));
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

    fn enter_block_scope(&mut self) -> usize {
        let parent_id = self.current_scope_id;
        let saved = self.current_scope_id;
        for i in self.scope_cursor..self.program.scope_table.len() {
            if let Some(ScopeId(p)) = self.program.scope_table[i].parent {
                if p as usize == parent_id {
                    self.scope_cursor = i + 1;
                    self.current_scope_id = i;
                    return saved;
                }
            }
        }
        panic!("no child scope found for scope {}", parent_id);
    }

    fn exit_block_scope(&mut self, saved: usize) {
        self.current_scope_id = saved;
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
        self.reg_counter = 0;
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

            SemanticStatement::SemanticDerefAssign { target, value, .. } => {
                // target is Expression::UnaryOperation { op: Deref, operand }
                let Expression::UnaryOperation {
                    op: UnaryOperator::Deref,
                    operand,
                } = target
                else {
                    panic!("DerefAssign target must be a Deref expression");
                };

                let (addr_reg, addr_instrs) = self.emit_expression(operand);
                let (val_reg, val_instrs) = self.emit_expression(value);
                let pointee_type = self.resolve_pointee_type(operand);

                rest.extend(addr_instrs);
                rest.extend(val_instrs);
                rest.push(IrInstruction::StoreIndirect {
                    ir_type: IrType::from(&pointee_type),
                    addr: addr_reg,
                    src: val_reg,
                });

                self.free_reg(addr_reg);
                self.free_reg(val_reg);
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
                        label: else_label.clone().unwrap(),
                        branch_type: BranchType::EqZero,
                    });
                } else {
                    rest.push(IrInstruction::Branch {
                        reg,
                        label: label.clone(),
                        branch_type: BranchType::EqZero,
                    });
                }

                let saved = self.enter_block_scope();
                for stmt in body {
                    self.emit_statement(stmt, allocs, rest);
                }
                self.exit_block_scope(saved);

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

                    let saved = self.enter_block_scope();
                    for stmt in else_body.clone().unwrap() {
                        self.emit_statement(&stmt, allocs, rest);
                    }
                    self.exit_block_scope(saved);

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

            SemanticStatement::SemanticWhile {
                body_label,
                body,
                cond_label,
                condition,
            } => {
                let (reg, instrs) = self.emit_expression(condition);

                rest.push(IrInstruction::Jump {
                    label: cond_label.clone(),
                });

                rest.push(IrInstruction::Label {
                    label_name: body_label.clone(),
                });

                let saved = self.enter_block_scope();
                for stmt in body {
                    self.emit_statement(stmt, allocs, rest);
                }
                self.exit_block_scope(saved);

                rest.push(IrInstruction::Label {
                    label_name: cond_label.clone(),
                });

                rest.extend(instrs);
                rest.push(IrInstruction::Branch {
                    reg,
                    label: body_label.clone(),
                    branch_type: BranchType::NeqZero,
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

            Expression::UnaryOperation { op, operand } => match op {
                UnaryOperator::AddressOf => match operand.as_ref() {
                    Expression::Identifier(name) => {
                        let sym_id = self.lookup_symbol_id(name);
                        let dest = self.fresh_reg();
                        (
                            dest,
                            vec![IrInstruction::LoadAddr {
                                dest,
                                symbol: sym_id,
                            }],
                        )
                    }
                    Expression::Index { base, index } => {
                        let elem_type = self.resolve_pointee_type(base);
                        let ir_elem_type = IrType::from(&elem_type);
                        let elem_size = ir_elem_type.size_bytes() as i32;

                        let (base_reg, base_instrs) = self.emit_expression(base);
                        let (index_reg, index_instrs) = self.emit_expression(index);
                        let mut instrs = base_instrs;
                        instrs.extend(index_instrs);

                        let addr_reg = self.fresh_reg();
                        let scale_reg = self.fresh_reg();
                        let size_reg = self.fresh_reg();
                        instrs.push(IrInstruction::StoreImm {
                            dest: size_reg,
                            value: IrValue::Immediate(elem_size),
                        });
                        instrs.push(IrInstruction::BinaryOp {
                            op: BinaryOperator::Mul,
                            dest: scale_reg,
                            left: index_reg,
                            right: size_reg,
                        });
                        instrs.push(IrInstruction::BinaryOp {
                            op: BinaryOperator::Add,
                            dest: addr_reg,
                            left: base_reg,
                            right: scale_reg,
                        });
                        self.free_reg(base_reg);
                        self.free_reg(index_reg);
                        self.free_reg(scale_reg);
                        self.free_reg(size_reg);

                        (addr_reg, instrs)
                    }
                    _ => panic!(
                        "AddressOf of non-lvalue reached IR — semantic analysis should have caught this"
                    ),
                },

                UnaryOperator::Deref => {
                    let (addr_reg, addr_instrs) = self.emit_expression(operand);
                    let pointee_type = self.resolve_pointee_type(operand); // need this helper, see below
                    let dest = self.fresh_reg();

                    let mut instrs = addr_instrs;
                    instrs.push(IrInstruction::LoadIndirect {
                        ir_type: IrType::from(&pointee_type),
                        dest,
                        addr: addr_reg,
                    });

                    self.free_reg(addr_reg);
                    (dest, instrs)
                }

                UnaryOperator::Not => {
                    let (operand_reg, operand_instrs) = self.emit_expression(operand);
                    let dest = self.fresh_reg();
                    let mut instrs = operand_instrs;
                    instrs.push(IrInstruction::UnaryOp {
                        op: UnaryOperator::Not,
                        dest,
                        operand: operand_reg,
                    });
                    (dest, instrs)
                }
            },

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

            Expression::Array { values, .. } => {
                if values.is_empty() {
                    let dest = self.fresh_reg();
                    return (
                        dest,
                        vec![IrInstruction::StoreImm {
                            dest,
                            value: IrValue::Immediate(0),
                        }],
                    );
                }

                let elem_type = self.resolve_expr_type(&values[0]);
                let ir_elem_type = IrType::from(&elem_type);
                let elem_size = ir_elem_type.size_bytes() as i32;

                let slot = self.spill_counter;
                self.spill_counter += 1;

                let mut instrs = Vec::new();

                instrs.push(IrInstruction::AllocArray {
                    slot,
                    elem_type: ir_elem_type.clone(),
                    count: values.len(),
                });

                let base_reg = self.fresh_reg();
                instrs.push(IrInstruction::LoadArrayBase {
                    dest: base_reg,
                    slot,
                });

                for (i, val) in values.iter().enumerate() {
                    let (val_reg, val_instrs) = self.emit_expression(val);
                    instrs.extend(val_instrs);

                    if i == 0 {
                        instrs.push(IrInstruction::StoreIndirect {
                            ir_type: ir_elem_type.clone(),
                            addr: base_reg,
                            src: val_reg,
                        });
                    } else {
                        let offset_reg = self.fresh_reg();
                        let addr_reg = self.fresh_reg();
                        instrs.push(IrInstruction::StoreImm {
                            dest: offset_reg,
                            value: IrValue::Immediate(i as i32 * elem_size),
                        });
                        instrs.push(IrInstruction::BinaryOp {
                            op: BinaryOperator::Add,
                            dest: addr_reg,
                            left: base_reg,
                            right: offset_reg,
                        });
                        instrs.push(IrInstruction::StoreIndirect {
                            ir_type: ir_elem_type.clone(),
                            addr: addr_reg,
                            src: val_reg,
                        });
                        self.free_reg(offset_reg);
                        self.free_reg(addr_reg);
                    }

                    self.free_reg(val_reg);
                }

                (base_reg, instrs) // base_reg holds pointer to first element
            }

            Expression::Index { base, index } => {
                let elem_type = self.resolve_pointee_type(base);
                let ir_elem_type = IrType::from(&elem_type);
                let elem_size = ir_elem_type.size_bytes() as i32;

                let (base_reg, base_instrs) = self.emit_expression(base);
                let (index_reg, index_instrs) = self.emit_expression(index);

                let mut instrs = base_instrs;
                instrs.extend(index_instrs);

                let addr_reg = self.fresh_reg();

                if elem_size == 1 {
                    // no scaling needed
                    instrs.push(IrInstruction::BinaryOp {
                        op: BinaryOperator::Add,
                        dest: addr_reg,
                        left: base_reg,
                        right: index_reg,
                    });
                } else {
                    // scale index by elem_size
                    let scale_reg = self.fresh_reg();
                    let size_reg = self.fresh_reg();
                    instrs.push(IrInstruction::StoreImm {
                        dest: size_reg,
                        value: IrValue::Immediate(elem_size),
                    });
                    instrs.push(IrInstruction::BinaryOp {
                        op: BinaryOperator::Mul,
                        dest: scale_reg,
                        left: index_reg,
                        right: size_reg,
                    });
                    instrs.push(IrInstruction::BinaryOp {
                        op: BinaryOperator::Add,
                        dest: addr_reg,
                        left: base_reg,
                        right: scale_reg,
                    });
                    self.free_reg(scale_reg);
                    self.free_reg(size_reg);
                }

                let dest = self.fresh_reg();
                instrs.push(IrInstruction::LoadIndirect {
                    ir_type: ir_elem_type,
                    dest,
                    addr: addr_reg,
                });

                self.free_reg(base_reg);
                self.free_reg(index_reg);
                self.free_reg(addr_reg);

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

    fn lookup_symbol_id(&self, name: &str) -> SymbolId {
        let mut scope_id = self.current_scope_id;
        loop {
            let scope = &self.program.scope_table[scope_id];
            if let Some(s) = scope.symbols.iter().find(|s| s.name == *name) {
                return s.id.clone();
            }
            match scope.parent {
                Some(ScopeId(id)) => scope_id = id as usize,
                None => panic!("symbol '{}' not found in scope chain", name),
            }
        }
    }

    fn resolve_expr_type(&self, expr: &Expression) -> Type {
        match expr {
            Expression::Identifier(name) => {
                let sym_id = self.lookup_symbol_id(name);
                match &self.lookup(&sym_id).kind {
                    SymbolKind::Variable { var_type, .. } => var_type.clone(),
                    SymbolKind::Function { return_type, .. } => return_type.clone(),
                }
            }
            Expression::UnaryOperation { op, operand } => match op {
                UnaryOperator::AddressOf => {
                    Type::Pointer(Box::new(self.resolve_expr_type(operand)))
                }
                UnaryOperator::Deref => match self.resolve_expr_type(operand) {
                    Type::Pointer(inner) => *inner,
                    other => panic!("cannot dereference non-pointer type {:?}", other),
                },
                UnaryOperator::Not => Type::Bool,
            },
            Expression::Integer(_) => Type::Int32,
            Expression::Bool(_) => Type::Bool,
            Expression::FunctionCall { .. } => {
                panic!("resolve_expr_type: function call type resolution not wired here yet")
            }
            Expression::BinaryOperation { left, .. } => self.resolve_expr_type(left),

            Expression::Array { values, .. } => {
                let elem_type = values
                    .first()
                    .map(|v| self.resolve_expr_type(v))
                    .unwrap_or(Type::Void);
                Type::Pointer(Box::new(elem_type))
            }

            Expression::Index { base, .. } => match self.resolve_expr_type(base) {
                Type::Pointer(inner) => *inner,
                other => panic!("cannot index non-pointer type {:?}", other),
            },
        }
    }

    fn resolve_pointee_type(&self, expr: &Expression) -> Type {
        match self.resolve_expr_type(expr) {
            Type::Pointer(inner) => *inner,
            other => panic!("expected pointer type, got {:?}", other),
        }
    }
}
