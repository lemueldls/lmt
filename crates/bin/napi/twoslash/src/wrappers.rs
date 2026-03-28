use std::ops;

use napi_derive::napi;
// use serde::{Deserialize, Serialize};
// use tsify::Tsify;
// use wasm_bindgen::prelude::*;

#[napi(object)]
// #[derive(Tsify, Serialize, Deserialize, Clone, Hash)]
// #[tsify(into_wasm_abi)]
pub struct Span {
    pub start: u32,
    pub end: u32,
}

impl From<ops::Range<usize>> for Span {
    fn from(range: ops::Range<usize>) -> Self {
        Self {
            start: range.start as u32,
            end: range.end as u32,
        }
    }
}

impl From<lmt_parser::SimpleSpan> for Span {
    fn from(span: lmt_parser::SimpleSpan) -> Self {
        Self {
            start: span.start as u32,
            end: span.end as u32,
        }
    }
}

impl From<&lmt_parser::SimpleSpan> for Span {
    fn from(span: &lmt_parser::SimpleSpan) -> Self {
        Self {
            start: span.start as u32,
            end: span.end as u32,
        }
    }
}
