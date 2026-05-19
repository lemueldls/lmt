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
        expr: Box<Expr>,
        span: Span,
    },
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
        span: Span,
    },
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
        span: Span,
    },
    FieldAccess {
        base: Box<Expr>,
        field: String,
        span: Span,
    },
    Block {
        statements: Vec<Statement>,
        tail: Option<Box<Expr>>,
        span: Span,
    },
    If {
        condition: Box<Expr>,
        then_branch: Box<Expr>,
        else_branch: Option<Box<Expr>>,
        span: Span,
    },
    Match {
        scrutinee: Box<Expr>,
        arms: Vec<MatchArm>,
        span: Span,
    },
    Variant {
        name: String,
        args: Vec<Expr>,
        span: Span,
    },
    Paren {
        expr: Box<Expr>,
        span: Span,
    },
    Refinement {
        base: Box<Expr>,
        binder: Option<String>,
        predicate: Box<Expr>,
        span: Span,
    },
    Ascription {
        left: Box<Expr>,
        right: Box<Expr>,
        span: Span,
    },
    Error {
        span: Span,
    },
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::LiteralInt { span, .. }
            | Expr::LiteralReal { span, .. }
            | Expr::LiteralString { span, .. }
            | Expr::LiteralBool { span, .. }
            | Expr::Var { span, .. }
            | Expr::Hole { span }
            | Expr::Unary { span, .. }
            | Expr::Binary { span, .. }
            | Expr::Call { span, .. }
            | Expr::FieldAccess { span, .. }
            | Expr::Block { span, .. }
            | Expr::If { span, .. }
            | Expr::Match { span, .. }
            | Expr::Variant { span, .. }
            | Expr::Paren { span, .. }
            | Expr::Refinement { span, .. }
            | Expr::Ascription { span, .. }
            | Expr::Error { span } => *span,
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
    Variant { name: String, args: Vec<Pattern> },
    Wildcard,
    Error,
}

#[derive(Facet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Program {
    pub statements: Vec<Statement>,
}
