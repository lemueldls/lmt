use std::collections::HashMap;

use cvc5_rs::{Kind, Solver, Term, TermManager};

use crate::{Quantifier, SmtExpr, SmtOp, SmtSolver, SmtSort, SolverError};

pub struct Cvc5Solver {
    tm: TermManager,
    solver: Solver,
    vars: HashMap<String, Term>,
}

impl Default for Cvc5Solver {
    fn default() -> Self {
        Self::new()
    }
}

impl Cvc5Solver {
    #[must_use]
    pub fn new() -> Self {
        let tm = TermManager::new();
        let mut solver = Solver::new(&tm);
        solver.set_logic("ALL");
        solver.set_option("produce-models", "true");

        Self {
            tm,
            solver,
            vars: HashMap::new(),
        }
    }

    fn get_var(&mut self, name: &str) -> Term {
        if let Some(t) = self.vars.get(name) {
            t.clone()
        } else {
            // Default to Int sort for now if undeclared
            let int_sort = self.tm.integer_sort();
            let var = self.tm.mk_const(int_sort, name);
            self.vars.insert(name.to_owned(), var.clone());

            var
        }
    }

    fn translate(&mut self, expr: &SmtExpr) -> Term {
        match expr {
            SmtExpr::Var(v) => self.get_var(v),
            SmtExpr::Int(i) => self.tm.mk_integer(*i),
            SmtExpr::Bool(b) => self.tm.mk_boolean(*b),
            SmtExpr::App(op, args) => {
                let kind = match op {
                    SmtOp::Eq => Kind::CVC5_KIND_EQUAL,
                    SmtOp::Gt => Kind::CVC5_KIND_GT,
                    SmtOp::GtEq => Kind::CVC5_KIND_GEQ,
                    SmtOp::Lt => Kind::CVC5_KIND_LT,
                    SmtOp::LtEq => Kind::CVC5_KIND_LEQ,
                    SmtOp::And => Kind::CVC5_KIND_AND,
                    SmtOp::Or => Kind::CVC5_KIND_OR,
                    SmtOp::Not => Kind::CVC5_KIND_NOT,
                    SmtOp::Implies => Kind::CVC5_KIND_IMPLIES,
                    SmtOp::Add => Kind::CVC5_KIND_ADD,
                    SmtOp::Sub => Kind::CVC5_KIND_SUB,
                    SmtOp::Mul => Kind::CVC5_KIND_MULT,
                    SmtOp::Div => Kind::CVC5_KIND_INTS_DIVISION,
                };
                let cvc_args: Vec<Term> = args.iter().map(|a| self.translate(a)).collect();

                self.tm.mk_term(kind, &cvc_args)
            }
            SmtExpr::Quantifier(q, vars, body) => {
                let kind = match q {
                    Quantifier::Forall => Kind::CVC5_KIND_FORALL,
                    Quantifier::Exists => Kind::CVC5_KIND_EXISTS,
                };
                let mut bvl_args = vec![];
                for (v, s) in vars {
                    let sort = match s {
                        SmtSort::Int => self.tm.integer_sort(),
                        SmtSort::Bool => self.tm.boolean_sort(),
                    };
                    let bv = self.tm.mk_var(sort, v);
                    self.vars.insert(v.clone(), bv.clone());
                    bvl_args.push(bv);
                }
                let bvl = self.tm.mk_term(Kind::CVC5_KIND_VARIABLE_LIST, &bvl_args);
                let cvc_body = self.translate(body);

                self.tm.mk_term(kind, &[bvl, cvc_body])
            }
        }
    }
}

impl SmtSolver for Cvc5Solver {
    fn check_validity(&mut self, env: &[SmtExpr], vc: &SmtExpr) -> Result<bool, SolverError> {
        self.solver.reset_assertions();
        for assertion in env {
            let term = self.translate(assertion);
            self.solver.assert_formula(term);
        }

        let vc_term = self.translate(vc);
        let not_vc = self.tm.mk_term(Kind::CVC5_KIND_NOT, &[vc_term]);
        self.solver.assert_formula(not_vc);
        let result = self.solver.check_sat();

        Ok(result.is_unsat())
    }

    fn check_sat(&mut self, assertions: &[SmtExpr]) -> Result<bool, SolverError> {
        self.solver.reset_assertions();
        for assertion in assertions {
            let term = self.translate(assertion);
            self.solver.assert_formula(term);
        }

        let result = self.solver.check_sat();

        Ok(result.is_sat())
    }
}
