// Same computation. JavaScript has no native decimal type, so this is
// double-precision float — fast, but not exact for money.
let total = 0.0;
let i = 0;
while (i < 100000) {
    const income = 300000 + i;
    const tax = (income - 300000) * 0.05;
    total = total + tax;
    i += 1;
}
console.log(total);
