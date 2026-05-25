mod printer;
mod renderer;

use std::fmt;

use facet::{Facet, PtrConst, Shape};
use facet_reflect::{HasFields, Peek};
use line_col::LineColLookup;
pub use printer::print;
pub use renderer::{ReportRender, ReportRenderItem};

use crate::{ModuleId, Span, graph::ModuleGraph, source::HasNamedSourceIngredient};

pub struct FieldDisplay {
    shape: &'static Shape,
    ptr: PtrConst,
}

impl fmt::Display for FieldDisplay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // SAFETY: The caller must guarantee that the pointer is valid for the shape.
        let x = unsafe { self.shape.call_display(self.ptr, f) };

        match x {
            Some(result) => result,
            None => todo!(),
        }
    }
}

// Span and field metadata used by the renderer
#[derive(Debug, Clone)]
pub struct FieldSpan {
    pub module_id: ModuleId,
    pub file_name: String,
    pub content: String, // TODO: Only store the relevant portion of the content
    pub start: usize,
    pub end: usize,
    pub start_line: usize,
    pub start_col: usize,
    pub end_line: usize,
    pub end_col: usize,
}

#[derive(Debug, Clone)]
pub struct FieldInfo {
    pub name: String,
    pub index: usize,
    pub span: Option<FieldSpan>,
    pub display: Option<String>,
    pub label: Option<String>,
}

/// Safely get a substring of `content` by byte offsets `start..end`.
/// If the byte indices are not on UTF-8 char boundaries this will
/// fallback to the nearest valid boundaries. Returns an owned String.
#[must_use]
pub fn safe_excerpt(content: &str, start: usize, end: usize) -> String {
    if start >= end || start >= content.len() {
        return String::new();
    }

    if let Some(s) = content.get(start..end) {
        return s.to_string();
    }

    // find nearest char boundary >= start and <= end
    let s_idx = content
        .char_indices()
        .find(|&(i, _)| i >= start)
        .map_or(content.len(), |(i, _)| i);
    let e_idx = content
        .char_indices()
        .find(|&(i, _)| i >= end)
        .map_or(content.len(), |(i, _)| i);

    content
        .get(s_idx..e_idx)
        .map(str::to_owned)
        .unwrap_or_default()
}

#[derive(Debug, Clone)]
pub struct Report {
    /// The span at which the message applies.
    pub span: Span,

    /// The report's severity. Can be omitted. If omitted it is up to the
    /// client to interpret reports as error, warning, info or hint.
    pub severity: Option<ReportSeverity>,

    /// The report's message.
    pub message: ReportRender,

    /// Optional help text rendered below the body.
    pub help: Option<ReportRender>,

    /// Structured fields extracted from the diagnostic variant.
    pub fields: Vec<FieldInfo>,

    /// An array of related report information, e.g. when symbol-names within
    /// a scope collide all definitions can be marked via this property.
    pub related_information: Option<Vec<ReportRelatedInformation>>,
}

#[repr(u8)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReportSeverity {
    /// Reports an error.
    Error,
    /// Reports a warning.
    Warning,
    /// Reports an information.
    Information,
    /// Reports a hint.
    Hint,
}

/// Represents a related message and source code location for a diagnostic. This
/// should be used to point to code locations that cause or related to a
/// diagnostics, e.g when duplicating a symbol in a scope.
#[derive(Debug, Clone)]
pub struct ReportRelatedInformation {
    /// The span of this related diagnostic information.
    pub span: Span,

    /// The message of this related diagnostic information.
    pub message: ReportRender,
}

/// Converts a facet-based diagnostic into a [`Report`].
///
/// # Panics
///
/// Panics if the facet cannot be converted into an enum or if the active variant cannot be determined.
pub fn from_diagnostic<'mem, 'facet, T: Facet<'facet> + ?Sized, DB: HasNamedSourceIngredient>(
    t: &'mem T,
    db: &DB,
    graph: &impl ModuleGraph,
) -> Report {
    let peek = Peek::new(t).into_enum().unwrap();
    let variant = peek.active_variant().unwrap();

    let variant_label = variant
        .get_attr(Some("diagnostics"), "label")
        .and_then(|attr| attr.get_as::<&str>())
        .map(<&str>::to_string);

    let variant_help = variant
        .get_attr(Some("diagnostics"), "help")
        .and_then(|attr| attr.get_as::<&str>())
        .map(<&str>::to_string);

    let variant_severity = variant
        .get_attr(Some("diagnostics"), "severity")
        .and_then(|attr| attr.get_as::<&str>())
        .map(<&str>::to_string);

    let mut infos = Vec::new();

    for (field, fpeek) in peek.fields() {
        let name = field.effective_name().to_string();

        let mut span_meta = None;
        if let Ok(span) = fpeek.get::<Span>() {
            match *span {
                Span::Known {
                    start,
                    end,
                    module_id,
                } => {
                    let source = graph.get(module_id);
                    let file_name = source.name(db).unwrap().to_string();
                    let content = source.content(db).unwrap();
                    drop(source);

                    let lookup = LineColLookup::new(&content);
                    let (start_line, start_col) = lookup.get(start);
                    let (end_line, end_col) = lookup.get(end);

                    span_meta = Some(FieldSpan {
                        module_id,
                        file_name,
                        content: content.clone(),
                        start,
                        end,
                        start_line,
                        start_col,
                        end_line,
                        end_col,
                    });
                }
                Span::Unknown => {}
            }
        }

        let shape = field.shape();
        let display = if shape.is_display() {
            Some(
                FieldDisplay {
                    shape,
                    ptr: fpeek.data(),
                }
                .to_string(),
            )
        } else {
            None
        };

        let label_attr = field
            .get_attr(Some("diagnostics"), "label")
            .and_then(|attr| attr.get_as::<&str>())
            .map(<&str>::to_string);

        infos.push(FieldInfo {
            name,
            index: infos.len(),
            span: span_meta,
            display,
            label: label_attr,
        });
    }

    let message = variant_label.as_deref().map_or_else(
        || ReportRender::text("diagnostic"),
        |label| ReportRender::parse_report_attr(label, &infos),
    );

    let help = variant_help
        .as_deref()
        .map(|help| ReportRender::parse_report_attr(help, &infos));

    let severity = variant_severity.as_deref().and_then(|severity| {
        match severity.to_ascii_lowercase().as_str() {
            "error" => Some(ReportSeverity::Error),
            "warning" => Some(ReportSeverity::Warning),
            "information" | "info" => Some(ReportSeverity::Information),
            "hint" => Some(ReportSeverity::Hint),
            _ => None,
        }
    });

    let span = infos
        .iter()
        .find_map(|info| info.span.as_ref())
        .map_or(Span::Unknown, |span| {
            Span::Known {
                start: span.start,
                end: span.end,
                module_id: span.module_id,
            }
        });

    let related_information = {
        let mut related = Vec::new();

        for info in &infos {
            if let Some(field_span) = &info.span {
                if let Span::Known {
                    start: primary_start,
                    end: primary_end,
                    module_id: primary_module_id,
                } = span
                    && field_span.start == primary_start
                    && field_span.end == primary_end
                    && field_span.module_id == primary_module_id
                {
                    continue;
                }

                let message = info
                    .label
                    .as_deref()
                    .map(|label| ReportRender::parse_report_attr(label, &infos))
                    .or_else(|| info.display.clone().map(ReportRender::text))
                    .unwrap_or_else(|| ReportRender::text(info.name.clone()));

                related.push(ReportRelatedInformation {
                    span: Span::Known {
                        start: field_span.start,
                        end: field_span.end,
                        module_id: field_span.module_id,
                    },
                    message,
                });
            }
        }

        if related.is_empty() {
            None
        } else {
            Some(related)
        }
    };

    Report {
        span,
        severity,
        message,
        help,
        fields: infos,
        related_information,
    }
}
