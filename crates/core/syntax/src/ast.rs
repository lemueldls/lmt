use facet::Facet;
use lmt_diagnostics::Span;
use ordered_float::OrderedFloat;

#[repr(u8)]
#[derive(Facet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum UnaryOp {
    Neg,
    Not,
}

#[repr(u8)]
#[derive(Facet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum BinaryOp {
    FieldAccess,
    Power,
    Mul,
    Div,
    Mod,
    Add,
    Sub,
    Intersection,
    Lt,
    Le,
    Gt,
    Ge,
    Eq,
    Ne,
    Refinement,
    And,
    Or,
}

#[repr(u8)]
#[derive(Facet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum Expr {
    LiteralInt {
        value: i64,
        span: Span,
    },
    LiteralReal {
        value: OrderedFloat<f64>,
        span: Span,
    },
    LiteralString {
        value: String,
        span: Span,
    },
    LiteralBool {
        value: bool,
        span: Span,
    },
    Var {
        name: String,
        span: Span,
    },
    Hole {
        span: Span,
    },
    Unary {
        op: UnaryOp,
        expr: Box<Self>,
        span: Span,
    },
    Binary {
        left: Box<Self>,
        op: BinaryOp,
        right: Box<Self>,
        span: Span,
    },
    Call {
        callee: Box<Self>,
        args: Vec<Self>,
        span: Span,
    },
    FieldAccess {
        base: Box<Self>,
        field: String,
        span: Span,
    },
    Block {
        statements: Vec<Statement>,
        tail: Option<Box<Self>>,
        span: Span,
    },
    If {
        condition: Box<Self>,
        then_branch: Box<Self>,
        else_branch: Option<Box<Self>>,
        span: Span,
    },
    Match {
        scrutinee: Box<Self>,
        arms: Vec<MatchArm>,
        span: Span,
    },
    Variant {
        name: String,
        args: Vec<Self>,
        span: Span,
    },
    Paren {
        expr: Box<Self>,
        span: Span,
    },
    Refinement {
        base: Box<Self>,
        binder: Option<String>,
        predicate: Box<Self>,
        span: Span,
    },
    Ascription {
        left: Box<Self>,
        right: Box<Self>,
        span: Span,
    },
    Error {
        span: Span,
    },
}

impl Expr {
    #[must_use]
    pub const fn span(&self) -> Span {
        match self {
            Self::LiteralInt { span, .. }
            | Self::LiteralReal { span, .. }
            | Self::LiteralString { span, .. }
            | Self::LiteralBool { span, .. }
            | Self::Var { span, .. }
            | Self::Hole { span }
            | Self::Unary { span, .. }
            | Self::Binary { span, .. }
            | Self::Call { span, .. }
            | Self::FieldAccess { span, .. }
            | Self::Block { span, .. }
            | Self::If { span, .. }
            | Self::Match { span, .. }
            | Self::Variant { span, .. }
            | Self::Paren { span, .. }
            | Self::Refinement { span, .. }
            | Self::Ascription { span, .. }
            | Self::Error { span } => *span,
        }
    }
}

#[repr(u8)]
#[derive(Facet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum Statement {
    Let(LetDecl),
    Use(UseDecl),
    Expr(Expr),
}

#[derive(Facet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct LetDecl {
    pub name: String,
    pub annotation: Option<Expr>,
    pub value: Option<Expr>,
}

#[derive(Facet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct UseDecl {
    pub path: Vec<String>,
}

#[derive(Facet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub body: Expr,
}

#[repr(u8)]
#[derive(Facet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum Pattern {
    LiteralInt(i64),
    LiteralReal(OrderedFloat<f64>),
    LiteralString(String),
    LiteralBool(bool),
    Ident(String),
    Variant { name: String, args: Vec<Self> },
    Wildcard,
    Error,
}

#[derive(Facet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Program {
    pub statements: Vec<Statement>,
}
