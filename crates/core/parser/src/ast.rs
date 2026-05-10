use facet::Facet;

#[derive(Debug, Clone, PartialEq, Facet)]
#[repr(u8)]
pub enum BaseType {
    Int,
    Bool,
    Real,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Facet)]
#[repr(u8)]
pub enum Type {
    Base(BaseType),
    Refined {
        base: BaseType,
        v: String, // Value variable name, e.g., 'v' in { v: Int | v > 0 }
        predicate: Expr,
    },
}

#[derive(Debug, Clone, PartialEq, Facet)]
#[repr(u8)]
pub enum Expr {
    Binary {
        left: Box<Expr>,
        op: BinOp,
        right: Box<Expr>,
    },
    Unary {
        op: UnOp,
        expr: Box<Expr>,
    },
    Literal(Lit),
    Var(String),
    Tuple(Vec<Expr>),
    List(Vec<Expr>),
    App {
        func: Box<Expr>,
        args: Vec<Expr>,
    },
    Lambda {
        params: Vec<(String, Option<Type>)>,
        body: Box<Expr>,
    },
    Match {
        expr: Box<Expr>,
        arms: Vec<(Pattern, Expr)>,
    },
    Let {
        name: String,
        ty: Option<Box<Type>>,
        value: Box<Expr>,
        body: Box<Expr>,
    },
}

#[derive(Debug, Clone, PartialEq, Facet)]
#[repr(u8)]
pub enum BinOp {
    And,
    Or,
    Implies,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    Add,
    Sub,
    Mul,
    Div,
    Cons,
}

#[derive(Debug, Clone, PartialEq, Facet)]
#[repr(u8)]
pub enum UnOp {
    Not,
    Neg,
}

#[derive(Debug, Clone, PartialEq, Facet)]
#[repr(u8)]
pub enum Lit {
    Int(i64),
    Bool(bool),
    Real(f64),
}

#[derive(Debug, Clone, PartialEq, Facet)]
#[repr(u8)]
pub enum Pattern {
    Wild,
    Var(String),
    Literal(Lit),
    Tuple(Vec<Pattern>),
    List(Vec<Pattern>),
    Cons(Box<Pattern>, Box<Pattern>),
}

#[derive(Debug, Clone, PartialEq, Facet)]
pub struct FunctionContract {
    pub name: String,
    pub params: Vec<(String, Type)>,
    pub return_type: Type,
}

#[derive(Debug, Clone, PartialEq, Facet)]
pub struct TypeAlias {
    pub name: String,
    pub ty: Type,
}

#[derive(Debug, Clone, PartialEq, Facet)]
pub struct Assertion {
    pub predicate: Expr,
}

#[derive(Debug, Clone, PartialEq, Facet)]
#[repr(u8)]
pub enum SpecItem {
    FunctionContract(FunctionContract),
    TypeAlias(TypeAlias),
    Assertion(Assertion),
}
