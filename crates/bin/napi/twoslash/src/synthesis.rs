use std::{collections::HashMap, ops::Range};

use lmt_parser::{Input, Parser, SimpleSpan};
use lmt_report::miette::{self, LabeledSpan};
use lmt_synthesis::{ModuleId, StaticSynthesis};
use napi_derive::napi;
use ropey::Rope;

// use serde::{Deserialize, Serialize};
// use tsify::Tsify;
// use wasm_bindgen::prelude::*;
use crate::wrappers;

#[napi(object)]
// #[derive(Tsify, Serialize)]
// #[tsify(into_wasm_abi)]
pub struct Synthesis {
    pub hovers: Vec<NodeHover>,
    pub errors: Vec<NodeError>,
}

// pub enum Node {
//     Highlight(NodeHighlight),
//     Hover(NodeHover),
//     Query(NodeQuery),
//     Completion(NodeCompletion),
//     Error(NodeError),
//     Tag(NodeTag),
// }

#[napi(object)]
// #[derive(Tsify, Serialize)]
// #[tsify(into_wasm_abi)]
pub struct NodeHover {
    #[napi(ts_type = "'hover'")]
    pub r#type: String,

    pub docs: Option<String>,
    pub tags: Option<()>,
    pub target: String,
    pub text: String,

    pub start: u32,
    pub line: u32,
    pub character: u32,
    pub length: u32,
}

#[napi(object)]
// #[derive(Tsify, Serialize)]
// #[tsify(into_wasm_abi)]
pub struct NodeError {
    #[napi(ts_type = "'error'")]
    pub r#type: String,

    pub id: Option<String>,
    #[napi(ts_type = "'warning' | 'error' | 'suggestion' | 'message'")]
    pub level: String,
    // pub code: Option<number | string>
    pub text: String,
    pub filename: String,

    pub start: u32,
    pub line: u32,
    pub character: u32,
    pub length: u32,
}

#[napi(object)]
// #[derive(Tsify, Serialize, Clone)]
// #[tsify(into_wasm_abi)]
pub struct SynthesisError {
    pub message: String,
    pub span: wrappers::Span,
}

#[napi(js_name = "synthesize")]
// #[wasm_bindgen(js_name = "synthesize")]
pub fn standalone_synthesis(src: String) -> Synthesis {
    let filename = "examples/test.lmt";

    let synthesis = StaticSynthesis::default();

    let (module_id, errors) = synthesis.load_module(&filename, &src);

    // let module = synthesis.module_synthesis_map.get(module_id);

    // let parse_errors = errors
    //     .into_iter()
    //     .map(|err| err.map_token(|ch| ch.to_string()))
    //     .chain(
    //         tokenize_errors
    //             .into_iter()
    //             .map(|err| err.map_token(|token| token.to_string())),
    //     );

    let rope = Rope::from_str(&src);

    let hovers: Vec<NodeHover> = synthesis
        .context
        .spanned_proofs
        .read()
        .iter()
        .map(|(span, type_id)| {
            let (start, line, character, length) = span_to_node(&rope, &span);

            let docs = None;
            let tags = None;
            let target = rope.byte_slice(span.start..span.end).to_string();
            let ty = synthesis.context.get_type_from_id(*type_id);
            let text = ty.proof.to_string();

            NodeHover {
                r#type: "hover".to_string(),

                docs,
                tags,
                target,
                text,

                start,
                line,
                character,
                length,
            }
        })
        .collect();

    let errors = errors
        .into_iter()
        .flat_map(|err| {
            let report = err.report_with_source(filename.to_string(), src.to_string());

            let level = match report.severity().unwrap() {
                miette::Severity::Advice => "suggestion",
                miette::Severity::Warning => "warning",
                miette::Severity::Error => "error",
            }
            .to_string();

            let nodes: Vec<NodeError> = report
                .labels()
                .unwrap()
                .map(|labeled_span| {
                    let (start, line, character, length) =
                        labeled_span_to_node(&rope, &labeled_span);

                    NodeError {
                        r#type: "error".to_string(),

                        id: None,
                        level: level.clone(),
                        filename: filename.to_string(),
                        text: labeled_span.label().unwrap().to_string(),

                        start,
                        line,
                        character,
                        length,
                    }
                })
                .collect();

            nodes
        })
        .collect();

    Synthesis { hovers, errors }
}

fn span_to_node(rope: &Rope, span: &SimpleSpan<usize, ModuleId>) -> (u32, u32, u32, u32) {
    let start = span.start as u32;

    let line_idx = rope.byte_to_line(span.start);
    let line = line_idx as u32;

    let character = start - rope.line_to_byte(line_idx) as u32;
    let length = (span.end - span.start) as u32;

    (start, line, character, length)
}

fn labeled_span_to_node(rope: &Rope, labeled_span: &LabeledSpan) -> (u32, u32, u32, u32) {
    let offset = labeled_span.offset();
    let start = offset as u32;

    let line_idx = rope.byte_to_line(offset);
    let line = line_idx as u32;

    let character = start - rope.line_to_byte(line_idx) as u32;
    let length = labeled_span.len() as u32;

    (start, line, character, length)
}
