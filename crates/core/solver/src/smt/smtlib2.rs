use crate::{Quantifier, SmtExpr, SmtOp, SmtSolver, SmtSort, SolverError};

pub struct SmtLib2Solver {
    pub output: String,
}

impl SmtLib2Solver {
    pub fn new() -> Self {
        Self {
            output: String::new(),
        }
    }

    pub fn to_smtlib2(expr: &SmtExpr) -> String {
        match expr {
            SmtExpr::Var(v) => v.clone(),
            SmtExpr::Int(i) => i.to_string(),
            SmtExpr::Bool(b) => {
                if *b {
                    "true".to_string()
                } else {
                    "false".to_string()
                }
            }
            SmtExpr::App(op, args) => {
                let op_str = match op {
                    SmtOp::Eq => "=",
                    SmtOp::Gt => ">",
                    SmtOp::GtEq => ">=",
                    SmtOp::Lt => "<",
                    SmtOp::LtEq => "<=",
                    SmtOp::And => "and",
                    SmtOp::Or => "or",
                    SmtOp::Not => "not",
                    SmtOp::Implies => "=>",
                    SmtOp::Add => "+",
                    SmtOp::Sub => "-",
                    SmtOp::Mul => "*",
                    SmtOp::Div => "div",
                };
                let args_str: Vec<String> = args.iter().map(|a| Self::to_smtlib2(a)).collect();

                format!("({} {})", op_str, args_str.join(" "))
            }
            SmtExpr::Quantifier(q, vars, body) => {
                let q_str = match q {
                    Quantifier::Forall => "forall",
                    Quantifier::Exists => "exists",
                };
                let vars_str: Vec<String> = vars
                    .iter()
                    .map(|(v, s)| {
                        let sort_str = match s {
                            SmtSort::Int => "Int",
                            SmtSort::Bool => "Bool",
                        };

                        format!("({} {})", v, sort_str)
                    })
                    .collect();

                format!(
                    "({} ({}) {})",
                    q_str,
                    vars_str.join(" "),
                    Self::to_smtlib2(body)
                )
            }
        }
    }
}

impl SmtSolver for SmtLib2Solver {
    fn check_validity(&mut self, env: &[SmtExpr], vc: &SmtExpr) -> Result<bool, SolverError> {
        self.output.clear();
        for assertion in env {
            self.output
                .push_str(&format!("(assert {})\n", Self::to_smtlib2(assertion)));
        }
        let not_vc = SmtExpr::app(SmtOp::Not, vec![vc.clone()]);
        self.output
            .push_str(&format!("(assert {})\n", Self::to_smtlib2(&not_vc)));
        self.output.push_str("(check-sat)\n");

        Err(SolverError::Unsupported(
            "SmtLib2Solver only generates strings".into(),
        ))
    }

    fn check_sat(&mut self, assertions: &[SmtExpr]) -> Result<bool, SolverError> {
        self.output.clear();
        for assertion in assertions {
            self.output
                .push_str(&format!("(assert {})\n", Self::to_smtlib2(assertion)));
        }
        self.output.push_str("(check-sat)\n");

        Err(SolverError::Unsupported(
            "SmtLib2Solver only generates strings".into(),
        ))
    }
}
