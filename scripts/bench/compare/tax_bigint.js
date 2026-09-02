// Exact, using BigInt scaled to paisa. JavaScript has no decimal type.
const n = BigInt(process.argv[2]);

let total = 0n;               // paisa
let i = 0n;
while (i < n) {
    const income = 30000000n + i * 100n;
    total += ((income - 30000000n) * 5n) / 100n;
    i += 1n;
}

const whole = total / 100n;
const frac = total % 100n;
console.log(frac === 0n ? `${whole}` : `${whole}.${String(frac).padStart(2, "0")}`);
