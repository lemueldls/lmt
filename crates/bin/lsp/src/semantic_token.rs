use std::{collections::HashMap, hash::BuildHasher, ops::Deref};

use async_lsp::lsp_types::SemanticTokenType;
use lmt_parser::{Block, Expr, Function, Spanned, Stmt};

#[derive(Debug)]
pub struct ImCompleteSemanticToken {
    pub start: usize,
    pub length: usize,
    pub token_type: usize,
}

pub const LEGEND_TYPE: &[SemanticTokenType] = &[
    SemanticTokenType::FUNCTION,
    SemanticTokenType::VARIABLE,
    SemanticTokenType::STRING,
    SemanticTokenType::COMMENT,
    SemanticTokenType::NUMBER,
    SemanticTokenType::KEYWORD,
    SemanticTokenType::OPERATOR,
    SemanticTokenType::PARAMETER,
];

/// # Panics
/// Should never panic.
#[inline]
#[must_use]
pub fn semantic_token_from_ast(block: &Spanned<Block>) -> Vec<ImCompleteSemanticToken> {
    let mut semantic_tokens = vec![];

    match &**block {
        Block::Multiline {
            return_typed_ident,
            stmts,
        } => {
            for stmt in stmts {
                match &**stmt {
                    Stmt::Function(function) => {
                        semantic_token_from_function(function, &mut semantic_tokens);
                    }
                    _ => {}
                }
            }
        }
        Block::Singleline { return_expr } => {
            semantic_token_from_expr(&*return_expr, &mut semantic_tokens);
        }
    }

    semantic_tokens
}

fn semantic_token_from_function(
    function: &Function,
    semantic_tokens: &mut Vec<ImCompleteSemanticToken>,
) {
    function.signature.args.iter().for_each(|arg| {
        let span = arg.span();

        semantic_tokens.push(ImCompleteSemanticToken {
            start: span.start,
            length: span.end - span.start,
            token_type: LEGEND_TYPE
                .iter()
                .position(|item| item == &SemanticTokenType::PARAMETER)
                .unwrap(),
        });
    });

    let span = &function.signature.name.span();

    semantic_tokens.push(ImCompleteSemanticToken {
        start: span.start,
        length: span.end - span.start,
        token_type: LEGEND_TYPE
            .iter()
            .position(|item| item == &SemanticTokenType::FUNCTION)
            .unwrap(),
    });

    // for expr in &function.body {
    //     semantic_token_from_expr(expr, &mut semantic_tokens);
    // }
}

/// # Panics
/// Should never panic.
#[inline]
pub fn semantic_token_from_stmt(
    stmt: &Spanned<Stmt>,
    semantic_tokens: &mut Vec<ImCompleteSemanticToken>,
) {
    match &**stmt {
        Stmt::Error => {}
        Stmt::Expr(expr) => semantic_token_from_expr(expr, semantic_tokens),
        Stmt::Let(typed_ident, value) => {
            let span = typed_ident.ident.span();

            semantic_tokens.push(ImCompleteSemanticToken {
                start: span.start,
                length: span.end - span.start,
                token_type: LEGEND_TYPE
                    .iter()
                    .position(|item| item == &SemanticTokenType::VARIABLE)
                    .unwrap(),
            });

            semantic_token_from_expr(value, semantic_tokens);
        }
        Stmt::Import(_) => {}
        Stmt::Function(function) => semantic_token_from_function(function, semantic_tokens),
        Stmt::Return(_) => {}
    }
}

/// # Panics
/// Should never panic.
#[inline]
pub fn semantic_token_from_expr(
    expr: &Spanned<Expr>,
    semantic_tokens: &mut Vec<ImCompleteSemanticToken>,
) {
    match expr.deref() {
        Expr::Error => {}
        Expr::List(_) => {}
        Expr::Literal(_) => {}
        Expr::Ident(ident) => {
            let span = ident.span();

            semantic_tokens.push(ImCompleteSemanticToken {
                start: span.start,
                length: span.end - span.start,
                token_type: LEGEND_TYPE
                    .iter()
                    .position(|item| item == &SemanticTokenType::VARIABLE)
                    .unwrap(),
            });
        }
        Expr::Binary(lhs, _op, rhs) => {
            semantic_token_from_expr(lhs, semantic_tokens);
            semantic_token_from_expr(rhs, semantic_tokens);
        }
        Expr::Call(expr, params) => {
            semantic_token_from_expr(expr, semantic_tokens);

            for p in &**params {
                semantic_token_from_expr(p, semantic_tokens);
            }
        }
        Expr::Block(block) => {
            match block {
                Block::Multiline { stmts, .. } => {
                    for stmt in stmts {
                        semantic_token_from_stmt(stmt, semantic_tokens);
                    }
                }
                Block::Singleline { return_expr, .. } => {
                    semantic_token_from_expr(return_expr, semantic_tokens);
                }
            }
        }
        Expr::Match(match_expr) => {
            semantic_token_from_expr(&match_expr.expr, semantic_tokens);

            for case in &match_expr.cases {
                semantic_token_from_expr(&case.condition, semantic_tokens);
                semantic_token_from_expr(&case.then, semantic_tokens);
            }
        }
    }
}
