use diag::report;
use facet::Facet;
use lmt_diagnostics::{self as diag, Span, graph::ModuleGraph, source::HasNamedSourceIngredient};

use crate::TokenKind;

#[repr(u8)]
#[derive(Facet, Debug, Clone, PartialEq)]
pub enum Diagnostic {
    #[facet(diag::label("Unexpected token `{kind}`"))]
    UnexpectedToken { kind: TokenKind, span: Span },

    #[facet(diag::label("Unknown token"))]
    UnknownToken { span: Span },

    #[facet(diag::label("Unterminated string literal"))]
    UnterminatedString { span: Span },

    #[facet(diag::label("Invalid integer literal"))]
    InvalidInteger { span: Span },

    #[facet(diag::label("Invalid real literal"))]
    InvalidReal { span: Span },

    #[facet(diag::label("Invalid directive"))]
    InvalidDirective { span: Span },

    #[facet(diag::label("Expected field after `.`"))]
    ExpectedField { span: Span },

    #[facet(diag::label("Expected variant name"))]
    ExpectedVariantName { span: Span },

    #[facet(diag::label("Expected variant in pattern"))]
    ExpectedVariantInPattern { span: Span },

    #[facet(diag::label("Unexpected pattern"))]
    UnexpectedPattern { span: Span },
}

impl Diagnostic {
    pub fn print<DB: HasNamedSourceIngredient>(&self, db: &DB, graph: &impl ModuleGraph) {
        let report = report::from_diagnostic(self, db, graph);
        report::print(&report)
    }

    pub fn to_report<DB: HasNamedSourceIngredient>(
        &self,
        db: &DB,
        graph: &impl ModuleGraph,
    ) -> report::Report {
        report::from_diagnostic(self, db, graph)
    }
}
