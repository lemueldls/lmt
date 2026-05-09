#![feature(trait_alias)]

pub mod ast;
pub mod lexer;
mod utils;

use std::path::PathBuf;

pub use ast::{
    BinaryOp, Block, Expr, Function, FunctionSignature, Literal, MatchExpr, MatchExprCase, Stmt,
    TypeAnnotation, TypedIdent,
};
pub use chumsky::{
    Parser, error, extra,
    input::Input,
    span::{SimpleSpan, Span},
};
use chumsky::{input::ValueInput, prelude::*};
pub use lexer::{Token, token_parser};
pub use utils::{Error, Spanned};

pub trait ParserInput<'tokens, 'src> =
    ValueInput<'tokens, Token = Token<'src>, Span = SimpleSpan> + Clone;

pub trait TokenParser<'tokens, 'src: 'tokens, I: ParserInput<'tokens, 'src>, O> =
    Parser<'tokens, I, O, extra::Err<Error<'tokens, Token<'src>>>> + Clone;

pub trait SpannedTokenParser<'tokens, 'src: 'tokens, I: ParserInput<'tokens, 'src>, O> =
    Parser<'tokens, I, Spanned<O>, extra::Err<Error<'tokens, Token<'src>>>> + Clone;

pub fn module_parser<'tokens, 'src: 'tokens, I: ParserInput<'tokens, 'src>>()
-> impl TokenParser<'tokens, 'src, I, Vec<Spanned<Stmt>>> + Clone {
    stmt_parser().repeated().collect::<Vec<Spanned<Stmt>>>()
}

pub fn stmt_parser<'tokens, 'src: 'tokens, I: ParserInput<'tokens, 'src>>()
-> impl SpannedTokenParser<'tokens, 'src, I, Stmt> {
    recursive(|stmt| {
        let expr = expr_parser(stmt.clone());
        let path = path();
        let typed_ident = typed_ident();

        let expr_stmt = expr
            .clone()
            // .then_ignore(just(Token::Ctrl(';')))
            .map(Stmt::Expr);

        let let_stmt = just(Token::Let)
            .ignore_then(typed_ident.clone())
            .then_ignore(just(Token::Op("=")))
            .then(expr.clone())
            // .then_ignore(just(Token::Ctrl(';')))
            .map(|(typed_ident, value)| Stmt::Let(typed_ident, Box::new(value)));

        let function_stmt = function_parser(stmt, expr.clone()).map(Stmt::Function);

        let import_stmt = just(Token::Import)
            .ignore_then(path)
            .map(Stmt::Import)
            // .then_ignore(just(Token::Ctrl(';')))
            ;

        let return_stmt = just(Token::Return).ignore_then(expr).map(Stmt::Return)
            // .then_ignore(just(Token::Ctrl(';')))
        ;

        expr_stmt
            .or(let_stmt)
            .or(function_stmt)
            .or(import_stmt)
            .or(return_stmt)
            .labelled("statement")
            .as_context()
            .recover_with(skip_then_retry_until(
                nested_delimiters(
                    Token::Ctrl('{'),
                    Token::Ctrl('}'),
                    [
                        (Token::Ctrl('('), Token::Ctrl(')')),
                        (Token::Ctrl('['), Token::Ctrl(']')),
                    ],
                    |span| Spanned::new(Stmt::Error, span),
                )
                .ignored(),
                one_of([
                    // Token::Ctrl(';'),
                    Token::Ctrl('}'),
                    Token::Ctrl(')'),
                    Token::Ctrl(']'),
                ])
                .ignored(),
            ))
            .map_with(|stmt, e| Spanned::new(stmt, e.span()))
    })
}

pub fn expr_parser<'tokens, 'src: 'tokens, I: ParserInput<'tokens, 'src>>(
    stmt: impl SpannedTokenParser<'tokens, 'src, I, Stmt> + 'tokens,
) -> impl SpannedTokenParser<'tokens, 'src, I, Expr> {
    let ident = ident();

    let val = select! {
        Token::Bool(x) => Expr::Literal(Literal::Boolean(x)),
        Token::Integer(n) => Expr::Literal(Literal::Integer(n)),
        Token::Decimal(d) => Expr::Literal(Literal::Decimal(d)),
        Token::Str(s) => Expr::Literal(Literal::String(s.to_owned())),
    }
    .labelled("value");

    recursive(|expr| {
        // A list of expressions
        let items = expr
            .clone()
            .separated_by(just(Token::Ctrl(',')))
            .allow_trailing()
            .collect::<Vec<_>>();

        let list = items
            .clone()
            .map(Expr::List)
            .delimited_by(just(Token::Ctrl('[')), just(Token::Ctrl(']')));

        // 'Atoms' are expressions that contain no ambiguity
        let atom = val
            .or(ident.map(Expr::Ident))
            .or(list)
            .map_with(|expr, e| Spanned::new(expr, e.span()))
            // Atoms can also just be normal expressions, but surrounded with parentheses
            .or(expr
                .clone()
                .delimited_by(just(Token::Ctrl('(')), just(Token::Ctrl(')'))))
            // Attempt to recover anything that looks like a parenthesised expression but contains errors
            .recover_with(via_parser(nested_delimiters(
                Token::Ctrl('('),
                Token::Ctrl(')'),
                [
                    (Token::Ctrl('['), Token::Ctrl(']')),
                    (Token::Ctrl('{'), Token::Ctrl('}')),
                ],
                |span| Spanned::new(Expr::Error, span),
            )))
            // Attempt to recover anything that looks like a list but contains errors
            .recover_with(via_parser(nested_delimiters(
                Token::Ctrl('['),
                Token::Ctrl(']'),
                [
                    (Token::Ctrl('('), Token::Ctrl(')')),
                    (Token::Ctrl('{'), Token::Ctrl('}')),
                ],
                |span| Spanned::new(Expr::Error, span),
            )))
            .boxed();

        // Function calls have very high precedence so we prioritise them
        let call = atom.foldl_with(
            items
                .delimited_by(just(Token::Ctrl('(')), just(Token::Ctrl(')')))
                .map_with(|args, e| Spanned::new(args, e.span()))
                .repeated(),
            |f, args, e| Spanned::new(Expr::Call(Box::new(f), args), e.span()),
        );

        // Product ops (multiply and divide) have equal precedence
        let product = {
            let op = just(Token::Op("*"))
                .to(BinaryOp::Mul)
                .or(just(Token::Op("/")).to(BinaryOp::Div));

            call.clone()
                .foldl_with(op.then(call).repeated(), |a, (op, b), e| {
                    Spanned::new(Expr::Binary(Box::new(a), op, Box::new(b)), e.span())
                })
        };

        // Sum ops (add and subtract) have equal precedence
        let sum = {
            let op = just(Token::Op("+"))
                .to(BinaryOp::Add)
                .or(just(Token::Op("-")).to(BinaryOp::Sub));

            product
                .clone()
                .foldl_with(op.then(product).repeated(), |a, (op, b), e| {
                    Spanned::new(Expr::Binary(Box::new(a), op, Box::new(b)), e.span())
                })
        };

        // Comparison ops (equal, less-than, greater-than, etc.) have equal precedence
        let compare = {
            let op = just(Token::Op("=="))
                .to(BinaryOp::Equal)
                .or(just(Token::Op("!=")).to(BinaryOp::NotEqual))
                .or(just(Token::Op("<")).to(BinaryOp::LessThan))
                // .or(just(Token::Op("<=")).to(BinaryOp::LessThanOrEqual))
                .or(just(Token::Op(">")).to(BinaryOp::GreaterThan))
                .or(just(Token::Op(">=")).to(BinaryOp::GreaterThanOrEqual));

            sum.clone()
                .foldl_with(op.then(sum).repeated(), |a, (op, b), e| {
                    Spanned::new(Expr::Binary(Box::new(a), op, Box::new(b)), e.span())
                })
        };

        let inline_expr = compare.labelled("expression").as_context();

        let block =
            block(stmt, expr).map(|block| Spanned::new(Expr::Block(block.inner), block.span));

        // let if_ = recursive(|if_| {
        //     just(Token::If)
        //         .ignore_then(inline_expr.clone())
        //         .then(block.clone())
        //         .then(
        //             just(Token::Else)
        //                 .ignore_then(block.clone().or(if_))
        //                 .or_not(),
        //         )
        //         .map_with(|((cond, a), b), e| {
        //             Spanned::new(
        //                 Expr::If(
        //                     Box::new(cond),
        //                     Box::new(a),
        //                     // If an `if` expression has no trailing `else` block, we magic up one that just produces null
        //                     Box::new(b.unwrap_or_else(|| {
        //                         Spanned::new(Expr::Literal(Literal::Nothing), e.span())
        //                     })),
        //                 ),
        //                 e.span(),
        //             )
        //         })
        // });

        let match_ = just(Token::Match)
            .ignore_then(inline_expr.clone())
            .then(
                inline_expr
                    .clone()
                    .then_ignore(just(Token::Then))
                    .then(inline_expr.clone())
                    .separated_by(just(Token::Ctrl(',')))
                    .allow_trailing()
                    // .repeated()
                    .collect::<Vec<(Spanned<Expr>, Spanned<Expr>)>>()
                    .delimited_by(just(Token::Ctrl('{')), just(Token::Ctrl('}'))),
                // .recover_with(via_parser(nested_delimiters(
                //     Token::Ctrl('{'),
                //     Token::Ctrl('}'),
                //     [
                //         (Token::Ctrl('('), Token::Ctrl(')')),
                //         (Token::Ctrl('['), Token::Ctrl(']')),
                //     ],
                //     |span| Spanned::new(Expr::Error, span),
                // ))),
            )
            .map_with(
                |(expr, cases): (Spanned<Expr>, Vec<(Spanned<Expr>, Spanned<Expr>)>), e| {
                    Spanned::new(
                        Expr::Match(Box::new(MatchExpr {
                            expr,
                            cases: cases
                                .into_iter()
                                .map(|(condition, then)| MatchExprCase { condition, then })
                                .collect(),
                        })),
                        e.span(),
                    )
                },
            );

        // // Both blocks and `if` are 'block expressions' and can appear in the place of statements
        // let block_expr = block.or(if_);
        // let block_expr = block;

        // block.or(inline_expr)
        // block.or(if_).or(inline_expr)
        // block_expr.or(inline_expr)

        // let block_chain = block_expr;
        //     .clone()
        //     .foldl_with(block_expr.clone().repeated(), |a, b, e| {
        //         Spanned::new(Expr::Then(Box::new(a), Box::new(b)), e.span())
        //     });

        // let block_recovery = nested_delimiters(
        //     Token::Ctrl('{'),
        //     Token::Ctrl('}'),
        //     [
        //         (Token::Ctrl('('), Token::Ctrl(')')),
        //         (Token::Ctrl('['), Token::Ctrl(']')),
        //     ],
        //     |span| Spanned::new(Expr::Error, span),
        // );

        block
            // Expressions, chained by semicolons, are statements
            .or(match_)
            .or(inline_expr)
        // .recover_with(skip_then_retry_until(
        //     block_recovery.ignored().or(any().ignored()),
        //     one_of([
        //         // Token::Ctrl(';'),
        //         Token::Ctrl('}'),
        //         Token::Ctrl(')'),
        //         Token::Ctrl(']'),
        //     ])
        //     .ignored(),
        // ))
        // .foldl_with(
        //     just(Token::Ctrl(';'))
        //         .ignore_then(inline_expr.or_not())
        //         .repeated(),
        //     |a, b, e| {
        //         let span: Span = e.span();
        //         (
        //             Expr::Then(
        //                 Box::new(a),
        //                 // If there is no b expression then its span is the end of the statement/block.
        //                 Box::new(
        //                     b.unwrap_or_else(|| (Expr::Literal(Literal::Nothing), span.to_end())),
        //                 ),
        //             ),
        //             span,
        //         )
        //     },
        // )
    })
}

fn block<'tokens, 'src: 'tokens, I: ParserInput<'tokens, 'src>>(
    stmt: impl SpannedTokenParser<'tokens, 'src, I, Stmt>,
    expr: impl SpannedTokenParser<'tokens, 'src, I, Expr>,
) -> impl SpannedTokenParser<'tokens, 'src, I, Block> {
    let multiline_block = just(Token::Op("->"))
        .ignore_then(typed_ident().map(Box::new))
        .or_not()
        .then(
            stmt.clone()
                .repeated()
                .collect::<Vec<Spanned<Stmt>>>()
                .delimited_by(just(Token::Ctrl('{')), just(Token::Ctrl('}'))),
            // Attempt to recover anything that looks like a block but contains errors
            // .recover_with(via_parser(nested_delimiters(
            //     Token::Ctrl('{'),
            //     Token::Ctrl('}'),
            //     [
            //         (Token::Ctrl('('), Token::Ctrl(')')),
            //         (Token::Ctrl('['), Token::Ctrl(']')),
            //     ],
            //     |span| Spanned::new(Expr::Error, span),
            // ))),
        )
        .map_with(|(return_typed_ident, stmts), e| {
            Spanned::new(
                Block::Multiline {
                    return_typed_ident,
                    stmts,
                },
                e.span(),
            )
        });

    let singleline_block = just(Token::Op("=>"))
        .ignore_then(expr.map(Box::new))
        .map_with(|return_expr, e| Spanned::new(Block::Singleline { return_expr }, e.span()));

    multiline_block.or(singleline_block).labelled("block")
}

fn function_parser<'tokens, 'src: 'tokens, I: ParserInput<'tokens, 'src>>(
    stmt: impl SpannedTokenParser<'tokens, 'src, I, Stmt>,
    expr: impl SpannedTokenParser<'tokens, 'src, I, Expr>,
) -> impl TokenParser<'tokens, 'src, I, Function> {
    let ident = ident();
    let typed_ident = typed_ident();

    // Argument lists are just identifiers separated by commas, surrounded by parentheses
    let args = typed_ident
        .clone()
        .separated_by(just(Token::Ctrl(',')))
        .allow_trailing()
        .collect()
        .delimited_by(just(Token::Ctrl('(')), just(Token::Ctrl(')')))
        .labelled("function args");

    let function = just(Token::Fn)
        .ignore_then(ident.labelled("function name"))
        .then(args)
        .map_with(|(name, args), e| Spanned::new(FunctionSignature { name, args }, e.span()))
        .labelled("function signature")
        .then(block(stmt, expr))
        .map(|(signature, body)| Function { signature, body })
        .labelled("function");

    function
}

pub fn ident<'tokens, 'src: 'tokens, I: ParserInput<'tokens, 'src>>()
-> impl SpannedTokenParser<'tokens, 'src, I, String> {
    select! { Token::Ident(ident) => ident }
        .map_with(|ident, e| Spanned::new(ident.to_owned(), e.span()))
        .labelled("identifier")
}

pub fn type_annotation<'tokens, 'src: 'tokens, I: ParserInput<'tokens, 'src>>()
-> impl SpannedTokenParser<'tokens, 'src, I, TypeAnnotation> {
    ident()
        .map_with(|ident, e| Spanned::new(TypeAnnotation::Expr(Expr::Ident(ident)), e.span()))
        .labelled("type annotation")
}

pub fn typed_ident<'tokens, 'src: 'tokens, I: ParserInput<'tokens, 'src>>()
-> impl SpannedTokenParser<'tokens, 'src, I, TypedIdent> {
    ident()
        .then(
            just(Token::Ctrl(':'))
                .ignore_then(type_annotation())
                .or_not(),
        )
        .map_with(|(ident, type_ann), e| Spanned::new(TypedIdent { ident, type_ann }, e.span()))
        .labelled("typed identifier")
}

pub fn path<'tokens, 'src: 'tokens, I: ParserInput<'tokens, 'src>>()
-> impl SpannedTokenParser<'tokens, 'src, I, PathBuf> {
    // select! { Token::Path(path) => path }
    //     .map(|path| path.to_path_buf())
    //     .or(ident().map(|ident| PathBuf::from(ident.as_str())))
    //     .map_with(|path, e| Spanned::new(path, e.span()))
    //     .labelled("path")
    select! { Token::Ctrl('.') => () }
        .then(select! { Token::Op("/") => () })
        .or_not()
        .then(
            ident()
                .separated_by(just(Token::Op("/")))
                .at_least(1)
                .collect::<Vec<_>>(),
        )
        .map_with(|(leading, segments), e| {
            let mut path = PathBuf::new();

            if leading.is_some() {
                path = path.join("./");
            }

            for segment in segments {
                path = path.join(segment.inner)
            }

            Spanned::new(path, e.span())
        })
}
