use anyhow::{Result, anyhow};
use cvc5_rs::{Solver, TermManager};
use lmt_parser::ast::{Expr, FunctionContract, Type};

use crate::{expr_conv::expr_to_term, solver::SolverEnv};

pub fn check_contract_consistency(contract: &FunctionContract) -> Result<()> {
    let tm = TermManager::new();
    let mut solver = Solver::new(&tm);
    solver.set_logic("ALL");

    let mut env = SolverEnv::new();

    for (name, ty) in &contract.params {
        env.insert_var_from_type(&tm, name, ty);
        if let Type::Refined { v, predicate, .. } = ty {
            let pred = substitute_var(predicate, v, name);
            solver.assert_formula(expr_to_term(&tm, &pred, env.vars())?);
        }
    }

    // Introduce canonical return variable `v` for postconditions and refined return type.
    env.insert_var_from_type(&tm, "v", &contract.return_type);

    for pre in &contract.pre_conditions {
        solver.assert_formula(expr_to_term(&tm, pre, env.vars())?);
    }

    if let Type::Refined { v, predicate, .. } = &contract.return_type {
        let pred = substitute_var(predicate, v, "v");
        solver.assert_formula(expr_to_term(&tm, &pred, env.vars())?);
    }

    for post in &contract.post_conditions {
        solver.assert_formula(expr_to_term(&tm, post, env.vars())?);
    }

    let sat = solver.check_sat();
    if sat.is_sat() {
        Ok(())
    } else {
        Err(anyhow!(
            "contract `{}` has inconsistent refinements/pre/post conditions",
            contract.name
        ))
    }
}

fn substitute_var(expr: &Expr, from: &str, to: &str) -> Expr {
    match expr {
        Expr::Var(name) if name == from => Expr::Var(to.to_string()),
        Expr::Var(name) => Expr::Var(name.clone()),
        Expr::Literal(l) => Expr::Literal(l.clone()),
        Expr::Unary { op, expr } => {
            Expr::Unary {
                op: op.clone(),
                expr: Box::new(substitute_var(expr, from, to)),
            }
        }
        Expr::Binary { left, op, right } => {
            Expr::Binary {
                left: Box::new(substitute_var(left, from, to)),
                op: op.clone(),
                right: Box::new(substitute_var(right, from, to)),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use lmt_parser::parser::Parser;

    use super::*;

    #[test]
    fn test_check_contract_consistency_ok() {
        let input = "fn clamp(x: Int, min: Int, max: { v: Int | v >= min }) -> { v: Int | v >= min && v <= max } @pre min <= max @post v >= min";
        let mut parser = Parser::new(input);
        let contract = parser.parse_function_contract();
        check_contract_consistency(&contract).expect("expected consistent contract");
    }

    #[test]
    fn test_check_contract_consistency_inconsistent() {
        let input = "fn bad(x: Int) -> { v: Int | v > 0 } @post v < 0";
        let mut parser = Parser::new(input);
        let contract = parser.parse_function_contract();
        let result = check_contract_consistency(&contract);
        assert!(result.is_err());
    }
}
