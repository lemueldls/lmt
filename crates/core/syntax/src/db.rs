use lmt_diagnostics::{ModuleId, source::*};
use picante::PicanteResult;

use crate::{
    lexer::Lexer,
    token::{Token, TokenKind},
};

#[picante::db(inputs(NamedSource), tracked(tokenize, tokenize_window))]
pub struct SyntaxDatabase {}

#[picante::tracked]
pub async fn tokenize<DB: SyntaxDatabaseTrait>(
    db: &DB,
    source: NamedSource,
) -> PicanteResult<Vec<Token>> {
    let content = source.content(db)?;
    let module_id = source.module_id(db)?;

    Ok(tokenize_source(&content, module_id))
}

pub fn tokenize_source(content: &str, module_id: ModuleId) -> Vec<Token> {
    let mut tokens = Vec::new();

    for (start, end) in token_windows(content, module_id) {
        let mut window_tokens = tokenize_source_window(content, start, end, module_id);

        if matches!(
            window_tokens.last().map(|token| &token.kind),
            Some(TokenKind::EOF)
        ) {
            window_tokens.pop();
        }

        tokens.extend(window_tokens);
    }

    tokens.push(Token::new(
        TokenKind::EOF,
        content.len(),
        content.len(),
        module_id,
    ));

    tokens
}

#[picante::tracked]
pub async fn tokenize_window<DB: SyntaxDatabaseTrait>(
    db: &DB,
    source: NamedSource,
    start: usize,
    end: usize,
) -> PicanteResult<Vec<Token>> {
    let content = source.content(db)?;
    let module_id = source.module_id(db)?;

    Ok(tokenize_source_window(&content, start, end, module_id))
}

fn tokenize_source_window(
    content: &str,
    start: usize,
    end: usize,
    module_id: ModuleId,
) -> Vec<Token> {
    let start = start.min(content.len());
    let end = end.min(content.len());

    if start >= end {
        return Vec::new();
    }

    let mut lexer = Lexer::new(&content[start..end], module_id);
    let mut tokens = Vec::new();

    loop {
        let mut token = lexer.next_token();
        token.span = token.span.offset(start);

        let is_eof = matches!(token.kind, TokenKind::EOF);
        tokens.push(token);

        if is_eof {
            break;
        }
    }

    tokens
}

fn token_windows(content: &str, module_id: ModuleId) -> Vec<(usize, usize)> {
    let mut windows = Vec::new();
    let mut lexer = Lexer::new(content, module_id);
    let mut window_start = 0usize;
    let mut brace_depth = 0usize;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;

    loop {
        let token = lexer.next_token();
        let window_end = token.span.end().unwrap();

        match token.kind {
            TokenKind::LBrace => brace_depth = brace_depth.saturating_add(1),
            TokenKind::RBrace => brace_depth = brace_depth.saturating_sub(1),
            TokenKind::LParen => paren_depth = paren_depth.saturating_add(1),
            TokenKind::RParen => paren_depth = paren_depth.saturating_sub(1),
            TokenKind::LBracket => bracket_depth = bracket_depth.saturating_add(1),
            TokenKind::RBracket => bracket_depth = bracket_depth.saturating_sub(1),
            _ => {}
        }

        let at_top_level = brace_depth == 0 && paren_depth == 0 && bracket_depth == 0;
        let closes_boundary = at_top_level
            && matches!(
                token.kind,
                TokenKind::Semi | TokenKind::RBrace | TokenKind::EOF
            );

        if closes_boundary {
            if window_start < window_end {
                windows.push((window_start, window_end));
            }
            window_start = window_end;
        }

        if matches!(token.kind, TokenKind::EOF) {
            break;
        }
    }

    if window_start < content.len() {
        windows.push((window_start, content.len()));
    }

    windows
}
