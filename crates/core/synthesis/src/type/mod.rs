use core::fmt;
use std::ops::Deref;

use lmt_parser::{Function, Literal, Spanned};

mod proof;
mod store;

pub use proof::Proof;
pub use store::{TypeId, TypeMap};

#[derive(Debug, Clone)]
// #[cfg_attr(feature = "serde", derive(bitcode::Encode, bitcode::Decode))]
pub struct Implication {
    pub if_equal: Literal,
    pub then_implies: Proof,
    pub for_type: Spanned<TypeId>,
}

#[derive(Debug, Clone)]
// #[cfg_attr(feature = "serde", derive(bitcode::Encode, bitcode::Decode))]
pub struct SynType {
    pub proof: Proof,
    pub implications: Vec<Implication>,
}

impl SynType {
    pub fn new(proof: Proof) -> Self {
        Self {
            proof,
            implications: Vec::new(),
        }
    }

    pub fn with_implication(mut self, implication: Implication) -> Self {
        self.implications.push(implication);

        self
    }

    pub fn upcast(&self) -> Option<Self> {
        // if let (equal_to, span) = self.proof.equal_to().unwrap().deref_spanned() {
        //     equal_to.upcast().map(|syn_type_kind| Self {
        //         proof: Proof::EqualTo(Spanned::new(syn_type_kind, span)),
        //         implications: self.implications.clone(),
        //     })
        // } else {
        //     None
        // }

        self.proof.upcast().map(|upcast| {
            Self {
                proof: upcast,
                implications: self.implications.clone(),
            }
        })
    }
}

#[derive(Debug, Clone)]
// #[cfg_attr(feature = "serde", derive(bitcode::Encode, bitcode::Decode))]
pub enum SynTypeKind {
    Unknown,
    Constant(Literal),
    Primitive(Primitive),
    List(Vec<SynType>),
    Function(Function),
}

impl SynTypeKind {
    pub fn upcast(&self) -> Option<Self> {
        match self {
            SynTypeKind::Unknown => None,
            SynTypeKind::Constant(literal) => {
                match literal {
                    Literal::Integer(..) => Some(SynTypeKind::Primitive(Primitive::Integer)),
                    Literal::Decimal(..) => Some(SynTypeKind::Primitive(Primitive::Decimal)),
                    Literal::String(..) => Some(SynTypeKind::Primitive(Primitive::String)),
                    Literal::Boolean(..) => Some(SynTypeKind::Primitive(Primitive::Boolean)),
                    Literal::Nothing => None,
                }
            }
            SynTypeKind::Primitive(..) => None,
            SynTypeKind::List(list) => {
                let mut is_upcasted = false;

                let upcasted_list = list
                    .iter()
                    .map(|r#type| {
                        if let Some(upcasted_type) = r#type.upcast() {
                            is_upcasted = true;

                            upcasted_type
                        } else {
                            r#type.clone()
                        }
                    })
                    .collect();

                is_upcasted.then_some(SynTypeKind::List(upcasted_list))
            }
            SynTypeKind::Function(..) => None,
        }
    }

    pub fn literal(&self) -> Option<&Literal> {
        match self {
            SynTypeKind::Unknown => None,
            SynTypeKind::Constant(constant) => Some(constant),
            SynTypeKind::Primitive(..) => None,
            SynTypeKind::List(..) => None,
            SynTypeKind::Function(..) => None,
        }
    }

    pub fn literal_mut(&mut self) -> Option<&mut Literal> {
        match self {
            SynTypeKind::Unknown => None,
            SynTypeKind::Constant(constant) => Some(constant),
            SynTypeKind::Primitive(..) => None,
            SynTypeKind::List(..) => None,
            SynTypeKind::Function(..) => None,
        }
    }
}

impl SynTypeKind {
    pub fn into_constant_value(self) -> Option<Literal> {
        match self {
            SynTypeKind::Unknown => None,
            SynTypeKind::Constant(constant) => Some(constant),
            SynTypeKind::Primitive(..) => None,
            SynTypeKind::List(..) => None,
            SynTypeKind::Function(..) => None,
        }
    }
}

#[derive(Debug, Clone)]
// #[cfg_attr(feature = "serde", derive(bitcode::Encode, bitcode::Decode))]
pub enum Primitive {
    Integer,
    Decimal,
    String,
    Boolean,
}

impl fmt::Display for Primitive {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Integer => write!(f, "integer"),
            Self::Decimal => write!(f, "decimal"),
            Self::String => write!(f, "string"),
            Self::Boolean => write!(f, "boolean"),
        }
    }
}

// impl Literal {
//     pub fn upcast(&self) -> Primitive {
//         match self {
//             Self::Integer(_) => Primitive::Integer,
//             Self::Decimal(_) => Primitive::Decimal,
//             Self::String(_) => Primitive::String,
//             Self::Boolean(_) => Primitive::Boolean,
//             Self::Nothing => Primitive::Nothing,
//         }
//     }
// }

// impl fmt::Display for Literal {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         match self {
//             Self::Integer(integer) => write!(f, "{}", integer.value),
//             Self::Decimal(float) => write!(f, "{float}"),
//             Self::String(string) => write!(f, "{string}"),
//             Self::Boolean(boolean) => write!(f, "{boolean}"),
//             Self::Nothing => write!(f, "{{nothing}}"),
//         }
//     }
// }

// impl<'src> From<Literal<'src>> for Literal {
//     fn from(literal: Literal) -> Self {
//         match literal {
//             Literal::Boolean(boolean) => Self::Boolean(boolean),
//             Literal::Integer(integer) => Self::Integer(Integer::new(integer)),
//             Literal::Decimal(float, precision) => Self::Decimal(Decimal::new(float, precision)),
//             Literal::String(string) => Self::String(Box::from(string)),
//             Literal::List(_) => todo!(),
//             Literal::Nothing => Self::Nothing,
//         }
//     }
// }

impl fmt::Display for SynTypeKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SynTypeKind::Unknown => write!(f, "{{unknown}}"),
            SynTypeKind::Constant(literal) => write!(f, "{literal}"),
            SynTypeKind::Primitive(primitive) => write!(f, "{primitive}"),
            SynTypeKind::List(list) => {
                write!(f, "[")?;

                for (i, r#type) in list.iter().enumerate() {
                    if i != 0 {
                        write!(f, ", ")?;
                    }

                    write!(f, "{}", r#type.proof)?;
                }

                write!(f, "]")
            }
            SynTypeKind::Function(function) => {
                write!(f, "<function:{}>", function.signature.name.deref())
            }
        }
    }
}

// pub fn synthesize(stmts: Vec<Spanned<Stmt>>) -> ModuleSynthesis {
//     let mut synthesis = ModuleSynthesis::default();

//     // {
//     //     let span = Span::new(0, 0);

//     //     let syn_type = Arc::new(RwLock::new(SynType::new(Proof::And(
//     //         Box::new(Proof::EqualTo(Spanned::new(
//     //             SynTypeKind::Primitive(Primitive::Integer),
//     //             span,
//     //         ))),
//     //         Box::new(Proof::LessThan(Spanned::new(
//     //             SynTypeKind::Constant(Literal::Integer(LmtInteger::new(2.into()))),
//     //             span,
//     //         ))),
//     //     ))));
//     //     // let syn_type = Arc::new(RwLock::new(SynType::new(Proof::LessThan(Spanned::new(
//     //     //     SynTypeKind::Constant(Literal::Integer(Integer::new(
//     //     //         2.into(),
//     //     //         // BigUint::zero(),
//     //     //     ))),
//     //     //     span,
//     //     // )))));

//     //     // synthesis
//     //     //     .ident_types
//     //     //     .insert(span, Some(Arc::clone(&syn_type)));

//     //     let name = "lt-two";

//     //     let name_spanned = Spanned::new(name, span);

//     //     if let Some(idents) = synthesis.ident_stacks.get_mut(name) {
//     //         idents.push(name_spanned.map(|name| name.to_string()));
//     //     } else {
//     //         synthesis.ident_stacks.insert(name.to_string(), vec![
//     //             name_spanned.map(|name| name.to_string()),
//     //         ]);
//     //     }

//     //     synthesis.ident_references.insert(span, Vec::new());

//     //     synthesis.ident_types.insert(span, syn_type);
//     // }

//     // synthesis.hoist_top_level(&module);

//     // for function in module.functions {
//     //     let span = function.signature.span();
//     //     let function_context = synthesis.eval_function(function);
//     //     synthesis.functions.insert(span, function_context);
//     // }

//     for stmt in stmts {
//         synthesis.eval_statement(stmt);
//     }

//     synthesis
// }
