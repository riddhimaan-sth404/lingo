use lingo_ast::*;
use lingo_diagnostics::Diagnostic;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq)]
pub enum SymbolKind {
    Variable { is_let: bool, ty: TypeAnnotation },
    Function { arity: usize, ret_ty: TypeAnnotation },
}

#[derive(Debug, Clone, Default)]
pub struct Scope {
    symbols: HashMap<String, SymbolKind>,
}

pub struct Resolver {
    scopes: Vec<Scope>,
    builtin_functions: HashSet<String>,
}

impl Resolver {
    pub fn new() -> Self {
        let mut builtin_functions = HashSet::new();
        builtin_functions.insert("print".into());
        builtin_functions.insert("println".into());
        builtin_functions.insert("len".into());
        builtin_functions.insert("Channel".into());
        builtin_functions.insert("channel_new".into());
        builtin_functions.insert("channel_send".into());
        builtin_functions.insert("channel_recv".into());
        builtin_functions.insert("sleep_ms".into());
        builtin_functions.insert("time_now_ms".into());
        builtin_functions.insert("file_read".into());
        builtin_functions.insert("file_write".into());
        builtin_functions.insert("file_exists".into());
        builtin_functions.insert("env_get".into());
        builtin_functions.insert("math_sqrt".into());
        builtin_functions.insert("math_sin".into());
        builtin_functions.insert("math_cos".into());
        builtin_functions.insert("math_floor".into());
        builtin_functions.insert("math_ceil".into());
        builtin_functions.insert("math_abs".into());

        Self {
            scopes: vec![Scope::default()],
            builtin_functions,
        }
    }

    fn push_scope(&mut self) {
        self.scopes.push(Scope::default());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: &str, kind: SymbolKind) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.symbols.insert(name.to_string(), kind);
        }
    }

    pub fn lookup(&self, name: &str) -> Option<&SymbolKind> {
        for scope in self.scopes.iter().rev() {
            if let Some(sym) = scope.symbols.get(name) {
                return Some(sym);
            }
        }
        None
    }

    pub fn is_builtin(&self, name: &str) -> bool {
        self.builtin_functions.contains(name)
    }

    pub fn resolve_program(&mut self, program: &Program) -> Result<(), Diagnostic> {
        // First pass: register top-level functions
        for stmt in &program.stmts {
            if let StmtKind::Function { name, params, ret_ty, .. } = &stmt.kind {
                self.declare(
                    name,
                    SymbolKind::Function {
                        arity: params.len(),
                        ret_ty: ret_ty.clone(),
                    },
                );
            }
            if let StmtKind::ExternFunction { name, params, ret_ty, .. } = &stmt.kind {
                self.declare(
                    name,
                    SymbolKind::Function {
                        arity: params.len(),
                        ret_ty: ret_ty.clone(),
                    },
                );
            }
        }

        // Second pass: resolve bodies
        for stmt in &program.stmts {
            self.resolve_stmt(stmt)?;
        }

        Ok(())
    }

    fn resolve_stmt(&mut self, stmt: &Stmt) -> Result<(), Diagnostic> {
        match &stmt.kind {
            StmtKind::Function { params, body, .. } => {
                self.push_scope();
                for p in params {
                    self.declare(&p.name, SymbolKind::Variable { is_let: false, ty: p.ty.clone() });
                }
                for s in body {
                    self.resolve_stmt(s)?;
                }
                self.pop_scope();
            }
            StmtKind::Let { name, ty, init } => {
                if let Some(init_expr) = init {
                    self.resolve_expr(init_expr)?;
                }
                self.declare(name, SymbolKind::Variable { is_let: true, ty: ty.clone() });
            }
            StmtKind::Assign { target, value } => {
                self.resolve_expr(value)?;
                if let ExprKind::Ident(ref name) = target.kind {
                    if self.lookup(name).is_none() {
                        // Dynamic variable declaration on assignment
                        self.declare(name, SymbolKind::Variable { is_let: false, ty: TypeAnnotation::Dynamic });
                    }
                } else {
                    self.resolve_expr(target)?;
                }
            }
            StmtKind::If { condition, then_branch, elif_branches, else_branch } => {
                self.resolve_expr(condition)?;
                self.push_scope();
                for s in then_branch {
                    self.resolve_stmt(s)?;
                }
                self.pop_scope();

                for (elif_cond, elif_body) in elif_branches {
                    self.resolve_expr(elif_cond)?;
                    self.push_scope();
                    for s in elif_body {
                        self.resolve_stmt(s)?;
                    }
                    self.pop_scope();
                }

                if let Some(else_stmts) = else_branch {
                    self.push_scope();
                    for s in else_stmts {
                        self.resolve_stmt(s)?;
                    }
                    self.pop_scope();
                }
            }
            StmtKind::While { condition, body } => {
                self.resolve_expr(condition)?;
                self.push_scope();
                for s in body {
                    self.resolve_stmt(s)?;
                }
                self.pop_scope();
            }
            StmtKind::For { var, iter, body } => {
                self.resolve_expr(iter)?;
                self.push_scope();
                self.declare(var, SymbolKind::Variable { is_let: false, ty: TypeAnnotation::Dynamic });
                for s in body {
                    self.resolve_stmt(s)?;
                }
                self.pop_scope();
            }
            StmtKind::Return(Some(expr)) => {
                self.resolve_expr(expr)?;
            }
            StmtKind::Return(None) => {}
            StmtKind::Expr(expr) => {
                self.resolve_expr(expr)?;
            }
            StmtKind::Import { module, .. } => {
                self.declare(module, SymbolKind::Variable { is_let: true, ty: TypeAnnotation::Dynamic });
            }
            StmtKind::ExternFunction { .. } => {}
        }
        Ok(())
    }

    fn resolve_expr(&self, expr: &Expr) -> Result<(), Diagnostic> {
        match &expr.kind {
            ExprKind::Ident(name) => {
                if self.lookup(name).is_none() && !self.is_builtin(name) {
                    return Err(Diagnostic::error(
                        format!("Use of undeclared identifier `{name}`"),
                        expr.span,
                    ));
                }
            }
            ExprKind::Binary { left, right, .. } => {
                self.resolve_expr(left)?;
                self.resolve_expr(right)?;
            }
            ExprKind::Unary { expr, .. } => {
                self.resolve_expr(expr)?;
            }
            ExprKind::Call { callee, args } => {
                self.resolve_expr(callee)?;
                for arg in args {
                    self.resolve_expr(arg)?;
                }
            }
            ExprKind::List(items) => {
                for item in items {
                    self.resolve_expr(item)?;
                }
            }
            ExprKind::Map(entries) => {
                for (k, v) in entries {
                    self.resolve_expr(k)?;
                    self.resolve_expr(v)?;
                }
            }
            ExprKind::Index { target, index } => {
                self.resolve_expr(target)?;
                self.resolve_expr(index)?;
            }
            ExprKind::Assign { target, value } => {
                self.resolve_expr(target)?;
                self.resolve_expr(value)?;
            }
            _ => {}
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_undeclared_variable_error() {
        let resolver = Resolver::new();
        let expr = Expr::new(ExprKind::Ident("undefined_var".into()), lingo_source::Span::default());
        let res = resolver.resolve_expr(&expr);
        assert!(res.is_err());
    }
}
