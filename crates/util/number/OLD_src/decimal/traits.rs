use std::{borrow::Cow, cmp, collections::HashMap, fmt, ops};

use num_bigint::BigInt;
use num_integer::Integer;
use num_rational::Ratio;
use num_traits::{One, Zero};

use crate::{LmtDecimal, LmtInteger};

impl<I: Into<LmtInteger>> From<I> for LmtDecimal {
    fn from(value: I) -> Self {
        Self::new_from_parts(value.into(), 0, 0, None)
    }
}

impl PartialEq for LmtDecimal {
    fn eq(&self, other: &Self) -> bool {
        self.integer_part.times_ten_to_power(other.point_position)
            == other.integer_part.times_ten_to_power(self.point_position)
    }
}

impl Eq for LmtDecimal {}

impl PartialOrd for LmtDecimal {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LmtDecimal {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let self_integer_part = self.integer_part.times_ten_to_power(other.point_position);
        let other_integer_part = other.integer_part.times_ten_to_power(self.point_position);

        self_integer_part.cmp(&other_integer_part)
    }
}

impl fmt::Display for LmtDecimal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let integer_part_string = self.integer_part.to_string();
        let integer_part_length = integer_part_string.len();

        let (before_point, mut after_point) = if integer_part_length >= self.point_position as usize
        {
            let (before_point, after_point) =
                integer_part_string.split_at(integer_part_length - self.point_position as usize);

            (before_point, Cow::Borrowed(after_point))
        } else {
            let leading_zeros = "0".repeat(self.point_position as usize - integer_part_length);

            ("", Cow::Owned(leading_zeros + &integer_part_string))
        };

        // let precision_usize = self.precision as usize;

        // if after_point.len() > precision_usize {
        //     let (truncated, _) = after_point.split_at(precision_usize);
        //     after_point = Cow::Owned(truncated.to_owned());
        // }

        if before_point.is_empty() {
            write!(f, "0")?;
        } else {
            write!(f, "{before_point}")?;
        }

        // let after_point_length = after_point.len();

        // if precision_usize > after_point_length {
        //     let trailing_zeros = "0".repeat(precision_usize - after_point_length);

        //     write!(f, ".{after_point}{trailing_zeros}")?;
        // } else {
        write!(f, ".{after_point}")?;
        // }

        if let Some(remainder) = &self.repetend {
            write!(f, "[{remainder}]")?;
        }

        Ok(())
    }
}

impl ops::Add for LmtDecimal {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let integer_part = (self.integer_part.times_ten_to_power(rhs.point_position))
            + (rhs.integer_part.times_ten_to_power(self.point_position));

        let bigger_precision = cmp::max(self.precision, rhs.precision);

        Self::new_from_parts(
            integer_part,
            self.point_position + rhs.point_position,
            bigger_precision,
            None,
        )
    }
}

// impl<T, U> ops::Add<U> for LmtDecimal<T>
// where T: ops::Add<U, Output = T>
// {
//     type Output = Self;

//     fn add(self, rhs: U) -> Self::Output {
//         self
//     }
// }

impl ops::AddAssign for LmtDecimal {
    fn add_assign(&mut self, rhs: Self) {
        *self = self.clone() + rhs;
    }
}

impl ops::Sub for LmtDecimal {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        let integer_part = (self.integer_part.times_ten_to_power(rhs.point_position))
            - (rhs.integer_part.times_ten_to_power(self.point_position));

        let bigger_precision = cmp::max(self.precision, rhs.precision);

        Self::new_from_parts(
            integer_part,
            self.point_position + rhs.point_position,
            bigger_precision,
            None,
        )
    }
}

impl ops::Mul for LmtDecimal {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        return self / (Self::one() / rhs);

        let bigger_precision = cmp::max(self.precision, rhs.precision);

        let integer_part = (self.integer_part.times_ten_to_power(rhs.point_position))
            * (rhs.integer_part.times_ten_to_power(self.point_position));

        Self::new_from_parts(
            integer_part,
            (self.point_position + rhs.point_position) * 2,
            bigger_precision,
            None,
        )
    }
}

impl ops::Div for LmtDecimal {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        let bigger_precision = cmp::max(self.precision, rhs.precision);

        let mut numerator = self.integer_part.times_ten_to_power(rhs.point_position);
        let mut denominator = rhs.integer_part.times_ten_to_power(self.point_position);

        let repeating_point = denominator.log_base_10();

        let mut scale = 0;
        let mut integer_remainder: Option<Ratio<LmtInteger>> = None;

        loop {
            // let numer_rem = &numerator % &denominator;

            // if numer_rem.is_zero() {
            //     break;
            // }

            if scale >= repeating_point {
                break;
            }

            numerator *= LmtInteger::big_ten();
            scale += 1;

            // This is not greater than or equal to (>=) to keep exactly
            // one more digit than the precision to prevent rounding errors.
            // if scale > bigger_precision {
            //     // dbg!(
            //     //     &numer_rem,
            //     //     &denominator,
            //     //     // &self.remainder,
            //     //     // &rhs.remainder,
            //     //     // scale,
            //     //     // bigger_precision
            //     // );
            //     integer_remainder = Some(Ratio::new(numer_rem, denominator.clone()));

            //     // dbg!(&integer_remainder);

            //     break;
            // }
        }
        // while !(&numerator % &denominator).is_zero() {
        //     numerator *= LmtInteger::big_ten();
        //     scale += 1;

        //     if scale >= bigger_precision {
        //         integer_remainder =
        //             Some(Ratio::new(&numerator % &denominator, denominator.clone()));

        //         break;
        //     }
        // }

        // let scale_precision_difference = bigger_precision - scale;

        // let remainder = if scale_precision_difference > 0 {
        //     // Some(Ratio::new(
        //     //     numerator.clone() % denominator.clone(),
        //     //     denominator.clone(),
        //     // ))
        //     None
        // } else {
        //     integer_remainder
        // };

        // let remainder = integer_remainder;
        // let remainder = integer_remainder;

        let integer_part = numerator.clone() / denominator.clone();

        let repetend = match (&self.repetend, &rhs.repetend) {
            (Some(self_repetend), Some(rhs_repetend)) => {
                dbg!(self_repetend, rhs_repetend);

                todo!()
            }
            (Some(..), None) => self.repetend.clone(),
            (None, Some(..)) => rhs.repetend.clone(),
            (None, None) => repetend(&numerator, &denominator),
        };

        Self::new_from_parts(integer_part, scale, bigger_precision, repetend)
    }
}

impl One for LmtDecimal {
    fn one() -> Self {
        Self::new_from_parts(LmtInteger::one(), 0, 0, None)
    }
}

impl Zero for LmtDecimal {
    fn zero() -> Self {
        Self::new_from_parts(LmtInteger::zero(), 0, 0, None)
    }

    fn is_zero(&self) -> bool {
        self.integer_part.is_zero()
    }
}

/// Divide the modulo of the numerator and denominator by the denominator.
fn wrap_ratio(ratio: &Ratio<LmtInteger>) -> Ratio<LmtInteger> {
    let numer = ratio.numer();
    let denom = ratio.denom();

    let remainder = numer % denom;

    Ratio::new(remainder, denom.clone())
}

/// TODO: optimize.
#[must_use]
pub fn repetend(numerator: &LmtInteger, denominator: &LmtInteger) -> Option<LmtInteger> {
    let multiplicative_order = denominator.multiplicative_order()?;
    let remainder = numerator % denominator;

    let inv_denom_repetend = LmtInteger::one()
        .times_ten_to_power(multiplicative_order + denominator.log_base_10())
        / denominator;

    let repetend = remainder * inv_denom_repetend;
    Some(
        repetend, // &repetend
                 //     / LmtInteger::big_ten()
                 //         .times_ten_to_power(multiplicative_order - repetend.log_base_10()),
    )
}
