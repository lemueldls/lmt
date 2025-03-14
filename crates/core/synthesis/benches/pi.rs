use criterion::{criterion_group, criterion_main, Criterion};
use lmt_number::fraction::{BigDecimal, One, Zero};

fn pow_bb(base: BigDecimal, exponent: usize) -> BigDecimal {
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
        let eight_k = eight.clone() * k;

        pi += one.clone() / pow_bb(sixteen.clone(), k)
            * ((four.clone() / (eight_k.clone() + one.clone()))
                - two.clone() / (eight_k.clone() + four.clone())
                - one.clone() / (eight_k.clone() + five.clone())
                - one.clone() / (eight_k.clone() + six.clone()));
    }

    pi
}

fn bbp_pi_compact(precision: usize) -> BigDecimal {
    let mut pi = BigDecimal::zero();

    let one = BigDecimal::one();
    let one_hundred_twenty = BigDecimal::from(120);
    let sixteen = BigDecimal::from(16);
    let one_hundred_fifty_one = BigDecimal::from(151);
    let forty_seven = BigDecimal::from(47);
    let five_hundred_twelve = BigDecimal::from(512);
    let one_thousand_twenty_four = BigDecimal::from(1024);
    let seven_hundred_twelve = BigDecimal::from(712);
    let one_hundred_ninety_four = BigDecimal::from(194);
    let fifteen = BigDecimal::from(15);

    for k in 0..precision {
        pi += one.clone() / pow_bb(sixteen.clone(), k)
            * ((one_hundred_twenty.clone() * k.pow(2)
                + one_hundred_fifty_one.clone() * k
                + forty_seven.clone())
                / (five_hundred_twelve.clone() * k.pow(4)
                    + one_thousand_twenty_four.clone() * k.pow(3)
                    + seven_hundred_twelve.clone() * k.pow(2)
                    + one_hundred_ninety_four.clone() * k
                    + fifteen.clone()));
    }

    pi * BigDecimal::from(0.015625)
}

fn criterion_benchmark(c: &mut Criterion) {
    // GROUP: ZERO PRECISION
    c.bench_function("bbp_pi(0)", |b| b.iter(|| bbp_pi(0)));
    c.bench_function("bbp_pi_compact(0)", |b| b.iter(|| bbp_pi_compact(0)));
    // GROUP: LOW PRECISION
    c.bench_function("bbp_pi(10)", |b| b.iter(|| bbp_pi(10)));
    c.bench_function("bbp_pi_compact(10)", |b| b.iter(|| bbp_pi_compact(10)));
    // GROUP: MEDIUM PRECISION
    c.bench_function("bbp_pi(100)", |b| b.iter(|| bbp_pi(100)));
    c.bench_function("bbp_pi_compact(100)", |b| b.iter(|| bbp_pi_compact(100)));
    // GROUP: HIGH PRECISION
    // c.bench_function("bbp_pi(1000)", |b| b.iter(|| bbp_pi((1000))));
    // c.bench_function("bbp_pi_compact(1000)", |b| b.iter(|| bbp_pi_compact((1000))));
    // GROUP: EXTREME PRECISION
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
