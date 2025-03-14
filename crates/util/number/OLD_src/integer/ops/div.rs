use std::ops;

use num_bigint::BigInt;

use crate::LmtInteger;

impl ops::Div for LmtInteger {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Small(a), Self::Small(b)) => {
                if let Some(c) = a.checked_div(b) {
                    Self::Small(c)
                } else {
                    Self::Big(BigInt::from(a) / BigInt::from(b))
                }
            }
            (Self::Small(a), Self::Big(b)) => Self::Big(BigInt::from(a) / b),
            (Self::Big(a), Self::Small(b)) => Self::Big(a / BigInt::from(b)),
            (Self::Big(a), Self::Big(b)) => Self::Big(a / b),
        }
    }
}

impl ops::Div for &LmtInteger {
    type Output = LmtInteger;

    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (LmtInteger::Small(a), LmtInteger::Small(b)) => {
                if let Some(c) = a.checked_div(*b) {
                    LmtInteger::Small(c)
                } else {
                    LmtInteger::Big(BigInt::from(*a) / BigInt::from(*b))
                }
            }
            (LmtInteger::Small(a), LmtInteger::Big(b)) => LmtInteger::Big(BigInt::from(*a) / b),
            (LmtInteger::Big(a), LmtInteger::Small(b)) => LmtInteger::Big(a / BigInt::from(*b)),
            (LmtInteger::Big(a), LmtInteger::Big(b)) => LmtInteger::Big(a / b),
        }
    }
}

impl ops::Div<&LmtInteger> for LmtInteger {
    type Output = LmtInteger;

    fn div(self, rhs: &LmtInteger) -> Self::Output {
        match (self, rhs) {
            (LmtInteger::Small(a), LmtInteger::Small(b)) => {
                if let Some(c) = a.checked_div(*b) {
                    LmtInteger::Small(c)
                } else {
                    LmtInteger::Big(BigInt::from(a) / BigInt::from(*b))
                }
            }
            (LmtInteger::Small(a), LmtInteger::Big(b)) => LmtInteger::Big(BigInt::from(a) / b),
            (LmtInteger::Big(a), LmtInteger::Small(b)) => LmtInteger::Big(a / BigInt::from(*b)),
            (LmtInteger::Big(a), LmtInteger::Big(b)) => LmtInteger::Big(a / b),
        }
    }
}

impl ops::Div<LmtInteger> for &LmtInteger {
    type Output = LmtInteger;

    fn div(self, rhs: LmtInteger) -> Self::Output {
        match (self, rhs) {
            (LmtInteger::Small(a), LmtInteger::Small(b)) => {
                if let Some(c) = a.checked_div(b) {
                    LmtInteger::Small(c)
                } else {
                    LmtInteger::Big(BigInt::from(*a) / BigInt::from(b))
                }
            }
            (LmtInteger::Small(a), LmtInteger::Big(b)) => LmtInteger::Big(BigInt::from(*a) / b),
            (LmtInteger::Big(a), LmtInteger::Small(b)) => LmtInteger::Big(a / BigInt::from(b)),
            (LmtInteger::Big(a), LmtInteger::Big(b)) => LmtInteger::Big(a / b),
        }
    }
}

impl<T, U> ops::Div<U> for LmtInteger<T>
where
    T: ops::Div<U, Output = T>,
    BigInt: ops::Div<U, Output = BigInt>,
{
    type Output = Self;

    fn div(self, rhs: U) -> Self::Output {
        match self {
            Self::Small(a) => Self::Small(a / rhs),
            Self::Big(a) => Self::Big(a / rhs),
        }
    }
}

impl ops::DivAssign for LmtInteger {
    fn div_assign(&mut self, rhs: Self) {
        *self = self.clone() / rhs;
    }
}
