use std::ops;

use num_bigint::BigInt;

use crate::LmtInteger;

impl ops::Mul for LmtInteger {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Small(a), Self::Small(b)) => {
                if let Some(c) = a.checked_mul(b) {
                    Self::Small(c)
                } else {
                    Self::Big(BigInt::from(a) * BigInt::from(b))
                }
            }
            (Self::Small(a), Self::Big(b)) => Self::Big(BigInt::from(a) * b),
            (Self::Big(a), Self::Small(b)) => Self::Big(a * BigInt::from(b)),
            (Self::Big(a), Self::Big(b)) => Self::Big(a * b),
        }
    }
}

impl ops::Mul for &LmtInteger {
    type Output = LmtInteger;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (LmtInteger::Small(a), LmtInteger::Small(b)) => {
                if let Some(c) = a.checked_mul(*b) {
                    LmtInteger::Small(c)
                } else {
                    LmtInteger::Big(BigInt::from(*a) * BigInt::from(*b))
                }
            }
            (LmtInteger::Small(a), LmtInteger::Big(b)) => LmtInteger::Big(BigInt::from(*a) * b),
            (LmtInteger::Big(a), LmtInteger::Small(b)) => LmtInteger::Big(a * BigInt::from(*b)),
            (LmtInteger::Big(a), LmtInteger::Big(b)) => LmtInteger::Big(a * b),
        }
    }
}

impl ops::Mul<&LmtInteger> for LmtInteger {
    type Output = LmtInteger;

    fn mul(self, rhs: &LmtInteger) -> Self::Output {
        match (self, rhs) {
            (LmtInteger::Small(a), LmtInteger::Small(b)) => {
                if let Some(c) = a.checked_mul(*b) {
                    LmtInteger::Small(c)
                } else {
                    LmtInteger::Big(BigInt::from(a) * BigInt::from(*b))
                }
            }
            (LmtInteger::Small(a), LmtInteger::Big(b)) => LmtInteger::Big(BigInt::from(a) * b),
            (LmtInteger::Big(a), LmtInteger::Small(b)) => LmtInteger::Big(a * BigInt::from(*b)),
            (LmtInteger::Big(a), LmtInteger::Big(b)) => LmtInteger::Big(a * b),
        }
    }
}

impl ops::Mul<LmtInteger> for &LmtInteger {
    type Output = LmtInteger;

    fn mul(self, rhs: LmtInteger) -> Self::Output {
        match (self, rhs) {
            (LmtInteger::Small(a), LmtInteger::Small(b)) => {
                if let Some(c) = a.checked_mul(b) {
                    LmtInteger::Small(c)
                } else {
                    LmtInteger::Big(BigInt::from(*a) * BigInt::from(b))
                }
            }
            (LmtInteger::Small(a), LmtInteger::Big(b)) => LmtInteger::Big(BigInt::from(*a) * b),
            (LmtInteger::Big(a), LmtInteger::Small(b)) => LmtInteger::Big(a * BigInt::from(b)),
            (LmtInteger::Big(a), LmtInteger::Big(b)) => LmtInteger::Big(a * b),
        }
    }
}

impl<T, U> ops::Mul<U> for LmtInteger<T>
where
    T: ops::Mul<U, Output = T>,
    BigInt: ops::Mul<U, Output = BigInt>,
{
    type Output = Self;

    fn mul(self, rhs: U) -> Self::Output {
        match self {
            Self::Small(a) => Self::Small(a * rhs),
            Self::Big(a) => Self::Big(a * rhs),
        }
    }
}

impl ops::MulAssign for LmtInteger {
    fn mul_assign(&mut self, rhs: Self) {
        *self = self.clone() * rhs;
    }
}
