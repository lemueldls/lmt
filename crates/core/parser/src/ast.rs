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
pub struct FunctionContract {
    pub name: String,
    pub params: Vec<(String, Type)>,
    pub return_type: Type,
    pub pre_conditions: Vec<Expr>,
    pub post_conditions: Vec<Expr>,
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
