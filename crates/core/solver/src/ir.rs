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
    App(SmtOp, Vec<SmtExpr>),
    Quantifier(Quantifier, Vec<(String, SmtSort)>, Box<SmtExpr>),
}

impl SmtExpr {
    pub fn var(name: impl Into<String>) -> Self {
        Self::Var(name.into())
    }

    pub fn app(op: SmtOp, args: Vec<SmtExpr>) -> Self {
        Self::App(op, args)
    }

    pub fn implies(lhs: SmtExpr, rhs: SmtExpr) -> Self {
        Self::app(SmtOp::Implies, vec![lhs, rhs])
    }

    pub fn substitute(&self, var: &str, replacement: &SmtExpr) -> Self {
        match self {
            SmtExpr::Var(v) => {
                if v == var {
                    replacement.clone()
                } else {
                    self.clone()
                }
            }
            SmtExpr::Int(i) => SmtExpr::Int(*i),
            SmtExpr::Bool(b) => SmtExpr::Bool(*b),
            SmtExpr::App(op, args) => {
                SmtExpr::App(
                    op.clone(),
                    args.iter()
                        .map(|a| a.substitute(var, replacement))
                        .collect(),
                )
            }
            SmtExpr::Quantifier(q, vars, body) => {
                if vars.iter().any(|(v, _)| v == var) {
                    self.clone()
                } else {
                    SmtExpr::Quantifier(
                        q.clone(),
                        vars.clone(),
                        Box::new(body.substitute(var, replacement)),
                    )
                }
            }
        }
    }
}
