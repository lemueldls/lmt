use std::{
    // collections::HashMap,
    fs,
    // hash::{DefaultHasher, Hash, Hasher},
    path::{Path, PathBuf},
};

use cvc5_rs::{Kind, Solver, Term, TermManager};
use lmt_number::{LmtDecimal, LmtInteger, num_traits::ToPrimitive};
use lmt_parser::{Expr, Literal, TypeAnnotation};
// use ahash::{AHashMap, AHasher};
// use ariadne::{sources, Color, Label, Report, ReportKind};
// use directories::ProjectDirs;
// use hashbrown::Equivalent;
use lmt_report::miette::{self, IntoDiagnostic, NamedSource};
use lmt_synthesis::{
    Graph, ModuleId, ModuleSynthesis, Primitive, Proof, StaticSynthesis, SynTypeKind,
};

fn main() -> miette::Result<()> {
    miette::set_panic_hook();

    let synthesis = StaticSynthesis::default();
    // let filename = env::args().nth(1).expect("no file given");

    let path = "examples/test.lmt";
    let src = fs::read_to_string(path).into_diagnostic()?;

    let (module_id, errors) = synthesis.load_module(path, &src);
    let module = synthesis.module_synthesis_map.get(module_id);

    for error in errors {
        eprintln!(
            "{:?}",
            error.report_with_source(path.to_string(), src.to_string())
        )
    }

    let tm = TermManager::new();
    let mut solver = Solver::new(&tm);

    solver.set_logic("ALL");
    solver.set_option("produce-models", "true");
    solver.set_option("sygus", "true");
    // solver.set_option("produce-proofs", "true");
    // solver.set_option("proof-mode", "full-proof");

    // let int_sort = tm.integer_sort();
    // let x = tm.mk_const(int_sort, "x");
    // let zero = tm.mk_integer(0);

    // let gt = tm.mk_term(Kind::CVC5_KIND_GT, &[x.clone(), zero.clone()]);
    // solver.assert_formula(gt);
    // let lt = tm.mk_term(Kind::CVC5_KIND_LT, &[x.clone(), zero]);
    // solver.assert_formula(lt);

    let types = synthesis.context.type_map.into_iter();

    for (i, r#type) in types.enumerate() {
        let name = format!("v{}", i);
        let term = proof_to_term(r#type.proof, &name, &tm);

        solver.assert_formula(term);
    }

    dbg!(solver.check_sat());

    dbg!(solver.check_synth());

    // let proof = solver.get_proof(cvc5_rs::ProofComponent::CVC5_PROOF_COMPONENT_FULL);
    // dbg!(proof);

    Ok(())
}

fn proof_to_term(proof: Proof, name: &str, tm: &TermManager) -> Term {
    match proof {
        Proof::And(lhs, rhs) => {
            let lhs_term = proof_to_term(*lhs, name, tm);
            let rhs_term = proof_to_term(*rhs, name, tm);

            tm.mk_term(Kind::CVC5_KIND_AND, &[lhs_term, rhs_term])
        }
        Proof::Or(lhs, rhs) => {
            let lhs_term = proof_to_term(*lhs, name, tm);
            let rhs_term = proof_to_term(*rhs, name, tm);

            tm.mk_term(Kind::CVC5_KIND_OR, &[lhs_term, rhs_term])
        }
        Proof::EqualTo(spanned) => {
            let kind = syn_type_kind_to_cvc5_term(&spanned.inner, tm);

            let var = tm.mk_const(kind.sort(), name);
            tm.mk_term(Kind::CVC5_KIND_EQUAL, &[var, kind])
            // tm.mk_term(Kind::CVC5_KIND_EQUAL, &[var.clone(), kind])
        }
        Proof::NotEqualTo(spanned) => {
            let term = syn_type_kind_to_cvc5_term(&spanned.inner, tm);

            let var = tm.mk_const(term.sort(), name);
            tm.mk_term(Kind::CVC5_KIND_DISTINCT, &[var, term])
            // tm.mk_term(Kind::CVC5_KIND_NOT, &[var.clone(), term])
        }
        Proof::LessThan(spanned) => {
            let term = syn_type_kind_to_cvc5_term(&spanned.inner, tm);

            let var = tm.mk_const(term.sort(), name);
            tm.mk_term(Kind::CVC5_KIND_LT, &[var, term])
        }
        Proof::GreaterThan(spanned) => {
            let term = syn_type_kind_to_cvc5_term(&spanned.inner, tm);

            let var = tm.mk_const(term.sort(), name);
            tm.mk_term(Kind::CVC5_KIND_GT, &[var, term])
        }
    }
}

fn syn_type_kind_to_cvc5_term(sym_type_kind: &SynTypeKind, tm: &TermManager) -> Term {
    match sym_type_kind {
        SynTypeKind::Unknown => tm.mk_anonymous_const(tm.mk_anonymous_uninterpreted_sort()),
        SynTypeKind::Constant(literal) => literal_to_cvc5_term(literal, tm),
        SynTypeKind::Primitive(primitive) => {
            let sort = match primitive {
                Primitive::Integer => tm.integer_sort(),
                Primitive::Decimal => tm.real_sort(),
                Primitive::String => tm.string_sort(),
                Primitive::Boolean => tm.boolean_sort(),
            };

            tm.mk_anonymous_const(sort)
        }
        SynTypeKind::List(syn_types) => {
            todo!()
        }
        SynTypeKind::Function(function) => {
            let domain_terms = function.signature.args.iter().map(|arg| {
                match &arg.type_ann {
                    Some(type_ann) => type_ann_to_cvc5_term(type_ann, tm),
                    None => tm.mk_anonymous_const(tm.mk_anonymous_uninterpreted_sort()),
                }
            });
            let mut domain_sorts = domain_terms.map(|term| term.sort()).collect::<Vec<_>>();

            if domain_sorts.is_empty() {
                domain_sorts.push(tm.mk_anonymous_uninterpreted_sort());
            }

            let codomain_sort = tm.mk_anonymous_uninterpreted_sort();

            let sort = tm.mk_fun_sort(&domain_sorts, codomain_sort);
            let term = tm.mk_const(sort, &function.signature.name);

            term
        }
    }
}

fn type_ann_to_cvc5_term(type_ann: &TypeAnnotation, tm: &TermManager) -> Term {
    match type_ann {
        TypeAnnotation::Expr(expr) => expr_to_cvc5_term(expr, tm),
        TypeAnnotation::Comparison(binary_op, expr) => todo!(),
        TypeAnnotation::And(type_annotation, type_annotation1) => todo!(),
    }
}

fn expr_to_cvc5_term(expr: &Expr, tm: &TermManager) -> Term {
    match expr {
        Expr::Error => todo!(),
        Expr::Literal(literal) => literal_to_cvc5_term(literal, tm),
        Expr::Ident(spanned) => todo!(),
        Expr::Block(block) => todo!(),
        Expr::Match(match_expr) => todo!(),
        Expr::List(spanneds) => todo!(),
        Expr::Binary(spanned, binary_op, spanned1) => todo!(),
        Expr::Call(spanned, spanned1) => todo!(),
    }
}

fn literal_to_cvc5_term(literal: &Literal, tm: &TermManager) -> Term {
    match literal {
        Literal::Boolean(value) => tm.mk_boolean(*value),
        Literal::Integer(lmt_integer) => tm.mk_integer(lmt_integer.value.to_i64().unwrap()),
        Literal::Decimal(lmt_decimal) => tm.mk_real(lmt_decimal.value.to_i64().unwrap()),
        Literal::String(string) => tm.mk_string(string, false),
        Literal::Nothing => tm.mk_anonymous_const(tm.mk_anonymous_uninterpreted_sort()),
    }
}
