use facet::Facet;

#[repr(u8)]
#[derive(Facet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum SmtSort {
    Int,
    Bool,
    // Future: Arrays, BitVectors, Uninterpreted Sorts, etc.
}

#[repr(u8)]
#[derive(Facet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum Quantifier {
    Forall,
    Exists,
}

#[repr(u8)]
#[derive(Facet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum SmtOp {
    Eq,
    Gt,
    GtEq,
    Lt,
    LtEq,
    And,
    Or,
    Not,
    Implies,
    Add,
    Sub,
    Mul,
    Div,
}

#[repr(u8)]
#[derive(Facet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum SmtExpr {
    Var(String),
    Int(i64),
    Bool(bool),
    App(SmtOp, Vec<Self>),
    Quantifier(Quantifier, Vec<(String, SmtSort)>, Box<Self>),
}

impl SmtExpr {
    pub fn var(name: impl Into<String>) -> Self {
        Self::Var(name.into())
    }

    #[must_use]
    pub const fn app(op: SmtOp, args: Vec<Self>) -> Self {
        Self::App(op, args)
    }

    #[must_use]
    pub fn implies(lhs: Self, rhs: Self) -> Self {
        Self::app(SmtOp::Implies, vec![lhs, rhs])
    }

    #[must_use]
    pub fn substitute(&self, var: &str, replacement: &Self) -> Self {
        match self {
            Self::Var(v) => {
                if v == var {
                    replacement.clone()
                } else {
                    self.clone()
                }
            }
            Self::Int(i) => Self::Int(*i),
            Self::Bool(b) => Self::Bool(*b),
            Self::App(op, args) => {
                Self::App(
                    op.clone(),
                    args.iter()
                        .map(|a| a.substitute(var, replacement))
                        .collect(),
                )
            }
            Self::Quantifier(q, vars, body) => {
                if vars.iter().any(|(v, _)| v == var) {
                    self.clone()
                } else {
                    Self::Quantifier(
                        q.clone(),
                        vars.clone(),
                        Box::new(body.substitute(var, replacement)),
                    )
                }
            }
        }
    }
}
