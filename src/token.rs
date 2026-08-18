#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Keywords
    Def,
    Class,
    Protocol,
    Impl,
    If,
    Elif,
    Else,
    While,
    For,
    In,
    Return,
    Let,
    Mut,
    Import,
    From,
    As,
    And,
    Or,
    Not,
    True,
    False,
    None,

    // Identifiers & Literals
    Ident(String),
    Int(i64),
    Float(f64),
    Str(String),

    // Symbols & Operators
    Colon,
    Comma,
    Dot,
    Arrow, // ->
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Assign, // =
    Eq,     // ==
    NotEq,  // !=
    Lt,     // <
    Gt,     // >
    LtEq,   // <=
    GtEq,   // >=
    Amp,    // &

    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,

    // Layout / Whitespace
    Newline,
    Indent,
    Dedent,
    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub col: usize,
}
