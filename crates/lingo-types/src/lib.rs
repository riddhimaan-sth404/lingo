use lingo_ast::*;
use lingo_diagnostics::Diagnostic;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Dynamic,
    Int,
    Float,
    Bool,
    Str,
    Nil,
    Void,
}

impl Type {
    pub fn from_annotation(ann: &TypeAnnotation) -> Self {
        match ann {
            TypeAnnotation::Dynamic => Type::Dynamic,
            TypeAnnotation::Int => Type::Int,
            TypeAnnotation::Float => Type::Float,
            TypeAnnotation::Bool => Type::Bool,
            TypeAnnotation::Str => Type::Str,
            TypeAnnotation::Custom(_) => Type::Dynamic,
        }
    }

    pub fn is_assignable_to(&self, target: &Type) -> bool {
        if *self == Type::Dynamic || *target == Type::Dynamic {
            return true;
        }
        self == target
    }
}

pub struct TypeChecker {
    env: HashMap<String, Type>,
}

impl TypeChecker {
    pub fn new() -> Self {
        Self {
            env: HashMap::new(),
        }
    }

    pub fn check_program(&mut self, program: &Program) -> Result<(), Diagnostic> {
        for stmt in &program.stmts {
            self.check_stmt(stmt)?;
        }
        Ok(())
    }

    fn check_stmt(&mut self, stmt: &Stmt) -> Result<(), Diagnostic> {
        match &stmt.kind {
            StmtKind::Function { name, params, ret_ty, body } => {
                for p in params {
                    self.env.insert(p.name.clone(), Type::from_annotation(&p.ty));
                }
                let expected_ret = Type::from_annotation(ret_ty);
                for s in body {
                    self.check_stmt(s)?;
                }
                self.env.insert(name.clone(), expected_ret);
            }
            StmtKind::ExternFunction { name, ret_ty, .. } => {
                let expected_ret = Type::from_annotation(ret_ty);
                self.env.insert(name.clone(), expected_ret);
            }
            StmtKind::Let { name, ty, init } => {
                let declared_ty = Type::from_annotation(ty);
                if let Some(init_expr) = init {
                    let actual_ty = self.infer_expr(init_expr)?;
                    if !actual_ty.is_assignable_to(&declared_ty) {
                        return Err(Diagnostic::error(
                            format!("Type mismatch: cannot assign {:?} to declared type {:?}", actual_ty, declared_ty),
                            init_expr.span,
                        ));
                    }
                }
                self.env.insert(name.clone(), declared_ty);
            }
            StmtKind::Assign { target, value } => {
                let val_ty = self.infer_expr(value)?;
                if let ExprKind::Ident(ref name) = target.kind {
                    if let Some(target_ty) = self.env.get(name) {
                        if !val_ty.is_assignable_to(target_ty) {
                            return Err(Diagnostic::error(
                                format!("Type mismatch: cannot assign {:?} to variable of type {:?}", val_ty, target_ty),
                                value.span,
                            ));
                        }
                    } else {
                        self.env.insert(name.clone(), Type::Dynamic);
                    }
                }
            }
            StmtKind::If { condition, then_branch, elif_branches, else_branch } => {
                self.infer_expr(condition)?;
                for s in then_branch {
                    self.check_stmt(s)?;
                }
                for (c, b) in elif_branches {
                    self.infer_expr(c)?;
                    for s in b {
                        self.check_stmt(s)?;
                    }
                }
                if let Some(else_b) = else_branch {
                    for s in else_b {
                        self.check_stmt(s)?;
                    }
                }
            }
            StmtKind::While { condition, body } => {
                self.infer_expr(condition)?;
                for s in body {
                    self.check_stmt(s)?;
                }
            }
            StmtKind::For { var, iter, body } => {
                self.infer_expr(iter)?;
                self.env.insert(var.clone(), Type::Dynamic);
                for s in body {
                    self.check_stmt(s)?;
                }
            }
            StmtKind::Return(Some(expr)) => {
                self.infer_expr(expr)?;
            }
            StmtKind::Return(None) => {}
            StmtKind::Expr(expr) => {
                self.infer_expr(expr)?;
            }
            StmtKind::Import { .. } => {}
        }
        Ok(())
    }

    pub fn infer_expr(&self, expr: &Expr) -> Result<Type, Diagnostic> {
        match &expr.kind {
            ExprKind::Int(_) => Ok(Type::Int),
            ExprKind::Float(_) => Ok(Type::Float),
            ExprKind::Bool(_) => Ok(Type::Bool),
            ExprKind::Str(_) => Ok(Type::Str),
            ExprKind::Nil => Ok(Type::Nil),
            ExprKind::Ident(name) => {
                Ok(self.env.get(name).cloned().unwrap_or(Type::Dynamic))
            }
            ExprKind::Binary { op, left, right } => {
                let l_ty = self.infer_expr(left)?;
                let r_ty = self.infer_expr(right)?;

                match op {
                    BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => {
                        if l_ty == Type::Int && r_ty == Type::Int {
                            Ok(Type::Int)
                        } else if l_ty == Type::Float || r_ty == Type::Float {
                            Ok(Type::Float)
                        } else if l_ty == Type::Str && *op == BinaryOp::Add {
                            Ok(Type::Str)
                        } else {
                            Ok(Type::Dynamic)
                        }
                    }
                    BinaryOp::Eq | BinaryOp::NotEq | BinaryOp::Lt | BinaryOp::LtEq | BinaryOp::Gt | BinaryOp::GtEq | BinaryOp::And | BinaryOp::Or => {
                        Ok(Type::Bool)
                    }
                }
            }
            ExprKind::Unary { op, expr } => {
                let ty = self.infer_expr(expr)?;
                match op {
                    UnaryOp::Neg => Ok(ty),
                    UnaryOp::Not => Ok(Type::Bool),
                }
            }
            ExprKind::Call { callee, args } => {
                if let ExprKind::Ident(ref name) = callee.kind {
                    if name == "print" || name == "println" {
                        return Ok(Type::Void);
                    }
                }
                for arg in args {
                    self.infer_expr(arg)?;
                }
                Ok(Type::Dynamic)
            }
            ExprKind::List(_) => Ok(Type::Dynamic),
            ExprKind::Map(entries) => {
                for (k, v) in entries {
                    self.infer_expr(k)?;
                    self.infer_expr(v)?;
                }
                Ok(Type::Dynamic)
            }
            ExprKind::Index { .. } => Ok(Type::Dynamic),
            ExprKind::Assign { value, .. } => self.infer_expr(value),
        }
    }
}
