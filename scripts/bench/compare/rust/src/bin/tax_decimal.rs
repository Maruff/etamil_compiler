//! Exact decimals, via the crate eTamil's VM uses for every value.
//!
//! The gap between this and `tax.qmz` is the cost of interpreting bytecode.

use rust_decimal::Decimal;
use std::str::FromStr;

fn main() {
    let n: i64 = std::env::args()
        .nth(1)
        .expect("need N")
        .parse()
        .expect("N must be a number");

    let rate = Decimal::from_str("0.05").unwrap();
    let base = Decimal::from(300000);

    let mut total = Decimal::ZERO;
    let mut i = 0i64;
    while i < n {
        let income = base + Decimal::from(i);
        let tax = (income - base) * rate;
        total += tax;
        i += 1;
    }

    println!("{}", total.normalize());
}
