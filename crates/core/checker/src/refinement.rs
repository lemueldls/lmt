use std::collections::{HashMap, HashSet};

use anyhow::{Result, anyhow};
use cvc5_rs::{Term, TermManager};
use lmt_parser::{
    SpecItem,
    ast::{Assertion, BaseType, Expr, FunctionContract, Pattern, Type, TypeAlias},
};

use crate::{
    expr_conv::expr_to_term,
    solver::{self, SolverEnv},
};

#[derive(Debug, Clone)]
struct ExpandedType {
    base: BaseType,
    predicates: Vec<Expr>,
}

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

    pub fn aliases(&self) -> &HashMap<String, TypeAlias> {
        &self.aliases
    }
}

pub fn check_spec_item(item: &SpecItem, env: &mut VerificationEnv) -> Result<()> {
    match item {
        SpecItem::TypeAlias(alias) => {
            check_type_alias_consistency(alias, env)?;
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
        let expanded = expand_type(ty, env.aliases(), name)?;
        solver_env.insert_var_from_base(&tm, name, &expanded.base);
        for predicate in expanded.predicates {
            assumptions.push(expr_to_term(&tm, &predicate, solver_env.vars())?);
        }
    }

    let expanded_return = expand_type(&contract.return_type, env.aliases(), "v")?;
    solver_env.insert_var_from_base(&tm, "v", &expanded_return.base);

    for fact in env.facts() {
        assumptions.push(expr_to_term(&tm, fact, solver_env.vars())?);
    }

    for predicate in expanded_return.predicates {
        assumptions.push(expr_to_term(&tm, &predicate, solver_env.vars())?);
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

fn check_type_alias_consistency(alias: &TypeAlias, env: &VerificationEnv) -> Result<()> {
    let value_name = "it";
    let expanded = expand_type(&alias.ty, env.aliases(), value_name)?;

    if expanded.predicates.is_empty() {
        Ok(())
    } else {
        let tm = TermManager::new();
        let mut solver_env = SolverEnv::new();
        solver_env.insert_var_from_base(&tm, value_name, &expanded.base);

        let assumptions = expanded
            .predicates
            .iter()
            .map(|predicate| expr_to_term(&tm, predicate, solver_env.vars()))
            .collect::<Result<Vec<_>>>()?;

        if solver::is_satisfiable(&tm, &assumptions)? {
            Ok(())
        } else {
            Err(anyhow!("type alias `{}` is inconsistent", alias.name))
        }
    }
}

pub fn is_subtype(sub: &Type, sup: &Type, env: &VerificationEnv) -> Result<bool> {
    let value_name = "it";
    let sub_expanded = expand_type(sub, env.aliases(), value_name)?;
    let sup_expanded = expand_type(sup, env.aliases(), value_name)?;

    if sub_expanded.base != sup_expanded.base {
        return Ok(false);
    }

    if sup_expanded.predicates.is_empty() {
        return Ok(true);
    }

    let tm = TermManager::new();
    let mut solver_env = SolverEnv::new();
    solver_env.insert_var_from_base(&tm, value_name, &sub_expanded.base);

    let assumptions = sub_expanded
        .predicates
        .iter()
        .map(|predicate| expr_to_term(&tm, predicate, solver_env.vars()))
        .collect::<Result<Vec<_>>>()?;

    let goal = conjunction(&tm, &sup_expanded.predicates, solver_env.vars())?;

    solver::proves(&tm, &assumptions, &goal)
}

fn conjunction(
    tm: &TermManager,
    predicates: &[Expr],
    vars: &HashMap<String, Term>,
) -> Result<Term> {
    let mut terms = predicates
        .iter()
        .map(|predicate| expr_to_term(tm, predicate, vars))
        .collect::<Result<Vec<_>>>()?;

    if terms.is_empty() {
        return Ok(tm.mk_true());
    }

    let mut acc = terms.remove(0);
    for term in terms {
        acc = tm.mk_term(cvc5_rs::Kind::CVC5_KIND_AND, &[acc, term]);
    }

    Ok(acc)
}

fn expand_type(
    ty: &Type,
    aliases: &HashMap<String, TypeAlias>,
    value_name: &str,
) -> Result<ExpandedType> {
    let mut visiting = HashSet::new();
    expand_type_with_stack(ty, aliases, value_name, &mut visiting)
}

fn expand_type_with_stack(
    ty: &Type,
    aliases: &HashMap<String, TypeAlias>,
    value_name: &str,
    visiting: &mut HashSet<String>,
) -> Result<ExpandedType> {
    match ty {
        Type::Base(base) => expand_base(base, aliases, value_name, visiting),
        Type::Refined { base, v, predicate } => {
            let mut expanded = expand_base(base, aliases, value_name, visiting)?;
            expanded
                .predicates
                .push(substitute_var(predicate, v, value_name));
            Ok(expanded)
        }
    }
}

fn expand_base(
    base: &BaseType,
    aliases: &HashMap<String, TypeAlias>,
    value_name: &str,
    visiting: &mut HashSet<String>,
) -> Result<ExpandedType> {
    match base {
        BaseType::Custom(name) => {
            if let Some(alias) = aliases.get(name) {
                if !visiting.insert(name.clone()) {
                    return Err(anyhow!("cyclic type alias detected at `{}`", name));
                }

                let expanded = expand_type_with_stack(&alias.ty, aliases, value_name, visiting);
                visiting.remove(name);
                expanded
            } else {
                Ok(ExpandedType {
                    base: base.clone(),
                    predicates: Vec::new(),
                })
            }
        }
        _ => {
            Ok(ExpandedType {
                base: base.clone(),
                predicates: Vec::new(),
            })
        }
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
        Expr::Tuple(elems) => {
            Expr::Tuple(elems.iter().map(|e| substitute_var(e, from, to)).collect())
        }
        Expr::List(elems) => {
            Expr::List(elems.iter().map(|e| substitute_var(e, from, to)).collect())
        }
        Expr::App { func, args } => {
            Expr::App {
                func: Box::new(substitute_var(func, from, to)),
                args: args.iter().map(|a| substitute_var(a, from, to)).collect(),
            }
        }
        Expr::Lambda { params, body } => {
            // if lambda shadows `from`, do not substitute in body
            let shadows = params.iter().any(|(n, _)| n == from);
            if shadows {
                Expr::Lambda {
                    params: params.clone(),
                    body: body.clone(),
                }
            } else {
                Expr::Lambda {
                    params: params.clone(),
                    body: Box::new(substitute_var(body, from, to)),
                }
            }
        }
        Expr::Match { expr, arms } => {
            Expr::Match {
                expr: Box::new(substitute_var(expr, from, to)),
                arms: arms
                    .iter()
                    .map(|(p, e)| {
                        if pattern_binds(p, from) {
                            (p.clone(), e.clone())
                        } else {
                            (p.clone(), substitute_var(e, from, to))
                        }
                    })
                    .collect(),
            }
        }
        Expr::Let {
            name,
            ty,
            value,
            body,
        } => {
            Expr::Let {
                name: name.clone(),
                ty: ty.clone(),
                value: Box::new(substitute_var(value, from, to)),
                body: if name == from {
                    body.clone() // Shadowing
                } else {
                    Box::new(substitute_var(body, from, to))
                },
            }
        }
    }
}

fn pattern_binds(pattern: &Pattern, name: &str) -> bool {
    match pattern {
        Pattern::Wild | Pattern::Literal(_) => false,
        Pattern::Var(bound) => bound == name,
        Pattern::Tuple(items) | Pattern::List(items) => {
            items.iter().any(|item| pattern_binds(item, name))
        }
        Pattern::Cons(left, right) => pattern_binds(left, name) || pattern_binds(right, name),
    }
}

#[cfg(test)]
mod tests {
    use lmt_parser::{SpecItem, parser::Parser};

    use super::*;

    #[test]
    fn test_check_contract_consistency_ok() {
        let input = "let clamp(x: Int, min: Int, max: Int | it >= min): (res: Int | res >= min && res <= max)";
        let mut parser = Parser::new(input);
        let contract = match parser.parse_spec_item() {
            SpecItem::FunctionContract(c) => c,
            _ => panic!("Expected FunctionContract"),
        };
        check_contract_consistency(&contract).expect("expected consistent contract");
    }

    #[test]
    fn test_check_contract_consistency_inconsistent() {
        let input = "let bad(x: Int): (res: Int | res > 0 && res < 0)";
        let mut parser = Parser::new(input);
        let contract = match parser.parse_spec_item() {
            SpecItem::FunctionContract(c) => c,
            _ => panic!("Expected FunctionContract"),
        };
        let result = check_contract_consistency(&contract);
        assert!(result.is_err());
    }

    #[test]
    fn test_check_assertion_in_sequence() {
        let mut env = VerificationEnv::new();
        let mut parser = Parser::new("assert 1 + 1 == 2");
        let item = SpecItem::Assertion(parser.parse_assertion());
        check_spec_item(&item, &mut env).expect("assertion should verify");
        assert_eq!(env.facts().len(), 1);
    }

    #[test]
    fn test_check_bad_assertion_fails() {
        let mut env = VerificationEnv::new();
        let mut parser = Parser::new("assert 1 < 0");
        let item = SpecItem::Assertion(parser.parse_assertion());
        assert!(check_spec_item(&item, &mut env).is_err());
    }

    #[test]
    fn test_type_alias_chain_consistency() {
        let mut env = VerificationEnv::new();
        let mut parser = Parser::new("let Nat = Int | it >= 0");
        let nat = parser.parse_spec_item();
        check_spec_item(&nat, &mut env).expect("Nat should be consistent");

        let mut parser = Parser::new("let SmallNat = Nat | it < 256");
        let small = parser.parse_spec_item();
        check_spec_item(&small, &mut env).expect("SmallNat should be consistent");
    }

    #[test]
    fn test_subtyping_refinements() {
        let env = VerificationEnv::new();
        let mut parser = Parser::new("Int | it > 0");
        let pos = parser.parse_type();
        let mut parser = Parser::new("Int | it >= 0");
        let nat = parser.parse_type();

        assert!(is_subtype(&pos, &nat, &env).expect("subtyping check"));
        assert!(!is_subtype(&nat, &pos, &env).expect("subtyping check"));
    }

    #[test]
    fn test_subtyping_with_aliases() {
        let mut env = VerificationEnv::new();

        let mut parser = Parser::new("let Nat = Int | it >= 0");
        let nat_item = parser.parse_spec_item();
        check_spec_item(&nat_item, &mut env).expect("Nat alias should verify");

        let mut parser = Parser::new("let PosInt = Nat | it > 0");
        let pos_item = parser.parse_spec_item();
        check_spec_item(&pos_item, &mut env).expect("PosInt alias should verify");

        let mut parser = Parser::new("PosInt");
        let pos_type = parser.parse_type();
        let mut parser = Parser::new("Nat");
        let nat_type = parser.parse_type();

        assert!(is_subtype(&pos_type, &nat_type, &env).expect("alias subtyping check"));
    }
}
