use lingo_source::Span;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Layout
    Indent,
    Dedent,
    Newline,
    Eof,

    // Literals
    Int(i64),
    Float(f64),
    Str(String),
    Ident(String),

    // Keywords
    Def,
    Extern,
    Let,
    If,
    Elif,
    Else,
    While,
    For,
    In,
    Return,
    And,
    Or,
    Not,
    True,
    False,
    Nil,
    Import,
    As,

    // Operators & Delimiters
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Eq,
    EqEq,
    BangEq,
    Lt,
    LtEq,
    Gt,
    GtEq,
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    Colon,
    Comma,
    Arrow, // ->
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }
}
