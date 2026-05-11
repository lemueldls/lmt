/// Convert checker diagnostics to LSP diagnostics with line/column mapping.
use lsp_types::{Diagnostic as LspDiagnostic, DiagnosticSeverity, Position, Range};

/// Convert checker diagnostics to LSP format with proper byte→line/column mapping.
pub fn to_lsp_diagnostics(
    text: &str,
    checker_diags: Vec<lmt_checker::Diagnostic>,
) -> Vec<LspDiagnostic> {
    let lines: Vec<&str> = text.lines().collect();

    checker_diags
        .into_iter()
        .map(|diag| {
            let range = match (diag.start_byte, diag.end_byte) {
                (Some(start), Some(end)) => byte_range_to_lsp_range(text, start, end, &lines),
                _ => {
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

            LspDiagnostic {
                range,
                severity: Some(DiagnosticSeverity::ERROR),
                code: None,
                source: Some("lmt".to_string()),
                message: diag.message,
                related_information: None,
                tags: None,
                code_description: None,
                data: None,
            }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_byte_to_position() {
        let text = "line1\nline2\nline3";
        let lines: Vec<&str> = text.lines().collect();

        // Start of first line
        let pos = byte_to_position(text, 0, &lines);
        assert_eq!(pos.line, 0);
        assert_eq!(pos.character, 0);

        // Start of second line (after first \n)
        let pos = byte_to_position(text, 6, &lines);
        assert_eq!(pos.line, 1);
        assert_eq!(pos.character, 0);
    }

    #[test]
    fn test_to_lsp_diagnostics() {
        let text = "let x = 1\nlet y = 2";
        let checker_diags = vec![lmt_checker::Diagnostic {
            message: "test error".to_string(),
            start_byte: Some(0),
            end_byte: Some(9),
        }];

        let lsp_diags = to_lsp_diagnostics(text, checker_diags);
        assert_eq!(lsp_diags.len(), 1);
        assert_eq!(lsp_diags[0].message, "test error");
        assert_eq!(lsp_diags[0].range.start.line, 0);
    }
}
