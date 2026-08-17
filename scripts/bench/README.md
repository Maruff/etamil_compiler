# Benchmarks

Reproducible measurements, plus the diagnostics behind the performance notes in
[docs/ARCHITECTURE.md](../../docs/ARCHITECTURE.md).

```bash
./scripts/bench/run.sh            # Linux / macOS
pwsh scripts/bench/run.ps1        # Windows
```

Build the release binary first: `cd etamil_compiler && cargo build --release`.

## What each file measures

| File | Measures |
|---|---|
| `tax.qmz` `tax_decimal.py` `tax_float.py` `tax.js` `tax.c` | The same 100,000 slab-tax calculations in four languages. All must print `249997500`. |
| `loop_only.qmz` | 9 bytecode instructions per iteration, almost no arithmetic — isolates per-instruction dispatch cost. |
| `loop_mul.qmz` | The same plus one decimal multiply per iteration. |
| `loop_vars.qmz` | The same plus four variable reads — isolates variable-lookup cost. |
| `append.qmz` | Array append at growing sizes. Shows the cost curve of `இணை`. |

## Method

Wall clock, minimum of several runs. **Startup is measured separately** with an
empty program of each language and subtracted, because for a 100 ms workload
process startup is not a rounding error — it is most of the number.

## Results, 2026-08-17, Windows 11, one core

eTamil 0.2.0 release, Python 3.14.6, Node 24.19.0, MSVC 19.44 `/O2`.

| Language | Startup | Arithmetic | Total |
|---|---|---|---|
| C — double | ~5 | 42 † | 47 ms |
| eTamil — decimal | 70 | 104 | 173 ms |
| JavaScript — double | 209 | 17 | 226 ms |
| Python — float | 814 | 7 | 821 ms |
| Python — Decimal | 813 | 57 | 870 ms |

† upper bound; C's arithmetic is below measurement resolution.

**Read the Python figure with care.** That is the Microsoft Store build, whose
~813 ms startup is pathological — a normal CPython starts in 20–40 ms. On Linux
Python's total would fall below eTamil's. The startup *finding* is real; the
*size* of the gap is an artefact of that machine. Rerun before quoting it.

## Diagnostics

Per-instruction cost, from `loop_only.qmz`: 900,000 instructions in 68 ms —
**≈75 ns per bytecode instruction**, and it stays near 75 ns whether or not the
instruction does arithmetic. A tuned bytecode VM is 5–15 ns. The cost is
dispatch, not decimal maths.

`append.qmz` compute time, startup subtracted:

| appends | time | growth |
|---|---|---|
| 4,000 | 360 ms | |
| 8,000 | 933 ms | ×2.6 |
| 16,000 | 4,997 ms | ×5.4 |

Quadratic. `இணை` clones the whole array per call, so building a list of *n*
items copies ~n²/2 elements. For the accounting framework — which builds ledgers
by appending — this dominates everything else, and it is the first thing to fix.
See ROADMAP.
