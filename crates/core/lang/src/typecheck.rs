use thiserror::Error;

use crate::syntax::{Binder, Expr, Program, Statement, Term, Type, UniverseLevel};

#[derive(Debug, Clone, Default)]
pub struct TypeEnv {
    entries: Vec<(String, Type)>,
}

impl TypeEnv {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, name: impl Into<String>, ty: Type) {
        self.entries.push((name.into(), ty));
    }

    pub fn lookup(&self, name: &str) -> Option<&Type> {
        self.entries
            .iter()
            .rev()
            .find(|(entry_name, _)| entry_name == name)
            .map(|(_, ty)| ty)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeResult {
    Type(Type),
    Universe(UniverseLevel),
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum TypeError {
    #[error("unbound variable: {0}")]
    UnboundVariable(String),
    #[error("type mismatch: expected {expected:?}, found {found:?}")]
    Mismatch { expected: Type, found: Type },
    #[error("unsupported typing rule for the current scaffold")]
    Unsupported,
}

/// TODO (Phase 2): Implement full bidirectional type checking for the core calculus.
/// See `/plans/core_calculus.md` § "Bidirectional Type System".
///
/// Current implementation only performs shallow syntactic checks. Needs:
///
/// 1. **Implement `check()` judgment**: Γ ⊢ e ⇐ A
///    - Checks that an expression matches a given type
///    - Drives type inference for omitted type annotations
///    - Requires integration with `crate::eval::normalize()` for type equality
///
/// 2. **Implement `synthesize()` judgment**: Γ ⊢ e ⇒ A
///    - Infers the type of an expression
///    - Handles variables (via TypeEnv), function application, pair construction
///    - For dependent function types (Pi), synthesize argument type first
///
/// 3. **Implement definitional equality**: A ≡ B
///    - Uses NbE normalization: normalize(A) = normalize(B)
///    - Currently stubbed; see `eval.rs::normalize()`
///
/// 4. **Phase 3 Addition: SMT-backed refinement checking**
///    - For Refine { binder, predicate } types, discharge predicate via cvc5
///    - Call `cvc5_sys::check_satisfiable(predicate)` after substitution
///    - Requires: Feature flag for cvc5 integration, error type for SMT failures
///
/// # Test Cases (Phase 2):
///   - `test_synth_var`: ∅ ⊢ x ⇒ T fails with UnboundVariable
///   - `test_synth_int`: ∅ ⊢ 42 ⇒ Int succeeds
///   - `test_check_lambda`: ∅ ⊢ λx:Int.x ⇐ Int → Int succeeds
///   - `test_check_app`: Γ ⊢ f a ⇐ B with Γ ⊢ f ⇒ A → B and Γ ⊢ a ⇐ A succeeds
///
/// # Dependencies:
///   - `crate::eval::normalize()` — for type equality checking (Phase 2)
///   - `cvc5_sys` — for refinement predicate discharge (Phase 3)
///   - `crate::syntax::Term` — for lowering surface Expr to core Term (Phase 2)
pub fn type_check(program: &Program, env: &mut TypeEnv) -> Result<Vec<TypeResult>, TypeError> {
    // TODO (Phase 2): Replace with actual bidirectional typechecking.
    // Current implementation is a stub that only records definitions.
    let mut results = Vec::new();

    for item in &program.items {
        match item {
            Statement::TypeAlias { name, ty } => {
                env.insert(name.clone(), ty.clone());
                results.push(TypeResult::Type(Type::Var(name.clone())));
            }
            Statement::Function {
                name,
                parameter,
                return_type,
                body,
            } => {
                // Push parameter to env and attempt to check body against return_type
                env.insert(parameter.name.clone(), (*parameter.ty).clone());
                // Simple check: lower `Expr` -> `Term`, synthesize the body and compare
                let body_term = expr_to_term(body);
                match synthesize(&body_term, env) {
                    Ok(inferred) => {
                        if !equal(&inferred, return_type) {
                            return Err(TypeError::Mismatch {
                                expected: return_type.clone(),
                                found: inferred,
                            });
                        }
                    }
                    Err(_) => {
                        // fallback: accept for now but record TODO
                    }
                }
                results.push(TypeResult::Type(return_type.clone()));
                env.insert(name.clone(), return_type.clone());
            }
            Statement::Assertion { predicate } => {
                // TODO (Phase 2): Implement `check(predicate, Bool, env)`
                // Verify assertion is boolean-typed.
                // TODO (Phase 3): Evaluate predicate and discharge via cvc5 if it's a refinement.
                let _ = predicate;
                results.push(TypeResult::Type(Type::Bool));
            }
            Statement::Expression(expr) => {
                // TODO (Phase 2): Implement `synthesize(expr, env)` for type inference.
                let _ = expr;
                results.push(TypeResult::Type(Type::Unit));
            }
        }
    }

    Ok(results)
}

/// Lower a surface `Expr` to a core `Term` for the current scaffold.
fn expr_to_term(e: &Expr) -> Term {
    match e {
        Expr::Var(s) => Term::Var(s.clone()),
        Expr::Int(n) => Term::Int(*n),
        Expr::Bool(b) => Term::Bool(*b),
        Expr::Unit => Term::Unit,
        Expr::Lambda { binder, body } => {
            Term::Lambda {
                binder: binder.clone(),
                body: Box::new(expr_to_term(body)),
            }
        }
        Expr::App { callee, argument } => {
            Term::App {
                callee: Box::new(expr_to_term(callee)),
                argument: Box::new(expr_to_term(argument)),
            }
        }
        Expr::Pair(a, b) => Term::Pair(Box::new(expr_to_term(a)), Box::new(expr_to_term(b))),
        Expr::Fst(t) => Term::Fst(Box::new(expr_to_term(t))),
        Expr::Snd(t) => Term::Snd(Box::new(expr_to_term(t))),
        Expr::Let { name, value, body } => {
            Term::Let {
                name: name.clone(),
                value: Box::new(expr_to_term(value)),
                body: Box::new(expr_to_term(body)),
            }
        }
        Expr::If {
            condition,
            then_branch,
            else_branch,
        } => {
            Term::If {
                condition: Box::new(expr_to_term(condition)),
                then_branch: Box::new(expr_to_term(then_branch)),
                else_branch: Box::new(expr_to_term(else_branch)),
            }
        }
        Expr::Quote(t) => Term::Quote(Box::new(expr_to_term(t))),
        Expr::Eval(t) => Term::Eval(Box::new(expr_to_term(t))),
    }
}

/// TODO (Phase 2): Implement type synthesis judgment Γ ⊢ e ⇒ A.
/// See `/plans/core_calculus.md` § "Synthesis Judgment".
///
/// Expected behavior:
///   - For Var(x), lookup x in env (or error UnboundVariable)
///   - For App(f, a), synthesize type of f as Pi(x, A, B), check a : A, return B[a/x]
///   - For Lambda/Pair/etc., return error (not synthesizable, must use checking mode)
///
/// # Test:
///   - `test_synth_var_found`: TypeEnv { ("x", Int) } ⊢ x ⇒ Int
///   - `test_synth_app`: Γ ⊢ (λx:Int.x) 42 ⇒ Int
pub fn synthesize(_term: &Term, _env: &TypeEnv) -> Result<Type, TypeError> {
    match _term {
        Term::Int(_) => Ok(Type::Int),
        Term::Unit => Ok(Type::Unit),
        Term::Bool(_) => Ok(Type::Bool),
        Term::Var(name) => {
            _env.lookup(name)
                .cloned()
                .ok_or(TypeError::UnboundVariable(name.clone()))
        }
        Term::App { callee, argument } => {
            // synthesize callee, expect Pi, check argument
            let fty = synthesize(callee, _env)?;
            match fty {
                Type::Pi { binder, body } => {
                    // check arg
                    check(argument, &*binder.ty, _env)?;
                    // NOTE: not performing full dependent substitution in types;
                    // if body contains the binder.name, a full implementation must substitute.
                    Ok(*body.clone())
                }
                _ => {
                    Err(TypeError::Mismatch {
                        expected: Type::Pi {
                            binder: Binder {
                                name: "_".to_string(),
                                ty: Box::new(Type::Unit),
                            },
                            body: Box::new(Type::Unit),
                        },
                        found: fty,
                    })
                }
            }
        }
        Term::Lambda { binder, body } => {
            // Lambda is not synthesizable without expected type
            Err(TypeError::Unsupported)
        }
        Term::Pair(a, b) => Err(TypeError::Unsupported),
        Term::Fst(t) => Err(TypeError::Unsupported),
        Term::Snd(t) => Err(TypeError::Unsupported),
        Term::Let { name, value, body } => {
            let vty = synthesize(&*value, _env)?;
            let mut env2 = _env.clone();
            env2.insert(name.clone(), vty);
            synthesize(&*body, &env2)
        }
        Term::If {
            condition,
            then_branch,
            else_branch,
        } => {
            check(condition, &Type::Bool, _env)?;
            let t1 = synthesize(then_branch, _env)?;
            let t2 = synthesize(else_branch, _env)?;
            if equal(&t1, &t2) {
                Ok(t1)
            } else {
                Err(TypeError::Mismatch {
                    expected: t1,
                    found: t2,
                })
            }
        }
        Term::Quote(_) | Term::Eval(_) => Err(TypeError::Unsupported),
    }
}

/// TODO (Phase 2): Implement type checking judgment Γ ⊢ e ⇐ A.
/// See `/plans/core_calculus.md` § "Checking Judgment".
///
/// Expected behavior:
///   - For Lambda(x, body), A = Pi(x, A1, A2): check body ⇐ A2[x:=fresh_var]
///   - For Pair(a, b), A = Sigma(x, A1, A2): check a ⇐ A1, check b ⇐ A2[a/x]
///   - For other e, A: synthesize e ⇒ B, check B ≡ A (via `normalize`)
///
/// # Test:
///   - `test_check_lambda_pi`: ∅ ⊢ λx:Int.x ⇐ Int → Int
///   - `test_check_int_int_ok`: ∅ ⊢ 42 ⇐ Int
///   - `test_check_int_bool_fail`: ∅ ⊢ 42 ⇐ Bool → Mismatch error
fn check(_term: &Term, _ty: &Type, _env: &TypeEnv) -> Result<(), TypeError> {
    match (_term, _ty) {
        (Term::Int(_), Type::Int) => Ok(()),
        (Term::Bool(_), Type::Bool) => Ok(()),
        (
            Term::Lambda { binder, body },
            Type::Pi {
                binder: pbind,
                body: pbody,
            },
        ) => {
            // check that parameter types are compatible
            if !equal(&*binder.ty, &*pbind.ty) {
                return Err(TypeError::Mismatch {
                    expected: (*pbind.ty).clone(),
                    found: (*binder.ty).clone(),
                });
            }
            // extend env and check body against pbody
            let mut env2 = _env.clone();
            env2.insert(binder.name.clone(), (*binder.ty).clone());
            check(&*body, &*pbody, &env2)
        }
        (..) => {
            // fallback: try to synthesize and compare
            let inferred = synthesize(_term, _env)?;
            if equal(&inferred, _ty) {
                Ok(())
            } else {
                Err(TypeError::Mismatch {
                    expected: _ty.clone(),
                    found: inferred,
                })
            }
        }
    }
}

/// TODO (Phase 2): Implement definitional equality check A ≡ B.
/// Uses NbE normalization: A ≡ B iff normalize(A) = normalize(B).
/// See `/plans/core_calculus.md` § "Definitional Equality".
fn equal(_a: &Type, _b: &Type) -> bool {
    // crude structural equality with normalization for Term-like parts
    if _a == _b {
        return true;
    }
    // For refinements and quoted terms, attempt normalization of inner terms
    match (_a, _b) {
        (
            Type::Refine {
                binder: ba,
                predicate: pa,
            },
            Type::Refine {
                binder: bb,
                predicate: pb,
            },
        ) => {
            // compare base types and normalized predicates as terms
            if ba.ty != bb.ty {
                return false;
            }
            // TODO: convert Expr -> Term or evaluate predicates via SMT; for now structural compare
            pa == pb
        }
        _ => false,
    }
}
