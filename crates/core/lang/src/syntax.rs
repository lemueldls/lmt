//! # Syntax Module
//!
//! Defines the abstract syntax tree (AST) for the LMT core calculus.
//! This module is primarily data-definition; see `eval.rs` and `typecheck.rs` for semantics.
//!
//! # Core Terms (De Bruijn Indexing)
//!
//! The `Term` enum represents the core calculus terms. See `/plans/core_calculus.md`
//! § "Syntax" for formal grammar. Note: Currently using named strings instead of De Bruijn
//! indices for readability; migrate to De Bruijn levels if performance becomes critical.
//!
//! # Future TODO: Surface Syntax Desugaring
//! TODO: Implement parser from `.lmt` surface syntax (via `lmt-parser`) to core `Term`.
//! This requires translation of:
//!   - Named variables to De Bruijn indices
//!   - Surface function definitions to core Lambda terms
//!   - Surface pattern matching to eliminator forms
//! See: `crates/core/parser/src/mapping.rs` (currently returns SpecItem, needs to lower to Term)

use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UniverseLevel(pub u32);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Binder {
    pub name: String,
    pub ty: Box<Type>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    Universe(UniverseLevel),
    Bool,
    Int,
    Unit,
    Var(String),
    Pi {
        binder: Binder,
        body: Box<Type>,
    },
    Sigma {
        binder: Binder,
        body: Box<Type>,
    },
    Refine {
        binder: Binder,
        predicate: Box<Expr>,
    },
    Quote(Box<Term>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Expr {
    Var(String),
    Int(i64),
    Bool(bool),
    Unit,
    Lambda {
        binder: Binder,
        body: Box<Expr>,
    },
    App {
        callee: Box<Expr>,
        argument: Box<Expr>,
    },
    Pair(Box<Expr>, Box<Expr>),
    Fst(Box<Expr>),
    Snd(Box<Expr>),
    Let {
        name: String,
        value: Box<Expr>,
        body: Box<Expr>,
    },
    If {
        condition: Box<Expr>,
        then_branch: Box<Expr>,
        else_branch: Box<Expr>,
    },
    Quote(Box<Expr>),
    Eval(Box<Expr>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Statement {
    TypeAlias {
        name: String,
        ty: Type,
    },
    Function {
        name: String,
        parameter: Binder,
        return_type: Type,
        body: Expr,
    },
    Assertion {
        predicate: Expr,
    },
    Expression(Expr),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Program {
    pub items: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Term {
    Var(String),
    Int(i64),
    Bool(bool),
    Unit,
    Lambda {
        binder: Binder,
        body: Box<Term>,
    },
    App {
        callee: Box<Term>,
        argument: Box<Term>,
    },
    Pair(Box<Term>, Box<Term>),
    Fst(Box<Term>),
    Snd(Box<Term>),
    Let {
        name: String,
        value: Box<Term>,
        body: Box<Term>,
    },
    If {
        condition: Box<Term>,
        then_branch: Box<Term>,
        else_branch: Box<Term>,
    },
    Quote(Box<Term>),
    Eval(Box<Term>),
}

impl Term {
    /// Substitute occurrences of `var` in `term` with `value`.
    /// This implementation avoids variable capture by alpha-renaming binders when needed.
    pub fn substitute(term: &Term, var: &str, value: &Term) -> Term {
        let value_fv = Term::free_vars(value);

        Term::substitute_capture_avoiding(term, var, value, &value_fv)
    }

    fn substitute_capture_avoiding(
        term: &Term,
        var: &str,
        value: &Term,
        value_fv: &HashSet<String>,
    ) -> Term {
        match term {
            Term::Var(name) if name == var => value.clone(),
            Term::Var(_) => term.clone(),
            Term::Int(_) | Term::Bool(_) | Term::Unit => term.clone(),
            Term::Lambda { binder, body } => {
                if binder.name == var {
                    // shadowed
                    term.clone()
                } else if value_fv.contains(&binder.name) {
                    // Prevent capture: alpha-rename binder before descending.
                    let fresh = Term::fresh_name(&binder.name, body, value_fv, var);
                    let renamed_body = Term::rename_bound_occurrences(body, &binder.name, &fresh);

                    Term::Lambda {
                        binder: Binder {
                            name: fresh,
                            ty: binder.ty.clone(),
                        },
                        body: Box::new(Term::substitute_capture_avoiding(
                            &renamed_body,
                            var,
                            value,
                            value_fv,
                        )),
                    }
                } else {
                    Term::Lambda {
                        binder: binder.clone(),
                        body: Box::new(Term::substitute_capture_avoiding(
                            body, var, value, value_fv,
                        )),
                    }
                }
            }
            Term::App { callee, argument } => {
                Term::App {
                    callee: Box::new(Term::substitute_capture_avoiding(
                        callee, var, value, value_fv,
                    )),
                    argument: Box::new(Term::substitute_capture_avoiding(
                        argument, var, value, value_fv,
                    )),
                }
            }
            Term::Pair(a, b) => {
                Term::Pair(
                    Box::new(Term::substitute_capture_avoiding(a, var, value, value_fv)),
                    Box::new(Term::substitute_capture_avoiding(b, var, value, value_fv)),
                )
            }
            Term::Fst(t) => {
                Term::Fst(Box::new(Term::substitute_capture_avoiding(
                    t, var, value, value_fv,
                )))
            }
            Term::Snd(t) => {
                Term::Snd(Box::new(Term::substitute_capture_avoiding(
                    t, var, value, value_fv,
                )))
            }
            Term::Let {
                name,
                value: v,
                body,
            } => {
                let new_value = Term::substitute_capture_avoiding(v, var, value, value_fv);

                if name == var {
                    Term::Let {
                        name: name.clone(),
                        value: Box::new(new_value),
                        body: body.clone(),
                    }
                } else if value_fv.contains(name) {
                    // Prevent capture in the `let` body binder.
                    let fresh = Term::fresh_name(name, body, value_fv, var);
                    let renamed_body = Term::rename_bound_occurrences(body, name, &fresh);

                    Term::Let {
                        name: fresh,
                        value: Box::new(new_value),
                        body: Box::new(Term::substitute_capture_avoiding(
                            &renamed_body,
                            var,
                            value,
                            value_fv,
                        )),
                    }
                } else {
                    Term::Let {
                        name: name.clone(),
                        value: Box::new(new_value),
                        body: Box::new(Term::substitute_capture_avoiding(
                            body, var, value, value_fv,
                        )),
                    }
                }
            }
            Term::If {
                condition,
                then_branch,
                else_branch,
            } => {
                Term::If {
                    condition: Box::new(Term::substitute_capture_avoiding(
                        condition, var, value, value_fv,
                    )),
                    then_branch: Box::new(Term::substitute_capture_avoiding(
                        then_branch,
                        var,
                        value,
                        value_fv,
                    )),
                    else_branch: Box::new(Term::substitute_capture_avoiding(
                        else_branch,
                        var,
                        value,
                        value_fv,
                    )),
                }
            }
            Term::Quote(t) => {
                Term::Quote(Box::new(Term::substitute_capture_avoiding(
                    t, var, value, value_fv,
                )))
            }
            Term::Eval(t) => {
                Term::Eval(Box::new(Term::substitute_capture_avoiding(
                    t, var, value, value_fv,
                )))
            }
        }
    }

    fn free_vars(term: &Term) -> HashSet<String> {
        match term {
            Term::Var(name) => {
                let mut set = HashSet::new();
                set.insert(name.clone());

                set
            }
            Term::Int(_) | Term::Bool(_) | Term::Unit => HashSet::new(),
            Term::Lambda { binder, body } => {
                let mut body_vars = Term::free_vars(body);
                body_vars.remove(&binder.name);

                body_vars
            }
            Term::App { callee, argument } => {
                let mut vars = Term::free_vars(callee);
                vars.extend(Term::free_vars(argument));

                vars
            }
            Term::Pair(a, b) => {
                let mut vars = Term::free_vars(a);
                vars.extend(Term::free_vars(b));

                vars
            }
            Term::Fst(t) | Term::Snd(t) | Term::Quote(t) | Term::Eval(t) => Term::free_vars(t),
            Term::Let { name, value, body } => {
                let mut vars = Term::free_vars(value);
                let mut body_vars = Term::free_vars(body);
                body_vars.remove(name);
                vars.extend(body_vars);

                vars
            }
            Term::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let mut vars = Term::free_vars(condition);
                vars.extend(Term::free_vars(then_branch));
                vars.extend(Term::free_vars(else_branch));

                vars
            }
        }
    }

    fn fresh_name(base: &str, scope: &Term, value_fv: &HashSet<String>, var: &str) -> String {
        let mut forbidden = Term::free_vars(scope);
        forbidden.extend(value_fv.iter().cloned());
        forbidden.insert(var.to_string());
        forbidden.insert(base.to_string());

        let mut idx: usize = 0;
        loop {
            let candidate = format!("{}_{}", base, idx);

            if !forbidden.contains(&candidate) {
                return candidate;
            }

            idx += 1;
        }
    }

    fn rename_bound_occurrences(term: &Term, old: &str, new: &str) -> Term {
        match term {
            Term::Var(name) if name == old => Term::Var(new.to_string()),
            Term::Var(_) | Term::Int(_) | Term::Bool(_) | Term::Unit => term.clone(),
            Term::Lambda { binder, body } => {
                if binder.name == old {
                    // Inner binder shadows `old`; stop descending here.
                    term.clone()
                } else {
                    Term::Lambda {
                        binder: binder.clone(),
                        body: Box::new(Term::rename_bound_occurrences(body, old, new)),
                    }
                }
            }
            Term::App { callee, argument } => {
                Term::App {
                    callee: Box::new(Term::rename_bound_occurrences(callee, old, new)),
                    argument: Box::new(Term::rename_bound_occurrences(argument, old, new)),
                }
            }
            Term::Pair(a, b) => {
                Term::Pair(
                    Box::new(Term::rename_bound_occurrences(a, old, new)),
                    Box::new(Term::rename_bound_occurrences(b, old, new)),
                )
            }
            Term::Fst(t) => Term::Fst(Box::new(Term::rename_bound_occurrences(t, old, new))),
            Term::Snd(t) => Term::Snd(Box::new(Term::rename_bound_occurrences(t, old, new))),
            Term::Let { name, value, body } => {
                let renamed_value = Term::rename_bound_occurrences(value, old, new);
                if name == old {
                    Term::Let {
                        name: name.clone(),
                        value: Box::new(renamed_value),
                        body: body.clone(),
                    }
                } else {
                    Term::Let {
                        name: name.clone(),
                        value: Box::new(renamed_value),
                        body: Box::new(Term::rename_bound_occurrences(body, old, new)),
                    }
                }
            }
            Term::If {
                condition,
                then_branch,
                else_branch,
            } => {
                Term::If {
                    condition: Box::new(Term::rename_bound_occurrences(condition, old, new)),
                    then_branch: Box::new(Term::rename_bound_occurrences(then_branch, old, new)),
                    else_branch: Box::new(Term::rename_bound_occurrences(else_branch, old, new)),
                }
            }
            Term::Quote(t) => Term::Quote(Box::new(Term::rename_bound_occurrences(t, old, new))),
            Term::Eval(t) => Term::Eval(Box::new(Term::rename_bound_occurrences(t, old, new))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Binder, Term, Type};

    fn unit_binder(name: &str) -> Binder {
        Binder {
            name: name.to_string(),
            ty: Box::new(Type::Unit),
        }
    }

    #[test]
    fn substitute_avoids_lambda_capture() {
        let term = Term::Lambda {
            binder: unit_binder("y"),
            body: Box::new(Term::Var("x".to_string())),
        };

        let replaced = Term::substitute(&term, "x", &Term::Var("y".to_string()));
        match replaced {
            Term::Lambda { binder, body } => {
                assert_ne!(binder.name, "y");
                assert_eq!(*body, Term::Var("y".to_string()));
            }
            _ => panic!("expected lambda"),
        }
    }

    #[test]
    fn substitute_respects_shadowing() {
        let term = Term::Lambda {
            binder: unit_binder("x"),
            body: Box::new(Term::Var("x".to_string())),
        };

        let replaced = Term::substitute(&term, "x", &Term::Int(1));
        assert_eq!(replaced, term);
    }

    #[test]
    fn substitute_avoids_let_capture() {
        let term = Term::Let {
            name: "y".to_string(),
            value: Box::new(Term::Int(0)),
            body: Box::new(Term::Var("x".to_string())),
        };

        let replaced = Term::substitute(&term, "x", &Term::Var("y".to_string()));
        match replaced {
            Term::Let { name, body, .. } => {
                assert_ne!(name, "y");
                assert_eq!(*body, Term::Var("y".to_string()));
            }
            _ => panic!("expected let"),
        }
    }
}
