use parser::{expression::Expression, program::Program, statement::Statement, types::Type};

use crate::{
    program::SemanticProgram,
    scope::{Scope, ScopeId, ScopeType},
    statement::SemanticStatement,
    symbol::{State, Symbol, SymbolId, SymbolKind},
    warning::{SemanticWarning, WarningType},
};

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

    pub fn analyze(&mut self) -> SemanticProgram {
        self.init_scope_table();

        let top_level = self.statements.body.clone();
        for stmt in &top_level {
            if let Some(sem) = self.analyze_statement(stmt) {
                self.body.push(sem);
            }
        }

        SemanticProgram {
            scope_table: self.scope_table.clone(),
            body: self.body.clone(),
            diagnostics: self.warnings.clone(),
        }
    }

    fn init_scope_table(&mut self) {
        let global_functions = self.collect_global_functions();
        self.scope_table.push(Scope {
            scope_id: ScopeId(0),
            kind: ScopeType::Global,
            parent: None,
            symbols: global_functions,
        });
    }

    fn collect_global_functions(&mut self) -> Vec<Symbol> {
        let mut functions = Vec::new();
        for stmt in &self.statements.body.clone() {
            if let Statement::FunctionDecleration {
                return_type,
                name,
                params,
                ..
            } = stmt
            {
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
        functions
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

    fn analyze_statement(&mut self, stmt: &Statement) -> Option<SemanticStatement> {
        match stmt {
            Statement::FunctionDecleration {
                return_type,
                name,
                body,
                line,
                ..
            } => {
                let (_, sym_idx) = self.find_symbol(name)?;
                let symbol_id = self.scope_table[0].symbols[sym_idx].id.clone();

                let fn_sym_id = symbol_id.clone();
                self.enter_scope(ScopeType::FunctionBody { parent: fn_sym_id });

                let mut sem_body = Vec::new();
                for s in body {
                    if let Some(sem) = self.analyze_statement(s) {
                        sem_body.push(sem);
                    }
                }

                self.exit_scope();

                // warn if non-void function has no return
                let has_return = sem_body
                    .iter()
                    .any(|s| matches!(s, SemanticStatement::SemanticReturn { .. }));
                if !has_return {
                    self.warnings.push(SemanticWarning {
                        warning_type: WarningType::MissingReturn,
                        name: name.to_string(),
                        message: WarningType::MissingReturn.get_message(name.to_string()),
                        line: *line,
                    });
                }

                Some(SemanticStatement::SemanticFunction {
                    symbol: symbol_id,
                    return_type: return_type.clone(),
                    body: sem_body,
                    line: *line,
                })
            }

            Statement::VariableDeclaration {
                var_name,
                var_type,
                operation,
                line,
            } => {
                // check RHS before inserting symbol - prevents i32 a = a;
                if let Some(expr) = operation {
                    self.analyze_expression(expr, *line);
                }

                if let Some(Expression::Integer(n)) = operation {
                    let fits = match var_type {
                        Type::Int8 => *n >= -128 && *n <= 127,
                        Type::Int16 => *n >= -32768 && *n <= 32767,
                        Type::Int32 => true,
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

                Some(SemanticStatement::SemanticVarDecl {
                    symbol: symbol_id,
                    initializer: operation.clone(),
                })
            }

            Statement::Assign {
                var_name,
                value,
                line,
            } => {
                self.analyze_expression(value, *line);

                if let Some((si, sym_i)) = self.find_symbol(var_name) {
                    let var_type_clone = match &self.scope_table[si].symbols[sym_i].kind {
                        SymbolKind::Variable { var_type, .. } => var_type.clone(),
                        _ => return None,
                    };
                    self.scope_table[si].symbols[sym_i].kind = SymbolKind::Variable {
                        var_type: var_type_clone,
                        state: State::Initialized,
                    };
                }

                let symbol_id = self
                    .find_symbol(var_name)
                    .map(|(si, sym_i)| self.scope_table[si].symbols[sym_i].id.clone())?;

                Some(SemanticStatement::SemanticAssign {
                    symbol: symbol_id,
                    value: value.clone(),
                })
            }

            Statement::Return { return_value } => {
                // analyze the return expression if there is one
                if let Some(expr) = return_value {
                    self.analyze_expression(expr, 0);
                }

                Some(SemanticStatement::SemanticReturn {
                    value: return_value.clone(),
                })
            }

            Statement::NewLine => None,

            _ => panic!("{:?} is not implemented in analyze_statement", stmt),
        }
    }

    fn analyze_expression(&mut self, expr: &Expression, line: usize) {
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

            Expression::BinaryOperation { left, right, .. } => {
                self.analyze_expression(left, line);
                self.analyze_expression(right, line);
            }

            // Literals carry no symbol references, nothing to check.
            Expression::Integer(_) | Expression::Bool(_) => {}
        }
    }
}
