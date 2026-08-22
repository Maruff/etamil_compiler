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

### 4. `திரும்பு` — not a gap, and this document was wrong about it

An earlier version of this file listed `திரும்பு` as unsupported. It is not.
Reading `codegen.rs:216` properly: the refusal fires only when
`self.in_function` is false — a `திரும்பு` at top level, where there is nothing
to return from. Inside a function it compiles to `LLVMBuildRet` like any other
return.

So the README's "numeric functions" claim stands, and this entry is kept only
because a corrected list is more useful than a quietly shortened one.

### 5. Only float arithmetic exists, which is why the workaround below works

Counted from source:

```
LLVMBuildFAdd  1     LLVMBuildAdd   1   (pointer arithmetic, not the language's)
LLVMBuildFSub  1     LLVMBuildSDiv  0
LLVMBuildFMul  1     LLVMBuildSRem  0
LLVMBuildFDiv  1
```

Every eTamil number goes through a double. LLVM has exact integer arithmetic
and this backend does not reach for it — which matters, because it makes the
cheap fix cheap. See "A workaround for money" below.

### 6. Strings, booleans and nil have no representation

`Expr::String` appears **zero** times in `codegen.rs`. Booleans, `இன்மை` and the
`?` operator appear only in the *labelling* function that writes refusal
messages. Arrays and records are narrower than absent: a local variable holding
one compiles, and indexing something the backend has not tracked as an array
does not.

### 7. The statements the VM has and this does not

Database, HTTP routing, files, scheduling. Each is refused explicitly, which is
correct behaviour, and none is reachable without gaps 1–3.

## A workaround for money, without a decimal runtime

Two-place decimals are what accounting needs, and there is a way to have them
exactly on this backend without building decimal arithmetic first.

**Keep money in paise, as whole numbers.** ₹2.05 is `205`. A double represents
every integer below 2^53 exactly — about ₹90,000 crore expressed in paise — so
addition, subtraction and multiplication by a quantity are exact today, with no
change to the backend at all. A percentage is exact too if it is computed as
integer arithmetic and rounded once:

    paise × rate_numerator + denominator/2, then divided and floored

`nUlakam/kAcu.qmz` is that, written in eTamil and tested on the VM. It never
writes a fractional literal, so it is not caught by the refusal, and it does the
one thing a naive implementation gets wrong: splitting an amount that does not
divide evenly distributes the odd paise instead of losing them.

**What still needs a backend change, and it is small.** Division of the paise
figure is the one step where a double can be off by one in the last place. The
fix is not a decimal runtime — it is to compile whole numbers as `i64` and use
`LLVMBuildSDiv`, which is exact. That is four or five instruction sites, against
the twenty-three a decimal representation would touch.

So the order below can be re-read: **integers as `i64` first** buys correct
accounting for a fraction of the work, and full decimal arithmetic can wait for
whoever needs three decimal places.

## Suggested order

1. **A parity number first.** `scripts/run_parity.sh` runs every example under
   both backends and reports what stopped each one, ranked. Run it on Ubuntu
   before deciding anything — the list below is reasoning, and that is
   measurement.
2. **Whole numbers as `i64`, with `LLVMBuildSDiv`.** Four or five sites, and it
   buys exact two-place money through `nUlakam/kAcu.qmz` — see the workaround
   above. This is the cheapest correctness win available and does not depend on
   anything else.
3. **The boxed value** (gap 3): a tagged struct the IR passes by pointer, the
   same shape `Value` has. Strings, arrays, records and results all wait on it.
4. **Builtins over the C ABI** (gap 2), mostly mechanical once 3 exists.
5. **Full decimal arithmetic** (gap 1), for whoever needs more than two places.
   Deliberately last: step 2 already covers accounting.

## What cannot be checked on Windows

`llvm-sys 180` needs LLVM 18 installed and the feature is Linux/macOS only, so
`codegen.rs` cannot even be type-checked on a Windows machine. That is why the
refusal logic lives in `src/codegen_limits.rs`, which walks the AST and links no
LLVM: it has eleven tests that run anywhere, and `codegen.rs` consults it in one
line.

**That one line is unverified.** It compiles nowhere this was written. Build with
`--features llvm` on Ubuntu first, and expect the possibility of a trivial
error there before anything else is judged.
