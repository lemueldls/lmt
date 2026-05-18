use lmt_diagnostics::{graph::ModuleGraph, source::HasNamedSourceIngredient};
use lmt_syntax::diagnostic::Diagnostic;
/// Convert checker diagnostics to LSP diagnostics with line/column mapping.
use lsp_types::{Diagnostic as LspDiagnostic, DiagnosticSeverity, Position, Range};

/// Convert checker diagnostics to LSP format with proper byte→line/column mapping.
pub fn to_lsp_diagnostics<DB: HasNamedSourceIngredient, G: ModuleGraph>(
    db: &DB,
    graph: &G,
    text: &str,
    source_diagnostics: Vec<Diagnostic>,
) -> Vec<LspDiagnostic> {
    let lines: Vec<&str> = text.lines().collect();

    source_diagnostics
        .into_iter()
        .filter_map(|diag| {
            let report = diag.to_report(db, graph);

            Some(LspDiagnostic {
                range: byte_range_to_lsp_range(
                    text,
                    report.span.start()?,
                    report.span.end()?,
                    &lines,
                ),
                severity: Some(DiagnosticSeverity::ERROR),
                code: None,
                source: Some("lmt".to_string()),
                message: report.message,
                related_information: None,
                tags: None,
                code_description: None,
                data: None,
            })
        })
        .collect()
}

/// Convert byte offsets to LSP line/column range.
fn byte_range_to_lsp_range(text: &str, start: usize, end: usize, lines: &[&str]) -> Range {
    Range {
        start: byte_to_position(text, start, lines),
        end: byte_to_position(text, end, lines),
    }
}

/// Convert byte offset to LSP Position (line/character).
fn byte_to_position(text: &str, offset: usize, lines: &[&str]) -> Position {
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
