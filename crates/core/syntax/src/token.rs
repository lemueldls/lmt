use std::fmt;

use facet::Facet;
use lmt_diagnostics::{ModuleId, Span};

use crate::diagnostic::Diagnostic;

#[derive(Facet, Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, start: usize, end: usize, module_id: ModuleId) -> Self {
        Self {
            kind,
            span: Span::new(start, end, module_id),
        }
    }
}

#[repr(u8)]
#[derive(Facet, Debug, Clone, PartialEq)]
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
    Error(TokenError),
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            TokenKind::Let => write!(f, "let"),
            TokenKind::If => write!(f, "if"),
            TokenKind::Else => write!(f, "else"),
            TokenKind::Match => write!(f, "match"),
            TokenKind::Use => write!(f, "use"),
            TokenKind::And => write!(f, "and"),
            TokenKind::Or => write!(f, "or"),
            TokenKind::Not => write!(f, "not"),
            TokenKind::Ident(name) => write!(f, "{name}"),
            TokenKind::Directive(name) => write!(f, "@{name}"),
            TokenKind::Int(value) => write!(f, "{value}"),
            TokenKind::Real(value) => write!(f, "{value}"),
            TokenKind::String(value) => write!(f, "\"{value}\""),
            TokenKind::True => write!(f, "true"),
            TokenKind::False => write!(f, "false"),
            TokenKind::Hole => write!(f, "?"),
            TokenKind::DoubleHole => write!(f, "??"),
            TokenKind::LParen => write!(f, "("),
            TokenKind::RParen => write!(f, ")"),
            TokenKind::LBrace => write!(f, "{{"),
            TokenKind::RBrace => write!(f, "}}"),
            TokenKind::LBracket => write!(f, "["),
            TokenKind::RBracket => write!(f, "]"),
            TokenKind::Dot => write!(f, "."),
            TokenKind::Comma => write!(f, ","),
            TokenKind::Semi => write!(f, ";"),
            TokenKind::Colon => write!(f, ":"),
            TokenKind::ColonColon => write!(f, "::"),
            TokenKind::Arrow => write!(f, "->"),
            TokenKind::FatArrow => write!(f, "=>"),
            TokenKind::Pipe => write!(f, "|"),
            TokenKind::EqEq => write!(f, "=="),
            TokenKind::Assign => write!(f, "="),
            TokenKind::Ne => write!(f, "!="),
            TokenKind::Lt => write!(f, "<"),
            TokenKind::Le => write!(f, "<="),
            TokenKind::Gt => write!(f, ">"),
            TokenKind::Ge => write!(f, ">="),
            TokenKind::Plus => write!(f, "+"),
            TokenKind::Minus => write!(f, "-"),
            TokenKind::Star => write!(f, "*"),
            TokenKind::Slash => write!(f, "/"),
            TokenKind::Percent => write!(f, "%"),
            TokenKind::Caret => write!(f, "^"),
            TokenKind::Amp => write!(f, "&"),
            TokenKind::EOF => write!(f, "<EOF>"),
            TokenKind::Error(error) => write!(f, "<ERROR: {error:?}>"),
        }
    }
}

#[repr(u8)]
#[derive(Facet, Debug, Clone, PartialEq)]
pub enum TokenError {
    UnexpectedToken(Box<TokenKind>),
    UnknownToken,
    UnterminatedString,
    InvalidInteger,
    InvalidReal,
    InvalidDirective,
}

impl TokenError {
    pub fn diagnostic(&self, span: Span) -> Diagnostic {
        match self {
            TokenError::UnexpectedToken(kind) => {
                Diagnostic::UnexpectedToken {
                    kind: *kind.clone(),
                    span,
                }
            }
            TokenError::UnknownToken => Diagnostic::UnknownToken { span },
            TokenError::UnterminatedString => Diagnostic::UnterminatedString { span },
            TokenError::InvalidInteger => Diagnostic::InvalidInteger { span },
            TokenError::InvalidReal => Diagnostic::InvalidReal { span },
            TokenError::InvalidDirective => Diagnostic::InvalidDirective { span },
        }
    }
}
