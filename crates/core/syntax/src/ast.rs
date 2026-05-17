use facet::Facet;

#[repr(u8)]
#[derive(Debug, Clone, PartialEq, Facet)]
pub enum UnaryOp {
    Neg,
    Not,
}

#[repr(u8)]
#[derive(Debug, Clone, PartialEq, Facet)]
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
    Ascription,
}

#[repr(u8)]
#[derive(Debug, Clone, PartialEq, Facet)]
pub enum Expr {
    LiteralInt(i64),
    LiteralReal(f64),
    LiteralString(String),
    LiteralBool(bool),
    Var(String),
    Hole,
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },
    FieldAccess {
        base: Box<Expr>,
        field: String,
    },
    Block {
        statements: Vec<Statement>,
        tail: Option<Box<Expr>>,
    },
    If {
        condition: Box<Expr>,
        then_branch: Box<Expr>,
        else_branch: Option<Box<Expr>>,
    },
    Match {
        scrutinee: Box<Expr>,
        arms: Vec<MatchArm>,
    },
    Variant {
        name: String,
        args: Vec<Expr>,
    },
    Paren(Box<Expr>),
    Refinement {
        base: Box<Expr>,
        binder: Option<String>,
        predicate: Box<Expr>,
    },
    Ascription {
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Error(String),
}

#[repr(u8)]
#[derive(Debug, Clone, PartialEq, Facet)]
pub enum Statement {
    Let(LetDecl),
    Use(UseDecl),
    Expr(Expr),
}

#[derive(Debug, Clone, PartialEq, Facet)]
pub struct LetDecl {
    pub name: String,
    pub annotation: Option<Expr>,
    pub value: Option<Expr>,
}

#[derive(Debug, Clone, PartialEq, Facet)]
pub struct UseDecl {
    pub path: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Facet)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub body: Expr,
}

#[repr(u8)]
#[derive(Debug, Clone, PartialEq, Facet)]
pub enum Pattern {
    LiteralInt(i64),
    LiteralReal(f64),
    LiteralString(String),
    LiteralBool(bool),
    Ident(String),
    Variant { name: String, args: Vec<Pattern> },
    Wildcard,
    Error(String),
}

#[derive(Debug, Clone, PartialEq, Facet)]
pub struct Program {
    pub statements: Vec<Statement>,
}
