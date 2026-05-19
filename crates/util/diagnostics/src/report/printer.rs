use std::cmp;

use crossterm::style::{Color, Stylize, style};
use rand::{rng, seq::SliceRandom};

use crate::report::{
    Report, ReportSeverity,
    renderer::{ReportRender, ReportRenderItem},
};

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

    if !report.message.items.is_empty() {
        eprintln!(
            "{} {}",
            severity_prefix(report.severity.as_ref()),
            format_report_render_items(&report.message).bold()
        );
    }

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
                let render = ReportRender::parse_report_attr(lbl, &infos);
                let rendered = format_report_render_items(&render);
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
        eprintln!("{spaces} ╧ {}", format_report_render_items(help));
    }
}

fn severity_prefix(severity: Option<&ReportSeverity>) -> String {
    let (text, color) = match severity {
        Some(ReportSeverity::Error) => ("Error:", Color::Red),
        Some(ReportSeverity::Warning) => ("Warning:", Color::Yellow),
        Some(ReportSeverity::Information) => ("Info:", Color::Blue),
        Some(ReportSeverity::Hint) => ("Hint:", Color::Cyan),
        None => ("Error:", Color::Red),
    };

    format!("{}", style(text).with(color))
}

pub fn format_report_render_items(render: &ReportRender) -> String {
    let mut out = String::new();

    for item in &render.items {
        match item {
            ReportRenderItem::Text(text) => out.push_str(&text),
            ReportRenderItem::Reference { text, span } => {
                let link = format!("{}:{}:{}", span.file_name, span.start_line, span.start_col);
                out.push_str(&format_osc8_link(&link, &text));
            }
        }
    }

    out
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
