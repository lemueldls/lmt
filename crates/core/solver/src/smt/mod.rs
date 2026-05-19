#[cfg(feature = "cvc5")]
pub mod cvc5;
pub mod smtlib2;

#[cfg(feature = "cvc5")]
pub use cvc5::Cvc5Solver;
use facet::Facet;
pub use smtlib2::SmtLib2Solver;

use crate::ir::SmtExpr;

pub trait SmtSolver {
    /// Checks the validity of a generated VC, given an environment of assertions.
    fn check_validity(&mut self, env: &[SmtExpr], vc: &SmtExpr) -> Result<bool, SolverError>;

    /// Checks the satisfiability of the given assertions.
    fn check_sat(&mut self, assertions: &[SmtExpr]) -> Result<bool, SolverError>;
}

#[repr(u8)]
#[derive(Facet, Debug)]
#[facet(derive(Error))]
pub enum SolverError {
    /// Solver encountered an error: {0}
    InternalError(String),

    /// Unsupported operation: {0}
    Unsupported(String),
}
