mod ops;

use core::fmt;
use std::{collections::HashMap, iter};

use num_bigint::BigInt;
use num_integer::Integer;
use num_traits::{Num, One, Pow, Zero};

#[derive(Debug, Clone, Hash)]
pub enum LmtInteger<T = i32> {
    Small(T),
    Big(BigInt),
}

impl LmtInteger {
    #[must_use]
    pub const fn small_ten() -> Self {
        Self::Small(10)
    }

    #[must_use]
    pub fn big_ten() -> Self {
        Self::Big(BigInt::from(10))
    }

    #[must_use]
    pub fn pow(&self, exponent: u32) -> Self {
        Pow::pow(self, exponent)
    }

    #[must_use]
    pub fn times_ten_to_power(&self, power: u32) -> Self {
        match self {
            Self::Small(value) => {
                if let Some(power) = 10_i32.checked_pow(power) {
                    Self::Small(value * power)
                } else {
                    Self::Big(BigInt::from(*value) * BigInt::from(10).pow(power))
                }
            }
            Self::Big(value) => Self::Big(value * BigInt::from(10).pow(power)),
        }
    }

    /// Find the repeating part of a rational number.
    /// We only need to find the multiplicative order of the denominator.
    #[must_use]
    pub fn multiplicative_order(&self) -> Option<u32> {
        let mut seen_remainders = HashMap::new();
        let mut current_remainder = LmtInteger::one();

        for i in 1_u32.. {
            current_remainder = (current_remainder * LmtInteger::big_ten()) % self;

            if current_remainder.is_zero() {
                return None;
            }

            if let Some(&j) = seen_remainders.get(&current_remainder) {
                return Some(i - j);
            }

            seen_remainders.insert(current_remainder.clone(), i);
        }

        None
    }

    pub fn log_base_10(&self) -> u32 {
        let mut current = self.clone();
        let mut count = 0;

        while current > LmtInteger::small_ten() {
            current /= LmtInteger::small_ten();
            count += 1;
        }

        count
    }
}

impl fmt::Display for LmtInteger {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Small(value) => write!(f, "{value}"),
            Self::Big(value) => write!(f, "{value}"),
        }
    }
}
