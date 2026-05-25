use std::slice;

#[cfg(feature = "cvc5")]
use lmt_solver::Cvc5Solver;
use lmt_solver::{SmtExpr, SmtLib2Solver, SmtOp, SmtSolver as _};

fn main() {
    let x = SmtExpr::var("x");
    let zero = SmtExpr::Int(0);
    let gt = SmtExpr::app(SmtOp::Gt, vec![x, zero]);

    println!("Testing SmtLib2Solver:");
    let mut string_solver = SmtLib2Solver::new();
    let _ = string_solver.check_sat(slice::from_ref(&gt));
    println!("{}", string_solver.output);

    #[cfg(feature = "cvc5")]
    {
        println!("Testing Cvc5Solver:");
        let mut native_solver = Cvc5Solver::new();
        match native_solver.check_sat(&[gt]) {
            Ok(is_sat) => println!("Is sat? {is_sat}"),
            Err(e) => println!("Error: {e}"),
        }
    }
}
