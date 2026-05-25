use crate::report::{FieldInfo, FieldSpan};

#[derive(Debug, Clone)]
pub enum ReportRenderItem {
    Text(String),
    Reference {
        text: String,
        span: FieldSpan,
        index: usize,
    },
}

#[derive(Debug, Clone)]
pub struct ReportRender {
    pub items: Vec<ReportRenderItem>,
}

impl ReportRender {
    pub fn text(text: impl Into<String>) -> Self {
        Self {
            items: vec![ReportRenderItem::Text(text.into())],
        }
    }

    // Parses `{name}` and `[text](field)` constructs, allowing nested
    // content inside link text. Uses `FieldInfo` to resolve placeholders and
    // produce styled links when a span is available.
    #[must_use]
    pub fn parse_report_attr(src: &str, infos: &[FieldInfo]) -> Self {
        let mut chars = src.chars().peekable();
        let mut items = Vec::new();

        while let Some(&ch) = chars.peek() {
            if ch == '{' {
                // consume '{'
                chars.next();

                let mut ident = String::new();

                while let Some(&c) = chars.peek() {
                    chars.next();

                    if c == '}' {
                        break;
                    }

                    ident.push(c);
                }

                if !ident.is_empty()
                    && let Some(info) = infos.iter().find(|i| i.name == ident)
                {
                    if let Some(span) = &info.span {
                        items.push(ReportRenderItem::Reference {
                            text: span.content[span.start..span.end].to_string(),
                            span: span.clone(),
                            index: info.index,
                        });
                    } else if let Some(disp) = &info.display {
                        items.push(ReportRenderItem::Text(disp.clone()));
                    } else {
                        items.push(ReportRenderItem::Text(ident.clone()));
                    }
                }
            } else if ch == '[' {
                // consume '['
                chars.next();

                let mut link_text = String::new();

                while let Some(&c) = chars.peek() {
                    chars.next();

                    if c == ']' {
                        break;
                    }

                    link_text.push(c);
                }

                if chars.peek() == Some(&'(') {
                    // consume '('
                    chars.next();

                    let mut field_name = String::new();

                    while let Some(&c) = chars.peek() {
                        chars.next();

                        if c == ')' {
                            break;
                        }

                        field_name.push(c);
                    }

                    if let Some(info) = infos.iter().find(|i| i.name == field_name) {
                        if let Some(span) = &info.span {
                            items.push(ReportRenderItem::Reference {
                                text: link_text.clone(),
                                span: span.clone(),
                                index: info.index,
                            });
                        } else if let Some(disp) = &info.display {
                            items.push(ReportRenderItem::Text(disp.clone()));
                        } else {
                            items.push(ReportRenderItem::Text(link_text.clone()));
                        }
                    } else {
                        items.push(ReportRenderItem::Text(link_text.clone()));
                    }
                } else {
                    items.push(ReportRenderItem::Text(link_text.clone()));
                }
            } else {
                // consume normal text
                let mut text = String::new();

                while let Some(&c) = chars.peek() {
                    if c == '{' || c == '[' {
                        break;
                    }

                    chars.next();
                    text.push(c);
                }

                items.push(ReportRenderItem::Text(text));
            }
        }

        Self { items }
    }
}
