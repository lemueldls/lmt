#[cfg(feature = "cvc5")]
use lmt_solver::Cvc5Solver as ActiveSolver;
#[cfg(not(feature = "cvc5"))]
use lmt_solver::SmtLib2Solver as ActiveSolver;
use lmt_solver::{SmtExpr, SmtSolver};
use lmt_syntax::{
    ast::{Expr, Program, Statement},
    diagnostic::Diagnostic,
};
use picante::PicanteResult;

use crate::types::{Env, Type};

#[picante::db(tracked(infer_expr, check_expr, is_subtype, check_program))]
pub struct CheckerDatabase {}

#[picante::tracked]
#[allow(clippy::needless_pass_by_value)]
pub fn infer_expr<DB: CheckerDatabaseTrait>(
    db: &DB,
    env: Env,
    expr: Expr,
) -> PicanteResult<Result<Type, Diagnostic>> {
    let _ = db;

    let result = match expr {
        Expr::LiteralInt { .. } => Ok(Type::Int),
        Expr::LiteralReal { .. } => Ok(Type::Real),
        Expr::LiteralBool { .. } => Ok(Type::Bool),
        Expr::LiteralString { .. } => Ok(Type::String),
        Expr::Var { name, span } => {
            env.get(&name)
                .ok_or(Diagnostic::VariableNotFound { var: span })
        }
        Expr::Ascription { .. } => {
            // Let's assume right is a Type for now
            // We need a function to evaluate Expr to Type
            // Check left against the type
            Ok(Type::Error)
        }
        _ => {
            // Stub for other expressions
            Ok(Type::Error)
        }
    };

    Ok(result)
}

#[picante::tracked]
pub async fn check_expr<DB: CheckerDatabaseTrait>(
    db: &DB,
    env: Env,
    expr: Expr,
    expected: Type,
) -> PicanteResult<Result<bool, Diagnostic>> {
    // Standard Bidirectional checking
    // E.g., for lambdas, we would decompose the expected Arrow type

    // Fallback to synthesis and subtyping
    let inferred = infer_expr(db, env.clone(), expr).await?;

    match inferred {
        Ok(inferred_ty) => {
            // Check if inferred type is a subtype of expected
            is_subtype(db, env, inferred_ty, expected).await.map(Ok)
        }
        Err(diag) => Ok(Err(diag)),
    }
}

#[picante::tracked]
pub async fn is_subtype<DB: CheckerDatabaseTrait>(
    db: &DB,
    env: Env,
    sub: Type,
    sup: Type,
) -> PicanteResult<bool> {
    // If they are exactly the same type, true
    if sub == sup {
        return Ok(true);
    }

    match (sub, sup) {
        (
            Type::Refinement {
                base: base1,
                binder: b1,
                predicate: p1,
            },
            Type::Refinement {
                base: base2,
                binder: b2,
                predicate: p2,
            },
        ) => {
            // Check base types
            let base_subtype = is_subtype(db, env.clone(), *base1, *base2).await?;
            if !base_subtype {
                return Ok(false);
            }

            // Extract assertions from the environment
            let mut env_assertions = Vec::new();
            for (name, ty) in &env.0 {
                if let Type::Refinement {
                    binder, predicate, ..
                } = ty
                {
                    let var_expr = SmtExpr::var(name);
                    let substituted_pred = predicate.substitute(binder, &var_expr);
                    env_assertions.push(substituted_pred);
                }
            }

            // Generate Verification Condition: \Gamma \wedge p1 \Rightarrow p2[b2/b1]
            // Add p1 to env_assertions (assumed to be true for the sub type)
            env_assertions.push(p1);

            // Substitute b2 with b1 in p2
            let var_b1 = SmtExpr::var(&b1);
            let p2_subst = p2.substitute(&b2, &var_b1);

            // Check validity of p2_subst given the environment assertions
            let mut solver = ActiveSolver::new();
            match solver.check_validity(&env_assertions, &p2_subst) {
                Ok(is_valid) => Ok(is_valid),
                Err(_) => {
                    // Fallback on error (could emit diagnostic instead)
                    Ok(false)
                }
            }
        }
        (Type::Refinement { base: base1, .. }, sup_ty) => {
            // Forget refinement
            is_subtype(db, env, *base1, sup_ty).await
        }
        (
            _sub_ty,
            Type::Refinement {
                base: _base2,
                binder: _b2,
                predicate: _p2,
            },
        ) => {
            // To prove sub_ty <: {v: B | P}, sub_ty <: B and \Gamma |- P[v/sub_ty]
            // We need to evaluate P with the term
            Ok(false)
        }
        _ => Ok(false),
    }
}

#[picante::tracked]
pub async fn check_program<DB: CheckerDatabaseTrait>(
    db: &DB,
    program: Program,
) -> PicanteResult<Result<(), Vec<Diagnostic>>> {
    let mut env = Env::new();
    let mut diagnostics = Vec::new();

    for stmt in program.statements {
        match stmt {
            Statement::Expr(expr) => {
                match infer_expr(db, env.clone(), expr).await? {
                    Ok(_) => {}
                    Err(d) => diagnostics.push(d),
                }
            }
            Statement::Let(decl) => {
                if let Some(val_expr) = decl.value {
                    match infer_expr(db, env.clone(), val_expr).await? {
                        Ok(ty) => {
                            env = env.extend(decl.name, ty);
                        }
                        Err(d) => diagnostics.push(d),
                    }
                }
            }
            Statement::Use(..) => todo!(),
        }
    }

    if diagnostics.is_empty() {
        Ok(Ok(()))
    } else {
        Ok(Err(diagnostics))
    }
}
