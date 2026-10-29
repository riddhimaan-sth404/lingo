pub mod token;

use lingo_diagnostics::Diagnostic;
use lingo_source::{SourceFile, Span};
pub use token::{Token, TokenKind};

pub struct Lexer<'a> {
    file: &'a SourceFile,
    chars: Vec<(usize, char)>,
    cursor: usize,
    indent_stack: Vec<usize>,
    pending_dedents: usize,
    at_line_start: bool,
    open_delimiters: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(file: &'a SourceFile) -> Self {
        let chars: Vec<(usize, char)> = file.src.char_indices().collect();
        Self {
            file,
            chars,
            cursor: 0,
            indent_stack: vec![0],
            pending_dedents: 0,
            at_line_start: true,
            open_delimiters: 0,
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.cursor).map(|(_, c)| *c)
    }

    fn peek_next(&self) -> Option<char> {
        self.chars.get(self.cursor + 1).map(|(_, c)| *c)
    }

    fn current_pos(&self) -> usize {
        self.chars.get(self.cursor).map(|(pos, _)| *pos).unwrap_or(self.file.src.len())
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.cursor += 1;
        Some(ch)
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, Diagnostic> {
        let mut tokens = Vec::new();

        while self.cursor < self.chars.len() || self.pending_dedents > 0 || self.indent_stack.len() > 1 {
            if self.pending_dedents > 0 {
                self.pending_dedents -= 1;
                tokens.push(Token::new(TokenKind::Dedent, Span::new(self.current_pos(), self.current_pos())));
                continue;
            }

            if self.cursor >= self.chars.len() {
                // End of file: emit dedents for any remaining indentation levels
                if self.indent_stack.len() > 1 {
                    self.indent_stack.pop();
                    tokens.push(Token::new(TokenKind::Dedent, Span::new(self.current_pos(), self.current_pos())));
                    continue;
                }
                break;
            }

            if self.at_line_start {
                let start_pos = self.current_pos();
                let mut spaces = 0;

                while let Some(ch) = self.peek() {
                    if ch == ' ' {
                        spaces += 1;
                        self.advance();
                    } else if ch == '\t' {
                        return Err(Diagnostic::error("Tabs are not allowed for indentation. Use spaces.", Span::new(self.current_pos(), self.current_pos() + 1)));
                    } else {
                        break;
                    }
                }

                // If blank line or comment line, ignore indentation
                if self.peek() == Some('\r') || self.peek() == Some('\n') || self.peek() == Some('#') {
                    self.skip_whitespace_and_comments();
                    continue;
                }

                self.at_line_start = false;

                if self.open_delimiters == 0 {
                    let current_indent = *self.indent_stack.last().unwrap();
                    if spaces > current_indent {
                        self.indent_stack.push(spaces);
                        tokens.push(Token::new(TokenKind::Indent, Span::new(start_pos, self.current_pos())));
                    } else if spaces < current_indent {
                        while let Some(&top) = self.indent_stack.last() {
                            if top == spaces {
                                break;
                            }
                            if top < spaces {
                                return Err(Diagnostic::error(
                                    format!("Unindent does not match any outer indentation level (spaces: {spaces})"),
                                    Span::new(start_pos, self.current_pos()),
                                ));
                            }
                            self.indent_stack.pop();
                            self.pending_dedents += 1;
                        }
                        while self.pending_dedents > 0 {
                            self.pending_dedents -= 1;
                            tokens.push(Token::new(TokenKind::Dedent, Span::new(start_pos, self.current_pos())));
                        }
                    }
                }
            }

            if let Some(tok) = self.next_token()? {
                tokens.push(tok);
            }
        }

        tokens.push(Token::new(TokenKind::Eof, Span::new(self.file.src.len(), self.file.src.len())));
        Ok(tokens)
    }

    fn skip_whitespace_and_comments(&mut self) {
        while let Some(ch) = self.peek() {
            if ch == ' ' || ch == '\t' || ch == '\r' {
                self.advance();
            } else if ch == '#' {
                while let Some(c) = self.peek() {
                    if c == '\n' {
                        break;
                    }
                    self.advance();
                }
            } else if ch == '\n' {
                self.advance();
                self.at_line_start = true;
                break;
            } else {
                break;
            }
        }
    }

    fn next_token(&mut self) -> Result<Option<Token>, Diagnostic> {
        let start = self.current_pos();
        let ch = match self.peek() {
            Some(c) => c,
            None => return Ok(None),
        };

        if ch == ' ' || ch == '\t' || ch == '\r' {
            self.advance();
            return Ok(None);
        }

        if ch == '#' {
            while let Some(c) = self.peek() {
                if c == '\n' {
                    break;
                }
                self.advance();
            }
            return Ok(None);
        }

        if ch == '\n' {
            self.advance();
            self.at_line_start = true;
            if self.open_delimiters == 0 {
                return Ok(Some(Token::new(TokenKind::Newline, Span::new(start, self.current_pos()))));
            } else {
                return Ok(None);
            }
        }

        // Operators & delimiters
        self.advance();
        let kind = match ch {
            '+' => TokenKind::Plus,
            '-' => {
                if self.peek() == Some('>') {
                    self.advance();
                    TokenKind::Arrow
                } else {
                    TokenKind::Minus
                }
            }
            '*' => TokenKind::Star,
            '/' => TokenKind::Slash,
            '%' => TokenKind::Percent,
            '=' => {
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::EqEq
                } else {
                    TokenKind::Eq
                }
            }
            '!' => {
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::BangEq
                } else {
                    return Err(Diagnostic::error("Unexpected character '!' (did you mean '!=')?", Span::new(start, start + 1)));
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
            '(' => {
                self.open_delimiters += 1;
                TokenKind::LParen
            }
            ')' => {
                self.open_delimiters = self.open_delimiters.saturating_sub(1);
                TokenKind::RParen
            }
            '[' => {
                self.open_delimiters += 1;
                TokenKind::LBracket
            }
            ']' => {
                self.open_delimiters = self.open_delimiters.saturating_sub(1);
                TokenKind::RBracket
            }
            '{' => {
                self.open_delimiters += 1;
                TokenKind::LBrace
            }
            '}' => {
                self.open_delimiters = self.open_delimiters.saturating_sub(1);
                TokenKind::RBrace
            }
            ':' => TokenKind::Colon,
            ',' => TokenKind::Comma,
            '"' => return self.lex_string(start).map(Some),
            c if c.is_ascii_digit() => return self.lex_number(start, c).map(Some),
            c if c.is_alphabetic() || c == '_' => return self.lex_ident_or_kw(start, c).map(Some),
            other => {
                return Err(Diagnostic::error(format!("Unexpected character '{other}'"), Span::new(start, start + 1)));
            }
        };

        Ok(Some(Token::new(kind, Span::new(start, self.current_pos()))))
    }

    fn lex_string(&mut self, start: usize) -> Result<Token, Diagnostic> {
        let mut s = String::new();
        while let Some(ch) = self.peek() {
            self.advance();
            if ch == '"' {
                return Ok(Token::new(TokenKind::Str(s), Span::new(start, self.current_pos())));
            } else if ch == '\\' {
                match self.advance() {
                    Some('n') => s.push('\n'),
                    Some('t') => s.push('\t'),
                    Some('r') => s.push('\r'),
                    Some('"') => s.push('"'),
                    Some('\\') => s.push('\\'),
                    Some(c) => s.push(c),
                    None => return Err(Diagnostic::error("Unterminated escape sequence in string", Span::new(start, self.current_pos()))),
                }
            } else {
                s.push(ch);
            }
        }

        Err(Diagnostic::error("Unterminated string literal", Span::new(start, self.current_pos())))
    }

    fn lex_number(&mut self, start: usize, first_digit: char) -> Result<Token, Diagnostic> {
        let mut num_str = first_digit.to_string();
        let mut is_float = false;

        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                num_str.push(ch);
                self.advance();
            } else if ch == '.' && !is_float && self.peek_next().map_or(false, |c| c.is_ascii_digit()) {
                is_float = true;
                num_str.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        let span = Span::new(start, self.current_pos());
        if is_float {
            let val = num_str.parse::<f64>().map_err(|e| Diagnostic::error(format!("Invalid float: {e}"), span))?;
            Ok(Token::new(TokenKind::Float(val), span))
        } else {
            let val = num_str.parse::<i64>().map_err(|e| Diagnostic::error(format!("Invalid integer: {e}"), span))?;
            Ok(Token::new(TokenKind::Int(val), span))
        }
    }

    fn lex_ident_or_kw(&mut self, start: usize, first: char) -> Result<Token, Diagnostic> {
        let mut ident = first.to_string();
        while let Some(ch) = self.peek() {
            if ch.is_alphanumeric() || ch == '_' {
                ident.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        let kind = match ident.as_str() {
            "def" => TokenKind::Def,
            "extern" => TokenKind::Extern,
            "let" => TokenKind::Let,
            "if" => TokenKind::If,
            "elif" => TokenKind::Elif,
            "else" => TokenKind::Else,
            "while" => TokenKind::While,
            "for" => TokenKind::For,
            "in" => TokenKind::In,
            "return" => TokenKind::Return,
            "and" => TokenKind::And,
            "or" => TokenKind::Or,
            "not" => TokenKind::Not,
            "true" | "True" => TokenKind::True,
            "false" | "False" => TokenKind::False,
            "nil" | "None" => TokenKind::Nil,
            "import" => TokenKind::Import,
            "as" => TokenKind::As,
            _ => TokenKind::Ident(ident),
        };

        Ok(Token::new(kind, Span::new(start, self.current_pos())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_indent_dedent_tokenization() {
        let src = "def main():\n    x = 42\n    return x\n";
        let file = SourceFile::new("test.ln", None, src.into());
        let mut lexer = Lexer::new(&file);
        let tokens = lexer.tokenize().expect("Tokenize successful");
        let kinds: Vec<TokenKind> = tokens.into_iter().map(|t| t.kind).collect();

        assert_eq!(kinds[0], TokenKind::Def);
        assert_eq!(kinds[1], TokenKind::Ident("main".into()));
        assert_eq!(kinds[2], TokenKind::LParen);
        assert_eq!(kinds[3], TokenKind::RParen);
        assert_eq!(kinds[4], TokenKind::Colon);
        assert_eq!(kinds[5], TokenKind::Newline);
        assert_eq!(kinds[6], TokenKind::Indent);
    }
}
