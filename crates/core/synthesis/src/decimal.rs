use core::{cmp, fmt, ops};

use bigdecimal::FromPrimitive;
use lmt_parser::fraction::{BigDecimal, BigFraction, BigInt, BigUint, One, Zero};

#[derive(Debug, Clone)]
// #[cfg_attr(feature = "serde", derive(bitcode::Encode, bitcode::Decode))]
pub struct Decimal {
    pub value: BigDecimal,
    pub min: BigDecimal,
    pub max: BigDecimal,
}

impl Decimal {
    pub fn new(decimal: BigDecimal, precision: BigUint) -> Self {
        Self {
            value: decimal.clone(),
            min: decimal.clone(),
            max: decimal,
        }
    }
}

impl ops::Add for &Decimal {
    type Output = Decimal;

    fn add(self, rhs: Self) -> Self::Output {
        Decimal {
            value: &self.value + &rhs.value,
            min: cmp::min(self.min.clone(), rhs.min.clone()),
            max: cmp::max(self.max.clone(), rhs.max.clone()),
        }
    }
}

impl ops::Div for &Decimal {
    type Output = Decimal;

    fn div(self, rhs: Self) -> Self::Output {
        Decimal {
            value: &self.value / &rhs.value,
            min: cmp::min(self.min.clone(), rhs.min.clone()),
            max: cmp::max(self.max.clone(), rhs.max.clone()),
        }
    }
}

impl PartialEq for Decimal {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl PartialOrd for Decimal {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.value.partial_cmp(&other.value)
    }
}

impl fmt::Display for Decimal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = self.value.to_string();

        let precision = self.value.get_precision();

        if let Some((.., after_dot)) = value.split_once('.') {
            let trailing_zeros = "0".repeat(precision - after_dot.len());

            write!(f, "{}{}", value, trailing_zeros)
        } else {
            let trailing_zeros = "0".repeat(precision);

            write!(f, "{}.{trailing_zeros}", value)
        }
    }
}

#[test]
fn series() {
    // type D = DynaDecimal<u32, u8>;
    type D = lmt_number::LmtDecimal;

    fn pow(base: BigDecimal, exponent: usize) -> BigDecimal {
        let mut result = BigDecimal::one();

        for _ in 0..exponent {
            result *= base.clone();
        }

        result
    }

    fn bbp_pi(precision: usize) -> BigDecimal {
        let mut pi = BigDecimal::zero();

        let sixteen = BigDecimal::from(16);
        let one = BigDecimal::one();
        let two = BigDecimal::from(2);
        let eight = BigDecimal::from(8);
        let five = BigDecimal::from(5);
        let six = BigDecimal::from(6);
        let four = BigDecimal::from(4);

        for k in 0..precision {
            let eight_k = eight.clone() * BigDecimal::from(k);

            pi += one.clone() / pow(sixteen.clone(), k)
                * ((four.clone() / (eight_k.clone() + one.clone()))
                    - two.clone() / (eight_k.clone() + four.clone())
                    - one.clone() / (eight_k.clone() + five.clone())
                    - one.clone() / (eight_k.clone() + six.clone()));

            // println!(
            //     "{} * {} = {}",
            //     one.clone() / sixteen.pow(k),
            //     ((four.clone() / (eight_k.clone() + one.clone()))
            //         - two.clone() / (eight_k.clone() + four.clone())
            //         - one.clone() / (eight_k.clone() + five.clone())
            //         - one.clone() / (eight_k.clone() + six.clone())),
            //     one.clone() / sixteen.pow(k)
            //         * ((four.clone() / (eight_k.clone() + one.clone()))
            //             - two.clone() / (eight_k.clone() + four.clone())
            //             - one.clone() / (eight_k.clone() + five.clone())
            //             - one.clone() / (eight_k.clone() + six.clone()))
            // );

            println!("{k:0>4}: {}", pi.clone().set_precision(k).to_string());
        }

        pi
    }

    fn d_pi(precision: u32) -> D {
        let mut pi = D::zero().with_precision(precision);

        let sixteen = D::from(16);
        let one = D::one();
        let two = D::from(2);
        let eight = D::from(8);
        let five = D::from(5);
        let six = D::from(6);
        let four = D::from(4);

        for k in 0..precision {
            let eight_k = eight.clone() * D::from(k);

            pi += one.clone() / sixteen.pow(k)
                * ((four.clone() / (eight_k.clone() + one.clone()))
                    - two.clone() / (eight_k.clone() + four.clone())
                    - one.clone() / (eight_k.clone() + five.clone())
                    - one.clone() / (eight_k.clone() + six.clone()));

            println!("{k:0>4}: {}", pi.clone().with_precision(k).to_string());
        }

        pi
    }

    fn factorial(n: usize) -> BigDecimal {
        let mut result = BigDecimal::one();

        for i in 1..=n {
            result *= BigDecimal::from(i);
        }

        result
    }

    fn rabinowitz_and_wagon_pi(precision: usize) -> BigDecimal {
        let mut pi = BigDecimal::zero();

        let one = BigDecimal::one();
        let two = BigDecimal::from(2);
        let four = BigDecimal::from(4);
        let five = BigDecimal::from(5);
        let six = BigDecimal::from(6);
        let eight = BigDecimal::from(8);
        let sixteen = BigDecimal::from(16);

        for k in 0..precision {
            let numerator = factorial(6 * k)
                * (BigDecimal::from(545140134) * BigDecimal::from(k) + BigDecimal::from(13591409));
            let denominator =
                factorial(3 * k) * pow(factorial(k), 3) * pow(BigDecimal::from(-640320), k);

            pi += numerator / denominator;
        }

        pi *= sixteen.clone();

        pi
    }

    fn sqrt(n: D) -> D {
        let mut x = n.clone();

        let two = D::from(2);
        for i in 0..n.precision() {
            x = (x.clone() + n.clone() / x) / two.clone();
            let now = std::time::Instant::now();
            // println!("DIVIDING BY X");
            // println!("[{:0>5}ms]: DIVIDING BY X", now.elapsed().as_millis());
            // let a = n.clone() / x.clone();
            // println!(
            //     "{} / {} = {a}",
            //     n.clone().to_string(),
            //     x.clone().to_string()
            // );
            // println!("ADDING X");
            // println!("[{:0>5}ms]: ADDING X", now.elapsed().as_millis());
            // let b = x.clone() + a.clone();
            // println!(
            //     "{} + {} = {b}",
            //     x.clone().to_string(),
            //     a.clone().to_string()
            // );
            // println!("DIVIDING BY 2");
            // println!("[{:0>5}ms]: DIVIDING BY 2", now.elapsed().as_millis());
            // x = b / two.clone();
            // println!("DONE WITH ITERATION {i}");
            // println!("[{:0>5}ms]: DONE {i}", now.elapsed().as_millis());

            // println!("{i:0>2}: {}", x.clone().to_string());

            println!(
                "{i:0>2}: DONE [{:0>5}ms]: {}",
                now.elapsed().as_millis(),
                x.clone().to_string()
            );
        }

        println!("{x}");

        x

        // dbg!(x)
    }

    fn fraction_sqrt(n: BigFraction, precision: usize) -> BigFraction {
        let mut x = n.clone();

        let two = BigFraction::from(2);
        for i in 0..precision {
            // x = (x.clone() + n.clone() / x) / two.clone();
            let now = std::time::Instant::now();
            // println!("DIVIDING BY X");
            println!("[{:0>5}ms]: {n} / {x}", now.elapsed().as_millis());
            let a = n.clone() / x.clone();
            // println!(
            //     "{} / {} = {a}",
            //     n.clone().to_string(),
            //     x.clone().to_string()
            // );
            println!("   = {a}");
            // println!("ADDING X");
            println!("[{:0>5}ms]: {x} + a", now.elapsed().as_millis());
            let b = x.clone() + a.clone();
            // println!(
            //     "{} + {} = {b}",
            //     x.clone().to_string(),
            //     a.clone().to_string()
            // );
            println!("   {b}",);
            // println!("DIVIDING BY 2");
            println!("[{:0>5}ms]: {b} / 2", now.elapsed().as_millis());
            x = b / two.clone();
            // println!("DONE WITH ITERATION {i}");
            println!("   = {x}");
            println!("[{:0>5}ms]: DONE {i}", now.elapsed().as_millis());

            // println!("{i:0>2}: {}", x.clone().to_string());

            // println!(
            //     "{i:0>2}: DONE [{:0>5}ms]: {}",
            //     now.elapsed().as_millis(),
            //     x.clone().to_string()
            // );
        }

        println!("{x}");

        x

        // dbg!(x)
    }

    sqrt(D::from(2).with_precision(50));
    // d_pi(4);
    // fraction_sqrt(BigFraction::from(2), 20);
    // bbp_pi(500);
}
