mod div;
mod mul;
mod rem;

use core::fmt;
use std::{iter, ops};

use num_bigint::BigInt;
use num_integer::Integer;
use num_traits::{Num, One, Pow, Zero};

use crate::LmtInteger;

impl Pow<u32> for &LmtInteger {
    type Output = LmtInteger;

    fn pow(self, rhs: u32) -> Self::Output {
        match self {
            LmtInteger::Small(value) => {
                if let Some(result) = value.checked_pow(rhs) {
                    LmtInteger::Small(result)
                } else {
                    LmtInteger::Big(BigInt::from(*value).pow(rhs))
                }
            }
            LmtInteger::Big(value) => LmtInteger::Big(value.pow(rhs)),
        }
    }
}

impl<I: Into<BigInt>> From<I> for LmtInteger {
    fn from(value: I) -> Self {
        Self::Big(value.into())
    }
}

impl One for LmtInteger {
    fn one() -> Self {
        Self::Small(1)
    }
}

impl Zero for LmtInteger {
    fn zero() -> Self {
        Self::Small(0)
    }

    fn is_zero(&self) -> bool {
        match self {
            Self::Small(value) => value.is_zero(),
            Self::Big(value) => value.is_zero(),
        }
    }
}

impl PartialEq for LmtInteger {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Small(a), Self::Small(b)) => a == b,
            (Self::Big(a), Self::Big(b)) => a == b,
            (Self::Small(a), Self::Big(b)) => BigInt::from(*a) == *b,
            (Self::Big(a), Self::Small(b)) => *a == BigInt::from(*b),
        }
    }
}

impl Eq for LmtInteger {}

impl PartialOrd for LmtInteger {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LmtInteger {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        match (self, other) {
            (Self::Small(a), Self::Small(b)) => a.cmp(b),
            (Self::Big(a), Self::Big(b)) => a.cmp(b),
            (Self::Small(a), Self::Big(b)) => BigInt::from(*a).cmp(b),
            (Self::Big(a), Self::Small(b)) => a.cmp(&BigInt::from(*b)),
        }
    }
}

impl ops::Neg for LmtInteger {
    type Output = Self;

    fn neg(self) -> Self::Output {
        match self {
            Self::Small(value) => Self::Small(-value),
            Self::Big(value) => Self::Big(-value),
        }
    }
}

impl ops::Add for LmtInteger {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Small(a), Self::Small(b)) => {
                if let Some(c) = a.checked_add(b) {
                    Self::Small(c)
                } else {
                    Self::Big(BigInt::from(a) + BigInt::from(b))
                }
            }
            (Self::Small(a), Self::Big(b)) => Self::Big(BigInt::from(a) + b),
            (Self::Big(a), Self::Small(b)) => Self::Big(a + BigInt::from(b)),
            (Self::Big(a), Self::Big(b)) => Self::Big(a + b),
        }
    }
}

impl ops::Add for &LmtInteger {
    type Output = LmtInteger;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (LmtInteger::Small(a), LmtInteger::Small(b)) => {
                if let Some(c) = a.checked_add(*b) {
                    LmtInteger::Small(c)
                } else {
                    LmtInteger::Big(BigInt::from(*a) + BigInt::from(*b))
                }
            }
            (LmtInteger::Small(a), LmtInteger::Big(b)) => LmtInteger::Big(BigInt::from(*a) + b),
            (LmtInteger::Big(a), LmtInteger::Small(b)) => LmtInteger::Big(a + BigInt::from(*b)),
            (LmtInteger::Big(a), LmtInteger::Big(b)) => LmtInteger::Big(a + b),
        }
    }
}

impl<T, U> ops::Add<U> for LmtInteger<T>
where
    T: ops::Add<U, Output = T>,
    BigInt: ops::Add<U, Output = BigInt>,
{
    type Output = Self;

    fn add(self, rhs: U) -> Self::Output {
        match self {
            Self::Small(a) => Self::Small(a + rhs),
            Self::Big(a) => Self::Big(a + rhs),
        }
    }
}

impl ops::AddAssign for LmtInteger {
    fn add_assign(&mut self, rhs: Self) {
        *self = self.clone() + rhs;
    }
}

impl ops::Sub for LmtInteger {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Small(a), Self::Small(b)) => {
                if let Some(c) = a.checked_sub(b) {
                    Self::Small(c)
                } else {
                    Self::Big(BigInt::from(a) - BigInt::from(b))
                }
            }
            (Self::Small(a), Self::Big(b)) => Self::Big(BigInt::from(a) - b),
            (Self::Big(a), Self::Small(b)) => Self::Big(a - BigInt::from(b)),
            (Self::Big(a), Self::Big(b)) => Self::Big(a - b),
        }
    }
}

impl Num for LmtInteger {
    type FromStrRadixErr = <BigInt as Num>::FromStrRadixErr;

    fn from_str_radix(s: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
        match BigInt::from_str_radix(s, radix) {
            Ok(value) => Ok(Self::Big(value)),
            Err(err) => Err(err),
        }
    }
}

impl Integer for LmtInteger {
    fn div_floor(&self, other: &Self) -> Self {
        match (self, other) {
            (Self::Small(a), Self::Small(b)) => Self::Small(a.div_floor(b)),
            (Self::Big(a), Self::Big(b)) => Self::Big(a.div_floor(b)),
            (Self::Small(a), Self::Big(b)) => Self::Big(BigInt::from(*a).div_floor(b)),
            (Self::Big(a), Self::Small(b)) => Self::Big(a.div_floor(&BigInt::from(*b))),
        }
    }

    fn mod_floor(&self, other: &Self) -> Self {
        match (self, other) {
            (Self::Small(a), Self::Small(b)) => Self::Small(a.mod_floor(b)),
            (Self::Big(a), Self::Big(b)) => Self::Big(a.mod_floor(b)),
            (Self::Small(a), Self::Big(b)) => Self::Big(BigInt::from(*a).mod_floor(b)),
            (Self::Big(a), Self::Small(b)) => Self::Big(a.mod_floor(&BigInt::from(*b))),
        }
    }

    fn div_mod_floor(&self, other: &Self) -> (Self, Self) {
        match (self, other) {
            (Self::Small(a), Self::Small(b)) => {
                let (div, mod_) = a.div_mod_floor(b);
                (Self::Small(div), Self::Small(mod_))
            }
            (Self::Big(a), Self::Big(b)) => {
                let (div, mod_) = a.div_mod_floor(b);
                (Self::Big(div), Self::Big(mod_))
            }
            (Self::Small(a), Self::Big(b)) => {
                let (div, mod_) = BigInt::from(*a).div_mod_floor(b);
                (Self::Big(div), Self::Big(mod_))
            }
            (Self::Big(a), Self::Small(b)) => {
                let (div, mod_) = a.div_mod_floor(&BigInt::from(*b));
                (Self::Big(div), Self::Big(mod_))
            }
        }
    }

    fn gcd(&self, other: &Self) -> Self {
        match (self, other) {
            (Self::Small(a), Self::Small(b)) => Self::Small(a.gcd(b)),
            (Self::Big(a), Self::Big(b)) => Self::Big(a.gcd(b)),
            (Self::Small(a), Self::Big(b)) => Self::Big(BigInt::from(*a).gcd(b)),
            (Self::Big(a), Self::Small(b)) => Self::Big(a.gcd(&BigInt::from(*b))),
        }
    }

    fn lcm(&self, other: &Self) -> Self {
        match (self, other) {
            (Self::Small(a), Self::Small(b)) => Self::Small(a.lcm(b)),
            (Self::Big(a), Self::Big(b)) => Self::Big(a.lcm(b)),
            (Self::Small(a), Self::Big(b)) => Self::Big(BigInt::from(*a).lcm(b)),
            (Self::Big(a), Self::Small(b)) => Self::Big(a.lcm(&BigInt::from(*b))),
        }
    }

    fn is_multiple_of(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Small(a), Self::Small(b)) => a.is_multiple_of(b),
            (Self::Big(a), Self::Big(b)) => a.is_multiple_of(b),
            (Self::Small(a), Self::Big(b)) => BigInt::from(*a).is_multiple_of(b),
            (Self::Big(a), Self::Small(b)) => a.is_multiple_of(&BigInt::from(*b)),
        }
    }

    fn is_even(&self) -> bool {
        match self {
            Self::Small(a) => a.is_even(),
            Self::Big(a) => a.is_even(),
        }
    }

    fn is_odd(&self) -> bool {
        match self {
            Self::Small(a) => a.is_odd(),
            Self::Big(a) => a.is_odd(),
        }
    }

    fn div_rem(&self, other: &Self) -> (Self, Self) {
        match (self, other) {
            (Self::Small(a), Self::Small(b)) => {
                let (div, rem) = a.div_rem(b);
                (Self::Small(div), Self::Small(rem))
            }
            (Self::Big(a), Self::Big(b)) => {
                let (div, rem) = a.div_rem(b);
                (Self::Big(div), Self::Big(rem))
            }
            (Self::Small(a), Self::Big(b)) => {
                let (div, rem) = BigInt::from(*a).div_rem(b);
                (Self::Big(div), Self::Big(rem))
            }
            (Self::Big(a), Self::Small(b)) => {
                let (div, rem) = a.div_rem(&BigInt::from(*b));
                (Self::Big(div), Self::Big(rem))
            }
        }
    }
}
