use std::collections::HashMap;

use anyhow::{Result, anyhow};
use cvc5_rs::{Kind, Term, TermManager};
use lmt_parser::ast::{BinOp, Expr, Lit, Pattern, UnOp};

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
                BinOp::Cons => {
                    return Err(anyhow!(
                        "list cons operator is not supported in SMT conversion"
                    ));
                }
            };
            Ok(tm.mk_term(kind, &[l, r]))
        }
        Expr::Tuple(_)
        | Expr::List(_)
        | Expr::App { .. }
        | Expr::Lambda { .. }
        | Expr::Match { .. } => convert_unsupported(tm, expr, vars),
        Expr::Let {
            name,
            ty: _,
            value,
            body,
        } => {
            let val_term = expr_to_term(tm, value, vars)?;
            let mut new_vars = vars.clone();
            new_vars.insert(name.clone(), val_term);
            expr_to_term(tm, body, &new_vars)
        }
    }
}

fn convert_unsupported(
    tm: &TermManager,
    expr: &Expr,
    vars: &HashMap<String, Term>,
) -> Result<Term> {
    match expr {
        Expr::Match { expr, arms } => convert_match(tm, expr, arms, vars),
        _ => {
            Err(anyhow!(
                "expression form not supported for SMT conversion yet"
            ))
        }
    }
}

fn convert_match(
    tm: &TermManager,
    scrutinee: &Expr,
    arms: &[(Pattern, Expr)],
    vars: &HashMap<String, Term>,
) -> Result<Term> {
    let scrutinee_term = expr_to_term(tm, scrutinee, vars)?;
    let catch_all_index = arms
        .iter()
        .position(|(pattern, _)| matches!(pattern, Pattern::Wild | Pattern::Var(_)))
        .ok_or_else(|| {
            anyhow!("non-exhaustive match expressions are not supported in SMT conversion")
        })?;

    let mut result = expr_to_term(
        tm,
        &arms[catch_all_index].1,
        &extend_bindings(vars, &arms[catch_all_index].0, &scrutinee_term)?,
    )?;

    for (pattern, arm_expr) in arms[..catch_all_index].iter().rev() {
        let (guard, arm_vars) = pattern_guard_and_bindings(tm, pattern, &scrutinee_term, vars)?;
        let arm_term = expr_to_term(tm, arm_expr, &arm_vars)?;
        result = tm.mk_term(Kind::CVC5_KIND_ITE, &[guard, arm_term, result]);
    }

    Ok(result)
}

fn pattern_guard_and_bindings(
    tm: &TermManager,
    pattern: &Pattern,
    scrutinee: &Term,
    vars: &HashMap<String, Term>,
) -> Result<(Term, HashMap<String, Term>)> {
    let mut bindings = vars.clone();
    let guard = match pattern {
        Pattern::Wild => tm.mk_true(),
        Pattern::Var(name) => {
            bindings.insert(name.clone(), scrutinee.clone());
            tm.mk_true()
        }
        Pattern::Literal(lit) => {
            let lit_term = match lit {
                Lit::Int(n) => tm.mk_integer(*n),
                Lit::Bool(true) => tm.mk_true(),
                Lit::Bool(false) => tm.mk_false(),
                Lit::Real(f) => tm.mk_real_from_str(&f.to_string()),
            };
            tm.mk_term(Kind::CVC5_KIND_EQUAL, &[scrutinee.clone(), lit_term])
        }
        Pattern::Tuple(_) | Pattern::List(_) | Pattern::Cons(..) => {
            return Err(anyhow!(
                "structural patterns are not supported in SMT conversion yet"
            ));
        }
    };

    Ok((guard, bindings))
}

fn extend_bindings(
    vars: &HashMap<String, Term>,
    pattern: &Pattern,
    scrutinee: &Term,
) -> Result<HashMap<String, Term>> {
    let mut bindings = vars.clone();
    if let Pattern::Var(name) = pattern {
        bindings.insert(name.clone(), scrutinee.clone());
    }
    Ok(bindings)
}

#[cfg(test)]
mod tests {
    use cvc5_rs::{Kind, Solver};
    use lmt_parser::{
        ast::{BinOp, Expr, Lit},
        parser::Parser,
    };

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

    #[test]
    fn test_expr_to_term_match_literal_and_catch_all() {
        let tm = TermManager::new();
        let mut parser = Parser::new("match x with | 0 => 1 | y => y + 2");
        let expr = parser.parse_expr();

        let x = tm.mk_const(tm.integer_sort(), "x");
        let mut vars = HashMap::new();
        vars.insert("x".to_string(), x.clone());

        let term = expr_to_term(&tm, &expr, &vars).expect("match conversion failed");

        let mut solver = Solver::new(&tm);
        solver.set_logic("QF_LIA");

        solver.assert_formula(tm.mk_term(Kind::CVC5_KIND_EQUAL, &[x.clone(), tm.mk_integer(0)]));
        solver.assert_formula(tm.mk_term(Kind::CVC5_KIND_EQUAL, &[term.clone(), tm.mk_integer(1)]));
        assert!(solver.check_sat().is_sat());

        let mut solver = Solver::new(&tm);
        solver.set_logic("QF_LIA");
        solver.assert_formula(tm.mk_term(Kind::CVC5_KIND_EQUAL, &[x, tm.mk_integer(3)]));
        solver.assert_formula(tm.mk_term(Kind::CVC5_KIND_EQUAL, &[term, tm.mk_integer(5)]));
        assert!(solver.check_sat().is_sat());
    }
}
