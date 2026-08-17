#!/usr/bin/env bash
#
# Time the same computation in every available language, with process startup
# measured separately and subtracted. Startup is not a rounding error at this
# workload size — it is most of the number.
#
#   ./scripts/bench/run.sh

set -uo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"
ETAMIL="${ETAMIL_BIN:-$ROOT/etamil_compiler/target/release/etamil}"
RUNS="${RUNS:-7}"

if [[ ! -x "$ETAMIL" ]]; then
    echo "error: $ETAMIL not found — (cd etamil_compiler && cargo build --release)"
    exit 1
fi

# Minimum wall-clock milliseconds over RUNS executions.
best_ms() {
    local best=""
    for _ in $(seq "$RUNS"); do
        local start end ms
        start=$(date +%s%N)
        "$@" >/dev/null 2>&1
        end=$(date +%s%N)
        ms=$(( (end - start) / 1000000 ))
        if [[ -z "$best" || $ms -lt $best ]]; then best=$ms; fi
    done
    echo "$best"
}

row() { printf '%-22s %9s %11s %9s\n' "$1" "$2" "$3" "$4"; }

echo "=== answers (all must be 249997500) ==="
printf '  %-14s %s\n' "eTamil"   "$("$ETAMIL" --vm "$HERE/tax.qmz" | tail -n 3 | head -n 1)"
command -v python3 >/dev/null && printf '  %-14s %s\n' "Python dec" "$(python3 "$HERE/tax_decimal.py")"
command -v python3 >/dev/null && printf '  %-14s %s\n' "Python float" "$(python3 "$HERE/tax_float.py")"
command -v node    >/dev/null && printf '  %-14s %s\n' "JavaScript" "$(node "$HERE/tax.js")"

if command -v cc >/dev/null; then
    cc -O2 -o "$HERE/tax_c" "$HERE/tax.c" 2>/dev/null && printf '  %-14s %s\n' "C" "$("$HERE/tax_c")"
fi

echo
echo "=== timings, minimum of $RUNS runs ==="
row "language" "total ms" "startup ms" "compute"

e_start=$(best_ms "$ETAMIL" --vm "$HERE/empty.qmz")
e_total=$(best_ms "$ETAMIL" --vm "$HERE/tax.qmz")
row "eTamil (decimal)" "$e_total" "$e_start" "$(( e_total - e_start ))"

if command -v python3 >/dev/null; then
    p_start=$(best_ms python3 "$HERE/empty.py")
    for v in decimal float; do
        t=$(best_ms python3 "$HERE/tax_$v.py")
        row "Python ($v)" "$t" "$p_start" "$(( t - p_start ))"
    done
fi

if command -v node >/dev/null; then
    n_start=$(best_ms node "$HERE/empty.js")
    t=$(best_ms node "$HERE/tax.js")
    row "JavaScript (double)" "$t" "$n_start" "$(( t - n_start ))"
fi

if [[ -x "$HERE/tax_c" ]]; then
    t=$(best_ms "$HERE/tax_c")
    row "C (double, -O2)" "$t" "~1" "$t"
fi

echo
echo "=== diagnostics: where the VM's time goes ==="
row "program" "total ms" "startup ms" "compute"
for f in loop_only loop_mul loop_vars append; do
    t=$(best_ms "$ETAMIL" --vm "$HERE/$f.qmz")
    row "$f" "$t" "$e_start" "$(( t - e_start ))"
done

echo
echo "loop_only runs 900,000 bytecode instructions. compute_ms * 1e6 / 900000"
echo "gives nanoseconds per instruction; a tuned bytecode VM sits at 5-15 ns."
