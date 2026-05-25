use std::fmt;

use facet::Facet;
use lmt_diagnostics::{ModuleId, Span};
use ordered_float::OrderedFloat;

use crate::diagnostic::Diagnostic;

#[derive(Facet, Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    #[must_use]
    pub const fn new(kind: TokenKind, start: usize, end: usize, module_id: ModuleId) -> Self {
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
    Real(OrderedFloat<f64>),
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
            Self::Let => write!(f, "let"),
            Self::If => write!(f, "if"),
            Self::Else => write!(f, "else"),
            Self::Match => write!(f, "match"),
            Self::Use => write!(f, "use"),
            Self::And => write!(f, "and"),
            Self::Or => write!(f, "or"),
            Self::Not => write!(f, "not"),
            Self::Ident(name) => write!(f, "{name}"),
            Self::Directive(name) => write!(f, "@{name}"),
            Self::Int(value) => write!(f, "{value}"),
            Self::Real(value) => write!(f, "{value}"),
            Self::String(value) => write!(f, "\"{value}\""),
            Self::True => write!(f, "true"),
            Self::False => write!(f, "false"),
            Self::Hole => write!(f, "?"),
            Self::DoubleHole => write!(f, "??"),
            Self::LParen => write!(f, "("),
            Self::RParen => write!(f, ")"),
            Self::LBrace => write!(f, "{{"),
            Self::RBrace => write!(f, "}}"),
            Self::LBracket => write!(f, "["),
            Self::RBracket => write!(f, "]"),
            Self::Dot => write!(f, "."),
            Self::Comma => write!(f, ","),
            Self::Semi => write!(f, ";"),
            Self::Colon => write!(f, ":"),
            Self::ColonColon => write!(f, "::"),
            Self::Arrow => write!(f, "->"),
            Self::FatArrow => write!(f, "=>"),
            Self::Pipe => write!(f, "|"),
            Self::EqEq => write!(f, "=="),
            Self::Assign => write!(f, "="),
            Self::Ne => write!(f, "!="),
            Self::Lt => write!(f, "<"),
            Self::Le => write!(f, "<="),
            Self::Gt => write!(f, ">"),
            Self::Ge => write!(f, ">="),
            Self::Plus => write!(f, "+"),
            Self::Minus => write!(f, "-"),
            Self::Star => write!(f, "*"),
            Self::Slash => write!(f, "/"),
            Self::Percent => write!(f, "%"),
            Self::Caret => write!(f, "^"),
            Self::Amp => write!(f, "&"),
            Self::EOF => write!(f, "<EOF>"),
            Self::Error(error) => write!(f, "<ERROR: {error:?}>"),
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
    #[must_use]
    pub fn diagnostic(&self, span: Span) -> Diagnostic {
        match self {
            Self::UnexpectedToken(kind) => {
                Diagnostic::UnexpectedToken {
                    kind: *kind.clone(),
                    token: span,
                }
            }
            Self::UnknownToken => Diagnostic::UnknownToken { token: span },
            Self::UnterminatedString => Diagnostic::UnterminatedString { string: span },
            Self::InvalidInteger => Diagnostic::InvalidInteger { integer: span },
            Self::InvalidReal => Diagnostic::InvalidReal { real: span },
            Self::InvalidDirective => Diagnostic::InvalidDirective { directive: span },
        }
    }
}
