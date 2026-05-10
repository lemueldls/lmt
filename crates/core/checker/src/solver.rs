use std::collections::HashMap;

use anyhow::{Result, anyhow};
use cvc5_rs::{Kind, Solver, Sort, Term, TermManager};
use lmt_parser::ast::{BaseType, Type};

pub struct SolverEnv {
    vars: HashMap<String, Term>,
}

impl SolverEnv {
    pub fn new() -> Self {
        Self {
            vars: HashMap::new(),
        }
    }

    pub fn vars(&self) -> &HashMap<String, Term> {
        &self.vars
    }

    pub fn insert_var_from_type(&mut self, tm: &TermManager, name: &str, ty: &Type) {
        let base = base_type_of(ty);
        let sort = sort_for_base(tm, base);
        let term = tm.mk_const(sort, name);
        self.vars.insert(name.to_string(), term);
    }
}

pub fn is_satisfiable(tm: &TermManager, assumptions: &[Term]) -> Result<bool> {
    let mut solver = Solver::new(tm);
    solver.set_logic("ALL");

    for assumption in assumptions {
        solver.assert_formula(assumption.clone());
    }

    let result = solver.check_sat();
    if result.is_unknown() {
        return Err(anyhow!("solver returned unknown for satisfiability check"));
    }

    Ok(result.is_sat())
}

pub fn proves(tm: &TermManager, assumptions: &[Term], goal: &Term) -> Result<bool> {
    let mut solver = Solver::new(tm);
    solver.set_logic("ALL");

    for assumption in assumptions {
        solver.assert_formula(assumption.clone());
    }

    let not_goal = tm.mk_term(Kind::CVC5_KIND_NOT, &[goal.clone()]);
    solver.assert_formula(not_goal);

    let result = solver.check_sat();
    if result.is_unknown() {
        return Err(anyhow!("solver returned unknown for proof obligation"));
    }

    Ok(result.is_unsat())
}

pub fn base_type_of(ty: &Type) -> &BaseType {
    match ty {
        Type::Base(base) => base,
        Type::Refined { base, .. } => base,
    }
}

pub fn sort_for_base(tm: &TermManager, base: &BaseType) -> Sort {
    match base {
        BaseType::Int => tm.integer_sort(),
        BaseType::Bool => tm.boolean_sort(),
        BaseType::Real => tm.real_sort(),
        // For now, model custom types as uninterpreted sorts.
        BaseType::Custom(name) => tm.mk_uninterpreted_sort(name),
    }
}
