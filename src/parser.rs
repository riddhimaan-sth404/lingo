use crate::ast::*;
use crate::token::{Token, TokenKind};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    pub fn parse_program(&mut self) -> Result<Program, String> {
        let mut statements = Vec::new();
        self.skip_newlines();

        while !self.is_at_end() {
            if self.peek_kind() == Some(&TokenKind::Eof) {
                break;
            }
            statements.push(self.parse_statement()?);
            self.skip_newlines();
        }

        Ok(Program { statements })
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn peek_kind(&self) -> Option<&TokenKind> {
        self.peek().map(|t| &t.kind)
    }

    fn advance(&mut self) -> Option<&Token> {
        if self.pos < self.tokens.len() {
            let tok = &self.tokens[self.pos];
            self.pos += 1;
            Some(tok)
        } else {
            None
        }
    }

    fn is_at_end(&self) -> bool {
        self.peek_kind().map_or(true, |k| k == &TokenKind::Eof)
    }

    fn consume(&mut self, expected: TokenKind, err_msg: &str) -> Result<Token, String> {
        if let Some(tok) = self.peek() {
            if tok.kind == expected {
                return Ok(self.advance().unwrap().clone());
            }
        }
        Err(format!(
            "SyntaxError: Expected {:?}, got {:?} at position {}. {}",
            expected,
            self.peek_kind(),
            self.pos,
            err_msg
        ))
    }

    fn skip_newlines(&mut self) {
        while let Some(k) = self.peek_kind() {
            if k == &TokenKind::Newline {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn parse_statement(&mut self) -> Result<Stmt, String> {
        self.skip_newlines();

        match self.peek_kind() {
            Some(TokenKind::Import) | Some(TokenKind::From) => self.parse_import(),
            Some(TokenKind::Def) => self.parse_def(),
            Some(TokenKind::Class) => self.parse_class(),
            Some(TokenKind::Protocol) => self.parse_protocol(),
            Some(TokenKind::Impl) => self.parse_impl(),
            Some(TokenKind::Let) => self.parse_let(),
            Some(TokenKind::If) => self.parse_if(),
            Some(TokenKind::While) => self.parse_while(),
            Some(TokenKind::For) => self.parse_for(),
            Some(TokenKind::Return) => self.parse_return(),
            _ => self.parse_expr_or_assign_stmt(),
        }
    }

    fn parse_import(&mut self) -> Result<Stmt, String> {
        if self.peek_kind() == Some(&TokenKind::Import) {
            self.advance();
            let mut module = Vec::new();
            if let Some(TokenKind::Ident(name)) = self.peek_kind().cloned() {
                self.advance();
                module.push(name);
                while self.peek_kind() == Some(&TokenKind::Dot) {
                    self.advance();
                    if let Some(TokenKind::Ident(sub)) = self.peek_kind().cloned() {
                        self.advance();
                        module.push(sub);
                    }
                }
            } else {
                return Err("Expected identifier after import".into());
            }

            let alias = if self.peek_kind() == Some(&TokenKind::As) {
                self.advance();
                if let Some(TokenKind::Ident(alias_name)) = self.peek_kind().cloned() {
                    self.advance();
                    Some(alias_name)
                } else {
                    return Err("Expected alias after 'as'".into());
                }
            } else {
                None
            };

            Ok(Stmt::Import { module, alias })
        } else {
            // from package import item
            self.advance(); // consume 'from'
            let mut module = Vec::new();
            if let Some(TokenKind::Ident(name)) = self.peek_kind().cloned() {
                self.advance();
                module.push(name);
            }
            self.consume(TokenKind::Import, "Expected 'import' after 'from package'")?;
            if let Some(TokenKind::Ident(item)) = self.peek_kind().cloned() {
                self.advance();
                module.push(item);
            }

            Ok(Stmt::Import {
                module,
                alias: None,
            })
        }
    }

    fn parse_def(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume def
        let name = match self.peek_kind() {
            Some(TokenKind::Ident(n)) => {
                let n = n.clone();
                self.advance();
                n
            }
            _ => return Err("Expected function name after 'def'".into()),
        };

        self.consume(TokenKind::LParen, "Expected '(' after function name")?;
        let params = self.parse_parameters()?;
        self.consume(TokenKind::RParen, "Expected ')' after parameters")?;

        let return_type = if self.peek_kind() == Some(&TokenKind::Arrow) {
            self.advance();
            self.parse_type_annotation()?
        } else {
            TypeAnnotation::Dynamic
        };

        self.consume(TokenKind::Colon, "Expected ':' after function signature")?;
        let body = self.parse_block()?;

        Ok(Stmt::Def {
            name,
            params,
            return_type,
            body,
        })
    }

    fn parse_parameters(&mut self) -> Result<Vec<Parameter>, String> {
        let mut params = Vec::new();
        while self.peek_kind() != Some(&TokenKind::RParen) && !self.is_at_end() {
            let is_self = if let Some(TokenKind::Ident(s)) = self.peek_kind() {
                s == "self"
            } else {
                false
            };

            let name = match self.peek_kind() {
                Some(TokenKind::Ident(n)) => {
                    let n = n.clone();
                    self.advance();
                    n
                }
                _ => return Err("Expected parameter name".into()),
            };

            let ty = if self.peek_kind() == Some(&TokenKind::Colon) {
                self.advance();
                self.parse_type_annotation()?
            } else {
                TypeAnnotation::Dynamic // Dynamic typing by default when omitted!
            };

            params.push(Parameter { name, ty, is_self });

            if self.peek_kind() == Some(&TokenKind::Comma) {
                self.advance();
            } else {
                break;
            }
        }
        Ok(params)
    }

    fn parse_type_annotation(&mut self) -> Result<TypeAnnotation, String> {
        if self.peek_kind() == Some(&TokenKind::Amp) {
            self.advance();
            let is_mut = if self.peek_kind() == Some(&TokenKind::Mut) {
                self.advance();
                true
            } else {
                false
            };
            let inner = self.parse_type_annotation()?;
            Ok(TypeAnnotation::Reference {
                mutable: is_mut,
                inner: Box::new(inner),
            })
        } else if let Some(TokenKind::Ident(name)) = self.peek_kind().cloned() {
            self.advance();
            if name == "dynamic" || name == "Any" {
                Ok(TypeAnnotation::Dynamic)
            } else if self.peek_kind() == Some(&TokenKind::Lt) {
                self.advance(); // consume <
                let mut type_args = Vec::new();
                while self.peek_kind() != Some(&TokenKind::Gt) && !self.is_at_end() {
                    let sub_ty = self.parse_type_annotation()?;
                    type_args.push(sub_ty);
                    if self.peek_kind() == Some(&TokenKind::Comma) {
                        self.advance();
                    } else {
                        break;
                    }
                }
                self.consume(TokenKind::Gt, "Expected '>' after generic type parameters")?;
                
                // Form composite named type string (e.g. HashMap<String, i32>)
                let args_str: Vec<String> = type_args.iter().map(|t| match t {
                    TypeAnnotation::Named(n) => n.clone(),
                    TypeAnnotation::Dynamic => "Value".to_string(),
                    _ => "Value".to_string(),
                }).collect();
                Ok(TypeAnnotation::Named(format!("{}<{}>", name, args_str.join(", "))))
            } else {
                Ok(TypeAnnotation::Named(name))
            }
        } else {
            Ok(TypeAnnotation::Dynamic)
        }
    }

    fn parse_class(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume class
        let name = match self.peek_kind() {
            Some(TokenKind::Ident(n)) => {
                let n = n.clone();
                self.advance();
                n
            }
            _ => return Err("Expected class name".into()),
        };

        self.consume(TokenKind::Colon, "Expected ':' after class name")?;
        self.skip_newlines();

        let mut fields = Vec::new();

        if self.peek_kind() == Some(&TokenKind::Indent) {
            self.advance(); // consume Indent
            self.skip_newlines();

            while self.peek_kind() != Some(&TokenKind::Dedent) && !self.is_at_end() {
                if let Some(TokenKind::Ident(fname)) = self.peek_kind().cloned() {
                    self.advance();
                    let ty = if self.peek_kind() == Some(&TokenKind::Colon) {
                        self.advance();
                        self.parse_type_annotation()?
                    } else {
                        TypeAnnotation::Dynamic
                    };
                    fields.push((fname, ty));
                }
                self.skip_newlines();
            }

            if self.peek_kind() == Some(&TokenKind::Dedent) {
                self.advance();
            }
        }

        Ok(Stmt::Class { name, fields })
    }

    fn parse_protocol(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume protocol
        let name = match self.peek_kind() {
            Some(TokenKind::Ident(n)) => {
                let n = n.clone();
                self.advance();
                n
            }
            _ => return Err("Expected protocol name".into()),
        };

        self.consume(TokenKind::Colon, "Expected ':' after protocol name")?;
        self.skip_newlines();

        let mut methods = Vec::new();

        if self.peek_kind() == Some(&TokenKind::Indent) {
            self.advance();
            self.skip_newlines();

            while self.peek_kind() != Some(&TokenKind::Dedent) && !self.is_at_end() {
                if self.peek_kind() == Some(&TokenKind::Def) {
                    self.advance();
                    if let Some(TokenKind::Ident(mname)) = self.peek_kind().cloned() {
                        self.advance();
                        self.consume(TokenKind::LParen, "Expected '('")?;
                        let params = self.parse_parameters()?;
                        self.consume(TokenKind::RParen, "Expected ')'")?;
                        let ret = if self.peek_kind() == Some(&TokenKind::Arrow) {
                            self.advance();
                            self.parse_type_annotation()?
                        } else {
                            TypeAnnotation::Dynamic
                        };
                        methods.push((mname, params, ret));
                    }
                }
                self.skip_newlines();
            }

            if self.peek_kind() == Some(&TokenKind::Dedent) {
                self.advance();
            }
        }

        Ok(Stmt::Protocol { name, methods })
    }

    fn parse_impl(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume impl
        let first_name = match self.peek_kind() {
            Some(TokenKind::Ident(n)) => {
                let n = n.clone();
                self.advance();
                n
            }
            _ => return Err("Expected trait or struct name after 'impl'".into()),
        };

        let (target, protocol) = if self.peek_kind() == Some(&TokenKind::For) {
            self.advance(); // consume for
            if let Some(TokenKind::Ident(target_name)) = self.peek_kind().cloned() {
                self.advance();
                (target_name, Some(first_name))
            } else {
                return Err("Expected struct name after 'for'".into());
            }
        } else {
            (first_name, None)
        };

        self.consume(TokenKind::Colon, "Expected ':' after impl declaration")?;
        let methods = self.parse_block()?;

        Ok(Stmt::Impl {
            target,
            protocol,
            methods,
        })
    }

    fn parse_let(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume let
        let is_mut = if self.peek_kind() == Some(&TokenKind::Mut) {
            self.advance();
            true
        } else {
            false
        };

        let name = match self.peek_kind() {
            Some(TokenKind::Ident(n)) => {
                let n = n.clone();
                self.advance();
                n
            }
            _ => return Err("Expected variable name after 'let'".into()),
        };

        let ty = if self.peek_kind() == Some(&TokenKind::Colon) {
            self.advance();
            self.parse_type_annotation()?
        } else {
            TypeAnnotation::Dynamic
        };

        self.consume(TokenKind::Assign, "Expected '=' in variable declaration")?;
        let init = self.parse_expr()?;

        Ok(Stmt::Let {
            name,
            is_mut,
            ty,
            init,
        })
    }

    fn parse_if(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume if
        let condition = self.parse_expr()?;
        self.consume(TokenKind::Colon, "Expected ':' after if condition")?;
        let then_branch = self.parse_block()?;

        let mut elif_branches = Vec::new();
        while self.peek_kind() == Some(&TokenKind::Elif) {
            self.advance();
            let elif_cond = self.parse_expr()?;
            self.consume(TokenKind::Colon, "Expected ':' after elif condition")?;
            let elif_body = self.parse_block()?;
            elif_branches.push((elif_cond, elif_body));
        }

        let else_branch = if self.peek_kind() == Some(&TokenKind::Else) {
            self.advance();
            self.consume(TokenKind::Colon, "Expected ':' after else")?;
            Some(self.parse_block()?)
        } else {
            None
        };

        Ok(Stmt::If {
            condition,
            then_branch,
            elif_branches,
            else_branch,
        })
    }

    fn parse_while(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume while
        let condition = self.parse_expr()?;
        self.consume(TokenKind::Colon, "Expected ':' after while condition")?;
        let body = self.parse_block()?;

        Ok(Stmt::While { condition, body })
    }

    fn parse_for(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume for
        let var = match self.peek_kind() {
            Some(TokenKind::Ident(n)) => {
                let n = n.clone();
                self.advance();
                n
            }
            _ => return Err("Expected loop variable name after 'for'".into()),
        };

        self.consume(TokenKind::In, "Expected 'in' after loop variable")?;
        let iterable = self.parse_expr()?;
        self.consume(TokenKind::Colon, "Expected ':' after for loop header")?;
        let body = self.parse_block()?;

        Ok(Stmt::For {
            var,
            iterable,
            body,
        })
    }

    fn parse_return(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume return
        if self.peek_kind() == Some(&TokenKind::Newline) || self.peek_kind() == Some(&TokenKind::Dedent) {
            Ok(Stmt::Return(None))
        } else {
            let expr = self.parse_expr()?;
            Ok(Stmt::Return(Some(expr)))
        }
    }

    fn parse_expr_or_assign_stmt(&mut self) -> Result<Stmt, String> {
        let expr = self.parse_expr()?;

        if self.peek_kind() == Some(&TokenKind::Assign) {
            self.advance();
            let value = self.parse_expr()?;
            Ok(Stmt::Assignment {
                target: expr,
                value,
            })
        } else {
            Ok(Stmt::ExprStmt(expr))
        }
    }

    fn parse_block(&mut self) -> Result<Vec<Stmt>, String> {
        self.skip_newlines();

        if self.peek_kind() == Some(&TokenKind::Indent) {
            self.advance(); // consume Indent
            let mut stmts = Vec::new();
            self.skip_newlines();

            while self.peek_kind() != Some(&TokenKind::Dedent) && !self.is_at_end() {
                stmts.push(self.parse_statement()?);
                self.skip_newlines();
            }

            if self.peek_kind() == Some(&TokenKind::Dedent) {
                self.advance();
            }

            Ok(stmts)
        } else {
            // Inline single line block
            let stmt = self.parse_statement()?;
            Ok(vec![stmt])
        }
    }

    // Expressions
    pub fn parse_expr(&mut self) -> Result<Expr, String> {
        self.parse_logical_or()
    }

    fn parse_logical_or(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_logical_and()?;
        while self.peek_kind() == Some(&TokenKind::Or) {
            self.advance();
            let right = self.parse_logical_and()?;
            left = Expr::Binary {
                left: Box::new(left),
                op: BinaryOp::Or,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_logical_and(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_equality()?;
        while self.peek_kind() == Some(&TokenKind::And) {
            self.advance();
            let right = self.parse_equality()?;
            left = Expr::Binary {
                left: Box::new(left),
                op: BinaryOp::And,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_equality(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_comparison()?;
        while let Some(k) = self.peek_kind() {
            let op = match k {
                TokenKind::Eq => BinaryOp::Eq,
                TokenKind::NotEq => BinaryOp::NotEq,
                _ => break,
            };
            self.advance();
            let right = self.parse_comparison()?;
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_term()?;
        while let Some(k) = self.peek_kind() {
            let op = match k {
                TokenKind::Lt => BinaryOp::Lt,
                TokenKind::Gt => BinaryOp::Gt,
                TokenKind::LtEq => BinaryOp::LtEq,
                TokenKind::GtEq => BinaryOp::GtEq,
                _ => break,
            };
            self.advance();
            let right = self.parse_term()?;
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_term(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_factor()?;
        while let Some(k) = self.peek_kind() {
            let op = match k {
                TokenKind::Plus => BinaryOp::Add,
                TokenKind::Minus => BinaryOp::Sub,
                _ => break,
            };
            self.advance();
            let right = self.parse_factor()?;
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_factor(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_unary()?;
        while let Some(k) = self.peek_kind() {
            let op = match k {
                TokenKind::Star => BinaryOp::Mul,
                TokenKind::Slash => BinaryOp::Div,
                TokenKind::Percent => BinaryOp::Mod,
                _ => break,
            };
            self.advance();
            let right = self.parse_unary()?;
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr, String> {
        if self.peek_kind() == Some(&TokenKind::Not) {
            self.advance();
            let expr = self.parse_unary()?;
            Ok(Expr::Unary {
                op: UnaryOp::Not,
                expr: Box::new(expr),
            })
        } else if self.peek_kind() == Some(&TokenKind::Minus) {
            self.advance();
            let expr = self.parse_unary()?;
            Ok(Expr::Unary {
                op: UnaryOp::Neg,
                expr: Box::new(expr),
            })
        } else if self.peek_kind() == Some(&TokenKind::Amp) {
            self.advance();
            let is_mut = if self.peek_kind() == Some(&TokenKind::Mut) {
                self.advance();
                true
            } else {
                false
            };
            let expr = self.parse_unary()?;
            Ok(Expr::Borrow {
                mutable: is_mut,
                expr: Box::new(expr),
            })
        } else {
            self.parse_postfix()
        }
    }

    fn parse_postfix(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_primary()?;

        loop {
            match self.peek_kind() {
                Some(TokenKind::LParen) => {
                    self.advance();
                    let mut args = Vec::new();
                    while self.peek_kind() != Some(&TokenKind::RParen) && !self.is_at_end() {
                        args.push(self.parse_expr()?);
                        if self.peek_kind() == Some(&TokenKind::Comma) {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    self.consume(TokenKind::RParen, "Expected ')' after arguments")?;
                    expr = Expr::Call {
                        callee: Box::new(expr),
                        args,
                    };
                }
                Some(TokenKind::Dot) => {
                    self.advance();
                    if let Some(TokenKind::Ident(field)) = self.peek_kind().cloned() {
                        self.advance();
                        expr = Expr::FieldAccess {
                            object: Box::new(expr),
                            field,
                        };
                    } else {
                        return Err("Expected field or method name after '.'".into());
                    }
                }
                Some(TokenKind::LBracket) => {
                    self.advance();
                    let index = self.parse_expr()?;
                    self.consume(TokenKind::RBracket, "Expected ']' after index")?;
                    expr = Expr::IndexAccess {
                        object: Box::new(expr),
                        index: Box::new(index),
                    };
                }
                Some(TokenKind::LBrace) => {
                    if let Expr::Var(ref struct_name) = expr {
                        let name = struct_name.clone();
                        self.advance(); // consume {
                        let mut fields = Vec::new();
                        while self.peek_kind() != Some(&TokenKind::RBrace) && !self.is_at_end() {
                            if let Some(TokenKind::Ident(fname)) = self.peek_kind().cloned() {
                                self.advance();
                                if self.peek_kind() == Some(&TokenKind::Colon) || self.peek_kind() == Some(&TokenKind::Assign) {
                                    self.advance();
                                }
                                let val = self.parse_expr()?;
                                fields.push((fname, val));
                                if self.peek_kind() == Some(&TokenKind::Comma) {
                                    self.advance();
                                } else {
                                    break;
                                }
                            } else {
                                break;
                            }
                        }
                        self.consume(TokenKind::RBrace, "Expected '}' after struct fields")?;
                        expr = Expr::StructInit { name, fields };
                    } else {
                        break;
                    }
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        let tok = match self.peek() {
            Some(t) => t.clone(),
            None => return Err("Unexpected end of input".into()),
        };

        match tok.kind {
            TokenKind::Int(i) => {
                self.advance();
                Ok(Expr::Literal(LiteralValue::Int(i)))
            }
            TokenKind::Float(f) => {
                self.advance();
                Ok(Expr::Literal(LiteralValue::Float(f)))
            }
            TokenKind::Str(s) => {
                self.advance();
                Ok(Expr::Literal(LiteralValue::Str(s)))
            }
            TokenKind::True => {
                self.advance();
                Ok(Expr::Literal(LiteralValue::Bool(true)))
            }
            TokenKind::False => {
                self.advance();
                Ok(Expr::Literal(LiteralValue::Bool(false)))
            }
            TokenKind::None => {
                self.advance();
                Ok(Expr::Literal(LiteralValue::Nil))
            }
            TokenKind::Ident(ref name) => {
                let name = name.clone();
                self.advance();

                // Check for struct init: Name(field=val, ...) or Name { ... }
                Ok(Expr::Var(name))
            }
            TokenKind::LParen => {
                self.advance();
                let expr = self.parse_expr()?;
                self.consume(TokenKind::RParen, "Expected ')'")?;
                Ok(expr)
            }
            TokenKind::LBracket => {
                self.advance();
                let mut elements = Vec::new();
                while self.peek_kind() != Some(&TokenKind::RBracket) && !self.is_at_end() {
                    elements.push(self.parse_expr()?);
                    if self.peek_kind() == Some(&TokenKind::Comma) {
                        self.advance();
                    } else {
                        break;
                    }
                }
                self.consume(TokenKind::RBracket, "Expected ']' after list elements")?;
                Ok(Expr::List(elements))
            }
            _ => Err(format!("Unexpected token {:?} in expression", tok.kind)),
        }
    }
}
