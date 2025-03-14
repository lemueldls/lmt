use std::{cmp, collections::HashMap, fmt, ops};

use lmt_parser::Spanned;

use crate::{Literal, Number, Primitive, SynTypeKind};

// TODO: just `Spanned<ProofTypeKind>` might do.
#[derive(Clone, Debug)]
pub enum ProofType {
    Literal(Literal),
    Type(Spanned<SynTypeKind>),
}

impl fmt::Display for ProofType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProofType::Literal(literal) => write!(f, "{literal}",),
            ProofType::Type(ty) => write!(f, "{ty}"),
        }
    }
}

pub struct Proof {
    bound: ProofBound,
}

#[derive(Clone, Debug)]
pub enum ProofBound {
    And(Box<ProofBound>, Box<ProofBound>),
    Or(Box<ProofBound>, Box<ProofBound>),
    EqualTo(ProofType),
    NotEqualTo(ProofType),
    LessThan(ProofType),
    GreaterThan(ProofType),
}

impl fmt::Display for ProofBound {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProofBound::And(left, right) => write!(f, "({left} and {right})"),
            ProofBound::Or(left, right) => {
                match (left, right) {
                    // (Proof::LessThan(left), Proof::EqualTo(right))
                    //     if crate::temp::equal(literal, rliteral) =>
                    // {
                    //     write!(f, "(<= {left})")
                    // }
                    // (Proof::GreaterThan(left), Proof::EqualTo(right))
                    //     if crate::temp::equal(literal, rliteral) =>
                    // {
                    //     write!(f, "(>= {left})")
                    // }
                    // (Proof::EqualTo(left), Proof::LessThan(right))
                    //     if crate::temp::equal(literal, rliteral) =>
                    // {
                    //     write!(f, "(<= {right})")
                    // }
                    // (Proof::EqualTo(left), Proof::GreaterThan(right))
                    //     if crate::temp::equal(literal, rliteral) =>
                    // {
                    //     write!(f, "(>= {right})")
                    // }
                    _ => write!(f, "({left} or {right})"),
                }
            }
            ProofBound::NotEqualTo(proof) => write!(f, "not {proof}"),
            ProofBound::EqualTo(value) => write!(f, "{value}"),
            ProofBound::LessThan(value) => write!(f, "< {value}"),
            ProofBound::GreaterThan(value) => write!(f, "> {value}"),
        }
    }
}

/// All applied proofs can only constraint, never expand the set
/// of possible values.
impl ProofBound {
    pub fn apply(&self, other: &ProofBound) -> Result<ProofBound, Box<str>> {
        match (self, other) {
            (ProofBound::And(left, right), _) => other.apply(&*left)?.apply(&*right),
            (ProofBound::Or(left, right), _) => other.or(left, right),
            (_, ProofBound::And(left, right)) => self.apply(left)?.apply(right),
            (_, ProofBound::Or(left, right)) => self.or(left, right),
            _ => {
                match self {
                    ProofBound::EqualTo(self_type) => {
                        match other {
                            ProofBound::EqualTo(other_type) => {
                                match (self_type, other_type) {
                                    (
                                        ProofType::Literal(self_literal),
                                        ProofType::Literal(other_literal),
                                    ) => {
                                        if crate::temp::equal(self_literal, other_literal)? {
                                            Ok(self.clone())
                                        } else {
                                            Err(format!("impossible to be both {self} and {other}")
                                                .into_boxed_str())
                                        }
                                    }
                                    _ => {
                                        Ok(ProofBound::And(
                                            Box::new(self.clone()),
                                            Box::new(other.clone()),
                                        ))
                                    }
                                }
                            }
                            ProofBound::NotEqualTo(other_type) => {
                                match (self_type, other_type) {
                                    (
                                        ProofType::Literal(self_literal),
                                        ProofType::Literal(other_literal),
                                    ) => {
                                        if crate::temp::equal(self_literal, other_literal)? {
                                            Err(format!("impossible to be both {self} and {other}")
                                                .into_boxed_str())
                                        } else {
                                            Ok(self.clone())
                                        }
                                    }
                                    _ => {
                                        Ok(ProofBound::And(
                                            Box::new(self.clone()),
                                            Box::new(other.clone()),
                                        ))
                                    }
                                }
                            }
                            ProofBound::GreaterThan(other_type) => {
                                match (self_type, other_type) {
                                    (
                                        ProofType::Literal(self_literal),
                                        ProofType::Literal(other_literal),
                                    ) => {
                                        if crate::temp::greater_than(self_literal, other_literal)? {
                                            Ok(self.clone())
                                        } else {
                                            Err(format!("impossible to be both {self} and {other}")
                                                .into_boxed_str())
                                        }
                                    }
                                    _ => {
                                        Ok(ProofBound::And(
                                            Box::new(self.clone()),
                                            Box::new(other.clone()),
                                        ))
                                    }
                                }
                            }
                            ProofBound::LessThan(other_type) => {
                                match (self_type, other_type) {
                                    (
                                        ProofType::Literal(self_literal),
                                        ProofType::Literal(other_literal),
                                    ) => {
                                        if crate::temp::less_than(self_literal, other_literal)? {
                                            Ok(self.clone())
                                        } else {
                                            Err(format!("impossible to be both {self} and {other}")
                                                .into_boxed_str())
                                        }
                                    }
                                    _ => {
                                        Ok(ProofBound::And(
                                            Box::new(self.clone()),
                                            Box::new(other.clone()),
                                        ))
                                    }
                                }
                            }
                            _ => unreachable!(),
                        }
                    }
                    ProofBound::NotEqualTo(self_type) => {
                        match other {
                            ProofBound::EqualTo(other_type) => {
                                match (self_type, other_type) {
                                    (
                                        ProofType::Literal(self_literal),
                                        ProofType::Literal(other_literal),
                                    ) => {
                                        if crate::temp::equal(self_literal, other_literal)? {
                                            Err(format!("impossible to be both {self} and {other}")
                                                .into_boxed_str())
                                        } else {
                                            Ok(other.clone())
                                        }
                                    }
                                    _ => {
                                        Ok(ProofBound::And(
                                            Box::new(self.clone()),
                                            Box::new(other.clone()),
                                        ))
                                    }
                                }
                            }
                            ProofBound::NotEqualTo(other_type) => {
                                match (self_type, other_type) {
                                    (
                                        ProofType::Literal(self_literal),
                                        ProofType::Literal(other_literal),
                                    ) => {
                                        if crate::temp::equal(self_literal, other_literal)? {
                                            Ok(self.clone())
                                        } else {
                                            Ok(ProofBound::And(
                                                Box::new(self.clone()),
                                                Box::new(other.clone()),
                                            ))
                                        }
                                    }
                                    _ => {
                                        Ok(ProofBound::And(
                                            Box::new(self.clone()),
                                            Box::new(other.clone()),
                                        ))
                                    }
                                }
                            }
                            ProofBound::GreaterThan(other_type) => {
                                match (self_type, other_type) {
                                    (
                                        ProofType::Literal(self_literal),
                                        ProofType::Literal(other_literal),
                                    ) => {
                                        if crate::temp::greater_than(self_literal, other_literal)? {
                                            Ok(ProofBound::And(
                                                Box::new(self.clone()),
                                                Box::new(other.clone()),
                                            ))
                                        } else {
                                            Ok(other.clone())
                                        }
                                    }
                                    _ => {
                                        Ok(ProofBound::And(
                                            Box::new(self.clone()),
                                            Box::new(other.clone()),
                                        ))
                                    }
                                }
                            }
                            ProofBound::LessThan(other_type) => {
                                match (self_type, other_type) {
                                    (
                                        ProofType::Literal(self_literal),
                                        ProofType::Literal(other_literal),
                                    ) => {
                                        if crate::temp::less_than(self_literal, other_literal)? {
                                            Ok(ProofBound::And(
                                                Box::new(self.clone()),
                                                Box::new(other.clone()),
                                            ))
                                        } else {
                                            Ok(other.clone())
                                        }
                                    }
                                    _ => {
                                        Ok(ProofBound::And(
                                            Box::new(self.clone()),
                                            Box::new(other.clone()),
                                        ))
                                    }
                                }
                            }
                            _ => unreachable!(),
                        }
                    }
                    ProofBound::GreaterThan(self_type) => {
                        match other {
                            ProofBound::EqualTo(other_type) => {
                                match (self_type, other_type) {
                                    (
                                        ProofType::Literal(self_literal),
                                        ProofType::Literal(other_literal),
                                    ) => {
                                        if crate::temp::greater_than(other_literal, self_literal)? {
                                            Ok(other.clone())
                                        } else {
                                            Err(format!("impossible to be both {self} and {other}")
                                                .into_boxed_str())
                                        }
                                    }
                                    _ => {
                                        Ok(ProofBound::And(
                                            Box::new(self.clone()),
                                            Box::new(other.clone()),
                                        ))
                                    }
                                }
                            }
                            ProofBound::NotEqualTo(other_type) => {
                                match (self_type, other_type) {
                                    (
                                        ProofType::Literal(self_literal),
                                        ProofType::Literal(other_literal),
                                    ) => {
                                        if crate::temp::greater_than(other_literal, self_literal)? {
                                            Ok(ProofBound::And(
                                                Box::new(self.clone()),
                                                Box::new(other.clone()),
                                            ))
                                        } else {
                                            Ok(self.clone())
                                        }
                                    }
                                    _ => {
                                        Ok(ProofBound::And(
                                            Box::new(self.clone()),
                                            Box::new(other.clone()),
                                        ))
                                    }
                                }
                            }
                            ProofBound::GreaterThan(other_type) => {
                                match (self_type, other_type) {
                                    (
                                        ProofType::Literal(self_literal),
                                        ProofType::Literal(other_literal),
                                    ) => {
                                        if crate::temp::less_than(other_literal, self_literal)? {
                                            Ok(self.clone())
                                        } else {
                                            Ok(other.clone())
                                        }
                                    }
                                    _ => {
                                        Ok(ProofBound::And(
                                            Box::new(self.clone()),
                                            Box::new(other.clone()),
                                        ))
                                    }
                                }
                            }
                            ProofBound::LessThan(other_type) => {
                                match (self_type, other_type) {
                                    (
                                        ProofType::Literal(self_literal),
                                        ProofType::Literal(other_literal),
                                    ) => {
                                        if crate::temp::greater_than(other_literal, self_literal)? {
                                            Ok(ProofBound::And(
                                                Box::new(self.clone()),
                                                Box::new(other.clone()),
                                            ))
                                        } else {
                                            Err(format!("impossible to be both {self} and {other}")
                                                .into_boxed_str())
                                        }
                                    }
                                    _ => {
                                        Ok(ProofBound::And(
                                            Box::new(self.clone()),
                                            Box::new(other.clone()),
                                        ))
                                    }
                                }
                            }
                            _ => unreachable!(),
                        }
                    }
                    ProofBound::LessThan(self_type) => {
                        match other {
                            ProofBound::EqualTo(other_type) => {
                                match (self_type, other_type) {
                                    (
                                        ProofType::Literal(self_literal),
                                        ProofType::Literal(other_literal),
                                    ) => {
                                        if crate::temp::less_than(other_literal, self_literal)? {
                                            Ok(other.clone())
                                        } else {
                                            Err(format!("impossible to be both {self} and {other}")
                                                .into_boxed_str())
                                        }
                                    }
                                    _ => {
                                        Ok(ProofBound::And(
                                            Box::new(self.clone()),
                                            Box::new(other.clone()),
                                        ))
                                    }
                                }
                            }
                            ProofBound::NotEqualTo(other_type) => {
                                match (self_type, other_type) {
                                    (
                                        ProofType::Literal(self_literal),
                                        ProofType::Literal(other_literal),
                                    ) => {
                                        if crate::temp::less_than(other_literal, self_literal)? {
                                            Ok(ProofBound::And(
                                                Box::new(self.clone()),
                                                Box::new(other.clone()),
                                            ))
                                        } else {
                                            Ok(self.clone())
                                        }
                                    }
                                    _ => {
                                        Ok(ProofBound::And(
                                            Box::new(self.clone()),
                                            Box::new(other.clone()),
                                        ))
                                    }
                                }
                            }
                            ProofBound::GreaterThan(other_type) => {
                                match (self_type, other_type) {
                                    (
                                        ProofType::Literal(self_literal),
                                        ProofType::Literal(other_literal),
                                    ) => {
                                        if crate::temp::less_than(other_literal, self_literal)? {
                                            Ok(ProofBound::And(
                                                Box::new(self.clone()),
                                                Box::new(other.clone()),
                                            ))
                                        } else {
                                            Err(format!("impossible to be both {self} and {other}")
                                                .into_boxed_str())
                                        }
                                    }
                                    _ => {
                                        Ok(ProofBound::And(
                                            Box::new(self.clone()),
                                            Box::new(other.clone()),
                                        ))
                                    }
                                }
                            }
                            ProofBound::LessThan(other_type) => {
                                match (self_type, other_type) {
                                    (
                                        ProofType::Literal(self_literal),
                                        ProofType::Literal(other_literal),
                                    ) => {
                                        if crate::temp::greater_than(other_literal, self_literal)? {
                                            Ok(self.clone())
                                        } else {
                                            Ok(other.clone())
                                        }
                                    }
                                    _ => {
                                        Ok(ProofBound::And(
                                            Box::new(self.clone()),
                                            Box::new(other.clone()),
                                        ))
                                    }
                                }
                            }
                            _ => unreachable!(),
                        }
                    }

                    _ => unreachable!(),
                }
            }
        }
    }

    fn or(&self, left: &Box<ProofBound>, right: &Box<ProofBound>) -> Result<ProofBound, Box<str>> {
        match (self.apply(left), self.apply(right)) {
            (Ok(left), Ok(right)) => Ok(ProofBound::Or(Box::new(left), Box::new(right))),
            (Ok(left), Err(_)) => Ok(left),
            (Err(_), Ok(right)) => Ok(right),
            (Err(left_err), Err(right_err)) => {
                Err(format!("{left_err} and {right_err}").into_boxed_str())
            }
        }
    }

    pub fn find_equal_to(&self) -> Option<&ProofType> {
        match self {
            // Proof::And(left, right) => {
            //     let left = left.find_equal_to();
            //     let right = right.find_equal_to();
            //     left.or(right)
            // }
            // Proof::Or(left, right) => {
            //     let left = left.find_equal_to();
            //     let right = right.find_equal_to();
            //     left.or(right)
            // }
            ProofBound::EqualTo(proof) => Some(proof),
            // _ => None,
            _ => todo!(),
        }
    }

    // fn max(&self) -> Result<&Proof, Box<str>> {
    //     match self {
    //         Proof::And(left, right) => {
    //             let left_max = left.max()?;
    //             let right_max = right.max()?;
    //             Ok(crate::temp::max(leftliteral, rightliteral))
    //         }
    //         Proof::Or(left, right) => {
    //             let left_max = left.max()?;
    //             let right_max = right.max()?;
    //             Ok(crate::temp::max(leftliteral, rightliteral))
    //         }
    //         Proof::Not(..) | Proof::EqualTo(..) | Proof::GreaterThan(..) | Proof::LessThan(..) => {
    //             Ok(self)
    //         }
    //     }
    // }

    // fn min(&self) -> Result<&ProofType, Box<str>> {
    //     match self {
    //         Proof::GreaterThan(value) => Ok(value),
    //         Proof::LessThan(value) => Ok(value),
    //         Proof::EqualTo(value) => Ok(value),
    //         Proof::And(left, right) => {
    //             let left_min = left.min()?;
    //             let right_min = right.min()?;
    //             Ok(crate::temp::min(leftliteral, rightliteral))
    //         }
    //         Proof::Or(left, right) => {
    //             let left_min = left.min()?;
    //             let right_min = right.min()?;
    //             Ok(crate::temp::min(leftliteral, rightliteral))
    //         }
    //     }
    // }

    pub fn negate(&self) -> ProofBound {
        match self {
            ProofBound::And(left, right) => {
                ProofBound::Or(Box::new(left.negate()), Box::new(right.negate()))
            }
            ProofBound::Or(left, right) => left.negate().apply(&right.negate()).unwrap(),
            ProofBound::EqualTo(value) => ProofBound::NotEqualTo(value.clone()),
            ProofBound::NotEqualTo(value) => ProofBound::EqualTo(value.clone()),
            ProofBound::GreaterThan(value) => {
                ProofBound::Or(
                    Box::new(ProofBound::LessThan(value.clone())),
                    Box::new(ProofBound::EqualTo(value.clone())),
                )
            }
            ProofBound::LessThan(value) => {
                ProofBound::Or(
                    Box::new(ProofBound::GreaterThan(value.clone())),
                    Box::new(ProofBound::EqualTo(value.clone())),
                )
            }
        }
    }
}

#[test]
fn test_u8() {
    fn new_u8(n: u8) -> ProofType {
        ProofType::Literal(Literal::Number(Number::U8(n)))
    }

    let u8_proof = ProofBound::And(
        Box::new(ProofBound::Or(
            Box::new(ProofBound::GreaterThan(new_u8(0))),
            Box::new(ProofBound::EqualTo(new_u8(0))),
        )),
        Box::new(ProofBound::Or(
            Box::new(ProofBound::LessThan(new_u8(255))),
            Box::new(ProofBound::EqualTo(new_u8(255))),
        )),
    );

    let mut a = u8_proof.apply(&ProofBound::LessThan(new_u8(100))).unwrap();
    let mut b = u8_proof.clone();

    println!("{a}");
    println!("{}", a.apply(&b).unwrap());
    // a = a.apply(&Proof::LessThan(new_u8(100))).unwrap();
    // a = a
    //     .apply(
    //         &Proof::LessThan(new_u8(100))
    //             .apply(&Proof::EqualTo(new_u8(2s0)))
    //             .unwrap(),
    //     )
    //     .unwrap();
    // println!("{a}");

    // let a_max = a.max().unwrap();
    // let b_max = b.max().unwrap();

    // println!("{} + {} <= 255", a, b);

    // let (min, max) = if a_max < b_max {
    //     (a_max, b_max)
    // } else {
    //     (b_max, a_max)
    // };

    // if max - min > 0 {
    //     eprintln!("{} + {} <= 255", a.proof, b.proof);
    // }
}

fn add(left: u8, right: u8) -> Result<u8, Box<str>> {
    if left <= 255 - right {
        Ok(left + right)
    } else {
        Err(format!("{} + {} <= 255", left, right).into_boxed_str())
    }
}
