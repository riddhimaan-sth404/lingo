use crate::token::{Token, TokenKind};

pub struct Lexer<'a> {
    _phantom: std::marker::PhantomData<&'a ()>,
    chars: Vec<char>,
    pos: usize,
    line: usize,
    col: usize,
    indent_stack: Vec<usize>,
    at_line_start: bool,
    paren_depth: usize,
    pending_tokens: Vec<Token>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            _phantom: std::marker::PhantomData,
            chars: input.chars().collect(),
            pos: 0,
            line: 1,
            col: 1,
            indent_stack: vec![0],
            at_line_start: true,
            paren_depth: 0,
            pending_tokens: Vec::new(),
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();

        while let Some(tok) = self.next_token()? {
            let is_eof = tok.kind == TokenKind::Eof;
            tokens.push(tok);
            if is_eof {
                break;
            }
        }

        Ok(tokens)
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn peek_next(&self) -> Option<char> {
        self.chars.get(self.pos + 1).copied()
    }

    fn advance(&mut self) -> Option<char> {
        if let Some(ch) = self.chars.get(self.pos).copied() {
            self.pos += 1;
            if ch == '\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
            Some(ch)
        } else {
            None
        }
    }

    fn next_token(&mut self) -> Result<Option<Token>, String> {
        if !self.pending_tokens.is_empty() {
            return Ok(Some(self.pending_tokens.remove(0)));
        }

        if self.at_line_start {
            self.at_line_start = false;

            // Handle indentation logic
            let mut indent_len = 0;
            while let Some(ch) = self.peek() {
                if ch == ' ' {
                    indent_len += 1;
                    self.advance();
                } else if ch == '\t' {
                    indent_len += 4;
                    self.advance();
                } else {
                    break;
                }
            }

            // If empty line or comment, skip
            if let Some(ch) = self.peek() {
                if ch == '\n' || ch == '\r' || ch == '#' {
                    self.skip_comment_or_newline();
                    self.at_line_start = true;
                    return self.next_token();
                }
            } else {
                // End of file reached
                return self.emit_final_dedents();
            }

            // Compare indent with stack
            let current_indent = *self.indent_stack.last().unwrap();
            if self.paren_depth == 0 {
                if indent_len > current_indent {
                    self.indent_stack.push(indent_len);
                    return Ok(Some(Token {
                        kind: TokenKind::Indent,
                        line: self.line,
                        col: self.col,
                    }));
                } else if indent_len < current_indent {
                    while *self.indent_stack.last().unwrap() > indent_len {
                        self.indent_stack.pop();
                        self.pending_tokens.push(Token {
                            kind: TokenKind::Dedent,
                            line: self.line,
                            col: self.col,
                        });
                    }

                    if *self.indent_stack.last().unwrap() != indent_len {
                        return Err(format!(
                            "IndentationError: unindent does not match any outer indentation level at line {}",
                            self.line
                        ));
                    }

                    return Ok(Some(self.pending_tokens.remove(0)));
                }
            }
        }

        self.skip_whitespace();

        let ch = match self.peek() {
            Some(c) => c,
            None => return self.emit_final_dedents(),
        };

        let start_line = self.line;
        let start_col = self.col;

        if ch == '#' {
            self.skip_comment();
            return self.next_token();
        }

        if ch == '\n' || ch == '\r' {
            if ch == '\r' && self.peek_next() == Some('\n') {
                self.advance();
            }
            self.advance();
            self.at_line_start = true;
            if self.paren_depth == 0 {
                return Ok(Some(Token {
                    kind: TokenKind::Newline,
                    line: start_line,
                    col: start_col,
                }));
            } else {
                return self.next_token();
            }
        }

        // Numbers
        if ch.is_ascii_digit() {
            return Ok(Some(self.read_number(start_line, start_col)));
        }

        // Strings
        if ch == '"' || ch == '\'' {
            return self.read_string(ch, start_line, start_col);
        }

        // Identifiers & Keywords
        if ch.is_alphabetic() || ch == '_' {
            return Ok(Some(self.read_ident_or_keyword(start_line, start_col)));
        }

        // Symbols & Operators
        self.advance();
        let kind = match ch {
            '(' => {
                self.paren_depth += 1;
                TokenKind::LParen
            }
            ')' => {
                if self.paren_depth > 0 {
                    self.paren_depth -= 1;
                }
                TokenKind::RParen
            }
            '[' => {
                self.paren_depth += 1;
                TokenKind::LBracket
            }
            ']' => {
                if self.paren_depth > 0 {
                    self.paren_depth -= 1;
                }
                TokenKind::RBracket
            }
            '{' => {
                self.paren_depth += 1;
                TokenKind::LBrace
            }
            '}' => {
                if self.paren_depth > 0 {
                    self.paren_depth -= 1;
                }
                TokenKind::RBrace
            }
            ':' => TokenKind::Colon,
            ',' => TokenKind::Comma,
            '.' => TokenKind::Dot,
            '+' => TokenKind::Plus,
            '*' => TokenKind::Star,
            '/' => TokenKind::Slash,
            '%' => TokenKind::Percent,
            '&' => TokenKind::Amp,
            '-' => {
                if self.peek() == Some('>') {
                    self.advance();
                    TokenKind::Arrow
                } else {
                    TokenKind::Minus
                }
            }
            '=' => {
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::Eq
                } else {
                    TokenKind::Assign
                }
            }
            '!' => {
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::NotEq
                } else {
                    return Err(format!("Unexpected character '!' at line {}", start_line));
                }
            }
            '<' => {
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::LtEq
                } else {
                    TokenKind::Lt
                }
            }
            '>' => {
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::GtEq
                } else {
                    TokenKind::Gt
                }
            }
            other => return Err(format!("Unexpected character '{}' at line {}", other, start_line)),
        };

        Ok(Some(Token {
            kind,
            line: start_line,
            col: start_col,
        }))
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch == ' ' || ch == '\t' {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn skip_comment(&mut self) {
        while let Some(ch) = self.peek() {
            if ch == '\n' || ch == '\r' {
                break;
            }
            self.advance();
        }
    }

    fn skip_comment_or_newline(&mut self) {
        while let Some(ch) = self.peek() {
            if ch == '\n' || ch == '\r' {
                if ch == '\r' && self.peek_next() == Some('\n') {
                    self.advance();
                }
                self.advance();
                break;
            } else if ch == '#' {
                self.skip_comment();
            } else {
                break;
            }
        }
    }

    fn read_number(&mut self, line: usize, col: usize) -> Token {
        let mut s = String::new();
        let mut is_float = false;

        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                s.push(ch);
                self.advance();
            } else if ch == '.' && !is_float && self.peek_next().map_or(false, |c| c.is_ascii_digit()) {
                is_float = true;
                s.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        if is_float {
            Token {
                kind: TokenKind::Float(s.parse().unwrap_or(0.0)),
                line,
                col,
            }
        } else {
            Token {
                kind: TokenKind::Int(s.parse().unwrap_or(0)),
                line,
                col,
            }
        }
    }

    fn read_string(&mut self, quote: char, line: usize, col: usize) -> Result<Option<Token>, String> {
        self.advance(); // consume quote
        let mut s = String::new();

        while let Some(ch) = self.peek() {
            if ch == quote {
                self.advance();
                return Ok(Some(Token {
                    kind: TokenKind::Str(s),
                    line,
                    col,
                }));
            } else if ch == '\\' {
                self.advance();
                match self.peek() {
                    Some('n') => {
                        s.push('\n');
                        self.advance();
                    }
                    Some('t') => {
                        s.push('\t');
                        self.advance();
                    }
                    Some('r') => {
                        s.push('\r');
                        self.advance();
                    }
                    Some('\\') => {
                        s.push('\\');
                        self.advance();
                    }
                    Some(c) if c == quote => {
                        s.push(quote);
                        self.advance();
                    }
                    Some(other) => {
                        s.push(other);
                        self.advance();
                    }
                    None => return Err(format!("Unterminated string literal at line {}", line)),
                }
            } else {
                s.push(ch);
                self.advance();
            }
        }

        Err(format!("Unterminated string literal at line {}", line))
    }

    fn read_ident_or_keyword(&mut self, line: usize, col: usize) -> Token {
        let mut s = String::new();
        while let Some(ch) = self.peek() {
            if ch.is_alphanumeric() || ch == '_' {
                s.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        let kind = match s.as_str() {
            "def" => TokenKind::Def,
            "class" => TokenKind::Class,
            "protocol" => TokenKind::Protocol,
            "impl" => TokenKind::Impl,
            "if" => TokenKind::If,
            "elif" => TokenKind::Elif,
            "else" => TokenKind::Else,
            "while" => TokenKind::While,
            "for" => TokenKind::For,
            "in" => TokenKind::In,
            "return" => TokenKind::Return,
            "let" => TokenKind::Let,
            "mut" => TokenKind::Mut,
            "import" => TokenKind::Import,
            "from" => TokenKind::From,
            "as" => TokenKind::As,
            "and" => TokenKind::And,
            "or" => TokenKind::Or,
            "not" => TokenKind::Not,
            "true" | "True" => TokenKind::True,
            "false" | "False" => TokenKind::False,
            "none" | "None" => TokenKind::None,
            _ => TokenKind::Ident(s),
        };

        Token { kind, line, col }
    }

    fn emit_final_dedents(&mut self) -> Result<Option<Token>, String> {
        if !self.pending_tokens.is_empty() {
            return Ok(Some(self.pending_tokens.remove(0)));
        }

        while self.indent_stack.len() > 1 {
            self.indent_stack.pop();
            self.pending_tokens.push(Token {
                kind: TokenKind::Dedent,
                line: self.line,
                col: self.col,
            });
        }

        if !self.pending_tokens.is_empty() {
            self.pending_tokens.push(Token {
                kind: TokenKind::Eof,
                line: self.line,
                col: self.col,
            });
            return Ok(Some(self.pending_tokens.remove(0)));
        }

        Ok(Some(Token {
            kind: TokenKind::Eof,
            line: self.line,
            col: self.col,
        }))
    }
}
