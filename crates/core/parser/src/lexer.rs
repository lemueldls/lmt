use std::{fmt, ops, path::PathBuf, str::FromStr};

use chumsky::{
    Parser, error, extra,
    input::{Input, SliceInput, ValueInput},
    prelude::*,
    span::{SimpleSpan, Span},
};
use lmt_number::{LmtDecimal, LmtInteger};

use crate::Spanned;

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

pub fn token_parser<'src>()
-> impl Parser<'src, &'src str, Vec<Spanned<Token<'src>>>, extra::Err<Rich<'src, char, SimpleSpan>>>
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
                    let signed_float = if is_negative { -float } else { float };

                    let precision = after_decimal.len() as u8 - 1;

                    Token::Decimal(signed_float.with_precision(precision))
                } else {
                    let int = LmtInteger::from_str(before_decimal).unwrap();
                    let signed_int = if is_negative { -int } else { int };

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
        .map_with(|token, e| Spanned::new(token, e.span()))
        .padded_by(comment.repeated())
        .padded()
        // If we encounter an error, skip and attempt to lex the next character as a token instead
        .recover_with(skip_then_retry_until(any().ignored(), end()))
        .repeated()
        .collect()
}
