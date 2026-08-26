# Continuation notes

Read this first when picking up eTamil work in a fresh session. It records what
was done, what is deliberately unfinished, and the traps that cost time.

Last updated: 2026-08-22.

## State

Everything below is committed to `main` locally and **not pushed** — the user
pushes by hand.

```bash
cd etamil_compiler && cargo test          # 373 tests
cd .. && bash scripts/run_examples.sh     # 66 as expected, 1 skipped
```

## What was built recently, newest first

| Area | Where |
|---|---|
| LLVM: whole numbers as `i64`, exact division, globals | `src/codegen.rs`, `src/codegen_limits.rs`, `examples/finance/pYcA_kaNakku.qmz` |
| Money as whole paise | `nUlakam/kAcu.qmz` |
| LLVM refusal for f64 arithmetic | `etamil_compiler/src/codegen_limits.rs`, `docs/llvm-backend-gaps.md` |
| Database attempt-and-report; `:memory:` never pooled | `interpreter.rs`, `db/pool.rs` |
| Insurance, customs and trade | `nUlakam/kAppItu/`, `nUlakam/cuwkam/` |
| MongoDB (`--features mongodb`) | `src/mongo.rs` |
| Array and record equality (was broken) | `src/vm/value.rs` |
| Redis, RESP implemented here | `src/redis.rs`, `nUlakam/qaLam/retis.qmz` |
| Depreciation, payroll, tax rate tables | `nUlakam/kaNakkiyal/` |
| UPI addresses, links and states | `nUlakam/upi/` |
| Fabric via REST gateway | `nUlakam/cawkili/` |
| Core banking: interest, loans, asset classification | `nUlakam/vawki/` |
| Tests written in eTamil; `வெளியேறு` | `nUlakam/cOqaZY.qmz` |
| REPL (`--repl`) | `src/repl.rs` |
| mTLS, ECDSA | `src/mtls.rs`, `src/signing.rs` |
| ODF/OOXML documents | `nUlakam/AvaNam.qmz`, package builtins |

## Next, in the order the user asked for

1. **Run the LLVM work on Ubuntu.** Whole numbers are `i64` now, division under
   `தரை`/`மேல்` is exact, top-level names are globals a function can read, and
   the statements that used to fake file and database work are refusals. All of
   it is type-checked and none of it has been *run*. See the section below.
2. **The boxed value** — the next LLVM gap, and the one everything else waits
   on. `docs/llvm-backend-gaps.md` has the order.
3. **eCommerce libraries** — the one item from the original seven never started.
   Builds on the accounting and GST modules, which exist.
4. **gRPC and protobuf** — only if direct Fabric peer access is wanted. The REST
   gateway already works, so this buys little.

### The LLVM work has not been run

`llvm-sys 180` needs LLVM 18 and the feature is Linux/macOS only, so nothing in
`codegen.rs` has executed on the machine it was written on. It *is* type-checked:

```bash
python scripts/check_llvm_backend.py     # works on Windows; no LLVM needed
```

That builds a signature-only stand-in for `llvm-sys` out of llvm-sys's own
source in the cargo registry cache and runs `cargo check --features llvm`. So
the Rust compiles, and the file is editable here at all — it had broken three
times without anyone noticing before this existed. What it cannot tell you is
whether the IR verifies or whether it answers what the VM answers.

```bash
(cd etamil_compiler && cargo build --release --features llvm)
./scripts/run_parity.sh
```

`run_parity.sh` runs every example under both backends, fails only where they
*disagree* — a refusal is expected and counted — and ranks what stopped each
one. `examples/finance/pYcA_kaNakku.qmz` is the example written to compile on
both, so it is the one to look at first: if it refuses, read why; if it
mismatches, that is the bug.

Expect the possibility of a trivial IR-verifier complaint before judging
anything else. The likely candidates, in order: a block left without a
terminator around `எனில்` with a `திரும்பு` in it, and `printf`/`write`
argument types.

## Two sessions, one checkout

A second session worked in this same working tree for a while — a wasm build for
the browser editor — and landed it in `3db8985`. One commit, `5bb806c`,
accidentally swept its `wasm.rs` in alongside MongoDB work before that.

**Check `git status` before staging, and stage files by name rather than `-A`.**
Better: give a concurrent session its own git worktree. Staging by name is what
kept `0dbffe1` clean while `3db8985` was being written beside it.

### Editor support drifts silently

`eTamil_Code/src/generated/` and `eTamil_Code/syntaxes/` are generated from the
compiler and from nUlakam, and nothing regenerates them for you:

```bash
python scripts/generate_editor_support.py --check    # what CI runs
python scripts/generate_editor_support.py            # fix it
```

**Run the check before pushing, whenever a nUlakam function or a builtin was
added or renamed.** An earlier note here guessed the outstanding drift came from
the wasm session's `interpreter.rs` edits; it did not. It was ten `stdlib`
entries from `nUlakam/kAcu.qmz`, drifting since `b2de386` — a plain nUlakam
commit, no builtins involved. Adding a function to nUlakam is enough to fail
that gate.

## Traps that cost real time

**Reserved words.** Many of the words a financial library most wants are
keywords and cannot be used as names: `காப்பீடு`, `இழப்பு`, `விலக்கு`, `சரக்கு`,
`கட்டணம்`, `பங்கு`, `தேய்மானம்`, `சொத்து`, `செலவு`, `ஊதியம்`, `நிகர`, `விலை`,
`அசல்`, `தொகை`, `முறை`, `உரை`, `உடல்`, `படி`, `எழுது`, `வரம்பு`, `மதிப்பீடு`,
`தலைப்பு`, `விசை`, `இல்லை`, `பொருள்`, `வரிசை`. Compounds lex as ordinary
identifiers, so `அசல்_தொகை` is fine. **Check before writing, not after:**

```bash
grep -cE '#\[regex\("WORD\|' etamil_compiler/src/lexer.rs
```

**A function cannot change a global.** Assigning to a name inside a `செயல்`
makes a local. That is why `cOqaZY.qmz` threads the test run through every
assertion instead of keeping a counter.

**`சோதனை_முடிவு` returns on success and only exits on failure**, so calling it
in a skip path prints a summary and then carries on into the tests it meant to
skip. Branch with `இன்றேல்` instead.

**A SQL NULL arrives as `nil`,** and `nil` equals nothing — test it with
`வகை(x) == "nil"`, not by comparing values. And eTamil has no `nil` to bind, so
a column that should be NULL is *omitted from the INSERT* rather than bound to
`""`.

**`%` is postfix percentage, not infix modulo.** `18%` is `0.18`, and
`சதவீதம்` divides by 100 itself — so it wants `18`, not `18%`. Writing the
natural thing gives an answer a hundred times too small and nothing complains.
Both `vari.qmz` and `vatti.qmz` pin this in their suites.

**`&` and `+` bind equally and associate left.** So
`அச்சு "மொத்தம்: " & அ + ஆ` is `("மொத்தம்: " & அ) + ஆ`, which is a number, and
the label vanishes from the output with nothing complaining. Bracket the sum:
`& (அ + ஆ)`. Cost a wrong line in `pYcA_kaNakku.qmz` before the VM run caught
it.

**Windows shell.** Heredocs mangle backslashes and backticks in shell
double-quotes are command substitution — a ROADMAP line lost all its code spans
that way. Write patches as files with the Write tool and run them with
`PYTHONIOENCODING=utf-8 python file.py`; console output needs that or Tamil
crashes on cp1252.

## The rule the libraries follow

Engine in eTamil, statutory figures in SQLite, supplied and verified by the
user. No rate, threshold or slab is hardcoded anywhere. Tables are
effective-dated so a return re-run next year produces what it produced when it
was filed, and `சரிபார்க்கப்படாதவை`-style functions list rows still marked
`PLACEHOLDER` so a program can refuse to file a figure nobody has vouched for.

`nUlakam/kaNakkiyal/vari_vikiqam.sql` ships the schema and the 36 states with
their GST codes, and **no rates at all**.
