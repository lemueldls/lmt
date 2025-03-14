pub use miette;
use miette::{Diagnostic, NamedSource, Report, SourceCode, SourceSpan};
pub use thiserror;
use thiserror::Error;

pub type Result<T> = core::result::Result<T, SynthesisReport>;

#[derive(Debug, Error, Diagnostic)]
pub enum SynthesisReport {
    #[error("tokenize error")]
    #[diagnostic(severity(Error))]
    TokenizeError {
        reason: String,
        #[label("{reason}")]
        span: SourceSpan,
    },

    #[error("{ident} is already defined")]
    #[diagnostic(severity(Error))]
    AlreadyDefined {
        ident: String,
        #[label]
        redefined_span: SourceSpan,
        originally_defined_span: SourceSpan,
    },
}

impl SynthesisReport {
    #[must_use]
    pub fn report_with_source(self, name: String, source: String) -> Report {
        Report::new(self).with_source_code(NamedSource::new(name, source).with_language("lmt"))
    }
}
