use std::ops;

use num_bigint::BigInt;

use crate::LmtInteger;

impl ops::Rem for LmtInteger {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Small(a), Self::Small(b)) => {
                if let Some(c) = a.checked_rem(b) {
                    Self::Small(c)
                } else {
                    Self::Big(BigInt::from(a) % BigInt::from(b))
                }
            }
            (Self::Small(a), Self::Big(b)) => Self::Big(BigInt::from(a) % b),
            (Self::Big(a), Self::Small(b)) => Self::Big(a % BigInt::from(b)),
            (Self::Big(a), Self::Big(b)) => Self::Big(a % b),
        }
    }
}

impl ops::Rem for &LmtInteger {
    type Output = LmtInteger;

    fn rem(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (LmtInteger::Small(a), LmtInteger::Small(b)) => {
                if let Some(c) = a.checked_rem(*b) {
                    LmtInteger::Small(c)
                } else {
                    LmtInteger::Big(BigInt::from(*a) % BigInt::from(*b))
                }
            }
            (LmtInteger::Small(a), LmtInteger::Big(b)) => LmtInteger::Big(BigInt::from(*a) % b),
            (LmtInteger::Big(a), LmtInteger::Small(b)) => LmtInteger::Big(a % BigInt::from(*b)),
            (LmtInteger::Big(a), LmtInteger::Big(b)) => LmtInteger::Big(a % b),
        }
    }
}

impl ops::Rem<&LmtInteger> for LmtInteger {
    type Output = LmtInteger;

    fn rem(self, rhs: &LmtInteger) -> Self::Output {
        match (self, rhs) {
            (LmtInteger::Small(a), LmtInteger::Small(b)) => {
                if let Some(c) = a.checked_rem(*b) {
                    LmtInteger::Small(c)
                } else {
                    LmtInteger::Big(BigInt::from(a) % BigInt::from(*b))
                }
            }
            (LmtInteger::Small(a), LmtInteger::Big(b)) => LmtInteger::Big(BigInt::from(a) % b),
            (LmtInteger::Big(a), LmtInteger::Small(b)) => LmtInteger::Big(a % BigInt::from(*b)),
            (LmtInteger::Big(a), LmtInteger::Big(b)) => LmtInteger::Big(a % b),
        }
    }
}
