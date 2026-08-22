# The LLVM backend: what is missing, measured

Everything here is counted from the source rather than estimated, so it can be
re-counted when it changes. Figures are from `src/codegen.rs` at 1,642 lines.

```bash
grep -c LLVMDoubleType etamil_compiler/src/codegen.rs      # 23
grep -c Decimal        etamil_compiler/src/codegen.rs      #  0
grep -c 'unsupported.push' etamil_compiler/src/codegen.rs  #  7
```

## The gaps, worst first

### 1. It computes in `f64` — a wrong answer, not a missing feature

Twenty-three references to `LLVMDoubleType` and none to `Decimal`. Every number
in eTamil is a fixed-point decimal, and the reason is the language's opening
claim: `0.1 + 0.2` is exactly `0.3`, and a ledger balances to the paisa.
Compiled through `--llvm` it is not, and until now nothing said so — the IR was
emitted, it ran, and it answered `0.30000000000000004`.

This is the one failure mode the project refuses everywhere else: a wrong answer
with no warning. `codegen.rs` even carries the principle in a comment on its own
`unsupported` field, and the README states it as "rejected rather than emitted
as incorrect IR". Decimal arithmetic escaped it.

**Mitigated, not fixed.** `src/codegen_limits.rs` now refuses any program whose
arithmetic the backend would get wrong — a fractional literal, a percentage, a
division — before any IR exists. Integer arithmetic is still accepted, because a
double holds every integer below 2^53 exactly, so those programs do agree with
the VM.

That turns silent wrongness into an explicit refusal. It does not make the
backend able to do money.

**To fix properly** the backend needs decimal arithmetic, which means a runtime
library: emit calls to `etamil_add`, `etamil_mul`, `etamil_div` in a small Rust
library compiled alongside, rather than `LLVMBuildFAdd` and friends. That is the
standard approach and it is not exotic — but it touches all twenty-three sites
and it is the prerequisite for everything below.

### 2. No builtin is reachable

`codegen.rs:1426` — a call that does not resolve to a `செயல்` the author wrote
is recorded unsupported and emits a `0.0` placeholder. So `நீளம்` is as
unavailable as `பொதி_மாற்று`. There are 59 builtins.

**To fix:** the same runtime library. Each builtin becomes a C-ABI symbol the
emitted IR calls. The Rust implementations already exist in
`src/vm/interpreter.rs` and would be re-exposed rather than rewritten — but they
take and return `Value`, which needs gap 3 first.

### 3. Values are unboxed doubles, so most of the language has no representation

Strings, arrays, records, results and nil have no form in the emitted IR. This
is why `array index`, `record field` and `ஒவ்வொரு` over a numeric array appear
in the refusal list.

**To fix:** a tagged value at runtime — the same shape `Value` has, as a struct
the IR can pass by pointer. This is the largest single piece of work and gates
gaps 2 and 4.

### 4. `திரும்பு` is refused

`codegen.rs:216`. A function cannot return, which is a surprising thing to find
listed as unsupported when the README claims numeric functions work. Worth
looking at first on Ubuntu: it may be narrower than it reads, and if it is
genuinely broken then "numeric functions" is an overstatement in the README.

### 5. The statements the VM has and this does not

Database, HTTP routing, files, scheduling. Each is refused explicitly, which is
correct behaviour, and none is reachable without gaps 1–3.

## Suggested order

1. **A parity number first.** `scripts/run_parity.sh` runs every example under
   both backends and reports what stopped each one, ranked. Run it on Ubuntu
   before deciding anything — the list below is reasoning, and that is
   measurement.
2. **The runtime library and the boxed value** (gaps 3 then 1). Nothing else
   moves until these do.
3. **Builtins over the C ABI** (gap 2), which is mostly mechanical afterwards.
4. **`திரும்பு`** (gap 4), which may be small enough to do at any point.

## What cannot be checked on Windows

`llvm-sys 180` needs LLVM 18 installed and the feature is Linux/macOS only, so
`codegen.rs` cannot even be type-checked on a Windows machine. That is why the
refusal logic lives in `src/codegen_limits.rs`, which walks the AST and links no
LLVM: it has eleven tests that run anywhere, and `codegen.rs` consults it in one
line.

**That one line is unverified.** It compiles nowhere this was written. Build with
`--features llvm` on Ubuntu first, and expect the possibility of a trivial
error there before anything else is judged.
