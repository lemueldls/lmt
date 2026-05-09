use std::collections::HashMap;

use cvc5_rs::{Sort, Term, TermManager};
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
