//! lmt-diagnostics - Diagnostic reporting and module management for LMT

pub mod graph;
mod index;
pub mod report;
pub mod source;
mod span;

pub use index::{ModuleId, ModuleMap, SecondaryModuleMap};
pub use span::Span;

facet::define_attr_grammar! {
    ns "diagnostics";
    crate_path ::lmt_diagnostics;

    pub enum Attr {
        Label(&'static str),
        Help(&'static str),
        Severity(&'static str),
    }
}
