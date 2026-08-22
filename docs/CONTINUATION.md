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
| Money as whole paise | `nUlakam/kAcu.qmz` |
| LLVM refusal for f64 arithmetic | `etamil_compiler/src/codegen_limits.rs`, `docs/llvm-backend-gaps.md` |
| Database attempt-and-report; `:memory:` never pooled | `interpreter.rs`, `db/pool.rs` |
| Insurance, customs and trade | `nUlakam/kAppIttu/`, `nUlakam/cuwkam/` |
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

1. **LLVM: whole numbers as `i64` with `LLVMBuildSDiv`.** This is the cheapest
   correctness win and it unlocks `kAcu.qmz` on the backend. See
   `docs/llvm-backend-gaps.md` — the gaps are counted from source, worst first,
   with a suggested order.
2. **eCommerce libraries** — the one item from the original seven never started.
   Builds on the accounting and GST modules, which exist.
3. **gRPC and protobuf** — only if direct Fabric peer access is wanted. The REST
   gateway already works, so this buys little.

### One unverified line

`codegen.rs` consults `codegen_limits::refusals` in one line that **compiles
nowhere it was written**: `llvm-sys 180` needs LLVM 18 installed and the feature
is Linux/macOS only, so a Windows machine cannot even type-check that file.
Build `--features llvm` on Ubuntu first and expect the possibility of a trivial
error there before judging anything else.

`scripts/run_parity.sh` runs every example under both backends and reports what
stopped each one, ranked. Run it before deciding what to fix.

## Two sessions, one checkout

Another session has been working in this same working tree throughout — a wasm
build for the browser editor (`src/wasm.rs`, `src/wasm_stubs.rs`,
`src/vm/host.rs`). One commit, `5bb806c`, accidentally swept its `wasm.rs` in
alongside MongoDB work.

**Check `git status` before staging, and stage files by name rather than `-A`.**
Better: give the other session its own git worktree.

The editor-support files (`eTamil_Code/src/generated/`) were deliberately *not*
regenerated in the last commit, because regenerating reads `interpreter.rs` and
would have baked the other session's uncommitted builtins into the data.
**Once the wasm work lands, run `python scripts/generate_editor_support.py` and
commit the result**, or the CI drift check will fail.

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

`nUlakam/kaNakkiyal/vari_viziqam.sql` ships the schema and the 36 states with
their GST codes, and **no rates at all**.
