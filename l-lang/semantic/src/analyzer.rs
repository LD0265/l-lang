use crate::{
    program::SemanticProgram,
    scope::{Scope, ScopeId, ScopeType},
    statement::{SemanticParam, SemanticStatement},
    symbol::{State, Symbol, SymbolId, SymbolKind},
    warning::{SemanticWarning, WarningType},
};
use parser::{
    expression::{BinaryOperator, Expression, UnaryOperator},
    program::Program,
    statement::Statement,
    types::Type,
};
use util::error::CompileError;
use util::error::Result;

pub struct Analyzer {
    statements: Program,
    scope_table: Vec<Scope>,
    body: Vec<SemanticStatement>,
    warnings: Vec<SemanticWarning>,
    current_scope_id: usize,
    current_symbol_id: usize,
}

impl Analyzer {
    pub fn new(ast: Program) -> Self {
        Self {
            statements: ast,
            scope_table: Vec::new(),
            body: Vec::new(),
            warnings: Vec::new(),
            current_scope_id: 0,
            current_symbol_id: 0,
        }
    }

    pub fn analyze(&mut self) -> Result<SemanticProgram> {
        self.init_scope_table()?;

        let mut top_level = self.statements.body.clone();
        for stmt in &mut top_level {
            if let Some(sem) = self.analyze_statement(stmt)? {
                self.body.push(sem);
            }
        }

        Ok(SemanticProgram {
            scope_table: self.scope_table.clone(),
            body: self.body.clone(),
            diagnostics: self.warnings.clone(),
        })
    }

    fn init_scope_table(&mut self) -> Result<()> {
        let global_functions = self.collect_global_functions();
        let funcs = match global_functions {
            Ok(vec) => vec,
            Err(e) => return Err(e),
        };

        self.scope_table.push(Scope {
            scope_id: ScopeId(0),
            kind: ScopeType::Global,
            parent: None,
            symbols: funcs,
        });

        Ok(())
    }

    fn collect_global_functions(&mut self) -> Result<Vec<Symbol>> {
        let mut functions = Vec::new();
        for stmt in &self.statements.body.clone() {
            if let Statement::FunctionDecleration {
                return_type,
                name,
                params,
                line,
                ..
            } = stmt
            {
                let already_declared = functions.iter().any(|f: &Symbol| f.name == *name);

                if already_declared {
                    return Err(CompileError::SemanticError {
                        message: format!("function '{}' already declared", name),
                        line: *line,
                    });
                }

                functions.push(Symbol {
                    name: name.to_string(),
                    id: SymbolId(self.current_symbol_id),
                    kind: SymbolKind::Function {
                        return_type: return_type.clone(),
                        params: params.clone(),
                    },
                    declared_scope: ScopeId(0),
                    line: 0, // TODO: tmp 0
                });
                self.current_symbol_id += 1;
            }
        }
        Ok(functions)
    }

    fn enter_scope(&mut self, kind: ScopeType) -> usize {
        let id = self.scope_table.len();
        self.scope_table.push(Scope {
            scope_id: ScopeId(id as i32),
            kind,
            parent: Some(ScopeId(self.current_scope_id as i32)),
            symbols: Vec::new(),
        });
        self.current_scope_id = id;
        id
    }

    fn exit_scope(&mut self) {
        // warn on any variables that were never initialized
        let symbols = self.scope_table[self.current_scope_id].symbols.clone();
        for sym in &symbols {
            if let SymbolKind::Variable {
                state: State::Uninitialized,
                ..
            } = &sym.kind
            {
                self.warnings.push(SemanticWarning {
                    warning_type: WarningType::UninitializedVariable,
                    name: sym.name.clone(),
                    message: WarningType::UninitializedVariable.get_message(sym.name.clone()),
                    line: 0,
                });
            }
        }

        let parent = match &self.scope_table[self.current_scope_id].parent {
            Some(ScopeId(id)) => *id as usize,
            None => 0,
        };
        self.current_scope_id = parent;
    }

    fn insert_symbol(&mut self, name: &str, kind: SymbolKind, line: usize) -> SymbolId {
        let id = SymbolId(self.current_symbol_id);
        self.current_symbol_id += 1;
        self.scope_table[self.current_scope_id]
            .symbols
            .push(Symbol {
                name: name.to_string(),
                id: id.clone(),
                kind,
                declared_scope: ScopeId(self.current_scope_id as i32),
                line,
            });
        id
    }

    fn find_symbol(&self, name: &str) -> Option<(usize, usize)> {
        let mut scope_id = self.current_scope_id;
        loop {
            let scope = &self.scope_table[scope_id];
            if let Some(i) = scope.symbols.iter().position(|s| s.name == name) {
                return Some((scope_id, i));
            }
            match scope.parent {
                Some(ScopeId(id)) => scope_id = id as usize,
                None => return None,
            }
        }
    }

    fn analyze_statement(&mut self, stmt: &mut Statement) -> Result<Option<SemanticStatement>> {
        match stmt {
            Statement::FunctionDecleration {
                return_type,
                name,
                params,
                body,
                line,
                ..
            } => {
                let (_, sym_idx) =
                    self.find_symbol(name)
                        .ok_or_else(|| CompileError::SemanticError {
                            message: format!("function '{}' not found in scope", name),
                            line: *line,
                        })?;
                let symbol_id = self.scope_table[0].symbols[sym_idx].id.clone();

                self.enter_scope(ScopeType::FunctionBody {
                    name: name.clone(),
                    parent: symbol_id.clone(),
                });

                let mut sem_params: Vec<SemanticParam> = Vec::new();

                // insert each param as a variable symbol in the function scope
                for param in params {
                    let already_declared = self.scope_table[self.current_scope_id]
                        .symbols
                        .iter()
                        .any(|s| s.name == param.name);

                    if already_declared {
                        return Err(CompileError::SemanticError {
                            message: format!("duplicate parameter name '{}'", param.name),
                            line: *line,
                        });
                    }

                    let param_id = self.insert_symbol(
                        &param.name,
                        SymbolKind::Variable {
                            var_type: param.param_type.clone(),
                            state: State::Initialized,
                        },
                        *line,
                    );

                    sem_params.push(SemanticParam {
                        name: param.name.clone(),
                        param_type: param.param_type,
                        symbol: param_id,
                    });
                }

                let mut sem_body = Vec::new();
                for s in body {
                    if let Some(sem) = self.analyze_statement(s)? {
                        sem_body.push(sem);
                    }
                }

                self.exit_scope();

                let has_return = sem_body
                    .iter()
                    .any(|s| matches!(s, SemanticStatement::SemanticReturn { .. }));
                if !has_return {
                    return Err(CompileError::SemanticError {
                        message: format!("function `{}` has no return statement", name),
                        line: *line,
                    });
                }

                Ok(Some(SemanticStatement::SemanticFunction {
                    name: name.clone(),
                    symbol: symbol_id,
                    return_type: return_type.clone(),
                    params: sem_params.clone(),
                    body: sem_body,
                    line: *line,
                }))
            }

            Statement::FunctionCall { name, args, line } => {
                // check function exists
                let Some((scope_idx, sym_idx)) = self.find_symbol(name) else {
                    return Err(CompileError::SemanticError {
                        message: format!("call to undeclared function '{}'", name),
                        line: *line,
                    });
                };

                let expected_params = match &self.scope_table[scope_idx].symbols[sym_idx].kind {
                    SymbolKind::Function { params, .. } => params.len(),
                    _ => {
                        return Err(CompileError::SemanticError {
                            message: format!("'{}' is not a function", name),
                            line: *line,
                        });
                    }
                };

                if args.len() != expected_params {
                    return Err(CompileError::SemanticError {
                        message: format!(
                            "'{}' expects {} args, got {}",
                            name,
                            expected_params,
                            args.len()
                        ),
                        line: *line,
                    });
                }

                for arg in args.iter_mut() {
                    self.analyze_expression(arg, *line);
                }

                Ok(Some(SemanticStatement::SemanticFunctionCall {
                    name: name.clone(),
                    args: args.clone(),
                    line: *line,
                }))
            }

            Statement::VariableDeclaration {
                var_name,
                var_type,
                operation,
                line,
            } => {
                // duplicate check
                let already_declared = self.scope_table[self.current_scope_id]
                    .symbols
                    .iter()
                    .any(|s| s.name == *var_name);

                if already_declared {
                    return Err(CompileError::SemanticError {
                        message: format!("variable '{}' already declared in this scope", var_name),
                        line: *line,
                    });
                }

                if let Some(expr) = operation.as_mut() {
                    self.analyze_expression(expr, *line);

                    // type mismatch check: declared type vs initializer type
                    let expr_type = self.resolve_type(expr);
                    if let Some(et) = expr_type {
                        if et != *var_type && *var_type != Type::Void {
                            self.warnings.push(SemanticWarning {
                                warning_type: WarningType::TypeMismatch,
                                name: var_name.to_string(),
                                message: format!(
                                    "variable '{}' declared as {:?} but assigned {:?}",
                                    var_name, var_type, et
                                ),
                                line: *line,
                            });
                        }
                    }
                }

                if let Some(Expression::Integer(n)) = operation {
                    let fits = match var_type {
                        Type::Int8 => *n >= -128 && *n <= 127,
                        Type::Int16 => *n >= -32768 && *n <= 32767,
                        _ => true,
                    };
                    if !fits {
                        self.warnings.push(SemanticWarning {
                            warning_type: WarningType::LiteralOverflow,
                            name: var_name.to_string(),
                            message: WarningType::LiteralOverflow.get_message(var_name.to_string()),
                            line: *line,
                        });
                    }
                }

                let state = if operation.is_some() {
                    State::Initialized
                } else {
                    State::Uninitialized
                };
                let symbol_id = self.insert_symbol(
                    var_name,
                    SymbolKind::Variable {
                        var_type: var_type.clone(),
                        state,
                    },
                    *line,
                );

                Ok(Some(SemanticStatement::SemanticVarDecl {
                    symbol: symbol_id,
                    initializer: operation.clone(),
                }))
            }

            Statement::Assign {
                var_name,
                value,
                line,
            } => {
                self.analyze_expression(value, *line);

                let expr_type = self.resolve_type(value);

                if let Some((si, sym_i)) = self.find_symbol(var_name) {
                    let var_type_clone = match &self.scope_table[si].symbols[sym_i].kind {
                        SymbolKind::Variable { var_type, .. } => var_type.clone(),
                        _ => return Ok(None),
                    };

                    // type mismatch check
                    if let Some(et) = expr_type {
                        if et != var_type_clone {
                            self.warnings.push(SemanticWarning {
                                warning_type: WarningType::TypeMismatch,
                                name: var_name.to_string(),
                                message: format!(
                                    "cannot assign {:?} to variable '{}' of type {:?}",
                                    et, var_name, var_type_clone
                                ),
                                line: *line,
                            });
                        }
                    }

                    self.scope_table[si].symbols[sym_i].kind = SymbolKind::Variable {
                        var_type: var_type_clone,
                        state: State::Initialized,
                    };
                }

                let symbol_id = self
                    .find_symbol(var_name)
                    .map(|(si, sym_i)| self.scope_table[si].symbols[sym_i].id.clone())
                    .ok_or_else(|| CompileError::SemanticError {
                        message: format!("assignment to undeclared variable '{}'", var_name),
                        line: *line,
                    })?;

                Ok(Some(SemanticStatement::SemanticAssign {
                    symbol: symbol_id,
                    value: value.clone(),
                }))
            }

            Statement::Return { return_value } => {
                if let Some(expr) = return_value {
                    self.analyze_expression(expr, 0);
                }
                Ok(Some(SemanticStatement::SemanticReturn {
                    value: return_value.clone(),
                }))
            }

            Statement::NewLine => Ok(None),

            /*
                assume that the assembly is correct and we don't check it
                which is kinda dangerous but you should know that anyway
            */
            Statement::Assembly { body } => Ok(Some(SemanticStatement::SemanticAssembly {
                body: body.clone(),
            })),

            // Had to comment this to make the compiler happy
            // _ => Err(CompileError::CompilerError {
            //     message: format!("{:?} is not implemented in analyze_statement", stmt),
            //     line: 0,
            // }),
            
            Statement::If {
                label,
                condition,
                body,
                else_label,
                else_body,
            } => {
                self.analyze_expression(condition, 0);

                self.enter_scope(ScopeType::IfBody {
                    parent: SymbolId(self.current_scope_id),
                });

                let mut sem_body = Vec::new();
                for s in body {
                    if let Some(sem) = self.analyze_statement(s)? {
                        sem_body.push(sem);
                    }
                }

                self.exit_scope();

                let sem_else_body = if let Some(else_stmts) = else_body {
                    self.enter_scope(ScopeType::IfBody {
                        parent: SymbolId(self.current_scope_id),
                    });

                    let mut else_sem = Vec::new();
                    for s in else_stmts {
                        if let Some(sem) = self.analyze_statement(s)? {
                            else_sem.push(sem);
                        }
                    }

                    self.exit_scope();
                    Some(else_sem)
                } else {
                    None
                };

                Ok(Some(SemanticStatement::SemanticIf {
                    label: label.clone(),
                    condition: condition.clone(),
                    body: sem_body,
                    else_label: else_label.clone(),
                    else_body: sem_else_body,
                }))
            }
        }
    }

    fn analyze_expression(&mut self, expr: &mut Expression, line: usize) {
        match expr {
            Expression::Identifier(name) => match self.find_symbol(name) {
                None => {}
                Some((scope_idx, sym_idx)) => {
                    let (is_uninit, var_type_clone) = {
                        let sym = &self.scope_table[scope_idx].symbols[sym_idx];
                        match &sym.kind {
                            SymbolKind::Variable { state, var_type } => {
                                (matches!(state, State::Uninitialized), var_type.clone())
                            }
                            _ => return,
                        }
                    };

                    if is_uninit {
                        let msg = WarningType::UninitializedVariable.get_message(name.to_string());
                        self.warnings.push(SemanticWarning {
                            warning_type: WarningType::UninitializedVariable,
                            name: name.to_string(),
                            message: msg,
                            line,
                        });
                    } else {
                        self.scope_table[scope_idx].symbols[sym_idx].kind = SymbolKind::Variable {
                            var_type: var_type_clone,
                            state: State::Used,
                        };
                    }
                }
            },

            Expression::FunctionCall {
                return_type,
                name,
                args,
            } => {
                // check function exists
                let Some((scope_idx, sym_idx)) = self.find_symbol(name) else {
                    self.warnings.push(SemanticWarning {
                        warning_type: WarningType::UndeclaredFunction,
                        name: name.clone(),
                        message: format!("call to undeclared function '{}'", name),
                        line,
                    });
                    return;
                };

                let (expected_params, resolved_return) =
                    match &self.scope_table[scope_idx].symbols[sym_idx].kind {
                        SymbolKind::Function {
                            params,
                            return_type,
                        } => (params.len(), return_type.clone()),
                        _ => {
                            self.warnings.push(SemanticWarning {
                                warning_type: WarningType::TypeMismatch,
                                name: name.clone(),
                                message: format!("'{}' is not a function", name),
                                line,
                            });
                            return;
                        }
                    };

                // check arg count matches param count
                if args.len() != expected_params {
                    self.warnings.push(SemanticWarning {
                        warning_type: WarningType::TypeMismatch,
                        name: name.clone(),
                        message: format!(
                            "'{}' expects {} args, got {}",
                            name,
                            expected_params,
                            args.len()
                        ),
                        line,
                    });
                }

                for arg in args {
                    self.analyze_expression(arg, line);
                }

                *return_type = Some(resolved_return);
            }

            Expression::BinaryOperation { op, left, right } => {
                self.analyze_expression(left, line);
                self.analyze_expression(right, line);

                let left_type = self.resolve_type(left);
                let right_type = self.resolve_type(right);

                let is_comparison = matches!(
                    op,
                    BinaryOperator::Eq
                        | BinaryOperator::NotEq
                        | BinaryOperator::Lt
                        | BinaryOperator::Gt
                        | BinaryOperator::LtEq
                        | BinaryOperator::GtEq
                );
                let is_logical = matches!(op, BinaryOperator::And | BinaryOperator::Or);
                let is_arithmetic = matches!(
                    op,
                    BinaryOperator::Add
                        | BinaryOperator::Sub
                        | BinaryOperator::Mul
                        | BinaryOperator::Div
                );

                if is_logical {
                    // both operands must be bool
                    if !matches!(left_type, Some(Type::Bool))
                        || !matches!(right_type, Some(Type::Bool))
                    {
                        self.warnings.push(SemanticWarning {
                            warning_type: WarningType::TypeMismatch,
                            name: "logical operators require bool operands".to_string(),
                            message: "logical operators require bool operands".to_string(),
                            line,
                        });
                    }
                } else if is_arithmetic {
                    // bool not valid in arithmetic
                    if matches!(left_type, Some(Type::Bool))
                        || matches!(right_type, Some(Type::Bool))
                    {
                        self.warnings.push(SemanticWarning {
                            warning_type: WarningType::TypeMismatch,
                            name: "bool cannot be used in arithmetic expression".to_string(),
                            message: WarningType::TypeMismatch.get_message(
                                "bool cannot be used in arithmetic expression".to_string(),
                            ),
                            line,
                        });
                    }
                    // warn if types differ
                    if let (Some(lt), Some(rt)) = (left_type, right_type) {
                        if lt != rt {
                            self.warnings.push(SemanticWarning {
                                warning_type: WarningType::TypeMismatch,
                                name: format!("type mismatch in expression: {:?} and {:?}", lt, rt),
                                message: format!(
                                    "type mismatch in expression: {:?} and {:?}",
                                    lt, rt
                                ),
                                line,
                            });
                        }
                    }
                }
                // comparisons just need matching types, which the type differ check covers
                if is_comparison {
                    if let (Some(lt), Some(rt)) = (left_type, right_type) {
                        if lt != rt {
                            self.warnings.push(SemanticWarning {
                                warning_type: WarningType::TypeMismatch,
                                message: format!("cannot compare {:?} and {:?}", lt, rt),
                                name: format!("cannot compare {:?} and {:?}", lt, rt),
                                line,
                            });
                        }
                    }
                }
            }

            Expression::UnaryOperation { op, operand } => {
                self.analyze_expression(operand, line);

                let operand_type = self.resolve_type(operand);
                match op {
                    UnaryOperator::Not => {
                        if !matches!(operand_type, Some(Type::Bool)) {
                            self.warnings.push(SemanticWarning {
                                warning_type: WarningType::TypeMismatch,
                                name: "not operator requires bool operand".to_string(),
                                message: "not operator requires bool operand".to_string(),
                                line,
                            });
                        }
                    }
                }
            }

            // Literals carry no symbol references, so we don't have to check anything
            Expression::Integer(_) | Expression::Bool(_) => {}
        }
    }

    fn resolve_type(&self, expr: &Expression) -> Option<Type> {
        match expr {
            Expression::Integer(_) => Some(Type::Int32),
            Expression::Bool(_) => Some(Type::Bool),

            Expression::Identifier(name) => {
                let (scope_idx, sym_idx) = self.find_symbol(name)?;
                match &self.scope_table[scope_idx].symbols[sym_idx].kind {
                    SymbolKind::Variable { var_type, .. } => Some(var_type.clone()),
                    SymbolKind::Function { return_type, .. } => Some(return_type.clone()),
                }
            }

            Expression::UnaryOperation { op, .. } => match op {
                UnaryOperator::Not => Some(Type::Bool),
            },

            Expression::BinaryOperation { op, left, .. } => {
                let is_comparison = matches!(
                    op,
                    BinaryOperator::Eq
                        | BinaryOperator::NotEq
                        | BinaryOperator::Lt
                        | BinaryOperator::Gt
                        | BinaryOperator::LtEq
                        | BinaryOperator::GtEq
                );
                let is_logical = matches!(op, BinaryOperator::And | BinaryOperator::Or);

                if is_comparison || is_logical {
                    Some(Type::Bool) // these always produce bool
                } else {
                    self.resolve_type(left) // arithmetic takes left operand type
                }
            }

            Expression::FunctionCall { return_type, .. } => return_type.clone(),
        }
    }
}
