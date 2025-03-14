#![feature(trait_alias)]

mod lexer;

use chumsky::prelude::*;
pub use lexer::*;
use lmt_report::{SynthesisReport, miette::SourceSpan};

pub fn parse_tokens(content: &str) -> (Option<Spanned<Block>>, Vec<SynthesisReport>) {
    let (tokens, tokenize_errors): (
        Option<Vec<(Token, SimpleSpan)>>,
        Vec<Rich<char, SimpleSpan>>,
    ) = lexer().parse(content).into_output_errors();

    let mut errors: Vec<SynthesisReport> = tokenize_errors
        .into_iter()
        .map(|error| {
            SynthesisReport::TokenizeError {
                reason: error.reason().to_string(),
                span: SourceSpan::from(error.span().into_range()),
            }
        })
        .collect();

    let stmts: Option<Vec<Spanned<Stmt>>> = tokens.and_then(|tokens| {
        let len = content.chars().count();
        let parse_result: ParseResult<Vec<Spanned<Stmt>>, Rich<Token>> = module_parser()
            .parse(tokens.map((len..len).into(), Box::new(|(token, span)| (token, span))));

        let parse_errors = parse_result.errors();
        errors.extend(parse_errors.map(|error| {
            SynthesisReport::TokenizeError {
                reason: error.reason().to_string(),
                span: SourceSpan::from(error.span().into_range()),
            }
        }));

        parse_result.into_output()
    });

    let block = stmts.map(|stmts| {
        Spanned::new(
            Block::Multiline {
                stmts,
                return_typed_ident: None,
            },
            SimpleSpan::from(0..content.len()),
        )
    });

    (block, errors)
}
