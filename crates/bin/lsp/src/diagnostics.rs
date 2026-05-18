use lmt_diagnostics::{
    Span,
    graph::ModuleGraph,
    report::{Report, ReportRelatedInformation, ReportSeverity, ReportTag},
    source::HasNamedSourceIngredient,
};
use lsp_types::{
    Diagnostic as LspDiagnostic, DiagnosticRelatedInformation, DiagnosticSeverity, Location,
    Position, Range, Url,
};

/// Convert structured reports to LSP format with byte→line/column mapping.
pub fn to_lsp_diagnostics<DB: HasNamedSourceIngredient, G: ModuleGraph>(
    db: &DB,
    graph: &G,
    reports: Vec<Report>,
) -> Vec<LspDiagnostic> {
    reports
        .into_iter()
        .map(|report| report_to_lsp_diagnostic(db, graph, report))
        .collect()
}

fn report_to_lsp_diagnostic<DB: HasNamedSourceIngredient, G: ModuleGraph>(
    db: &DB,
    graph: &G,
    report: Report,
) -> LspDiagnostic {
    let (text, lines): (String, Vec<String>) = match report.span {
        Span::Known { module_id, .. } => {
            let source = graph.get(module_id);
            let content = source.content(db).unwrap().to_string();
            let lines = content
                .lines()
                .map(|line| line.to_string())
                .collect::<Vec<_>>();

            (content, lines)
        }
        Span::Unknown => (String::new(), Vec::new()),
    };

    let range = match report.span {
        Span::Known { start, end, .. } => byte_range_to_lsp_range(&text, start, end, &lines),
        Span::Unknown => {
            Range {
                start: Position {
                    line: 0,
                    character: 0,
                },
                end: Position {
                    line: 0,
                    character: 0,
                },
            }
        }
    };

    let severity = report.severity.as_ref().map(|severity| {
        match severity {
            ReportSeverity::Error => DiagnosticSeverity::ERROR,
            ReportSeverity::Warning => DiagnosticSeverity::WARNING,
            ReportSeverity::Information => DiagnosticSeverity::INFORMATION,
            ReportSeverity::Hint => DiagnosticSeverity::HINT,
        }
    });

    let related_information = report
        .related_information
        .as_ref()
        .map(|items| {
            items
                .iter()
                .filter_map(|item| report_related_information_to_lsp(db, graph, item))
                .collect::<Vec<_>>()
        })
        .filter(|items| !items.is_empty());

    let message = if let Some(help) = &report.help {
        format!("{}\n{}", report.message, help)
    } else {
        report.message
    };

    LspDiagnostic {
        range,
        severity,
        code: report.code.clone().map(lsp_types::NumberOrString::String),
        source: Some("lmt".to_string()),
        message,
        related_information,
        tags: report.tags.as_ref().map(|tags| {
            tags.iter()
                .filter_map(|tag| {
                    match tag {
                        ReportTag::Unnecessary => Some(lsp_types::DiagnosticTag::UNNECESSARY),
                        ReportTag::Deprecated => Some(lsp_types::DiagnosticTag::DEPRECATED),
                    }
                })
                .collect()
        }),
        code_description: None,
        data: None,
    }
}

fn report_related_information_to_lsp<DB: HasNamedSourceIngredient, G: ModuleGraph>(
    db: &DB,
    graph: &G,
    item: &ReportRelatedInformation,
) -> Option<DiagnosticRelatedInformation> {
    let location = match item.span {
        Span::Known {
            start,
            end,
            module_id,
        } => {
            let source = graph.get(module_id);
            let uri = Url::parse(&source.name(db).ok()?.to_string()).ok()?;
            let content = source.content(db).unwrap().to_string();
            let lines = content
                .lines()
                .map(|line| line.to_string())
                .collect::<Vec<_>>();
            let range = byte_range_to_lsp_range(&content, start, end, &lines);

            Location::new(uri, range)
        }
        Span::Unknown => return None,
    };

    Some(DiagnosticRelatedInformation {
        location,
        message: item.message.clone(),
    })
}

/// Convert byte offsets to LSP line/column range.
fn byte_range_to_lsp_range(text: &str, start: usize, end: usize, lines: &[String]) -> Range {
    Range {
        start: byte_to_position(text, start, lines),
        end: byte_to_position(text, end, lines),
    }
}

/// Convert byte offset to LSP Position (line/character).
fn byte_to_position(text: &str, offset: usize, lines: &[String]) -> Position {
    let offset = offset.min(text.len());
    let mut byte_count = 0;

    for (line_idx, line) in lines.iter().enumerate() {
        let line_len = line.len() + 1; // +1 for newline
        if byte_count + line_len > offset {
            let char_idx = offset.saturating_sub(byte_count);
            return Position {
                line: line_idx as u32,
                character: char_idx as u32,
            };
        }
        byte_count += line_len;
    }

    // Fallback to EOF
    Position {
        line: (lines.len().saturating_sub(1)) as u32,
        character: lines.last().map(|l| l.len()).unwrap_or(0) as u32,
    }
}
