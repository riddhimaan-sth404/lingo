use lingo_ast::*;
use lingo_types::Type;

#[derive(Debug, Clone, PartialEq)]
pub enum IrValue {
    Nil,
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
    Var(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum IrInstruction {
    Assign {
        target: String,
        value: IrExpr,
    },
    Call {
        target: Option<String>,
        callee: String,
        args: Vec<IrValue>,
    },
    Return(Option<IrValue>),
    If {
        condition: IrValue,
        then_branch: Vec<IrInstruction>,
        else_branch: Option<Vec<IrInstruction>>,
    },
    While {
        cond_prep: Vec<IrInstruction>,
        condition: IrValue,
        body: Vec<IrInstruction>,
    },
    IndexAssign {
        target: IrValue,
        index: IrValue,
        value: IrValue,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum IrExpr {
    Val(IrValue),
    Binary {
        op: BinaryOp,
        left: IrValue,
        right: IrValue,
    },
    Unary {
        op: UnaryOp,
        expr: IrValue,
    },
    List(Vec<IrValue>),
    Map(Vec<(IrValue, IrValue)>),
    Index {
        target: IrValue,
        index: IrValue,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct IrFunction {
    pub name: String,
    pub params: Vec<(String, Type)>,
    pub ret_ty: Type,
    pub instructions: Vec<IrInstruction>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IrExternFunction {
    pub name: String,
    pub params: Vec<(String, Type)>,
    pub ret_ty: Type,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IrModule {
    pub name: String,
    pub functions: Vec<IrFunction>,
    pub extern_functions: Vec<IrExternFunction>,
}

pub struct IrBuilder {
    temp_counter: usize,
}

impl IrBuilder {
    pub fn new() -> Self {
        Self { temp_counter: 0 }
    }

    fn new_temp(&mut self) -> String {
        self.temp_counter += 1;
        format!("_t{}", self.temp_counter)
    }

    pub fn build_module(&mut self, program: &Program) -> IrModule {
        let mut functions = Vec::new();
        let mut extern_functions = Vec::new();

        for stmt in &program.stmts {
            if let StmtKind::Function { name, params, ret_ty, body } = &stmt.kind {
                let ir_params = params
                    .iter()
                    .map(|p| (p.name.clone(), Type::from_annotation(&p.ty)))
                    .collect();
                let ir_ret_ty = Type::from_annotation(ret_ty);
                let instructions = self.lower_stmts(body);

                functions.push(IrFunction {
                    name: name.clone(),
                    params: ir_params,
                    ret_ty: ir_ret_ty,
                    instructions,
                });
            } else if let StmtKind::ExternFunction { name, params, ret_ty } = &stmt.kind {
                let ir_params = params
                    .iter()
                    .map(|p| (p.name.clone(), Type::from_annotation(&p.ty)))
                    .collect();
                let ir_ret_ty = Type::from_annotation(ret_ty);
                extern_functions.push(IrExternFunction {
                    name: name.clone(),
                    params: ir_params,
                    ret_ty: ir_ret_ty,
                });
            }
        }

        IrModule {
            name: "main".into(),
            functions,
            extern_functions,
        }
    }

    fn lower_stmts(&mut self, stmts: &[Stmt]) -> Vec<IrInstruction> {
        let mut instrs = Vec::new();
        for s in stmts {
            self.lower_stmt(s, &mut instrs);
        }
        instrs
    }

    fn lower_stmt(&mut self, stmt: &Stmt, instrs: &mut Vec<IrInstruction>) {
        match &stmt.kind {
            StmtKind::Let { name, init, .. } => {
                if let Some(init_expr) = init {
                    if let ExprKind::Call { callee, args } = &init_expr.kind {
                        let arg_vals: Vec<IrValue> = args.iter().map(|a| self.lower_expr(a, instrs)).collect();
                        let callee_name = match &callee.kind {
                            ExprKind::Ident(s) => s.clone(),
                            _ => "unknown".into(),
                        };
                        instrs.push(IrInstruction::Call {
                            target: Some(name.clone()),
                            callee: callee_name,
                            args: arg_vals,
                        });
                        return;
                    }
                    let val = self.lower_expr(init_expr, instrs);
                    instrs.push(IrInstruction::Assign {
                        target: name.clone(),
                        value: IrExpr::Val(val),
                    });
                }
            }
            StmtKind::Assign { target, value } => {
                if let ExprKind::Index { target: inner_tgt, index: inner_idx } = &target.kind {
                    let t_val = self.lower_expr(inner_tgt, instrs);
                    let i_val = self.lower_expr(inner_idx, instrs);
                    let v_val = self.lower_expr(value, instrs);
                    instrs.push(IrInstruction::IndexAssign {
                        target: t_val,
                        index: i_val,
                        value: v_val,
                    });
                    return;
                }
                if let ExprKind::Call { callee, args } = &value.kind {
                    if let ExprKind::Ident(ref target_name) = target.kind {
                        let arg_vals: Vec<IrValue> = args.iter().map(|a| self.lower_expr(a, instrs)).collect();
                        let callee_name = match &callee.kind {
                            ExprKind::Ident(s) => s.clone(),
                            _ => "unknown".into(),
                        };
                        instrs.push(IrInstruction::Call {
                            target: Some(target_name.clone()),
                            callee: callee_name,
                            args: arg_vals,
                        });
                        return;
                    }
                }
                let val = self.lower_expr(value, instrs);
                if let ExprKind::Ident(ref name) = target.kind {
                    instrs.push(IrInstruction::Assign {
                        target: name.clone(),
                        value: IrExpr::Val(val),
                    });
                }
            }
            StmtKind::If { condition, then_branch, elif_branches, else_branch } => {
                let cond_val = self.lower_expr(condition, instrs);
                let then_instrs = self.lower_stmts(then_branch);

                let mut current_else = else_branch.as_ref().map(|b| self.lower_stmts(b));

                for (elif_cond, elif_body) in elif_branches.iter().rev() {
                    let mut elif_instrs = Vec::new();
                    let elif_cond_val = self.lower_expr(elif_cond, &mut elif_instrs);
                    let elif_then = self.lower_stmts(elif_body);

                    elif_instrs.push(IrInstruction::If {
                        condition: elif_cond_val,
                        then_branch: elif_then,
                        else_branch: current_else,
                    });
                    current_else = Some(elif_instrs);
                }

                instrs.push(IrInstruction::If {
                    condition: cond_val,
                    then_branch: then_instrs,
                    else_branch: current_else,
                });
            }
            StmtKind::While { condition, body } => {
                let mut cond_prep = Vec::new();
                let cond_val = self.lower_expr(condition, &mut cond_prep);
                let body_instrs = self.lower_stmts(body);
                instrs.push(IrInstruction::While {
                    cond_prep,
                    condition: cond_val,
                    body: body_instrs,
                });
            }
            StmtKind::Return(opt_expr) => {
                let ret_val = opt_expr.as_ref().map(|e| self.lower_expr(e, instrs));
                instrs.push(IrInstruction::Return(ret_val));
            }
            StmtKind::Expr(expr) => {
                self.lower_expr(expr, instrs);
            }
            _ => {}
        }
    }

    fn lower_expr(&mut self, expr: &Expr, instrs: &mut Vec<IrInstruction>) -> IrValue {
        match &expr.kind {
            ExprKind::Nil => IrValue::Nil,
            ExprKind::Bool(b) => IrValue::Bool(*b),
            ExprKind::Int(i) => IrValue::Int(*i),
            ExprKind::Float(f) => IrValue::Float(*f),
            ExprKind::Str(s) => IrValue::Str(s.clone()),
            ExprKind::Ident(s) => IrValue::Var(s.clone()),
            ExprKind::Binary { op, left, right } => {
                let l_val = self.lower_expr(left, instrs);
                let r_val = self.lower_expr(right, instrs);

                // Constant folding optimization pass!
                if let (IrValue::Int(a), IrValue::Int(b)) = (&l_val, &r_val) {
                    match op {
                        BinaryOp::Add => return IrValue::Int(a + b),
                        BinaryOp::Sub => return IrValue::Int(a - b),
                        BinaryOp::Mul => return IrValue::Int(a * b),
                        BinaryOp::Div if *b != 0 => return IrValue::Int(a / b),
                        BinaryOp::Mod if *b != 0 => return IrValue::Int(a % b),
                        BinaryOp::Eq => return IrValue::Bool(a == b),
                        BinaryOp::NotEq => return IrValue::Bool(a != b),
                        BinaryOp::Lt => return IrValue::Bool(a < b),
                        BinaryOp::LtEq => return IrValue::Bool(a <= b),
                        BinaryOp::Gt => return IrValue::Bool(a > b),
                        BinaryOp::GtEq => return IrValue::Bool(a >= b),
                        _ => {}
                    }
                }

                let temp = self.new_temp();
                instrs.push(IrInstruction::Assign {
                    target: temp.clone(),
                    value: IrExpr::Binary {
                        op: op.clone(),
                        left: l_val,
                        right: r_val,
                    },
                });
                IrValue::Var(temp)
            }
            ExprKind::Unary { op, expr } => {
                let val = self.lower_expr(expr, instrs);
                if let IrValue::Int(i) = val {
                    if *op == UnaryOp::Neg {
                        return IrValue::Int(-i);
                    }
                }
                if let IrValue::Bool(b) = val {
                    if *op == UnaryOp::Not {
                        return IrValue::Bool(!b);
                    }
                }
                let temp = self.new_temp();
                instrs.push(IrInstruction::Assign {
                    target: temp.clone(),
                    value: IrExpr::Unary { op: op.clone(), expr: val },
                });
                IrValue::Var(temp)
            }
            ExprKind::Call { callee, args } => {
                let arg_vals: Vec<IrValue> = args.iter().map(|a| self.lower_expr(a, instrs)).collect();
                let callee_name = match &callee.kind {
                    ExprKind::Ident(s) => s.clone(),
                    _ => "unknown".into(),
                };

                let temp = self.new_temp();
                instrs.push(IrInstruction::Call {
                    target: Some(temp.clone()),
                    callee: callee_name,
                    args: arg_vals,
                });
                IrValue::Var(temp)
            }
            ExprKind::List(items) => {
                let val_items: Vec<IrValue> = items.iter().map(|i| self.lower_expr(i, instrs)).collect();
                let temp = self.new_temp();
                instrs.push(IrInstruction::Assign {
                    target: temp.clone(),
                    value: IrExpr::List(val_items),
                });
                IrValue::Var(temp)
            }
            ExprKind::Map(entries) => {
                let val_entries: Vec<(IrValue, IrValue)> = entries
                    .iter()
                    .map(|(k, v)| (self.lower_expr(k, instrs), self.lower_expr(v, instrs)))
                    .collect();
                let temp = self.new_temp();
                instrs.push(IrInstruction::Assign {
                    target: temp.clone(),
                    value: IrExpr::Map(val_entries),
                });
                IrValue::Var(temp)
            }
            ExprKind::Index { target, index } => {
                let t_val = self.lower_expr(target, instrs);
                let i_val = self.lower_expr(index, instrs);
                let temp = self.new_temp();
                instrs.push(IrInstruction::Assign {
                    target: temp.clone(),
                    value: IrExpr::Index { target: t_val, index: i_val },
                });
                IrValue::Var(temp)
            }
            _ => IrValue::Nil,
        }
    }
}

pub struct Optimizer;

impl Optimizer {
    pub fn optimize_module(module: &mut IrModule) {
        for func in &mut module.functions {
            Self::optimize_function(func);
        }
    }

    fn optimize_function(func: &mut IrFunction) {
        // Dead-code elimination after return
        let mut cleaned = Vec::new();
        for inst in func.instructions.drain(..) {
            let is_ret = matches!(inst, IrInstruction::Return(_));
            cleaned.push(inst);
            if is_ret {
                break;
            }
        }
        func.instructions = cleaned;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constant_folding() {
        let mut builder = IrBuilder::new();
        let expr = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::Add,
                left: Box::new(Expr::new(ExprKind::Int(10), lingo_source::Span::default())),
                right: Box::new(Expr::new(ExprKind::Int(20), lingo_source::Span::default())),
            },
            lingo_source::Span::default(),
        );

        let mut instrs = Vec::new();
        let val = builder.lower_expr(&expr, &mut instrs);
        assert_eq!(val, IrValue::Int(30));
    }
}
