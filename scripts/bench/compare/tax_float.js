// Binary float. Fast, and not exact for money.
const n = Number(process.argv[2]);

let total = 0.0;
let i = 0;
while (i < n) {
    const income = 300000.0 + i;
    const tax = (income - 300000.0) * 0.05;
    total = total + tax;
    i += 1;
}
console.log(total.toFixed(2));
