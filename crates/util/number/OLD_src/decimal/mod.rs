mod traits;

use num_rational::Ratio;
use num_traits::{pow, Pow, Zero};

use crate::LmtInteger;

#[derive(Debug, Clone)]
pub struct LmtDecimal {
    integer_part: LmtInteger,
    point_position: u32,
    precision: u32,
    // remainder: Option<Ratio<LmtInteger>>,
    repetend: Option<LmtInteger>,
}

impl LmtDecimal {
    #[must_use]
    pub const fn new_from_parts(
        integer_part: LmtInteger,
        point_position: u32,
        precision: u32,
        // remainder: Option<Ratio<LmtInteger>>,
        repetend: Option<LmtInteger>,
    ) -> Self {
        Self {
            integer_part,
            point_position,
            precision,
            // remainder,
            repetend,
        }
    }

    #[must_use]
    pub const fn precision(&self) -> u32 {
        self.precision
    }

    #[must_use]
    pub fn with_precision(mut self, precision: u32) -> Self {
        self.precision = precision;

        self
    }

    #[must_use]
    pub fn pow(&self, exponent: u32) -> Self {
        Pow::pow(self, exponent)
    }
}

impl Pow<u32> for &LmtDecimal {
    type Output = LmtDecimal;

    fn pow(self, exponent: u32) -> Self::Output {
        let mut clone = self.clone();

        for _ in 0..exponent {
            clone = clone * self.clone();
        }

        clone
    }
}

#[test]
fn test() {
    // let a = LmtDecimal::new_from_parts(LmtInteger::from(46), 1, 5, None);
    // let b = LmtDecimal::new_from_parts(LmtInteger::from(82), 2, 5, None);
    // let a = LmtDecimal::new_from_parts(LmtInteger::from(8), 0, 5, None);
    // let b = LmtDecimal::new_from_parts(LmtInteger::from(9), 0, 5, None);
    // let c = LmtDecimal::new_from_parts(LmtInteger::from(4), 0, 5, None);
    // let d = LmtDecimal::new_from_parts(LmtInteger::from(7), 0, 5, None);
    let a = LmtDecimal::new_from_parts(LmtInteger::from(5), 0, 7, None);
    let b = LmtDecimal::new_from_parts(LmtInteger::from(6), 0, 7, None);
    // let c = LmtDecimal::new_from_parts(LmtInteger::from(5), 0, 7, None);
    // let d = LmtDecimal::new_from_parts(LmtInteger::from(7), 0, 7, None);

    // println!("({a}/{b}) = {}", a.clone() / b.clone());
    // println!("({c}/{d}) = {}", c.clone() / d.clone());
    // let x = (a / b) / (c / d);
    let x = a / b;

    dbg!(&x);

    println!("   = {x}");
    // println!("{x}");
}
