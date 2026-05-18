use core::fmt;
use std::{cmp, collections::HashMap, iter::Peekable, str::Chars};

use crossterm::style::{Color, Stylize, style};
use facet::{Facet, PtrConst, Shape};
use facet_reflect::{HasFields, Peek};
use line_col::LineColLookup;
use rand::{rng, seq::SliceRandom};

use crate::{Span, graph::ModuleGraph, source::HasNamedSourceIngredient};

pub fn report<'mem, 'facet, T: Facet<'facet> + ?Sized, DB: HasNamedSourceIngredient>(
    t: &'mem T,
    db: &DB,
    graph: &impl ModuleGraph,
) {
    let peek = Peek::new(t).into_enum().unwrap();

    let variant = peek.active_variant().unwrap();
    let variant_label = variant.get_attr(Some("diagnostics"), "label");
    let variant_help = variant.get_attr(Some("diagnostics"), "help");

    let mut colors = {
        let mut colors = [Color::Green, Color::Blue, Color::Magenta, Color::Cyan];
        colors.shuffle(&mut rng());

        colors.into_iter().cycle()
    };

    let field_names = peek
        .fields()
        .map(|(field, _)| field.effective_name())
        .collect::<Vec<_>>();

    let mut field_colors = HashMap::new();
    for name in field_names {
        field_colors.insert(name, colors.next().unwrap());
    }

    // variant label will be rendered after we collect `infos`

    let mut infos = Vec::new();
    let mut max_end_line = 0usize;

    for (field, fpeek) in peek.fields() {
        let name = field.effective_name().to_string();
        let color = *field_colors.get(field.effective_name()).unwrap();

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

                    max_end_line = cmp::max(max_end_line, end_line);

                    span_meta = Some(FieldSpan {
                        file_name: file_name.clone(),
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
            .and_then(|a| Some(a.get_as::<&str>().unwrap().to_string()));

        infos.push(FieldInfo {
            name,
            color,
            span: span_meta,
            display,
            label: label_attr,
        });
    }

    let alignment = if max_end_line == 0 {
        3
    } else {
        max_end_line.to_string().len() + 1
    };
    let spaces = " ".repeat(alignment);

    // Render variant label now that we have field metadata available
    if let Some(attr) = variant_label {
        let label = attr.get_as::<&str>().unwrap();
        let rendered = render_recursive(label, &infos);
        eprintln!("{} {}", "Error:".bold().red(), rendered.bold());
    }

    // Helper: render attribute-like strings with {field} and [text](field) links
    let render_attr = |s: &str| {
        let mut out = String::new();

        let mut chars = s.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '{' {
                let mut ident = String::new();
                while let Some(&c) = chars.peek() {
                    chars.next();
                    if c == '}' {
                        break;
                    }
                    ident.push(c);
                }

                if let Some(info) = infos.iter().find(|i| i.name == ident) {
                    if let Some(span) = &info.span {
                        if span.start <= span.end && span.end <= span.content.len() {
                            let excerpt = &span.content[span.start..span.end];
                            out.push_str(&format!("`{}`", excerpt));
                        }
                    } else if let Some(disp) = &info.display {
                        out.push_str(disp);
                    }
                }
            } else if ch == '[' {
                let mut text = String::new();
                while let Some(&c) = chars.peek() {
                    chars.next();
                    if c == ']' {
                        break;
                    }
                    text.push(c);
                }

                if chars.peek() == Some(&'(') {
                    chars.next();
                    let mut link_ident = String::new();
                    while let Some(&c) = chars.peek() {
                        chars.next();
                        if c == ')' {
                            break;
                        }
                        link_ident.push(c);
                    }

                    if let Some(info) = infos.iter().find(|i| i.name == link_ident) {
                        let styled = format!("{}", style(text).with(info.color));

                        if let Some(span) = &info.span {
                            let link = format!(
                                "{}:{}:{}",
                                span.file_name, span.start_line, span.start_col
                            );
                            out.push_str(&format_osc8_link(&link, &styled));
                        } else {
                            out.push_str(&styled);
                        }
                    }
                } else {
                    out.push_str(&text);
                }
            } else {
                out.push(ch);
            }
        }

        out
    };

    // Render codeblocks (for fields that are spans)
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
                    file_name, start_line, start_col, end_line, end_col,
                ))
                .cyan()
                .dim(),
            );

            eprintln!("{spaces} │");

            let line_start = start_line - 1;
            let line_end = end_line - 1;

            let lines = content.lines().collect::<Vec<_>>();

            let col_offset = start_col - 1;
            let mut range_size = span.end - span.start;

            let mut colored_lines = Vec::new();

            for line in line_start..=line_end {
                let mut line_code = lines[line].to_string();
                let end_range = cmp::min(line_code.len() - col_offset, range_size);

                let line_range = col_offset..(col_offset + end_range);

                if line != line_end {
                    range_size -= end_range - col_offset - 1;
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
                // print field label centered under the span (render placeholders inside the label)
                let rendered = render_recursive(lbl, &infos);
                let alignment_col = start_col + (end_col - start_col) / 2;
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

    if let Some(attr) = variant_help {
        let help = attr.get_as::<&str>().unwrap();

        eprintln!("{spaces} ╧ {}", render_attr(help))
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
pub struct FieldSpan {
    pub file_name: String,
    pub content: String,
    pub start: usize,
    pub end: usize,
    pub start_line: usize,
    pub start_col: usize,
    pub end_line: usize,
    pub end_col: usize,
}

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
    fn parse_inner(chars: &mut Peekable<Chars<'_>>, infos: &Vec<FieldInfo>) -> String {
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
                    inner.push_str(&parse_inner(chars, infos));
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
            } else if ch == ']' || ch == ')' || ch == '}' {
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

    parse_inner(&mut chars, infos)
}
