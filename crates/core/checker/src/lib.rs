use anyhow::Result;
use cvc5_rs::{Kind, Solver, TermManager};
mod expr_conv;
mod refinement;
mod solver;

use std::path::Path;

use anyhow::Context;
use lmt_parser::{StructuralMapper, parser::Parser};

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

pub fn check_path(path: &str) -> Result<()> {
    let path = Path::new(path);
    if path.is_file() {
        return check_file(path);
    }

    if path.is_dir() {
        for file in collect_supported_files(path)? {
            check_file(&file)?;
        }
        return Ok(());
    }

    Err(anyhow::anyhow!("path does not exist: {}", path.display()))
}

pub fn check_contract_spec(spec: &str) -> Result<()> {
    let mut parser = Parser::new(spec);
    let contract = parser.parse_function_contract();
    refinement::check_contract_consistency(&contract)
}

fn check_file(path: &Path) -> Result<()> {
    let mapper = StructuralMapper::new();
    let mappings = mapper
        .map_file_items(path)
        .with_context(|| format!("failed to map file {}", path.display()))?;

    let mut env = refinement::VerificationEnv::new();

    for mapping in mappings {
        refinement::check_spec_item(&mapping.item, &mut env).with_context(|| {
            let location = mapping
                .target_range
                .as_ref()
                .map(|range| format!("bytes {}..{}", range.start, range.end))
                .unwrap_or_else(|| "top-level item".to_string());
            format!(
                "verification failed for {} in {} ({})",
                location,
                path.display(),
                item_summary(&mapping.item)
            )
        })?;
    }

    Ok(())
}

fn item_summary(item: &lmt_parser::SpecItem) -> String {
    match item {
        lmt_parser::SpecItem::TypeAlias(alias) => format!("type alias `{}`", alias.name),
        lmt_parser::SpecItem::FunctionContract(contract) => {
            format!("function contract `{}`", contract.name)
        }
        lmt_parser::SpecItem::Assertion(_) => "assertion".to_string(),
    }
}

fn collect_supported_files(root: &Path) -> Result<Vec<std::path::PathBuf>> {
    let mut out = Vec::new();
    collect_supported_files_impl(root, &mut out)?;
    Ok(out)
}

fn collect_supported_files_impl(dir: &Path, out: &mut Vec<std::path::PathBuf>) -> Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_supported_files_impl(&path, out)?;
        } else if is_supported_source_file(&path) {
            out.push(path);
        }
    }

    Ok(())
}

fn is_supported_source_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("rs" | "py" | "js" | "ts" | "lmt")
    )
}

#[cfg(test)]
mod tests {
    use lmt_parser::ast::{BinOp, Expr, Lit};

    use super::*;

    #[test]
    fn test_expr_conv_term_check() {
        let expr = Expr::Binary {
            left: Box::new(Expr::Var("x".to_string())),
            op: BinOp::Gt,
            right: Box::new(Expr::Literal(Lit::Int(10))),
        };

        let tm = TermManager::new();
        let x = tm.mk_const(tm.integer_sort(), "x");
        let vars = std::collections::HashMap::from([("x".to_string(), x)]);
        let term = crate::expr_conv::expr_to_term(&tm, &expr, &vars).expect("term conversion");
        assert_eq!(term.kind(), Kind::CVC5_KIND_GT);
    }

    #[test]
    fn test_check_contract_spec_ok() {
        let spec = "fn id_pos(x: { v: Int | v > 0 }) -> { v: Int | v > 0 } @post v > 0";
        check_contract_spec(spec).expect("expected consistent contract");
    }

    #[test]
    fn test_check_path_lmt_program() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../parser/tests/fixtures/simple.lmt");
        check_path(path.to_str().expect("valid path")).expect("expected lmt program to check");
    }
}
