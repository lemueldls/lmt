use core::fmt;
use std::{cmp, iter::Peekable, str::Chars};

use crossterm::style::{Color, Stylize, style};
use facet::{Facet, PtrConst, Shape};
use facet_reflect::{HasFields, Peek};
use line_col::LineColLookup;
use rand::{rng, seq::SliceRandom};

use crate::{ModuleId, Span, graph::ModuleGraph, source::HasNamedSourceIngredient};

fn severity_prefix(severity: Option<&ReportSeverity>) -> &'static str {
    match severity {
        Some(ReportSeverity::Error) => "Error:",
        Some(ReportSeverity::Warning) => "Warning:",
        Some(ReportSeverity::Information) => "Info:",
        Some(ReportSeverity::Hint) => "Hint:",
        None => "Error:",
    }
}

fn severity_prefix_colored(severity: Option<&ReportSeverity>) -> String {
    let (text, color) = match severity {
        Some(ReportSeverity::Error) => ("Error:", Color::Red),
        Some(ReportSeverity::Warning) => ("Warning:", Color::Yellow),
        Some(ReportSeverity::Information) => ("Info:", Color::Blue),
        Some(ReportSeverity::Hint) => ("Hint:", Color::Cyan),
        None => ("Error:", Color::Red),
    };

    format!("{}", style(text).with(color))
}

pub fn print(report: &Report) {
    let mut colors = {
        let mut colors = [Color::Green, Color::Blue, Color::Magenta, Color::Cyan];
        colors.shuffle(&mut rng());
        colors.into_iter().cycle()
    };

    let mut infos = report.fields.clone();
    for info in &mut infos {
        info.color = colors.next().unwrap();
    }

    let mut max_end_line = 0usize;
    for info in &infos {
        if let Some(span) = &info.span {
            max_end_line = cmp::max(max_end_line, span.end_line);
        }
    }

    let alignment = if max_end_line == 0 {
        3
    } else {
        max_end_line.to_string().len() + 1
    };
    let spaces = " ".repeat(alignment);

    if !report.message.is_empty() {
        eprintln!(
            "{} {}",
            severity_prefix_colored(report.severity.as_ref()),
            render_recursive(&report.message, &infos).bold()
        );
    }

    let render_attr = |s: &str| render_recursive(s, &infos);

    for info in &infos {
        if let Some(span) = &info.span {
            let file_name = &span.file_name;
            let content = &span.content;
            let start_line = span.start_line;
            let start_col = span.start_col;
            let end_line = span.end_line;
            let end_col = span.end_col;

            eprintln!(
                "{spaces} ╭─[{}]",
                style(format!(
                    "{}:{}:{}-{}:{}",
                    file_name, start_line, start_col, end_line, end_col
                ))
                .cyan()
                .dim(),
            );

            eprintln!("{spaces} │");

            let line_start = start_line.saturating_sub(1);
            let line_end = end_line.saturating_sub(1);
            let lines = content.lines().collect::<Vec<_>>();
            let col_offset = start_col.saturating_sub(1);
            let mut range_size = span.end.saturating_sub(span.start);

            let mut colored_lines = Vec::new();

            for line in line_start..=line_end {
                let mut line_code = lines.get(line).copied().unwrap_or("").to_string();
                let line_len = line_code.len().saturating_sub(col_offset);
                let end_range = cmp::min(line_len, range_size);
                let line_range = col_offset..(col_offset + end_range);

                if line != line_end {
                    range_size = range_size
                        .saturating_sub(end_range.saturating_sub(col_offset).saturating_sub(1));
                }

                let content_fragment = style_with_color(&line_code[line_range.clone()], info.color);
                let link = format_osc8_link(
                    &format!("{}:{}:{}", file_name, start_line, start_col),
                    &content_fragment,
                );

                line_code.replace_range(line_range, &link);
                colored_lines.push((line, line_code));
            }

            for (i, line) in colored_lines {
                eprintln!("{} │ {}", format!("{:>1$}", i + 1, alignment).dim(), line);
            }

            if let Some(lbl) = &info.label {
                let rendered = render_recursive(lbl, &infos);
                let alignment_col = start_col + (end_col.saturating_sub(start_col)) / 2;
                eprintln!(
                    "{}",
                    rendered
                        .lines()
                        .enumerate()
                        .map(|(i, line)| {
                            if i == 0 {
                                format!(
                                    "{spaces} ╵{}{}",
                                    " ".repeat(alignment_col),
                                    format!("╰╴{}", line).with(info.color)
                                )
                            } else {
                                format!("{spaces} ╵{}{}", " ".repeat(alignment_col + 2), line)
                            }
                        })
                        .collect::<Vec<_>>()
                        .join("\n")
                );
            }

            eprintln!("{spaces} ╵");
        }
    }

    if let Some(help) = &report.help {
        eprintln!("{spaces} ╧ {}", render_attr(help));
    }
}

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
#[derive(Clone)]
pub struct FieldSpan {
    pub module_id: ModuleId,
    pub file_name: String,
    pub content: String,
    pub start: usize,
    pub end: usize,
    pub start_line: usize,
    pub start_col: usize,
    pub end_line: usize,
    pub end_col: usize,
}

#[derive(Clone)]
pub struct FieldInfo {
    pub name: String,
    pub color: Color,
    pub span: Option<FieldSpan>,
    pub display: Option<String>,
    pub label: Option<String>,
}

/// Safely get a substring of `content` by byte offsets `start..end`.
/// If the byte indices are not on UTF-8 char boundaries this will
/// fallback to the nearest valid boundaries. Returns an owned String.
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
        .map(|(i, _)| i)
        .unwrap_or(content.len());
    let e_idx = content
        .char_indices()
        .find(|&(i, _)| i >= end)
        .map(|(i, _)| i)
        .unwrap_or(content.len());

    content
        .get(s_idx..e_idx)
        .map(|s| s.to_string())
        .unwrap_or_default()
}

/// Return a styled string for `text` using the provided `color`.
pub fn style_with_color(text: &str, color: Color) -> String {
    format!("{}", style(text).with(color))
}

/// Produce an OSC 8 hyperlink wrapper to produce clickable
/// links in supporting terminals.
pub fn format_osc8_link(link: &str, content: &str) -> String {
    format!("\u{1b}]8;;{}\u{1b}\\{}\u{1b}]8;;\u{1b}\\", link, content)
}

// Parses `{name[:format]}` and `[text](field)` constructs, allowing nested
// content inside link text. Uses `FieldInfo` to resolve placeholders and
// produce styled links when a span is available.
pub fn render_recursive(src: &str, infos: &Vec<FieldInfo>) -> String {
    fn parse_inner(
        chars: &mut Peekable<Chars<'_>>,
        infos: &Vec<FieldInfo>,
        stop_at_bracket: bool,
    ) -> String {
        let mut out = String::new();

        while let Some(&ch) = chars.peek() {
            if ch == '{' {
                // consume '{'
                chars.next();

                let mut ident = String::new();
                let mut fmt_ = String::new();
                let mut seen_colon = false;

                while let Some(&c) = chars.peek() {
                    chars.next();

                    if c == '}' {
                        break;
                    }

                    if c == ':' && !seen_colon {
                        seen_colon = true;
                        continue;
                    }

                    if seen_colon {
                        fmt_.push(c);
                    } else {
                        ident.push(c);
                    }
                }

                if !ident.is_empty() {
                    if let Some(info) = infos.iter().find(|i| i.name == ident) {
                        if let Some(span) = &info.span {
                            if span.start <= span.end {
                                let excerpt = safe_excerpt(&span.content, span.start, span.end);
                                out.push_str(&format!("`{}`", excerpt));
                            }
                        } else if let Some(disp) = &info.display {
                            out.push_str(disp);
                        }
                    }
                }
            } else if ch == '[' {
                // parse link text
                chars.next(); // consume '['
                let mut inner = String::new();
                while let Some(&c) = chars.peek() {
                    if c == ']' {
                        chars.next(); // consume ']'
                        break;
                    }
                    inner.push_str(&parse_inner(chars, infos, true));
                }

                // now expect '('link')' or just treat as plain text
                if chars.peek() == Some(&'(') {
                    chars.next(); // consume '('
                    let mut link_ident = String::new();
                    while let Some(&c) = chars.peek() {
                        chars.next();
                        if c == ')' {
                            break;
                        }
                        link_ident.push(c);
                    }

                    if let Some(info) = infos.iter().find(|i| i.name == link_ident) {
                        let styled = style_with_color(&inner, info.color);
                        if let Some(span) = &info.span {
                            let link = format!(
                                "{}:{}:{}",
                                span.file_name, span.start_line, span.start_col
                            );
                            out.push_str(&format_osc8_link(&link, &styled));
                        } else {
                            out.push_str(&styled);
                        }
                    } else {
                        out.push_str(&inner);
                    }
                } else {
                    out.push_str(&inner);
                }
            } else if stop_at_bracket && ch == ']' {
                // return to caller to handle closing bracket
                break;
            } else {
                out.push(ch);
                chars.next();
            }
        }

        out
    }

    let mut chars = src.chars().peekable();

    parse_inner(&mut chars, infos, false)
}

#[derive(Clone)]
pub struct Report {
    /// The span at which the message applies.
    pub span: Span,

    /// The report's severity. Can be omitted. If omitted it is up to the
    /// client to interpret reports as error, warning, info or hint.
    pub severity: Option<ReportSeverity>,

    /// The report's code. Can be omitted.
    pub code: Option<String>,

    /// An optional property to describe the error code.
    pub code_description: Option<String>,

    /// The report's message.
    pub message: String,

    /// Optional help text rendered below the body.
    pub help: Option<String>,

    /// Structured fields extracted from the diagnostic variant.
    pub fields: Vec<FieldInfo>,

    /// An array of related report information, e.g. when symbol-names within
    /// a scope collide all definitions can be marked via this property.
    pub related_information: Option<Vec<ReportRelatedInformation>>,

    /// Additional metadata about the report.
    pub tags: Option<Vec<ReportTag>>,
}

#[repr(u8)]
#[derive(Facet, Clone, PartialEq, Eq)]
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

#[repr(u8)]
#[derive(Facet, Clone, PartialEq, Eq)]
pub enum ReportTag {
    /// Unused or unnecessary code.
    /// Clients are allowed to render diagnostics with this tag faded out instead of having
    /// an error squiggle.
    Unnecessary,

    /// Deprecated or obsolete code.
    /// Clients are allowed to rendered diagnostics with this tag strike through.
    Deprecated,
}

/// Represents a related message and source code location for a diagnostic. This
/// should be used to point to code locations that cause or related to a
/// diagnostics, e.g when duplicating a symbol in a scope.
#[derive(Facet, Clone, PartialEq, Eq)]
pub struct ReportRelatedInformation {
    /// The span of this related diagnostic information.
    pub span: Span,

    /// The message of this related diagnostic information.
    pub message: String,
}

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
        .map(|attr| attr.to_string());

    let variant_help = variant
        .get_attr(Some("diagnostics"), "help")
        .and_then(|attr| attr.get_as::<&str>())
        .map(|attr| attr.to_string());

    let variant_severity = variant
        .get_attr(Some("diagnostics"), "severity")
        .and_then(|attr| attr.get_as::<&str>())
        .map(|attr| attr.to_string());

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

                    let lookup = LineColLookup::new(&content);
                    let (start_line, start_col) = lookup.get(start);
                    let (end_line, end_col) = lookup.get(end);

                    span_meta = Some(FieldSpan {
                        module_id,
                        file_name,
                        content: content.to_string(),
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
            .map(|attr| attr.to_string());

        infos.push(FieldInfo {
            name,
            color: Color::Cyan,
            span: span_meta,
            display,
            label: label_attr,
        });
    }

    let message = variant_label
        .as_deref()
        .map(|label| render_recursive(label, &infos))
        .unwrap_or_else(|| "diagnostic".to_string());

    let help = variant_help
        .as_deref()
        .map(|help| render_recursive(help, &infos));

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
        .map(|span| {
            Span::Known {
                start: span.start,
                end: span.end,
                module_id: span.module_id,
            }
        })
        .unwrap_or(Span::Unknown);

    let related_information = {
        let mut related = Vec::new();

        for info in &infos {
            if let Some(field_span) = &info.span {
                if let Span::Known {
                    start: primary_start,
                    end: primary_end,
                    module_id: primary_module_id,
                } = span
                {
                    if field_span.start == primary_start
                        && field_span.end == primary_end
                        && field_span.module_id == primary_module_id
                    {
                        continue;
                    }
                }

                let message = info
                    .label
                    .as_deref()
                    .map(|label| render_recursive(label, &infos))
                    .or_else(|| info.display.clone())
                    .unwrap_or_else(|| info.name.clone());

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
        code: None,
        code_description: None,
        message,
        help,
        fields: infos,
        related_information,
        tags: None,
    }
}
