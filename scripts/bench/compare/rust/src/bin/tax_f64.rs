//! Binary float, for the other half of the table.

fn main() {
    let n: i64 = std::env::args()
        .nth(1)
        .expect("need N")
        .parse()
        .expect("N must be a number");

    let mut total = 0.0f64;
    let mut i = 0i64;
    while i < n {
        let income = 300000.0 + i as f64;
        let tax = (income - 300000.0) * 0.05;
        total += tax;
        i += 1;
    }

    println!("{:.2}", total);
}
