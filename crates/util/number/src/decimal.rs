use core::{cmp, fmt, ops, str};

use fraction::{
    error, BigDecimal, BigFraction, BigInt, BigUint, DynaDecimal, DynaFraction, GenericDecimal,
    One, Zero,
};

type D = DynaDecimal<usize, u8>;
type F = DynaFraction<usize>;

#[derive(Debug, Clone)]
// // #[cfg_attr(feature = "serde", derive(bitcode::Encode, bitcode::Decode))]
pub struct LmtDecimal {
    pub value: D,
    // pub min: F,
    // pub max: F,
}

impl LmtDecimal {
    pub fn new(decimal: D) -> Self {
        Self {
            value: decimal.clone(),
            // min: decimal_to_fraction(decimal)
            // max: decimal,
        }
    }

    #[must_use]
    pub fn with_precision(self, precision: u8) -> Self {
        Self::new(self.value.set_precision(precision))
    }
}

impl ops::Add for &LmtDecimal {
    type Output = LmtDecimal;

    fn add(self, rhs: Self) -> Self::Output {
        LmtDecimal {
            value: &self.value + &rhs.value,
            // min: cmp::min(self.min.clone(), rhs.min.clone()),
            // max: cmp::max(self.max.clone(), rhs.max.clone()),
        }
    }
}

impl ops::Div for &LmtDecimal {
    type Output = LmtDecimal;

    fn div(self, rhs: Self) -> Self::Output {
        LmtDecimal {
            value: &self.value / &rhs.value,
            // min: cmp::min(self.min.clone(), rhs.min.clone()),
            // max: cmp::max(self.max.clone(), rhs.max.clone()),
        }
    }
}

impl PartialEq for LmtDecimal {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl PartialOrd for LmtDecimal {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.value.partial_cmp(&other.value)
    }
}

impl fmt::Display for LmtDecimal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = self.value.to_string();

        let precision = self.value.get_precision() as usize;

        if let Some((.., after_dot)) = value.split_once('.') {
            let trailing_zeros = "0".repeat(precision - after_dot.len());

            write!(f, "{}{}", value, trailing_zeros)
        } else {
            let trailing_zeros = "0".repeat(precision);

            write!(f, "{}.{trailing_zeros}", value)
        }
    }
}

impl str::FromStr for LmtDecimal {
    type Err = error::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        D::from_str(s).map(Self::new)
    }
}

impl ops::Neg for LmtDecimal {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::new(self.value.neg())
    }
}

// impl ops::Deref for LmtDecimal {
//     type Target = D;

//     fn deref(&self) -> &Self::Target {
//         &self.value
//     }
// }
