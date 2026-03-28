use std::{collections::HashMap, ops::Range};

use lmt_parser::{Input, Parser, SimpleSpan};
use lmt_synthesis::ModuleSynthesis;
// use serde::{Deserialize, Serialize};
// use tsify::Tsify;
use wasm_bindgen::prelude::*;

// #[derive(Tsify, Serialize)]
#[tsify(into_wasm_abi)]
#[serde(rename = "Synthesis")]
pub struct SynthesisWrapper {
    types: HashMap<Range<usize>, Box<str>>,
    errors: Vec<SynthesisError>,
}

// #[derive(Tsify, Serialize)]
#[tsify(into_wasm_abi)]
pub struct SynthesisError {
    message: Box<str>,
    span: Range<usize>,
}

#[wasm_bindgen(js_name = "synthesize")]
pub fn standalone_synthesis(src: &str) -> SynthesisWrapper {
    // let filename = env::args().nth(1).expect("no file given");
    let filename = "examples/test.lmt";

    let (tokens, errs) = lmt_parser::lexer().parse(src).into_output_errors();

    let (synthesis, tokenize_errors) = if let Some(tokens) = tokens.as_ref() {
        let len = src.chars().count();
        let (ast, parse_errs) = lmt_parser::module_parser()
            .parse(tokens.spanned((len..len).into()))
            .into_output_errors();

        let synthesis = ast.map(lmt_synthesis::synthesize);

        (synthesis, parse_errs)
    } else {
        (None, Vec::new())
    };

    let parse_errors = errs
        .into_iter()
        .map(|err| err.map_token(|ch| ch.to_string()))
        .chain(
            tokenize_errors
                .into_iter()
                .map(|err| err.map_token(|token| token.to_string())),
        );

    // for err in parse_errors {
    //     err.print(sources([(filename, src)])).unwrap()
    // }

    let types = synthesis
        .map(|synthesis| {
            synthesis
                .ident_types
                .into_iter()
                .map(|(span, ty)| {
                    (
                        span.into_range(),
                        ty.borrow().proof.to_string().into_boxed_str(),
                    )
                })
                .collect()
        })
        .unwrap_or_default();

    let errors = parse_errors
        .map(|err| {
            SynthesisError {
                message: err.to_string().into_boxed_str(),
                span: err.span().into_range(),
            }
        })
        .collect();

    SynthesisWrapper { types, errors }
}
