use std::collections::HashMap;

use anyhow::{Result, anyhow};
use cvc5_rs::{Kind, Term, TermManager};
use lmt_parser::ast::{BinOp, Expr, Lit, UnOp};

pub fn expr_to_term(tm: &TermManager, expr: &Expr, vars: &HashMap<String, Term>) -> Result<Term> {
    match expr {
        Expr::Literal(l) => {
            Ok(match l {
                Lit::Int(n) => tm.mk_integer(*n),
                Lit::Bool(true) => tm.mk_true(),
                Lit::Bool(false) => tm.mk_false(),
                Lit::Real(f) => tm.mk_real_from_str(&f.to_string()),
            })
        }
        Expr::Var(name) => {
            vars.get(name)
                .cloned()
                .ok_or_else(|| anyhow!("unknown variable in expression: {}", name))
        }
        Expr::Unary { op, expr } => {
            let inner = expr_to_term(tm, expr, vars)?;
            match op {
                UnOp::Not => Ok(tm.mk_term(Kind::CVC5_KIND_NOT, &[inner])),
                UnOp::Neg => Ok(tm.mk_term(Kind::CVC5_KIND_SUB, &[tm.mk_integer(0), inner])),
            }
        }
        Expr::Binary { left, op, right } => {
            let l = expr_to_term(tm, left, vars)?;
            let r = expr_to_term(tm, right, vars)?;
            let kind = match op {
                BinOp::And => Kind::CVC5_KIND_AND,
                BinOp::Or => Kind::CVC5_KIND_OR,
                BinOp::Implies => Kind::CVC5_KIND_IMPLIES,
                BinOp::Eq => Kind::CVC5_KIND_EQUAL,
                BinOp::Ne => Kind::CVC5_KIND_DISTINCT,
                BinOp::Lt => Kind::CVC5_KIND_LT,
                BinOp::Le => Kind::CVC5_KIND_LEQ,
                BinOp::Gt => Kind::CVC5_KIND_GT,
                BinOp::Ge => Kind::CVC5_KIND_GEQ,
                BinOp::Add => Kind::CVC5_KIND_ADD,
                BinOp::Sub => Kind::CVC5_KIND_SUB,
                BinOp::Mul => Kind::CVC5_KIND_MULT,
                BinOp::Div => return Err(anyhow!("division is not implemented yet")),
            };
            Ok(tm.mk_term(kind, &[l, r]))
        }
    }
}

#[cfg(test)]
mod tests {
    use cvc5_rs::Solver;
    use lmt_parser::ast::{BinOp, Expr, Lit};

    use super::*;

    #[test]
    fn test_expr_to_term_sat() {
        let tm = TermManager::new();
        let mut solver = Solver::new(&tm);
        solver.set_logic("QF_LIA");

        let x = tm.mk_const(tm.integer_sort(), "x");
        let mut vars = HashMap::new();
        vars.insert("x".to_string(), x);

        let expr = Expr::Binary {
            left: Box::new(Expr::Var("x".to_string())),
            op: BinOp::Gt,
            right: Box::new(Expr::Literal(Lit::Int(10))),
        };

        let term = expr_to_term(&tm, &expr, &vars).expect("expr conversion failed");
        solver.assert_formula(term);
        assert!(solver.check_sat().is_sat());
    }
}
