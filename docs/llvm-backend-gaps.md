# The LLVM backend: what is missing, measured

Everything here is counted from the source rather than estimated, so it can be
re-counted when it changes.

```bash
grep -c LLVMDoubleType etamil_compiler/src/codegen.rs   # 0, was 23
grep -c Decimal        etamil_compiler/src/codegen.rs   # 0
grep -c LLVMBuildSDiv  etamil_compiler/src/codegen.rs   # 1, was 0
wc -l                  etamil_compiler/src/codegen.rs   # 1,565, was 1,642
```

**This has not been built on the machine it was written on.** `llvm-sys 180`
needs LLVM 18 installed and the feature is Linux/macOS only, so a Windows
machine cannot build it. It *has* been type-checked, against a generated
signature-only stand-in for `llvm-sys` — so the Rust is known to compile, and
what is not known is whether the emitted IR verifies and whether it answers
what the VM answers. `scripts/run_parity.sh` is the thing that decides that,
and CI runs it. Run it first.

## What changed: whole numbers as `i64`

The backend used to compute in `f64`. Every number in eTamil is a fixed-point
decimal, and the reason is the language's opening claim: `0.1 + 0.2` is exactly
`0.3`, and a ledger balances to the paisa. Compiled through `--llvm` it was
not — the IR was emitted, it ran, and it answered `0.30000000000000004`.

`src/codegen_limits.rs` already refuses any program with a fractional literal
in it, before any IR exists. So every number that reaches the backend is a
whole number, and an integer register fits it better in every respect:

|  | `f64` | `i64` |
|---|---|---|
| Exact to | 2^53, about ₹90,000 crore in paise | 2^63 |
| `a / b` | out by one in the last place — a paisa | exact, under `தரை` or `மேல்` |
| A whole number printed | `%.0f`, and `6.0` for the VM's `6` | `%lld` |

### The division, which is the whole point

`sdiv` truncates towards zero. That is the floor when the quotient is positive
and the ceiling when it is negative, so exactly one of the two directions needs
a correction of one, and which one depends on the sign of the answer — which is
the sign of the two operands xor'd together:

```
q = sdiv a, b          r = srem a, b
correct when r ≠ 0 and (a ^ b) < 0   → floor   is q - 1
correct when r ≠ 0 and (a ^ b) ≥ 0   → ceiling is q + 1
```

`build_integer_division` in `codegen.rs` is that, as five instructions and a
`select`. Getting the correction wrong is one paisa, in the direction that
makes a split fail to add up to the amount that was split.

**A bare division is still refused.** The true quotient of two whole numbers
usually is not one, and an `i64` has nowhere to keep the rest. It is exact only
under `தரை` or `மேல்`, and only when the division is the call's whole
argument: `தரை(1 + அ / ஆ)` and `தரை(அ / ஆ * இ)` are refused, because rounding a
sum is not rounding the quotient inside it.

**Dividing by zero is checked.** The VM stops with `பூஜ்ஜியத்தால் வகுத்தல்`.
An `sdiv` by zero is undefined behaviour and on x86 it faults, so the emitted
code compares the divisor against zero, writes the same message to file
descriptor 2 and calls `exit(1)`. `write` and `exit` rather than
`fprintf(stderr, …)`, because the `stderr` FILE* is a different symbol on glibc
than on macOS.

**Still not exact:** `i64::MIN / -1` overflows `sdiv`, which is undefined. It
is not guarded. It needs a figure at the far end of the `i64` range, negated,
and a program holding money that large has other problems.

### Three builtins are now reachable

`codegen_limits::whole_number_builtin` is the table, and both files read it, so
the guard and the backend cannot drift into a program that one accepts and the
other refuses:

| | on a whole number | on a division |
|---|---|---|
| `தரை(n)` | the number | exact floor division |
| `மேல்(n)` | the number | exact ceiling division |
| `வட்டமிடு(n, இடங்கள்)` | the number, at any places | refused — it rounds half away from zero, not down |

A `செயல்` the program defines shadows a builtin of the same name, and then the
exception does not apply: a `தரை` the program wrote is whatever the program
wrote.

The other fifty-six builtins take or return a string, an array, a record or a
result, and wait on gap 2 below.

### Globals, so a function can read one

A `செயல்` can read a top-level name — nUlakam is full of functions that do,
starting with `kAcu.qmz` reading `பைசா_ஒரு_ரூபாய்`. A function's stack frame
cannot reach `main`'s, and the backend used to look the name up in an empty map
and compile `0.0` in its place, so `ரூபாயாக(2)` answered 0 rather than 200 and
nothing said so.

Top-level names are module globals now, declared before any function body is
compiled. Assigning to a name *inside* a function still makes a local, which is
what the VM does and what nUlakam is written around.

An unknown name is a refusal rather than a zero. That is the more important
half of the change.

### Two things that were quietly wrong, and are refusals now

**Statements that pretended.** File, database and HTTP statements were
"handled": each printed a log line the VM never prints, or stored a placeholder
zero, and `கோப்பு_படி` read stdin instead of the file it named. None of it
failed — it produced different output from the same source run on the VM, which
is worse. 553 lines of it are gone and `stmt_label` names each one in the
refusal list instead. `src/fileio/` is no longer reached from anywhere as a
result.

**A comparison used as a value.** `அச்சு (1 < 2)` prints `true` on the VM. An
`i64` 1 is not a narrower answer, it is a different one, so a comparison is
compiled only where it is a branch condition. Anything else in a condition is
"not zero", which is what the VM does with a number — checked against it, not
assumed: 5 takes the branch and 0 does not.

### And two that now work

**A guard clause.** `(கீழ் == 0) எனில் { திரும்பு 0; }` used to emit a branch
after the `ret` — invalid IR. Unreachable in practice until a function with a
guard clause could compile at all, which is the first thing a money function
has.

**A text literal.** `அச்சு "வணக்கம்"` needs no value representation, only
bytes. The arm that handled it looked for a shape the parser stopped producing,
so every banner line in every example was refused. It goes through `%s` rather
than being used as the format itself, so a `%` in the text stays a `%`.

## The gaps that remain, worst first

### 1. No decimal arithmetic

Two-place money is covered by holding it in paise — see below — and that is
most of what accounting needs. Three places is not covered, and neither is
anything that wants `0.1` to be `0.1`.

**To fix:** a runtime library. Emit calls to `etamil_add`, `etamil_mul`,
`etamil_div` in a small Rust library compiled alongside, rather than
`LLVMBuildAdd` and friends. Standard, not exotic, and deliberately last: it is
a second artefact to link, and step 2 already covers accounting.

### 2. Fifty-six of the fifty-nine builtins are unreachable

A call that does not resolve to a `செயல்` the author wrote, or to one of the
three above, is recorded unsupported. So `நீளம்` is as unavailable as
`பொதி_மாற்று`.

**To fix:** each becomes a C-ABI symbol the emitted IR calls. The Rust
implementations already exist in `src/vm/interpreter.rs` and would be
re-exposed rather than rewritten — but they take and return `Value`, which
needs gap 3 first.

### 3. Values are unboxed integers, so most of the language has no form

Strings, arrays, records, results, booleans and nil have no representation in
the emitted IR. This is why `array index`, `record field` and `ஒவ்வொரு` over
anything the backend has not tracked as an array appear in the refusal list,
and why a text value is refused everywhere except as a literal being printed.

**To fix:** a tagged value at runtime — the same shape `Value` has, as a struct
the IR can pass by pointer. This is the largest single piece of work and it
gates gaps 1 and 2.

### 4. The statements the VM has and this does not

Database, HTTP routing, files, scheduling. Each is refused explicitly and named
— which it now actually is, see above — and none is reachable without gap 3.

## The workaround for money, and how far it reaches

**Keep money in paise, as whole numbers.** ₹2.05 is `205`. Addition,
subtraction and multiplication by a quantity are exact; a percentage is exact
if it is computed as integer arithmetic and rounded once:

    தரை((பைசா × மேல் + தரை(கீழ் / 2)) / கீழ்)

`nUlakam/kAcu.qmz` is that, written in eTamil and tested on the VM.
`examples/finance/paicA_kaNakku.qmz` is the same idea written inside the
backend's limits — no `இறக்கு`, no arrays, no text beyond the labels — so that
both backends run it and `run_parity.sh` can compare the answers. That example
is the proof this change works; everything above it is the reasoning.

`kAcu.qmz` itself is still refused, and it is worth being exact about why. Its
arithmetic compiles now — `ரூபாயும்_பைசாவும்`, `காசு_மடங்கு`,
`விழுக்காடு_காசு`. What does not is `காசு_உரை`, which builds a string, and
`சமமாகப்_பிரி` and `விகிதத்தில்_பிரி`, which return arrays. Those wait on gap
3, not on arithmetic.

## Measured, on Ubuntu, against 68 programs

`scripts/run_parity.sh`, LLVM 18, clang present so every accepted program was
compiled *and run*:

```
7 match, 0 mismatch, 57 refused, 0 compiled-only, 4 skipped
all 68 accounted for
```

**Nothing disagreed.** Where the backend accepted a program, the compiled binary
answered what the VM answered. Zero `IR REJECTED` also means clang's verifier
accepted all seven modules, which is the part `check_llvm_backend.py` cannot
tell you: the blocks-and-terminators work — `எனில்` containing `திரும்பு`,
entry-block allocas, the division guard's own basic blocks — produced valid IR.

The four skipped are the three route examples the VM cannot run either, plus
`mYcIkul_qaLam.qmz` behind `ETAMIL_TEST_MYSQL`.

Six of the seven matches are `examples/backend/*.qmz`, and every one of them was
*refused* before this round — the print path was matching a string-literal shape
the parser had stopped producing, so every banner line in the corpus was a
refusal. The seventh is `examples/finance/paicA_kaNakku.qmz`.

### What stopped the other 57, ranked

Counted by `run_parity.sh` from the refusal lines themselves:

```
48  array index
48  a comparison used as a value
47  ஒவ்வொரு (for-each) over a numeric array
45  function call நீளம்
45  function call சரி
43  function call தவறு
41  expression உரை (a text value)
41  expression a boolean literal
38  expression a logical operator
36  வகுத்தல் without தரை() or மேல்
33  the name விடை
32  expression a record literal
27  function call இணை
25  function call மதிப்பு
19  function call சொல்லாக்கு
15  statement an expression statement
15  function call தவறா
13  record field தொடக்கம்
13  expression இல்லை (not)
```

**Read that as reasons, not as programs.** A program with eight reasons needs all
eight before it compiles, so the ranking says what is *common*, not what is on
the critical path. Clearing the top line alone would move the match count by
approximately nothing.

Two entries in the first run of this list were the backend misreporting itself,
and both are fixed rather than left in the table above:

- A bare division appeared **twice**, at 36 and 35, because `codegen_limits` and
  `codegen.rs` refused it in two different words. It is one cause, and it is the
  commonest arithmetic gap in the corpus — the split reading hid that. Both now
  push the same constant.
- `the name விடை (nothing here defines it)`, 33 of them, named a cause that does
  not exist. `nUlakam/aNi.qmz` builds up `விடை = []` and returns it; the name is
  perfectly well defined, it just holds an array, and an array is not a value
  here. Those 33 belong to gap 3 and now say so.

A refusal list is a roadmap, so a wrong reason in it is worse than a missing
one. Both were mine.

## Suggested order from here

The measurement agrees with the reasoning above, which was not guaranteed:

1. **The boxed value** (gap 3): a tagged struct the IR passes by pointer, the
   same shape `Value` has. It gates the top of the list outright — array index,
   for-each, text, record literal, the 33 `விடை`-shaped returns — and it gates
   the builtins below it, because `நீளம்` and `இணை` take and return exactly
   those. Include `சரி`/`தவறு` in it: the result type is 128 occurrences across
   `நீளம்`, `சரி`, `தவறு`, `மதிப்பு` and `தவறா`, because that is how nUlakam
   reports failure.
2. **Booleans, which are cheaper than they look and were not in the old plan.**
   Comparisons-as-values, boolean literals, logical operators and `இல்லை` come
   to about 140 occurrences, and none of them needs a heap — an i64 0 or 1 does
   it. What they need is knowing *at the print site* that a value is a boolean,
   because `அச்சு` renders one as `true`, not `1`. A two-state static type
   (number or boolean) threaded through `compile_expr` buys all four, and
   booleans only ever arise from comparisons, logical operators, `இல்லை`,
   literals, and calls returning them. Worth doing before 1 if the aim is to
   raise the match count for the least work; worth doing after 1 if the aim is
   nUlakam, which needs the boxed value regardless.
3. **Builtins over the C ABI** (gap 2), mostly mechanical once 1 exists.
4. **Full decimal arithmetic** (gap 1), for whoever needs more than two places.

Before starting any of them, re-run the parity script: these numbers are from
one commit, and the two corrected reasons above will redraw the ranking.

## Type-checking this file on a machine that cannot build it

`src/codegen_limits.rs` walks the AST and links no LLVM, so its seventeen tests
run anywhere — that is why the refusal rules live there rather than here.

`codegen.rs` cannot be built without LLVM, but it can be *type-checked*
anywhere, because `llvm-sys`'s own source sits in the cargo registry cache even
where it cannot be compiled:

```bash
python scripts/check_llvm_backend.py
```

It generates a signature-only crate from llvm-sys's source — the same
declarations, nothing behind them — points cargo at it through a path override,
runs `cargo check --features llvm --all-targets`, and puts the checkout back
afterwards including `Cargo.lock`. `--clean` undoes an interrupted run.

**Check that it is really looking at `codegen.rs` before trusting a clean run.**
Put a deliberately undefined LLVM symbol in the file and confirm the error: a
`cfg`'d out file passes everything, and this whole file is `cfg`'d out.

This catches every type error and every borrow error. It cannot catch invalid
IR, and it cannot catch a wrong answer. Those are what Ubuntu and
`run_parity.sh` are for, and a clean run here is not evidence the backend
works.
