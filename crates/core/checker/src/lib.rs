use anyhow::Result;
use cvc5_rs::{Kind, Solver, TermManager};

pub fn check_simple_arithmetic() -> Result<()> {
    let tm = TermManager::new();
    let mut solver = Solver::new(&tm);

    solver.set_logic("QF_LIA");
    solver.set_option("produce-models", "true");

    let integer_sort = tm.integer_sort();
    let x = tm.mk_const(integer_sort, "x");
    let ten = tm.mk_integer(10);

    // x > 10
    let assertion = tm.mk_term(Kind::CVC5_KIND_GT, &[x.clone(), ten]);
    solver.assert_formula(assertion);

    let result = solver.check_sat();
    if result.is_sat() {
        println!("Satisfiable!");
        let x_val = solver.get_value(x);
        println!("x = {}", x_val);
    } else {
        println!("Unsatisfiable!");
    }

    Ok(())
}

pub fn check() -> Result<()> {
    check_simple_arithmetic()
}
