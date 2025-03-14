use core::{cmp, fmt, ops};

use fraction::{error, BigInt, Bounded, DynaInt, Num};

type I = DynaInt<isize, BigInt>;

#[derive(Debug, Clone)]
// // #[cfg_attr(feature = "serde", derive(bitcode::Encode, bitcode::Decode))]
pub struct LmtInteger {
    pub value: I,
    pub min: I,
    pub max: I,
}

impl LmtInteger {
    pub fn new(integer: I) -> Self {
        Self {
            value: integer.clone(),
            min: integer.clone(),
            max: integer,
        }
    }
}

impl ops::Add for &LmtInteger {
    type Output = LmtInteger;

    fn add(self, rhs: Self) -> Self::Output {
        LmtInteger {
            value: &self.value + &rhs.value,
            min: cmp::min(self.min.clone(), rhs.min.clone()),
            max: cmp::max(self.max.clone(), rhs.max.clone()),
        }
    }
}

impl ops::Div for &LmtInteger {
    type Output = LmtInteger;

    fn div(self, rhs: Self) -> Self::Output {
        LmtInteger {
            value: &self.value / &rhs.value,
            min: cmp::min(self.min.clone(), rhs.min.clone()),
            max: cmp::max(self.max.clone(), rhs.max.clone()),
        }
    }
}

impl PartialEq for LmtInteger {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl PartialOrd for LmtInteger {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.value.partial_cmp(&other.value)
    }
}

impl fmt::Display for LmtInteger {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

// impl Bounded for LmtInteger {
//     fn min_value() -> Self {
//         self.min
//     }

//     fn max_value() -> Self {
//         self.max
//     }
// }

impl core::str::FromStr for LmtInteger {
    type Err = <BigInt as Num>::FromStrRadixErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Num::from_str_radix(s, 10).map(Self::new)
    }
}

impl ops::Neg for LmtInteger {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::new(self.value.neg())
    }
}

// impl ops::Deref for LmtInteger {
//     type Target = I;

//     fn deref(&self) -> &Self::Target {
//         &self.value
//     }
// }
