use facet::Facet;

#[repr(u8)]
#[derive(Debug, Clone, PartialEq, Facet)]
pub enum TokenKind {
    Let,
    If,
    Else,
    Match,
    Use,
    And,
    Or,
    Not,
    Ident(String),
    Directive(String),
    Int(i64),
    Real(f64),
    String(String),
    True,
    False,
    Hole,
    DoubleHole,
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Dot,
    Comma,
    Semi,
    Colon,
    ColonColon,
    Arrow,
    FatArrow,
    Pipe,
    EqEq,
    Assign,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Caret,
    Amp,
    EOF,
    Error(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Facet)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq, Facet)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, start: usize, end: usize) -> Self {
        Self {
            kind,
            span: Span { start, end },
        }
    }
}
