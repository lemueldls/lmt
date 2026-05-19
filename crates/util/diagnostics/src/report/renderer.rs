use crate::report::{FieldInfo, FieldSpan};

#[repr(u8)]
#[derive(Clone)]
pub enum ReportRenderItem {
    Text(String),
    Reference { text: String, span: FieldSpan },
}

#[derive(Clone)]
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
    pub fn parse_report_attr(src: &str, infos: &Vec<FieldInfo>) -> Self {
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

                if !ident.is_empty() {
                    if let Some(info) = infos.iter().find(|i| i.name == ident) {
                        if let Some(span) = &info.span {
                            items.push(ReportRenderItem::Reference {
                                text: info.display.clone().unwrap_or_else(|| ident.clone()),
                                span: span.clone(),
                            });
                        } else if let Some(disp) = &info.display {
                            items.push(ReportRenderItem::Text(disp.clone()));
                        } else {
                            items.push(ReportRenderItem::Text(ident.clone()));
                        }
                    }
                }
            } else {
                let mut text = String::new();
                while let Some(&c) = chars.peek() {
                    if c == '{' {
                        break;
                    }

                    text.push(c);
                    chars.next();
                }

                items.push(ReportRenderItem::Text(text));
            }
        }

        Self { items }
    }
}
