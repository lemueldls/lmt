use crate::syntax::{Binder, Term, Type};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Neutral {
    Var(String),
    App {
        callee: Box<Neutral>,
        argument: Box<Value>,
    },
    Fst(Box<Neutral>),
    Snd(Box<Neutral>),
    If {
        condition: Box<Neutral>,
        then_value: Box<Value>,
        else_value: Box<Value>,
    },
    Quote(Box<Term>),
    Eval(Box<Neutral>),
    Stuck(Term),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Neutral(Neutral),
    Int(i64),
    Bool(bool),
    Unit,
    Pair(Box<Value>, Box<Value>),
    Closure {
        env: Env,
        binder: Binder,
        body: Box<Term>,
    },
}

type Env = Vec<(String, Value)>;

fn lookup_env(env: &Env, name: &str) -> Option<Value> {
    env.iter()
        .rev()
        .find(|(entry_name, _)| entry_name == name)
        .map(|(_, value)| value.clone())
}

fn extend_env(env: &Env, name: String, value: Value) -> Env {
    let mut next = env.clone();
    next.push((name, value));
    next
}

fn substitute_type_term(ty: &Type, var: &str, value: &Term) -> Type {
    match ty {
        Type::Universe(_) | Type::Bool | Type::Int | Type::Unit | Type::Var(_) => ty.clone(),
        Type::Pi { binder, body } => {
            if binder.name == var {
                Type::Pi {
                    binder: binder.clone(),
                    body: body.clone(),
                }
            } else {
                Type::Pi {
                    binder: binder.clone(),
                    body: Box::new(substitute_type_term(body, var, value)),
                }
            }
        }
        Type::Sigma { binder, body } => {
            if binder.name == var {
                Type::Sigma {
                    binder: binder.clone(),
                    body: body.clone(),
                }
            } else {
                Type::Sigma {
                    binder: binder.clone(),
                    body: Box::new(substitute_type_term(body, var, value)),
                }
            }
        }
        Type::Refine { binder, predicate } => Type::Refine {
            binder: binder.clone(),
            predicate: predicate.clone(),
        },
        Type::Quote(term) => Type::Quote(Box::new(Term::substitute(term, var, value))),
    }
}

fn apply_value(callee: Value, argument: Value) -> Value {
    match callee {
        Value::Closure { env, binder, body } => {
            let next = extend_env(&env, binder.name, argument);
            meaning(&body, &next)
        }
        Value::Neutral(neutral) => Value::Neutral(Neutral::App {
            callee: Box::new(neutral),
            argument: Box::new(argument),
        }),
        other => Value::Neutral(Neutral::Stuck(Term::App {
            callee: Box::new(quote_value(&other)),
            argument: Box::new(quote_value(&argument)),
        })),
    }
}

fn project_fst(value: Value) -> Value {
    match value {
        Value::Pair(left, _) => *left,
        Value::Neutral(neutral) => Value::Neutral(Neutral::Fst(Box::new(neutral))),
        other => Value::Neutral(Neutral::Stuck(Term::Fst(Box::new(quote_value(&other))))),
    }
}

fn project_snd(value: Value) -> Value {
    match value {
        Value::Pair(_, right) => *right,
        Value::Neutral(neutral) => Value::Neutral(Neutral::Snd(Box::new(neutral))),
        other => Value::Neutral(Neutral::Stuck(Term::Snd(Box::new(quote_value(&other))))),
    }
}

fn meaning(term: &Term, env: &Env) -> Value {
    match term {
        Term::Var(name) => lookup_env(env, name).unwrap_or_else(|| Value::Neutral(Neutral::Var(name.clone()))),
        Term::Int(value) => Value::Int(*value),
        Term::Bool(value) => Value::Bool(*value),
        Term::Unit => Value::Unit,
        Term::Lambda { binder, body } => Value::Closure {
            env: env.clone(),
            binder: binder.clone(),
            body: body.clone(),
        },
        Term::App { callee, argument } => {
            let callee_value = meaning(callee, env);
            let argument_value = meaning(argument, env);
            apply_value(callee_value, argument_value)
        }
        Term::Pair(left, right) => {
            let left_value = meaning(left, env);
            let right_value = meaning(right, env);
            Value::Pair(Box::new(left_value), Box::new(right_value))
        }
        Term::Fst(pair) => {
            let pair_value = meaning(pair, env);
            project_fst(pair_value)
        }
        Term::Snd(pair) => {
            let pair_value = meaning(pair, env);
            project_snd(pair_value)
        }
        Term::Let { name, value, body } => {
            let value_sem = meaning(value, env);
            let next = extend_env(env, name.clone(), value_sem);
            meaning(body, &next)
        }
        Term::If {
            condition,
            then_branch,
            else_branch,
        } => {
            let cond = meaning(condition, env);
            match cond {
                Value::Bool(true) => meaning(then_branch, env),
                Value::Bool(false) => meaning(else_branch, env),
                Value::Neutral(neutral_cond) => {
                    let then_sem = meaning(then_branch, env);
                    let else_sem = meaning(else_branch, env);
                    Value::Neutral(Neutral::If {
                        condition: Box::new(neutral_cond),
                        then_value: Box::new(then_sem),
                        else_value: Box::new(else_sem),
                    })
                }
                other => Value::Neutral(Neutral::Stuck(Term::If {
                    condition: Box::new(quote_value(&other)),
                    then_branch: then_branch.clone(),
                    else_branch: else_branch.clone(),
                })),
            }
        }
        Term::Quote(inner) => Value::Neutral(Neutral::Quote(Box::new(quote_value(&meaning(inner, env))))),
        Term::Eval(inner) => {
            let quoted = meaning(inner, env);
            match quoted {
                Value::Neutral(Neutral::Quote(term)) => meaning(&term, env),
                Value::Neutral(neutral) => Value::Neutral(Neutral::Eval(Box::new(neutral))),
                other => other,
            }
        }
    }
}

fn fresh_binder_name(name: &str) -> String {
    if name.is_empty() {
        "x_nbe".to_string()
    } else {
        format!("{}_nbe", name)
    }
}

fn quote_neutral(neutral: &Neutral) -> Term {
    match neutral {
        Neutral::Var(name) => Term::Var(name.clone()),
        Neutral::App { callee, argument } => Term::App {
            callee: Box::new(quote_neutral(callee)),
            argument: Box::new(quote_value(argument)),
        },
        Neutral::Fst(pair) => Term::Fst(Box::new(quote_neutral(pair))),
        Neutral::Snd(pair) => Term::Snd(Box::new(quote_neutral(pair))),
        Neutral::If {
            condition,
            then_value,
            else_value,
        } => Term::If {
            condition: Box::new(quote_neutral(condition)),
            then_branch: Box::new(quote_value(then_value)),
            else_branch: Box::new(quote_value(else_value)),
        },
        Neutral::Quote(term) => Term::Quote(term.clone()),
        Neutral::Eval(neutral) => Term::Eval(Box::new(quote_neutral(neutral))),
        Neutral::Stuck(term) => term.clone(),
    }
}

fn quote_value(value: &Value) -> Term {
    match value {
        Value::Int(n) => Term::Int(*n),
        Value::Bool(b) => Term::Bool(*b),
        Value::Unit => Term::Unit,
        Value::Pair(left, right) => {
            Term::Pair(Box::new(quote_value(left)), Box::new(quote_value(right)))
        }
        Value::Neutral(neutral) => quote_neutral(neutral),
        Value::Closure { binder, .. } => {
            let parameter = fresh_binder_name(&binder.name);
            let reified_binder = Binder {
                name: parameter.clone(),
                ty: binder.ty.clone(),
            };
            let arg = Value::Neutral(Neutral::Var(parameter));
            let body_value = apply_value(value.clone(), arg);
            Term::Lambda {
                binder: reified_binder,
                body: Box::new(quote_value(&body_value)),
            }
        }
    }
}

pub fn reflect(ty: &Type, neutral: Neutral) -> Value {
    match ty {
        Type::Refine { binder, .. } => reflect(&binder.ty, neutral),
        _ => Value::Neutral(neutral),
    }
}

pub fn reify(ty: &Type, value: &Value) -> Term {
    match ty {
        Type::Int => match value {
            Value::Int(n) => Term::Int(*n),
            _ => quote_value(value),
        },
        Type::Bool => match value {
            Value::Bool(b) => Term::Bool(*b),
            _ => quote_value(value),
        },
        Type::Unit => Term::Unit,
        Type::Refine { binder, .. } => reify(&binder.ty, value),
        Type::Pi { binder, body } => {
            let parameter = fresh_binder_name(&binder.name);
            let parameter_neutral = Neutral::Var(parameter.clone());
            let parameter_value = reflect(&binder.ty, parameter_neutral.clone());
            let applied = apply_value(value.clone(), parameter_value.clone());

            let parameter_term = reify(&binder.ty, &parameter_value);
            let body_ty = substitute_type_term(body, &binder.name, &parameter_term);
            let body_nf = reify(&body_ty, &applied);

            Term::Lambda {
                binder: Binder {
                    name: parameter,
                    ty: binder.ty.clone(),
                },
                body: Box::new(body_nf),
            }
        }
        Type::Sigma { binder, body } => {
            let first = project_fst(value.clone());
            let first_nf = reify(&binder.ty, &first);
            let body_ty = substitute_type_term(body, &binder.name, &first_nf);
            let second = project_snd(value.clone());
            let second_nf = reify(&body_ty, &second);
            Term::Pair(Box::new(first_nf), Box::new(second_nf))
        }
        Type::Quote(_) | Type::Universe(_) | Type::Var(_) => quote_value(value),
    }
}

pub fn eval(term: &Term) -> Value {
    meaning(term, &Vec::new())
}

pub fn normalize(term: &Term) -> Term {
    quote_value(&eval(term))
}

#[cfg(test)]
mod tests {
    use super::{Neutral, normalize, reify, reflect};
    use crate::syntax::{Binder, Term, Type};

    fn int_binder(name: &str) -> Binder {
        Binder {
            name: name.to_string(),
            ty: Box::new(Type::Int),
        }
    }

    #[test]
    fn normalize_beta_reduces_application() {
        let term = Term::App {
            callee: Box::new(Term::Lambda {
                binder: int_binder("x"),
                body: Box::new(Term::Var("x".to_string())),
            }),
            argument: Box::new(Term::Int(42)),
        };

        assert_eq!(normalize(&term), Term::Int(42));
    }

    #[test]
    fn normalize_let_eliminates_binding() {
        let term = Term::Let {
            name: "x".to_string(),
            value: Box::new(Term::Bool(true)),
            body: Box::new(Term::Var("x".to_string())),
        };

        assert_eq!(normalize(&term), Term::Bool(true));
    }

    #[test]
    fn normalize_pair_projection() {
        let term = Term::Fst(Box::new(Term::Pair(
            Box::new(Term::Int(7)),
            Box::new(Term::Bool(false)),
        )));

        assert_eq!(normalize(&term), Term::Int(7));
    }

    #[test]
    fn reify_eta_expands_neutral_function() {
        let ty = Type::Pi {
            binder: int_binder("x"),
            body: Box::new(Type::Int),
        };
        let reflected = reflect(&ty, Neutral::Var("f".to_string()));

        let reified = reify(&ty, &reflected);
        let expected = Term::Lambda {
            binder: Binder {
                name: "x_nbe".to_string(),
                ty: Box::new(Type::Int),
            },
            body: Box::new(Term::App {
                callee: Box::new(Term::Var("f".to_string())),
                argument: Box::new(Term::Var("x_nbe".to_string())),
            }),
        };

        assert_eq!(reified, expected);
    }
}
