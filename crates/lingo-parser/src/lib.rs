use lingo_ast::*;
use lingo_diagnostics::Diagnostic;
use lingo_lexer::{Token, TokenKind};
use lingo_source::Span;

pub struct Parser {
    tokens: Vec<Token>,
    cursor: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, cursor: 0 }
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.cursor).unwrap_or_else(|| self.tokens.last().unwrap())
    }

    fn peek_kind(&self) -> &TokenKind {
        &self.peek().kind
    }

    fn advance(&mut self) -> Token {
        if self.cursor < self.tokens.len() {
            let tok = self.tokens[self.cursor].clone();
            self.cursor += 1;
            tok
        } else {
            self.tokens.last().cloned().unwrap()
        }
    }

    fn expect(&mut self, kind: TokenKind) -> Result<Token, Diagnostic> {
        if self.peek_kind() == &kind {
            Ok(self.advance())
        } else {
            let tok = self.peek();
            Err(Diagnostic::error(
                format!("Expected {:?}, found {:?}", kind, tok.kind),
                tok.span,
            ))
        }
    }

    fn match_token(&mut self, kind: TokenKind) -> bool {
        if self.peek_kind() == &kind {
            self.advance();
            true
        } else {
            false
        }
    }

    fn skip_newlines(&mut self) {
        while self.peek_kind() == &TokenKind::Newline {
            self.advance();
        }
    }

    pub fn parse_program(&mut self) -> Result<Program, Diagnostic> {
        let mut stmts = Vec::new();
        self.skip_newlines();

        while self.peek_kind() != &TokenKind::Eof {
            stmts.push(self.parse_stmt()?);
            self.skip_newlines();
        }

        let span = if let (Some(first), Some(last)) = (stmts.first(), stmts.last()) {
            first.span.merge(last.span)
        } else {
            Span::default()
        };

        Ok(Program { stmts, span })
    }

    fn parse_stmt(&mut self) -> Result<Stmt, Diagnostic> {
        self.skip_newlines();
        match self.peek_kind() {
            TokenKind::Extern => self.parse_extern_function(),
            TokenKind::Def => self.parse_function_def(),
            TokenKind::Let => self.parse_let_stmt(),
            TokenKind::If => self.parse_if_stmt(),
            TokenKind::While => self.parse_while_stmt(),
            TokenKind::For => self.parse_for_stmt(),
            TokenKind::Return => self.parse_return_stmt(),
            TokenKind::Import => self.parse_import_stmt(),
            _ => self.parse_expr_or_assign_stmt(),
        }
    }

    fn parse_block(&mut self) -> Result<Vec<Stmt>, Diagnostic> {
        self.expect(TokenKind::Colon)?;
        self.expect(TokenKind::Newline)?;
        self.skip_newlines();
        self.expect(TokenKind::Indent)?;

        let mut stmts = Vec::new();
        while self.peek_kind() != &TokenKind::Dedent && self.peek_kind() != &TokenKind::Eof {
            stmts.push(self.parse_stmt()?);
            self.skip_newlines();
        }

        self.expect(TokenKind::Dedent)?;
        Ok(stmts)
    }

    fn parse_extern_function(&mut self) -> Result<Stmt, Diagnostic> {
        let start = self.expect(TokenKind::Extern)?.span;
        self.expect(TokenKind::Def)?;
        let name_tok = self.advance();
        let name = match name_tok.kind {
            TokenKind::Ident(s) => s,
            _ => return Err(Diagnostic::error("Expected function name after 'extern def'", name_tok.span)),
        };

        self.expect(TokenKind::LParen)?;
        let mut params = Vec::new();
        while self.peek_kind() != &TokenKind::RParen && self.peek_kind() != &TokenKind::Eof {
            let p_tok = self.advance();
            let p_name = match p_tok.kind {
                TokenKind::Ident(s) => s,
                _ => return Err(Diagnostic::error("Expected parameter name", p_tok.span)),
            };

            let ty = if self.match_token(TokenKind::Colon) {
                self.parse_type_annotation()?
            } else {
                TypeAnnotation::Dynamic
            };

            params.push(Parameter {
                name: p_name,
                ty,
                span: p_tok.span,
            });

            if !self.match_token(TokenKind::Comma) {
                break;
            }
        }
        self.expect(TokenKind::RParen)?;

        let ret_ty = if self.match_token(TokenKind::Arrow) {
            self.parse_type_annotation()?
        } else {
            TypeAnnotation::Dynamic
        };

        self.match_token(TokenKind::Newline);
        let span = start.merge(name_tok.span);

        Ok(Stmt::new(
            StmtKind::ExternFunction {
                name,
                params,
                ret_ty,
            },
            span,
        ))
    }

    fn parse_function_def(&mut self) -> Result<Stmt, Diagnostic> {
        let start = self.expect(TokenKind::Def)?.span;
        let name_tok = self.advance();
        let name = match name_tok.kind {
            TokenKind::Ident(s) => s,
            _ => return Err(Diagnostic::error("Expected function name", name_tok.span)),
        };

        self.expect(TokenKind::LParen)?;
        let mut params = Vec::new();
        while self.peek_kind() != &TokenKind::RParen && self.peek_kind() != &TokenKind::Eof {
            let p_tok = self.advance();
            let p_name = match p_tok.kind {
                TokenKind::Ident(s) => s,
                _ => return Err(Diagnostic::error("Expected parameter name", p_tok.span)),
            };

            let ty = if self.match_token(TokenKind::Colon) {
                self.parse_type_annotation()?
            } else {
                TypeAnnotation::Dynamic
            };

            params.push(Parameter {
                name: p_name,
                ty,
                span: p_tok.span,
            });

            if !self.match_token(TokenKind::Comma) {
                break;
            }
        }
        self.expect(TokenKind::RParen)?;

        let ret_ty = if self.match_token(TokenKind::Arrow) {
            self.parse_type_annotation()?
        } else {
            TypeAnnotation::Dynamic
        };

        let body = self.parse_block()?;
        let end = body.last().map(|s| s.span).unwrap_or(start);

        Ok(Stmt::new(
            StmtKind::Function {
                name,
                params,
                ret_ty,
                body,
            },
            start.merge(end),
        ))
    }

    fn parse_type_annotation(&mut self) -> Result<TypeAnnotation, Diagnostic> {
        let tok = self.advance();
        match tok.kind {
            TokenKind::Ident(ref s) if s == "int" || s == "i64" || s == "i32" => Ok(TypeAnnotation::Int),
            TokenKind::Ident(ref s) if s == "float" || s == "f64" => Ok(TypeAnnotation::Float),
            TokenKind::Ident(ref s) if s == "bool" => Ok(TypeAnnotation::Bool),
            TokenKind::Ident(ref s) if s == "str" || s == "String" => Ok(TypeAnnotation::Str),
            TokenKind::Ident(s) => Ok(TypeAnnotation::Custom(s)),
            _ => Err(Diagnostic::error("Expected type name", tok.span)),
        }
    }

    fn parse_let_stmt(&mut self) -> Result<Stmt, Diagnostic> {
        let start = self.expect(TokenKind::Let)?.span;
        let name_tok = self.advance();
        let name = match name_tok.kind {
            TokenKind::Ident(s) => s,
            _ => return Err(Diagnostic::error("Expected variable name after 'let'", name_tok.span)),
        };

        let ty = if self.match_token(TokenKind::Colon) {
            self.parse_type_annotation()?
        } else {
            TypeAnnotation::Dynamic
        };

        let init = if self.match_token(TokenKind::Eq) {
            Some(self.parse_expr(0)?)
        } else {
            None
        };

        self.match_token(TokenKind::Newline);
        let end = init.as_ref().map(|e| e.span).unwrap_or(start);

        Ok(Stmt::new(
            StmtKind::Let { name, ty, init },
            start.merge(end),
        ))
    }

    fn parse_if_stmt(&mut self) -> Result<Stmt, Diagnostic> {
        let start = self.expect(TokenKind::If)?.span;
        let condition = self.parse_expr(0)?;
        let then_branch = self.parse_block()?;

        let mut elif_branches = Vec::new();
        while self.peek_kind() == &TokenKind::Elif {
            self.advance();
            let elif_cond = self.parse_expr(0)?;
            let elif_body = self.parse_block()?;
            elif_branches.push((elif_cond, elif_body));
        }

        let else_branch = if self.peek_kind() == &TokenKind::Else {
            self.advance();
            Some(self.parse_block()?)
        } else {
            None
        };

        let end = else_branch
            .as_ref()
            .and_then(|b| b.last())
            .map(|s| s.span)
            .unwrap_or(start);

        Ok(Stmt::new(
            StmtKind::If {
                condition,
                then_branch,
                elif_branches,
                else_branch,
            },
            start.merge(end),
        ))
    }

    fn parse_while_stmt(&mut self) -> Result<Stmt, Diagnostic> {
        let start = self.expect(TokenKind::While)?.span;
        let condition = self.parse_expr(0)?;
        let body = self.parse_block()?;
        let end = body.last().map(|s| s.span).unwrap_or(start);

        Ok(Stmt::new(StmtKind::While { condition, body }, start.merge(end)))
    }

    fn parse_for_stmt(&mut self) -> Result<Stmt, Diagnostic> {
        let start = self.expect(TokenKind::For)?.span;
        let var_tok = self.advance();
        let var = match var_tok.kind {
            TokenKind::Ident(s) => s,
            _ => return Err(Diagnostic::error("Expected loop variable identifier", var_tok.span)),
        };

        self.expect(TokenKind::In)?;
        let iter = self.parse_expr(0)?;
        let body = self.parse_block()?;
        let end = body.last().map(|s| s.span).unwrap_or(start);

        Ok(Stmt::new(StmtKind::For { var, iter, body }, start.merge(end)))
    }

    fn parse_return_stmt(&mut self) -> Result<Stmt, Diagnostic> {
        let start = self.expect(TokenKind::Return)?.span;
        let expr = if self.peek_kind() != &TokenKind::Newline && self.peek_kind() != &TokenKind::Eof && self.peek_kind() != &TokenKind::Dedent {
            Some(self.parse_expr(0)?)
        } else {
            None
        };

        self.match_token(TokenKind::Newline);
        let end = expr.as_ref().map(|e| e.span).unwrap_or(start);
        Ok(Stmt::new(StmtKind::Return(expr), start.merge(end)))
    }

    fn parse_import_stmt(&mut self) -> Result<Stmt, Diagnostic> {
        let start = self.expect(TokenKind::Import)?.span;
        let mod_tok = self.advance();
        let module = match mod_tok.kind {
            TokenKind::Ident(s) => s,
            _ => return Err(Diagnostic::error("Expected module name after 'import'", mod_tok.span)),
        };

        let alias = if self.match_token(TokenKind::As) {
            let a_tok = self.advance();
            match a_tok.kind {
                TokenKind::Ident(s) => Some(s),
                _ => return Err(Diagnostic::error("Expected alias name after 'as'", a_tok.span)),
            }
        } else {
            None
        };

        self.match_token(TokenKind::Newline);
        Ok(Stmt::new(StmtKind::Import { module, alias }, start.merge(mod_tok.span)))
    }

    fn parse_expr_or_assign_stmt(&mut self) -> Result<Stmt, Diagnostic> {
        let expr = self.parse_expr(0)?;

        if self.match_token(TokenKind::Eq) {
            let value = self.parse_expr(0)?;
            self.match_token(TokenKind::Newline);
            let span = expr.span.merge(value.span);
            Ok(Stmt::new(StmtKind::Assign { target: expr, value }, span))
        } else {
            self.match_token(TokenKind::Newline);
            let span = expr.span;
            Ok(Stmt::new(StmtKind::Expr(expr), span))
        }
    }

    fn parse_expr(&mut self, min_bp: u8) -> Result<Expr, Diagnostic> {
        let mut lhs = self.parse_primary()?;

        loop {
            // Postfix operators: Call `()`, Index `[]`
            if self.peek_kind() == &TokenKind::LParen {
                self.advance();
                let mut args = Vec::new();
                while self.peek_kind() != &TokenKind::RParen && self.peek_kind() != &TokenKind::Eof {
                    args.push(self.parse_expr(0)?);
                    if !self.match_token(TokenKind::Comma) {
                        break;
                    }
                }
                let end = self.expect(TokenKind::RParen)?.span;
                let span = lhs.span.merge(end);
                lhs = Expr::new(ExprKind::Call { callee: Box::new(lhs), args }, span);
                continue;
            }

            if self.peek_kind() == &TokenKind::LBracket {
                self.advance();
                let index = self.parse_expr(0)?;
                let end = self.expect(TokenKind::RBracket)?.span;
                let span = lhs.span.merge(end);
                lhs = Expr::new(ExprKind::Index { target: Box::new(lhs), index: Box::new(index) }, span);
                continue;
            }

            let (op, l_bp, r_bp) = match self.peek_kind() {
                TokenKind::Or => (BinaryOp::Or, 1, 2),
                TokenKind::And => (BinaryOp::And, 3, 4),
                TokenKind::EqEq => (BinaryOp::Eq, 5, 6),
                TokenKind::BangEq => (BinaryOp::NotEq, 5, 6),
                TokenKind::Lt => (BinaryOp::Lt, 7, 8),
                TokenKind::LtEq => (BinaryOp::LtEq, 7, 8),
                TokenKind::Gt => (BinaryOp::Gt, 7, 8),
                TokenKind::GtEq => (BinaryOp::GtEq, 7, 8),
                TokenKind::Plus => (BinaryOp::Add, 9, 10),
                TokenKind::Minus => (BinaryOp::Sub, 9, 10),
                TokenKind::Star => (BinaryOp::Mul, 11, 12),
                TokenKind::Slash => (BinaryOp::Div, 11, 12),
                TokenKind::Percent => (BinaryOp::Mod, 11, 12),
                _ => break,
            };

            if l_bp < min_bp {
                break;
            }

            self.advance();
            let rhs = self.parse_expr(r_bp)?;
            let span = lhs.span.merge(rhs.span);
            lhs = Expr::new(
                ExprKind::Binary {
                    op,
                    left: Box::new(lhs),
                    right: Box::new(rhs),
                },
                span,
            );
        }

        Ok(lhs)
    }

    fn parse_primary(&mut self) -> Result<Expr, Diagnostic> {
        let tok = self.advance();
        match tok.kind {
            TokenKind::Int(i) => Ok(Expr::new(ExprKind::Int(i), tok.span)),
            TokenKind::Float(f) => Ok(Expr::new(ExprKind::Float(f), tok.span)),
            TokenKind::Str(s) => Ok(Expr::new(ExprKind::Str(s), tok.span)),
            TokenKind::True => Ok(Expr::new(ExprKind::Bool(true), tok.span)),
            TokenKind::False => Ok(Expr::new(ExprKind::Bool(false), tok.span)),
            TokenKind::Nil => Ok(Expr::new(ExprKind::Nil, tok.span)),
            TokenKind::Ident(s) => Ok(Expr::new(ExprKind::Ident(s), tok.span)),
            TokenKind::Minus => {
                let inner = self.parse_expr(13)?;
                let span = tok.span.merge(inner.span);
                Ok(Expr::new(ExprKind::Unary { op: UnaryOp::Neg, expr: Box::new(inner) }, span))
            }
            TokenKind::Not => {
                let inner = self.parse_expr(13)?;
                let span = tok.span.merge(inner.span);
                Ok(Expr::new(ExprKind::Unary { op: UnaryOp::Not, expr: Box::new(inner) }, span))
            }
            TokenKind::LParen => {
                let inner = self.parse_expr(0)?;
                let end = self.expect(TokenKind::RParen)?.span;
                Ok(Expr::new(inner.kind, tok.span.merge(end)))
            }
            TokenKind::LBracket => {
                let mut items = Vec::new();
                while self.peek_kind() != &TokenKind::RBracket && self.peek_kind() != &TokenKind::Eof {
                    items.push(self.parse_expr(0)?);
                    if !self.match_token(TokenKind::Comma) {
                        break;
                    }
                }
                let end = self.expect(TokenKind::RBracket)?.span;
                Ok(Expr::new(ExprKind::List(items), tok.span.merge(end)))
            }
            TokenKind::LBrace => {
                let mut entries = Vec::new();
                while self.peek_kind() != &TokenKind::RBrace && self.peek_kind() != &TokenKind::Eof {
                    let key = self.parse_expr(0)?;
                    self.expect(TokenKind::Colon)?;
                    let val = self.parse_expr(0)?;
                    entries.push((key, val));
                    if !self.match_token(TokenKind::Comma) {
                        break;
                    }
                }
                let end = self.expect(TokenKind::RBrace)?.span;
                Ok(Expr::new(ExprKind::Map(entries), tok.span.merge(end)))
            }
            _ => Err(Diagnostic::error(format!("Unexpected token in expression: {:?}", tok.kind), tok.span)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lingo_lexer::Lexer;
    use lingo_source::SourceFile;

    #[test]
    fn test_parse_simple_program() {
        let src = "def main():\n    x = 10 + 20 * 2\n    return x\n";
        let file = SourceFile::new("test.ln", None, src.into());
        let mut lexer = Lexer::new(&file);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let program = parser.parse_program().unwrap();

        assert_eq!(program.stmts.len(), 1);
        if let StmtKind::Function { name, body, .. } = &program.stmts[0].kind {
            assert_eq!(name, "main");
            assert_eq!(body.len(), 2);
        } else {
            panic!("Expected function");
        }
    }
}
