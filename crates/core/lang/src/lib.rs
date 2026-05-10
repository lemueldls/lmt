//! # LMT Language Core
//!
//! This crate implements the core calculus for the LMT language as specified in
//! `/plans/core_calculus.md`. It provides:
//!
//! - **Syntax** (`syntax.rs`): Core term AST, types, and universe stratification
//! - **Evaluation** (`eval.rs`): Normalization-by-evaluation (NbE) for type equality
//! - **Type Checking** (`typecheck.rs`): Bidirectional type checker with SMT-backed refinements
//!
//! # Implementation Phases
//!
//! **Phase 2** (Core Evaluator): Basic NbE without dependent types
//! **Phase 3** (Bidirectional Checker): Full type checking with Pi/Sigma and refinements
//! **Phase 4** (Advanced): Reflection, termination checking, advanced tactics
//!
//! See `/plans/core_calculus.md` and `/docs/LMT_STATIC_ANALYSIS.md` for formal specifications.

pub mod eval;
pub mod syntax;
pub mod typecheck;

pub use eval::{Neutral, Value, eval, normalize, reflect, reify};
pub use syntax::{Binder, Expr, Program, Statement, Term, Type, UniverseLevel};
pub use typecheck::{TypeEnv, TypeError, TypeResult, type_check};
