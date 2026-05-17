use picante::PicanteResult;

use crate::{
    input::{
        HasSourceFileIngredient, SourceFile, SourceFileDataIngredient, SourceFileKeysIngredient,
        make_source_file_data, make_source_file_keys,
    },
    lexer::Lexer,
    token::{Token, TokenKind},
};

#[picante::db(inputs(SourceFile), tracked(tokenize, tokenize_window))]
pub struct Database {}

#[picante::tracked]
pub async fn tokenize<DB: DatabaseTrait>(db: &DB, file: SourceFile) -> PicanteResult<Vec<Token>> {
    let content = file.content(db)?;

    Ok(tokenize_source(&content))
}

pub fn tokenize_source(content: &str) -> Vec<Token> {
    let mut tokens = Vec::new();

    for (start, end) in token_windows(content) {
        let mut window_tokens = tokenize_source_window(content, start, end);

        if matches!(
            window_tokens.last().map(|token| &token.kind),
            Some(TokenKind::EOF)
        ) {
            window_tokens.pop();
        }

        tokens.extend(window_tokens);
    }

    tokens.push(Token::new(TokenKind::EOF, content.len(), content.len()));

    tokens
}

#[picante::tracked]
pub async fn tokenize_window<DB: DatabaseTrait>(
    db: &DB,
    file: SourceFile,
    start: usize,
    end: usize,
) -> PicanteResult<Vec<Token>> {
    let content = file.content(db)?;

    Ok(tokenize_source_window(&content, start, end))
}

fn tokenize_source_window(content: &str, start: usize, end: usize) -> Vec<Token> {
    let start = start.min(content.len());
    let end = end.min(content.len());

    if start >= end {
        return Vec::new();
    }

    let mut lexer = Lexer::new(&content[start..end]);
    let mut tokens = Vec::new();

    loop {
        let mut token = lexer.next_token();
        token.span.start += start;
        token.span.end += start;

        let is_eof = matches!(token.kind, TokenKind::EOF);
        tokens.push(token);

        if is_eof {
            break;
        }
    }

    tokens
}

fn token_windows(content: &str) -> Vec<(usize, usize)> {
    let mut windows = Vec::new();
    let mut lexer = Lexer::new(content);
    let mut window_start = 0usize;
    let mut brace_depth = 0usize;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;

    loop {
        let token = lexer.next_token();
        let window_end = token.span.end;

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
