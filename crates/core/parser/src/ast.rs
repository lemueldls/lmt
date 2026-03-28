use std::path::PathBuf;

use lmt_number::{LmtDecimal, LmtInteger};

use crate::Spanned;

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
    pub body: Spanned<Block>,
}
