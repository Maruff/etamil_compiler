# Testing eTamil on Ubuntu

Every command here is meant to be run from a fresh clone. Where something is
expected to fail, that is stated — a few examples fail deliberately, and a
run that "passes" them would mean a regression.

```bash
git clone https://github.com/Maruff/etamil_compiler.git
cd etamil_compiler
```

---

## 1. Prerequisites

```bash
sudo apt update
sudo apt install -y build-essential pkg-config curl git
```

Rust **1.85 or newer** (the crate uses edition 2024):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
rustc --version          # expect 1.85.0 or newer
```

Optional, only for the `--llvm` backend — note it must be **LLVM 18**, since
`llvm-sys` is pinned to `"180"`:

```bash
sudo apt install -y llvm-18-dev libpolly-18-dev
export LLVM_SYS_180_PREFIX=/usr/lib/llvm-18
```

---

## 2. Build

```bash
cd etamil_compiler
cargo build --release
```

The binary is `target/release/etamil`. Put it on your PATH for the rest of
this document:

```bash
export PATH="$PWD/target/release:$PATH"
export ETAMIL_PATH="$PWD/.."          # so இறக்கு can find nUlakam/
etamil --version
etamil --help
```

**Expected:** a version line and a usage block. If `etamil --version` hangs,
you are on an old build — that bug is fixed.

---

## 3. The test suite

```bash
cd etamil_compiler
cargo test
```

**Expected: 105 passed, 0 failed**, plus the unit tests inside `src/http/`
and `src/fileio/`.

The suite asserts on **program results**, not exit codes. An older suite
checked only that examples exited 0, which is why several wrong-answer bugs
survived in it — do not reintroduce that style.

Useful variations:

```bash
cargo test foreach            # one group
cargo test -- --nocapture     # see program output
cargo test --release          # optimised
```

---

## 4. Run every example

```bash
cd ..            # repository root
./scripts/run_examples.sh
```

**Expected: 20 ran, 3 failed by design.** The script fails the build if any
example other than those three breaks, or if one of those three unexpectedly
succeeds.

The three deliberate failures, all reporting "not implemented in the VM yet":

| Example | Why |
|---|---|
| `examples/api/simple_api.qmz` | `வழி` route statements are not executable |
| `examples/db_samples/multi_db_test.qmz` | no database layer |
| `examples/db_samples/test_db_connectivity.qmz` | no database layer |

That is the intended behaviour: statements the VM cannot execute fail loudly
rather than being silently skipped.

---

## 5. Run examples individually

### Language features

```bash
etamil --vm examples/language/ceyalkaL.qmz
```
Functions, recursion, `சரி`/`தவறு` results and the `?` operator.
**Expect:** `காரணீயம்(6) = 720`, an average of `333.33`, a caught error.

```bash
etamil --vm examples/language/aNikaL_poruLkaL.qmz
```
Arrays (columns), records (rows), iteration, `புலம்_எடு`.
**Expect:** a total outstanding of `59800`.

### Finance

```bash
etamil --vm examples/finance/vaNikavari_pattiyal.qmz
```
A GST invoice with CGST/SGST split and Indian digit grouping.
**Expect:** amounts like `₹1,64,997.00`, tax split evenly across CGST/SGST.

```bash
etamil --vm examples/finance/campaLam.qmz
```
A payroll run over an array of employee records with slab tax and PF.
**Expect:** four employees and a total in `₹` with lakh/crore grouping.

### Basics and I/O

```bash
echo "950000" | etamil --vm examples/basic_samples/example.qmz
```
**Expect:** `High Tax Bracket` then `30000`.

```bash
echo "500000" | etamil --vm examples/basic_samples/example.qmz
```
**Expect:** `Low Tax Bracket (No Tax)`.

```bash
cd /tmp && etamil --vm ~/etamil_compiler/examples/io_samples/simple_fileio.qmz
```
These write files into the **current directory**, so run them somewhere
disposable. **Expect:** `கோப்பிலிருந்து வருவாய்: 125000` and an `output.txt`.

### The standard library

```bash
etamil --vm nUlakam/paNam.qmz     # loads cleanly, defines functions, prints nothing
```

To exercise it:

```bash
cat > /tmp/money.qmz <<'EOF'
இறக்கு "nUlakam/paNam.qmz";
அச்சு ரூபாய்(12345678.5);
அச்சு காசு_வடிவம்(0 - 4500.5);
EOF
etamil --vm /tmp/money.qmz
```

**Expect:**
```
₹1,23,45,678.50
-4,500.50
```

Note the Indian grouping — three digits, then pairs — not `12,345,678.50`.

---

## 5b. The accounting framework

```bash
etamil --vm examples/finance/kaNakkiyal.qmz
```

A full cycle: opening capital, a GST sale, a GST purchase, rent, a receipt,
and a deliberately unbalanced entry that must be refused.

**Expect:**

| | |
|---|---|
| Unbalanced entry | `✓ சமநிலையற்ற பரிவர்த்தனை மறுக்கப்பட்டது` |
| Trial balance | `பற்று 7,86,000.00   வரவு 7,86,000.00` and `✓ சமநிலை` |
| Net profit | `₹1,05,000.00` |
| Balance sheet | assets `₹6,91,000.00` and `✓ சொத்து = பொறுப்பு + பங்கு` |

If the trial balance does not agree, the ledger is being written by something
other than `பதிவிடு`, since posting refuses anything unbalanced.

---

## 6. Check decimal correctness

This is the property the language exists for, so it is worth verifying
directly:

```bash
cat > /tmp/exact.qmz <<'EOF'
அச்சு 0.1 + 0.2;
அச்சு 99.99 * 3;
அச்சு 20%;
EOF
etamil --vm /tmp/exact.qmz
```

**Expect exactly:**
```
0.3
299.97
0.2
```

If you see `0.30000000000000004`, the build has regressed to `f64`.

---

## 7. The HTTP server

```bash
etamil --server --port 8080 examples/backend/hello_server.qmz &
sleep 1
curl -i http://localhost:8080/
curl -s http://localhost:8080/health
```

**Expect:** an HTTP response, and a worker-count line at startup.

### Verify it is actually concurrent

The server used to handle one connection at a time. Twenty parallel requests
should all return promptly:

```bash
time (seq 1 20 | xargs -P 20 -I{} curl -s -o /dev/null -w "%{http_code}\n" http://localhost:8080/)
```

**Expect:** twenty `200`s, finishing in well under a second. If the elapsed
time scales linearly with request count, the worker pool has regressed to
serial handling.

Tune the pool:

```bash
ETAMIL_WORKERS=4 etamil --server --port 8080 examples/backend/hello_server.qmz
```

Stop it with `kill %1`.

`--async` currently prints a warning and runs the same synchronous server;
that is expected until roadmap item 4 lands.

---

## 7b. The database layer

SQLite is compiled in by default (`--features sqlite`, on unless you pass
`--no-default-features`). `rusqlite` is bundled, so no system SQLite is
needed — but it does compile C, which is why `build-essential` is required.

```bash
cd /tmp
etamil --vm ~/etamil_compiler/examples/db_samples/kaNakku_qaLam.qmz
```

**Expect:** a ledger created, four rows inserted, credits and debits listed
with `₹` formatting, and a closing balance of `₹2,17,300.00`. It writes
`kaNakku.db` in the current directory, hence running it from `/tmp`.

Check that parameters really are bound rather than interpolated:

```bash
cat > /tmp/inject.qmz <<'EOF'
தளம்_இணை சீகுலைட், ":memory:";
தளம்_செய் "CREATE TABLE t (peyar TEXT)", [];
தளம்_செய் "INSERT INTO t VALUES (?)", ["Ravi'; DROP TABLE t; --"];
தளம்_வினா "SELECT peyar FROM t WHERE peyar = ?", ["Ravi'; DROP TABLE t; --"], r;
அச்சு நீளம்(r);
EOF
etamil --vm /tmp/inject.qmz
```

**Expect `1`.** The table survives and the hostile string is matched as data.
If the table were dropped, the value would have reached SQLite as SQL.

A build without the driver should say so rather than fail obscurely:

```bash
cd etamil_compiler && cargo build --release --no-default-features
./target/release/etamil --vm /tmp/inject.qmz
```

**Expect:** `this build has no SQLite support: rebuild with --features sqlite`.

---

## 8. The LLVM backend

Linux only, and the least-exercised path in the repository — treat a failure
here as likely rather than surprising.

```bash
cd etamil_compiler
cargo build --release --features llvm
./target/release/etamil --llvm ../examples/basic_samples/example.qmz
cat output.ll
```

**Expect:** LLVM IR written to `output.ll`. Without the feature, `--llvm`
prints an explanatory error and exits 1, which is also correct.

---

## 9. Romanization audit

```bash
python3 scripts/transliterate.py --check
```

**Expect: 19 keywords reported off-scheme.** These are known and catalogued
in `docs/ROADMAP.md` item 6. The number should go **down**, never up. CI runs
this as a non-gating step.

Transliterate a word by hand:

```bash
python3 scripts/transliterate.py வணக்கம் நிதி நாணயம்
```

Regenerate the keyword reference after changing the token list:

```bash
python3 scripts/generate_keywords.py
git diff --stat docs/reference/KEYWORDS.md      # expect no diff if unchanged
```

---

## 10. Full check, in one go

```bash
cd etamil_compiler && cargo build --release && cargo test && cd .. \
  && ./scripts/run_examples.sh \
  && python3 scripts/transliterate.py --check ; echo "audit exit: $?"
```

Everything except the audit should exit 0.

---

## Troubleshooting

**`linker 'cc' not found`** — install `build-essential`.

**`edition2024 is required`** — Rust is older than 1.85. Run `rustup update`.

**`error: could not find native static library 'Polly'`** — LLVM 18 dev
packages are missing or `LLVM_SYS_180_PREFIX` is unset. Only affects
`--features llvm`.

**`cannot open module 'nUlakam/...'`** — set `ETAMIL_PATH` to the repository
root, or run from a directory where the relative path resolves. `இறக்கு`
searches beside the importing file, then `ETAMIL_PATH`, then next to the
binary.

**A file example says `cannot read`** — run it from a writable directory; the
I/O examples create their input files in the current directory.

**`bad interpreter: /bin/bash^M`** — the checkout has CRLF line endings.
`.gitattributes` should prevent this; if it happens, run
`git config core.autocrlf false && git rm --cached -r . && git reset --hard`.

---

## What to report back

If something fails, the useful details are:

1. `rustc --version` and `uname -a`
2. the exact command and its full output
3. for a test failure, `cargo test -- --nocapture` output for that test
4. whether `cargo test` alone passes, since that isolates compiler bugs from
   example or environment problems
