use std::collections::HashMap;

use anyhow::{Result, anyhow};
use cvc5_rs::{Term, TermManager};
use lmt_parser::{
    SpecItem,
    ast::{Assertion, Expr, FunctionContract, Type, TypeAlias},
};

use crate::{
    expr_conv::expr_to_term,
    solver::{self, SolverEnv},
};

#[derive(Debug, Default)]
pub struct VerificationEnv {
    facts: Vec<Expr>,
    aliases: HashMap<String, TypeAlias>,
}

impl VerificationEnv {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn facts(&self) -> &[Expr] {
        &self.facts
    }

    pub fn add_fact(&mut self, fact: Expr) {
        self.facts.push(fact);
    }

    pub fn insert_alias(&mut self, alias: TypeAlias) {
        self.aliases.insert(alias.name.clone(), alias);
    }
}

pub fn check_spec_item(item: &SpecItem, env: &mut VerificationEnv) -> Result<()> {
    match item {
        SpecItem::TypeAlias(alias) => {
            check_type_alias_consistency(alias)?;
            env.insert_alias(alias.clone());
            Ok(())
        }
        SpecItem::FunctionContract(contract) => check_contract_consistency_with_env(contract, env),
        SpecItem::Assertion(assertion) => {
            check_assertion(assertion, env)?;
            env.add_fact(assertion.predicate.clone());
            Ok(())
        }
    }
}

pub fn check_contract_consistency(contract: &FunctionContract) -> Result<()> {
    let env = VerificationEnv::new();
    check_contract_consistency_with_env(contract, &env)
}

fn check_contract_consistency_with_env(
    contract: &FunctionContract,
    env: &VerificationEnv,
) -> Result<()> {
    let tm = TermManager::new();
    let mut solver_env = SolverEnv::new();
    let mut assumptions: Vec<Term> = Vec::new();

    for (name, ty) in &contract.params {
        solver_env.insert_var_from_type(&tm, name, ty);
        if let Type::Refined { v, predicate, .. } = ty {
            let pred = substitute_var(predicate, v, name);
            let pred_term = expr_to_term(&tm, &pred, solver_env.vars())?;
            assumptions.push(pred_term);
        }
    }

    solver_env.insert_var_from_type(&tm, "v", &contract.return_type);

    for fact in env.facts() {
        assumptions.push(expr_to_term(&tm, fact, solver_env.vars())?);
    }

    for pre in &contract.pre_conditions {
        assumptions.push(expr_to_term(&tm, pre, solver_env.vars())?);
    }

    if let Type::Refined { v, predicate, .. } = &contract.return_type {
        let pred = substitute_var(predicate, v, "v");
        assumptions.push(expr_to_term(&tm, &pred, solver_env.vars())?);
    }

    for post in &contract.post_conditions {
        assumptions.push(expr_to_term(&tm, post, solver_env.vars())?);
    }

    if solver::is_satisfiable(&tm, &assumptions)? {
        Ok(())
    } else {
        Err(anyhow!(
            "contract `{}` has inconsistent refinements/pre/post conditions",
            contract.name
        ))
    }
}

fn check_type_alias_consistency(alias: &TypeAlias) -> Result<()> {
    if let Type::Refined { v, predicate, .. } = &alias.ty {
        let tm = TermManager::new();
        let mut solver_env = SolverEnv::new();
        solver_env.insert_var_from_type(&tm, v, &alias.ty);
        let predicate_term = expr_to_term(&tm, predicate, solver_env.vars())?;
        if solver::is_satisfiable(&tm, &[predicate_term])? {
            Ok(())
        } else {
            Err(anyhow!("type alias `{}` is inconsistent", alias.name))
        }
    } else {
        Ok(())
    }
}

fn check_assertion(assertion: &Assertion, env: &VerificationEnv) -> Result<()> {
    let tm = TermManager::new();
    let solver_env = SolverEnv::new();
    let assumptions: Vec<Term> = env
        .facts()
        .iter()
        .map(|fact| expr_to_term(&tm, fact, solver_env.vars()))
        .collect::<Result<Vec<_>>>()?;
    let goal = expr_to_term(&tm, &assertion.predicate, solver_env.vars())?;

    if solver::proves(&tm, &assumptions, &goal)? {
        Ok(())
    } else {
        Err(anyhow!("assertion is not provable"))
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
    use lmt_parser::{SpecItem, parser::Parser};

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

    #[test]
    fn test_check_assertion_in_sequence() {
        let mut env = VerificationEnv::new();
        let mut parser = Parser::new("@assert 1 + 1 == 2");
        let item = SpecItem::Assertion(parser.parse_assertion());
        check_spec_item(&item, &mut env).expect("assertion should verify");
        assert_eq!(env.facts().len(), 1);
    }

    #[test]
    fn test_check_bad_assertion_fails() {
        let mut env = VerificationEnv::new();
        let mut parser = Parser::new("@assert 1 < 0");
        let item = SpecItem::Assertion(parser.parse_assertion());
        assert!(check_spec_item(&item, &mut env).is_err());
    }
}
