#![feature(trait_alias)]

use std::{
    collections::HashMap,
    fmt::{self, write},
    ops::{self, Deref, Neg},
    path::{Path, PathBuf},
    str::FromStr,
};

pub use chumsky::{
    Parser, error, extra,
    input::Input,
    span::{SimpleSpan, Span},
};
use chumsky::{input::MapExtra, prelude::*};
use lmt_number::{
    LmtDecimal, LmtInteger,
    fraction::{BigDecimal, BigInt, BigUint, FromPrimitive},
};

pub type Error<'tokens, T> = Rich<'tokens, T, SimpleSpan>;

#[derive(Clone, Debug, PartialEq)]
pub enum Token<'src> {
    Bool(bool),
    Integer(LmtInteger),
    Decimal(LmtDecimal),
    Str(&'src str),
    // Path(&'src Path),
    Op(&'src str),
    Ctrl(char),
    Ident(&'src str),
    Fn,
    Let,
    Import,
    Match,
    Then,
    And,
    Or,
    Not,
    Return,
}

impl<'src> fmt::Display for Token<'src> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Token::Bool(x) => write!(f, "{x}"),
            Token::Integer(n) => write!(f, "{n}"),
            Token::Decimal(n, ..) => write!(f, "{n}"),
            Token::Str(s) => write!(f, "{s}"),
            Token::Op(s) => write!(f, "{s}"),
            Token::Ctrl(c) => write!(f, "{c}"),
            Token::Ident(s) => write!(f, "{s}"),
            Token::Fn => write!(f, "fn"),
            Token::Let => write!(f, "let"),
            Token::Import => write!(f, "import"),
            Token::Match => write!(f, "match"),
            Token::Then => write!(f, "then"),
            Token::And => write!(f, "and"),
            Token::Or => write!(f, "or"),
            Token::Not => write!(f, "not"),
            Token::Return => write!(f, "return"),
        }
    }
}

pub fn lexer<'src>()
-> impl Parser<'src, &'src str, Vec<(Token<'src>, SimpleSpan)>, extra::Err<Rich<'src, char, SimpleSpan>>>
{
    // A parser for numbers
    let num = just('-')
        .or_not()
        .then(text::int(10))
        .then(just('.').ignore_then(text::digits(10)).to_slice().or_not())
        .map(
            |((negative, before_decimal), after_decimal): ((_, _), Option<&str>)| {
                let is_negative = negative.is_some();

                if let Some(after_decimal) = after_decimal {
                    let float =
                        LmtDecimal::from_str(&format!("{before_decimal}{after_decimal}")).unwrap();
                    let signed_float = if is_negative { float.neg() } else { float };

                    let precision = after_decimal.len() as u8 - 1;

                    Token::Decimal(signed_float.with_precision(precision))
                } else {
                    let int = LmtInteger::from_str(before_decimal).unwrap();
                    let signed_int = if is_negative { int.neg() } else { int };

                    Token::Integer(signed_int)
                }
            },
        );

    // A parser for strings
    let string = just('"')
        .ignore_then(none_of('"').repeated())
        .then_ignore(just('"'))
        .to_slice()
        .map(Token::Str);

    // A parser for operators
    let op = one_of("+*-/!=<>")
        .repeated()
        .at_least(1)
        .to_slice()
        .map(Token::Op);

    // A parser for control characters (delimiters, semicolons, etc.)
    let ctrl = one_of("()[]{};:,.").map(Token::Ctrl);

    // Idents must start with a letter or underscore, and are followed by any number of letters, numbers, underscores, or hyphens.
    // Hyphens are not allowed and the end of an identifier, or between numbers or underscores, so must be between letters.

    // A parser for identifiers and keywords
    let ident = any()
        .filter(|c: &char| c.is_ascii_alphabetic() || *c == '_')
        .then(
            any()
                .filter(|c: &char| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
                .repeated(),
        )
        .to_slice()
        .map(|ident: &str| {
            match ident {
                "fn" => Token::Fn,
                "let" => Token::Let,
                "true" => Token::Bool(true),
                "false" => Token::Bool(false),
                "import" => Token::Import,
                "match" => Token::Match,
                "and" => Token::And,
                "or" => Token::Or,
                "not" => Token::Not,
                "then" => Token::Then,
                "return" => Token::Return,
                _ => Token::Ident(ident),
            }
        });

    // let path = just('.')
    //     .then(just('/'))
    //     .or_not()
    //     .then(ident.separated_by(just('/')).at_least(1).to_slice())
    //     .to_slice()
    //     .map(|slice| Token::Path(Path::new(slice)));

    // A single token can be one of the above
    let token = num
        .or(string)
        .or(op)
        .or(ctrl)
        // .or(path)
        .or(ident);

    let comment = just("//")
        .then(any().and_is(just('\n').not()).repeated())
        .padded();

    token
        .map_with(|token, e| (token, e.span()))
        .padded_by(comment.repeated())
        .padded()
        // If we encounter an error, skip and attempt to lex the next character as a token instead
        .recover_with(skip_then_retry_until(any().ignored(), end()))
        .repeated()
        .collect()
}

#[derive(Clone, Debug, PartialEq)]
// #[cfg_attr(feature = "serde", derive(bitcode::Encode, bitcode::Decode))]
pub enum Literal {
    Boolean(bool),
    Integer(LmtInteger),
    Decimal(LmtDecimal),
    String(String),
    // List(Vec<Self>),
    Nothing,
}

// impl<'src> Literal {
//     fn num(self, span: Span) -> Result<f64, Error<'src, 'src, String>> {
//         if let Literal::Number(x) = self {
//             Ok(x)
//         } else {
//             Err(Rich::custom(span, format!("'{self}' is not a number")))
//         }
//     }
// }

impl<'src> std::fmt::Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Boolean(x) => write!(f, "{x}"),
            Self::Integer(x) => write!(f, "{x}"),
            Self::Decimal(x, ..) => write!(f, "{x}"),
            Self::String(x) => write!(f, "{x}"),
            // Self::List(xs) => {
            //     write!(
            //         f,
            //         "[{}]",
            //         xs.iter()
            //             .map(|x| x.to_string())
            //             .collect::<Vec<_>>()
            //             .join(", ")
            //     )
            // }
            Self::Nothing => write!(f, "{{nothing}}"),
        }
    }
}

#[derive(Clone, Debug)]
// #[cfg_attr(feature = "serde", derive(bitcode::Encode, bitcode::Decode))]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
}

#[derive(Debug, Clone)]
// #[cfg_attr(feature = "serde", derive(bitcode::Encode, bitcode::Decode))]
pub struct Spanned<T> {
    token: T,
    span: SimpleSpan,
}

impl<T> Spanned<T> {
    pub fn new(inner: T, span: SimpleSpan) -> Self {
        Self { token: inner, span }
    }

    pub fn span(&self) -> SimpleSpan {
        self.span
    }

    pub fn into_deref(self) -> T {
        self.token
    }

    pub fn deref_spanned(&self) -> (&T, SimpleSpan) {
        (&self.token, self.span)
    }

    pub fn into_deref_spanned(self) -> (T, SimpleSpan) {
        (self.token, self.span)
    }

    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Spanned<U> {
        Spanned::new(f(self.token), self.span)
    }

    pub fn span_mut(&mut self) -> &mut SimpleSpan {
        &mut self.span
    }
}

impl<T: fmt::Display> fmt::Display for Spanned<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.token)
    }
}

impl<T> ops::Deref for Spanned<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.token
    }
}

impl<T> ops::DerefMut for Spanned<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.token
    }
}

#[derive(Debug, Clone)]
// #[cfg_attr(feature = "serde", derive(bitcode::Encode, bitcode::Decode))]
pub enum Stmt {
    Error,
    Import(Spanned<PathBuf>),
    Function(Function),
    Let(Spanned<TypedIdent>, Box<Spanned<Expr>>),
    Expr(Spanned<Expr>),
    Return(Spanned<Expr>),
}

#[derive(Debug, Clone)]
// #[cfg_attr(feature = "serde", derive(bitcode::Encode, bitcode::Decode))]
pub struct TypedIdent {
    pub ident: Spanned<String>,
    pub type_ann: Option<Spanned<TypeAnnotation>>,
}

#[derive(Debug, Clone)]
// #[cfg_attr(feature = "serde", derive(bitcode::Encode, bitcode::Decode))]
pub enum Expr {
    Error,
    Literal(Literal),
    Ident(Spanned<String>),
    Block(Block),
    Match(Box<MatchExpr>),
    List(Vec<Spanned<Self>>),
    Binary(Box<Spanned<Self>>, BinaryOp, Box<Spanned<Self>>),
    Call(Box<Spanned<Self>>, Spanned<Vec<Spanned<Self>>>),
}

#[derive(Debug, Clone)]
pub enum Block {
    Multiline {
        return_typed_ident: Option<Box<Spanned<TypedIdent>>>,
        stmts: Vec<Spanned<Stmt>>,
    },
    Singleline {
        return_expr: Box<Spanned<Expr>>,
    },
}

#[derive(Debug, Clone)]
pub struct MatchExpr {
    pub expr: Spanned<Expr>,
    pub cases: Vec<MatchExprCase>,
}

#[derive(Debug, Clone)]
pub struct MatchExprCase {
    pub condition: Spanned<Expr>,
    pub then: Spanned<Expr>,
}

#[derive(Debug, Clone)]
// #[cfg_attr(feature = "serde", derive(bitcode::Encode, bitcode::Decode))]
pub enum TypeAnnotation {
    /// `type X = 5`
    Expr(Expr),
    Comparison(BinaryOp, Expr),
    And(Box<TypeAnnotation>, Box<TypeAnnotation>),
}

#[derive(Debug, Clone)]
// #[cfg_attr(feature = "serde", derive(bitcode::Encode, bitcode::Decode))]
pub struct FunctionSignature {
    pub name: Spanned<String>,
    pub args: Vec<Spanned<TypedIdent>>,
}

#[derive(Debug, Clone)]
// #[cfg_attr(feature = "serde", derive(bitcode::Encode, bitcode::Decode))]
pub struct Function {
    pub signature: Spanned<FunctionSignature>,
    pub body: Block,
}

// The type of the input that our parser operates on. The input is the `&[(Token, Span)]` token buffer generated by the
// lexer, wrapped in a `SpannedInput` which 'splits' it apart into its constituent parts, tokens and spans, for chumsky
// to understand.
type ParserInput<'tokens, 'src> =
    chumsky::input::SpannedInput<Token<'src>, SimpleSpan, &'tokens [(Token<'src>, SimpleSpan)]>;

trait TokenParser<'tokens, 'src: 'tokens, I> = Parser<'tokens, ParserInput<'tokens, 'src>, Spanned<I>, extra::Err<Error<'tokens, Token<'src>>>>
    + Clone;

fn ident<'tokens, 'src: 'tokens>() -> impl TokenParser<'tokens, 'src, String> {
    select! { Token::Ident(ident) => ident }
        .map_with(|ident, e| Spanned::new(ident.to_owned(), e.span()))
        .labelled("identifier")
}

fn type_annotation<'tokens, 'src: 'tokens>() -> impl TokenParser<'tokens, 'src, TypeAnnotation> {
    ident()
        .map_with(|ident, e| Spanned::new(TypeAnnotation::Expr(Expr::Ident(ident)), e.span()))
        .labelled("type annotation")
}

fn typed_ident<'tokens, 'src: 'tokens>() -> impl TokenParser<'tokens, 'src, TypedIdent> {
    ident()
        .then(
            just(Token::Ctrl(':'))
                .ignore_then(type_annotation())
                .or_not(),
        )
        .map_with(|(ident, type_ann), e| Spanned::new(TypedIdent { ident, type_ann }, e.span()))
        .labelled("typed identifier")
}

fn path<'tokens, 'src: 'tokens>() -> impl TokenParser<'tokens, 'src, PathBuf> {
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
                .to_slice(),
        )
        .map_with(
            |(leading, segments): (Option<((), ())>, &[(Token<'_>, _)]),
             e: &mut chumsky::input::MapExtra<'_, '_, _, _>| {
                let mut path = PathBuf::new();

                if leading.is_some() {
                    path = path.join("./");
                }

                for (segment, _span) in segments {
                    match segment {
                        Token::Ident(ident) => path = path.join(ident),
                        Token::Op(..) => {}
                        _ => unreachable!(),
                    }
                }

                Spanned::new(path, e.span())
            },
        )
}

fn stmt_parser<'tokens, 'src: 'tokens>() -> impl TokenParser<'tokens, 'src, Stmt> {
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

fn expr_parser<'tokens, 'src: 'tokens>(
    stmt: impl TokenParser<'tokens, 'src, Stmt> + 'tokens,
) -> impl TokenParser<'tokens, 'src, Expr> {
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
            block(stmt, expr).map_with(|block, e| Spanned::new(Expr::Block(block), e.span()));

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

fn block<'tokens, 'src: 'tokens>(
    stmt: impl TokenParser<'tokens, 'src, Stmt>,
    expr: impl TokenParser<'tokens, 'src, Expr>,
) -> impl Parser<
    'tokens,
    ParserInput<'tokens, 'src>,
    Block,
    extra::Err<Rich<'tokens, Token<'src>, SimpleSpan>>,
> + Clone {
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
        .map(|(return_typed_ident, stmts)| {
            Block::Multiline {
                return_typed_ident,
                stmts,
            }
        });

    let singleline_block = just(Token::Op("=>"))
        .ignore_then(expr.map(Box::new))
        .map(|return_expr| Block::Singleline { return_expr });

    multiline_block.or(singleline_block).labelled("block")
}

fn function_parser<'tokens, 'src: 'tokens>(
    stmt: impl TokenParser<'tokens, 'src, Stmt>,
    expr: impl TokenParser<'tokens, 'src, Expr>,
) -> impl Parser<
    'tokens,
    ParserInput<'tokens, 'src>,
    Function,
    extra::Err<Rich<'tokens, Token<'src>, SimpleSpan>>,
> + Clone {
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

pub fn module_parser<'tokens, 'src: 'tokens>() -> impl Parser<
    'tokens,
    ParserInput<'tokens, 'src>,
    Vec<Spanned<Stmt>>,
    extra::Err<Rich<'tokens, Token<'src>, SimpleSpan>>,
> + Clone {
    stmt_parser().repeated().collect::<Vec<Spanned<Stmt>>>()
}
