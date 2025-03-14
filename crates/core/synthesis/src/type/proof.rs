use std::{fmt, ops::Deref};

use lmt_parser::{Literal, Spanned};

use super::SynTypeKind;

trait Operation {}

// struct Add

enum Operator {
    EqualTo,
    NotEqualTo,
    LessThan,
    GreaterThan,
}

struct Equation {
    left: Box<dyn Operation>,
    operator: Operator,
    right: Box<dyn Operation>,
}

#[test]
fn isolate() {}

// Theory of Isolation
//
// a + b = c
// a = c + (-b)
// a = c - b
//
// (a - 3)^2 = a^2 - 6a + 9
// 2a - 6 = 0
// a = 3
// where all operations require an inverse
// > add(a, b) = c
// > sub(add(a, b), b) = sub(c, b)
//
// or all operands require an inverse
// > add(a, b) = c
// > a = add(c, inverse(b))
//
// where the `Add<Rhs>` trait requires `Inverse`
//
// a % b = c
// a % b = c
//
// let s = { t, v: list(t), n: integer in v => v[n] >= v[n + 1] }
//
// let t
// let v = list(t)
// let n = v in t
// let s = match v[n] {
//    >= v[n + 1]
//    .. => @panic()
// }
//
// fn Vec(t) -> vec {
//    let vec = struct {
//       ptr: raw t,
//       len: uint32,
//
//       fn get(index) {
//          match index {
//             <= len => unsafe { self.ptr.get(index) },
//             .. => @panic("index out of bounds")
//          }
//       }
//    }
// }
//
// fn s() -> v {
//    let t
//    let v = list(t)
//
//    let n = integer
//
//    match n {
//      < v.len =>
//         match v.get(n) {
//            <= v.get(n + 1)
//            .. => @panic()
//         }
//      .. => {}
//    }
// }
//

#[derive(Clone, Debug)]
// #[cfg_attr(feature = "serde", derive(bitcode::Encode, bitcode::Decode))]
pub enum Proof {
    And(Box<Proof>, Box<Proof>),
    Or(Box<Proof>, Box<Proof>),
    EqualTo(Spanned<SynTypeKind>),
    NotEqualTo(Spanned<SynTypeKind>),
    LessThan(Spanned<SynTypeKind>),
    GreaterThan(Spanned<SynTypeKind>),
}

impl fmt::Display for Proof {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // Proof::And(left, right) => write!(f, "({left} and {right})"),
            Proof::And(left, right) => write!(f, "({left} {right})"),
            Proof::Or(left, right) => {
                write!(f, "({left} or {right})")
            }
            Proof::NotEqualTo(proof) => write!(f, "not {proof}"),
            Proof::EqualTo(value) => write!(f, "{value}"),
            Proof::LessThan(value) => write!(f, "< {value}"),
            Proof::GreaterThan(value) => write!(f, "> {value}"),
        }
    }
}

/// All applied proofs can only constraint, never expand the set
/// of possible values.
impl Proof {
    pub fn literal(&self) -> Option<&Literal> {
        self.equal_to()?.literal()
    }

    pub fn upcast(&self) -> Option<Self> {
        match self {
            Proof::And(left, right) => {
                match (left.upcast(), right.upcast()) {
                    (Some(left_upcast), Some(right_upcast)) => {
                        Some(Proof::And(Box::new(left_upcast), Box::new(right_upcast)))
                    }
                    (Some(left_upcast), None) => {
                        Some(Proof::And(Box::new(left_upcast), right.clone()))
                    }
                    (None, Some(right_upcast)) => {
                        Some(Proof::And(left.clone(), Box::new(right_upcast)))
                    }
                    (None, None) => None,
                }
            }
            Proof::Or(left, right) => {
                match (left.upcast(), right.upcast()) {
                    (Some(left_upcast), Some(right_upcast)) => {
                        Some(Proof::Or(Box::new(left_upcast), Box::new(right_upcast)))
                    }
                    (Some(left_upcast), None) => {
                        Some(Proof::Or(Box::new(left_upcast), right.clone()))
                    }
                    (None, Some(right_upcast)) => {
                        Some(Proof::Or(left.clone(), Box::new(right_upcast)))
                    }
                    (None, None) => None,
                }
            }
            Proof::EqualTo(kind) => {
                Some(Proof::EqualTo(Spanned::new(
                    kind.deref().upcast()?,
                    kind.span(),
                )))
            }
            Proof::NotEqualTo(kind) => {
                Some(Proof::NotEqualTo(Spanned::new(
                    kind.deref().upcast()?,
                    kind.span(),
                )))
            }
            Proof::LessThan(kind) => {
                Some(Proof::LessThan(Spanned::new(
                    kind.deref().upcast()?,
                    kind.span(),
                )))
            }
            Proof::GreaterThan(kind) => {
                Some(Proof::GreaterThan(Spanned::new(
                    kind.deref().upcast()?,
                    kind.span(),
                )))
            }
        }
    }

    pub fn apply(&self, other: &Proof) -> Result<Proof, Box<str>> {
        match (self, other) {
            (Proof::And(left, right), _) => other.apply(left)?.apply(right),
            (Proof::Or(left, right), _) => other.or(left, right),
            (_, Proof::And(left, right)) => self.apply(left)?.apply(right),
            (_, Proof::Or(left, right)) => self.or(left, right),
            _ => {
                match self {
                    Proof::EqualTo(self_type) => {
                        match other {
                            Proof::EqualTo(other_type) => {
                                if let (Some(self_literal), Some(other_literal)) =
                                    (self_type.literal(), other_type.literal())
                                {
                                    if crate::temp::equal(self_literal, other_literal)? {
                                        Ok(self.clone())
                                    } else {
                                        Err(format!("impossible to be both {self} and {other}")
                                            .into_boxed_str())
                                    }
                                } else {
                                    Ok(Proof::And(Box::new(self.clone()), Box::new(other.clone())))
                                }
                            }
                            Proof::NotEqualTo(other_type) => {
                                if let (Some(self_literal), Some(other_literal)) =
                                    (self_type.literal(), other_type.literal())
                                {
                                    if crate::temp::equal(self_literal, other_literal)? {
                                        Err(format!("impossible to be both {self} and {other}")
                                            .into_boxed_str())
                                    } else {
                                        Ok(self.clone())
                                    }
                                } else {
                                    Ok(Proof::And(Box::new(self.clone()), Box::new(other.clone())))
                                }
                            }
                            Proof::GreaterThan(other_type) => {
                                if let (Some(self_literal), Some(other_literal)) =
                                    (self_type.literal(), other_type.literal())
                                {
                                    if crate::temp::greater_than(self_literal, other_literal)? {
                                        Ok(self.clone())
                                    } else {
                                        Err(format!("impossible to be both {self} and {other}")
                                            .into_boxed_str())
                                    }
                                } else {
                                    Ok(Proof::And(Box::new(self.clone()), Box::new(other.clone())))
                                }
                            }
                            Proof::LessThan(other_type) => {
                                if let (Some(self_literal), Some(other_literal)) =
                                    (self_type.literal(), other_type.literal())
                                {
                                    if crate::temp::less_than(self_literal, other_literal)? {
                                        Ok(self.clone())
                                    } else {
                                        Err(format!("impossible to be both {self} and {other}")
                                            .into_boxed_str())
                                    }
                                } else {
                                    Ok(Proof::And(Box::new(self.clone()), Box::new(other.clone())))
                                }
                            }
                            _ => unreachable!(),
                        }
                    }
                    Proof::NotEqualTo(self_type) => {
                        match other {
                            Proof::EqualTo(other_type) => {
                                if let (Some(self_literal), Some(other_literal)) =
                                    (self_type.literal(), other_type.literal())
                                {
                                    if crate::temp::equal(self_literal, other_literal)? {
                                        Err(format!("impossible to be both {self} and {other}")
                                            .into_boxed_str())
                                    } else {
                                        Ok(other.clone())
                                    }
                                } else {
                                    Ok(Proof::And(Box::new(self.clone()), Box::new(other.clone())))
                                }
                            }
                            Proof::NotEqualTo(other_type) => {
                                if let (Some(self_literal), Some(other_literal)) =
                                    (self_type.literal(), other_type.literal())
                                {
                                    if crate::temp::equal(self_literal, other_literal)? {
                                        Ok(self.clone())
                                    } else {
                                        Ok(Proof::And(
                                            Box::new(self.clone()),
                                            Box::new(other.clone()),
                                        ))
                                    }
                                } else {
                                    Ok(Proof::And(Box::new(self.clone()), Box::new(other.clone())))
                                }
                            }
                            Proof::GreaterThan(other_type) => {
                                if let (Some(self_literal), Some(other_literal)) =
                                    (self_type.literal(), other_type.literal())
                                {
                                    if crate::temp::greater_than(self_literal, other_literal)? {
                                        Ok(Proof::And(
                                            Box::new(self.clone()),
                                            Box::new(other.clone()),
                                        ))
                                    } else {
                                        Ok(other.clone())
                                    }
                                } else {
                                    Ok(Proof::And(Box::new(self.clone()), Box::new(other.clone())))
                                }
                            }
                            Proof::LessThan(other_type) => {
                                if let (Some(self_literal), Some(other_literal)) =
                                    (self_type.literal(), other_type.literal())
                                {
                                    if crate::temp::less_than(self_literal, other_literal)? {
                                        Ok(Proof::And(
                                            Box::new(self.clone()),
                                            Box::new(other.clone()),
                                        ))
                                    } else {
                                        Ok(other.clone())
                                    }
                                } else {
                                    Ok(Proof::And(Box::new(self.clone()), Box::new(other.clone())))
                                }
                            }
                            _ => unreachable!(),
                        }
                    }
                    Proof::GreaterThan(self_type) => {
                        match other {
                            Proof::EqualTo(other_type) => {
                                if let (Some(self_literal), Some(other_literal)) =
                                    (self_type.literal(), other_type.literal())
                                {
                                    if crate::temp::greater_than(other_literal, self_literal)? {
                                        Ok(other.clone())
                                    } else {
                                        Err(format!("impossible to be both {self} and {other}")
                                            .into_boxed_str())
                                    }
                                } else {
                                    Ok(Proof::And(Box::new(self.clone()), Box::new(other.clone())))
                                }
                            }
                            Proof::NotEqualTo(other_type) => {
                                if let (Some(self_literal), Some(other_literal)) =
                                    (self_type.literal(), other_type.literal())
                                {
                                    if crate::temp::greater_than(other_literal, self_literal)? {
                                        Ok(Proof::And(
                                            Box::new(self.clone()),
                                            Box::new(other.clone()),
                                        ))
                                    } else {
                                        Ok(self.clone())
                                    }
                                } else {
                                    Ok(Proof::And(Box::new(self.clone()), Box::new(other.clone())))
                                }
                            }
                            Proof::GreaterThan(other_type) => {
                                if let (Some(self_literal), Some(other_literal)) =
                                    (self_type.literal(), other_type.literal())
                                {
                                    if crate::temp::less_than(other_literal, self_literal)? {
                                        Ok(self.clone())
                                    } else {
                                        Ok(other.clone())
                                    }
                                } else {
                                    Ok(Proof::And(Box::new(self.clone()), Box::new(other.clone())))
                                }
                            }
                            Proof::LessThan(other_type) => {
                                if let (Some(self_literal), Some(other_literal)) =
                                    (self_type.literal(), other_type.literal())
                                {
                                    if crate::temp::greater_than(other_literal, self_literal)? {
                                        Ok(Proof::And(
                                            Box::new(self.clone()),
                                            Box::new(other.clone()),
                                        ))
                                    } else {
                                        Err(format!("impossible to be both {self} and {other}")
                                            .into_boxed_str())
                                    }
                                } else {
                                    Ok(Proof::And(Box::new(self.clone()), Box::new(other.clone())))
                                }
                            }
                            _ => unreachable!(),
                        }
                    }
                    Proof::LessThan(self_type) => {
                        match other {
                            Proof::EqualTo(other_type) => {
                                if let (Some(self_literal), Some(other_literal)) =
                                    (self_type.literal(), other_type.literal())
                                {
                                    if crate::temp::less_than(other_literal, self_literal)? {
                                        Ok(other.clone())
                                    } else {
                                        Err(format!("impossible to be both {self} and {other}")
                                            .into_boxed_str())
                                    }
                                } else {
                                    Ok(Proof::And(Box::new(self.clone()), Box::new(other.clone())))
                                }
                            }
                            Proof::NotEqualTo(other_type) => {
                                if let (Some(self_literal), Some(other_literal)) =
                                    (self_type.literal(), other_type.literal())
                                {
                                    if crate::temp::less_than(other_literal, self_literal)? {
                                        Ok(Proof::And(
                                            Box::new(self.clone()),
                                            Box::new(other.clone()),
                                        ))
                                    } else {
                                        Ok(self.clone())
                                    }
                                } else {
                                    Ok(Proof::And(Box::new(self.clone()), Box::new(other.clone())))
                                }
                            }
                            Proof::GreaterThan(other_type) => {
                                if let (Some(self_literal), Some(other_literal)) =
                                    (self_type.literal(), other_type.literal())
                                {
                                    if crate::temp::less_than(other_literal, self_literal)? {
                                        Ok(Proof::And(
                                            Box::new(self.clone()),
                                            Box::new(other.clone()),
                                        ))
                                    } else {
                                        Err(format!("impossible to be both {self} and {other}")
                                            .into_boxed_str())
                                    }
                                } else {
                                    Ok(Proof::And(Box::new(self.clone()), Box::new(other.clone())))
                                }
                            }
                            Proof::LessThan(other_type) => {
                                if let (Some(self_literal), Some(other_literal)) =
                                    (self_type.literal(), other_type.literal())
                                {
                                    if crate::temp::greater_than(other_literal, self_literal)? {
                                        Ok(self.clone())
                                    } else {
                                        Ok(other.clone())
                                    }
                                } else {
                                    Ok(Proof::And(Box::new(self.clone()), Box::new(other.clone())))
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

    fn or(&self, left: &Box<Proof>, right: &Box<Proof>) -> Result<Proof, Box<str>> {
        match (self.apply(left), self.apply(right)) {
            (Ok(left), Ok(right)) => Ok(Proof::Or(Box::new(left), Box::new(right))),
            (Ok(left), Err(_)) => Ok(left),
            (Err(_), Ok(right)) => Ok(right),
            (Err(left_err), Err(right_err)) => {
                Err(format!("{left_err} and {right_err}").into_boxed_str())
            }
        }
    }

    /// Theoretically (hopefully), all proofs have at least one [Proof::EqualTo].
    pub fn equal_to(&self) -> Option<&Spanned<SynTypeKind>> {
        // dbg!("[[EQUAL TO]]");

        match self {
            Proof::And(left, right) => {
                let left = left.equal_to();
                let right = right.equal_to();

                match (left, right) {
                    (Some(..), Some(..)) => todo!("huh??"),
                    (Some(..), None) => left,
                    (None, Some(_)) => right,
                    (None, None) => None,
                }
            }
            // Proof::Or(left, right) => {
            //     let left = left.equal_to();
            //     let right = right.equal_to();
            //     left.or(right)
            // }
            Proof::Or(..) => None,
            Proof::EqualTo(proof) => Some(proof),
            _ => None,
            // _ => unreachable!(),
        }
    }

    // fn max(&self) -> Result<&Spanned<SynTypeKind>, Box<str>> {
    //     match self {
    //         Proof::And(left, right) => {
    //             let left_max = left.deref().max();
    //             let right_max = right.deref().max();

    //             match (left_max, right_max) {
    //                 (Ok(left_max), Ok(right_max)) => {
    //                     let left_max = left_max.deref();
    //                     let right_max = right_max.deref();
    //                     match (left_max.deref(), right_max.deref()) {
    //                         (
    //                             SynTypeKind::Constant(left_literal),
    //                             SynTypeKind::Constant(right_literal),
    //                         ) => {
    //                             if crate::temp::equal(left_literal, right_literal)? {
    //                                 match (left.deref(), right.deref()) {
    //                                     (Proof::EqualTo(..), Proof::LessThan(..)) => Ok(left),
    //                                     (Proof::EqualTo(..), Proof::GreaterThan(..)) => Ok(right),
    //                                     (Proof::LessThan(..), Proof::EqualTo(..)) => Ok(right),
    //                                     (Proof::GreaterThan(..), Proof::EqualTo(..)) => Ok(left),
    //                                     _ => unreachable!(),
    //                                 }
    //                             } else if crate::temp::less_than(left_literal, right_literal)? {
    //                                 Ok(right)
    //                             } else {
    //                                 Ok(left)
    //                             }
    //                         }
    //                         _ => unreachable!(),
    //                     }
    //                 }
    //                 (Ok(..), Err(_)) => Ok(left),
    //                 (Err(_), Ok(..)) => Ok(right),
    //                 (Err(left_err), Err(right_err)) => {
    //                     Err(format!("{left_err} and {right_err}").into_boxed_str())
    //                 }
    //             }

    //             // match (left_max.deref(), right_max.deref()) {
    //             //     (SynTypeKind::Constant(left_literal ), SynTypeKind::Constant(right_literal)) => {
    //             //         crate::temp::max(left_literal, right_literal)?.clone();

    //             //     }
    //             //     _ => todo!(),
    //             // }
    //         }
    //         Proof::Or(left, right) => {
    //             let left_max = left.deref().max()?;
    //             let right_max = right.deref().max()?;
    //             Ok(crate::temp::max(left, right))
    //         }
    //         Proof::EqualTo(..) | Proof::GreaterThan(..) | Proof::LessThan(..) => Ok(self),
    //         Proof::NotEqualTo(..) => Err("not equal to".into()),
    //     }
    // }

    // fn min(&self) -> Result<&Spanned<SynTypeKind>, Box<str>> {
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

    pub fn negate(&self) -> Proof {
        match self {
            Proof::And(left, right) => Proof::Or(Box::new(left.negate()), Box::new(right.negate())),
            Proof::Or(left, right) => left.negate().apply(&right.negate()).unwrap(),
            Proof::EqualTo(value) => Proof::NotEqualTo(value.clone()),
            Proof::NotEqualTo(value) => Proof::EqualTo(value.clone()),
            Proof::GreaterThan(value) => {
                Proof::Or(
                    Box::new(Proof::LessThan(value.clone())),
                    Box::new(Proof::EqualTo(value.clone())),
                )
            }
            Proof::LessThan(value) => {
                Proof::Or(
                    Box::new(Proof::GreaterThan(value.clone())),
                    Box::new(Proof::EqualTo(value.clone())),
                )
            }
        }
    }

    pub fn search_linear(&self, f: &mut impl FnMut(&Proof)) {
        match self {
            Proof::And(left, right) => {
                left.search_linear(f);
                right.search_linear(f);
            }
            Proof::Or(left, right) => {
                left.search_linear(f);
                right.search_linear(f);
            }
            Proof::EqualTo(_)
            | Proof::NotEqualTo(_)
            | Proof::LessThan(_)
            | Proof::GreaterThan(_) => f(self),
        }
    }

    pub fn search_linear_mut(&mut self, f: &mut impl FnMut(&mut Proof)) {
        match self {
            Proof::And(left, right) => {
                left.search_linear_mut(f);
                right.search_linear_mut(f);
            }
            Proof::Or(left, right) => {
                left.search_linear_mut(f);
                right.search_linear_mut(f);
            }
            Proof::EqualTo(_)
            | Proof::NotEqualTo(_)
            | Proof::LessThan(_)
            | Proof::GreaterThan(_) => f(self),
        }
    }
}
