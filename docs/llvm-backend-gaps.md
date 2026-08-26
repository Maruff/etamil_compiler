# The LLVM backend: what it does, and what it still refuses

Last rewritten when the backend stopped holding values in registers. The
history matters here, because the shape of the thing is a reaction to two
earlier attempts that were wrong in the same way.

## Two versions that traded the language for a register

**It computed in `f64`.** Every number in eTamil is a fixed-point decimal, and
the reason is the language's opening claim: `0.1 + 0.2` is exactly `0.3` and a
ledger balances to the paisa. Compiled through `--llvm` it was not, and nothing
said so — the IR was emitted, it ran, and it answered `0.30000000000000004`.

**Then it computed in `i64`.** Better: exact to 2^63 instead of 2^53, and
`sdiv` under `தரை` was exactly a floor where `fdiv` could be out by a paisa.
That version is what `examples/finance/pYcA_kaNakku.qmz` was written for, and
it worked — 7 of 68 examples matched the VM, none disagreed.

But both were the same bargain in different clothes. The IR held the value
directly, so the language shrank to whatever fits in a register. Numbers fit.
Strings, arrays, records, results and `இன்மை` did not. Neither did a decimal:

```
1 / 3   →   0.3333333333333333333333333333    on the VM
```

There is no register that holds that, so `i64` had to refuse any fractional
literal and any bare division. Which is most of what a tax calculation is.

### The obvious workaround, and why it was not taken

Fixed two-place decimals — every number an `i64` scaled by 100, which is what
the paise convention in `nUlakam/kAcu.qmz` already does by hand. Measured
against the VM before committing to it:

| | fixed 2-place | VM |
|---|---|---|
| `0.1 + 0.2` | `0.3` | `0.3` ✓ |
| `1 / 4` | `0.25` | `0.25` ✓ |
| `1000 * 18%` | `180` | `180` ✓ |
| `1 / 3` | **`0.33`** | `0.3333333333333333333333333333` ✗ |

Three out of four is not the standard here. The fourth is a silent wrong
answer, so inexact division would have to stay refused — and then the workaround
buys fractional literals and nothing else, while strings, arrays, records and
results stay out of reach because the IR has no heap to allocate them from.

## What it does now: the IR stops holding values

Every eTamil value in the emitted IR is an `i64` **handle** into an arena that
lives in `src/runtime.rs`, and every operation on one is a C-ABI call into that
module. The IR carries the control flow — branches, loops, calls, returns — and
knows nothing about what a value is.

That is the ordinary way to compile a dynamically typed language. It costs a
call per operation and buys three things, none of them a coincidence:

1. **Decimals are exact, because the runtime is the same `rust_decimal` the VM
   uses.** Not an approximation of the type — the type. `1 / 3` prints all
   twenty-eight digits in both backends for the same reason rather than by
   luck.
2. **Formatting cannot drift**, because printing calls `Value::to_string`, the
   function the VM prints through. Trailing zeros are trimmed in one place, not
   two: `1000 * 18%` is `180.00` by scale and prints as `180` on both sides.
3. **All fifty-nine builtins work at once**, because `etamil_call` dispatches
   through `VM::invoke_builtin` — the interpreter's own table. `நீளம்` is not
   reimplemented, so it cannot disagree.

`index_of` and `invoke_builtin` are public on the VM for exactly this reason:
the two backends reach the same code, rather than two implementations that
agree until someone edits one of them.

### What that removed

`src/codegen_limits.rs` is gone. It existed to refuse, before any IR existed,
the arithmetic the backend would get wrong — fractional literals, bare
division, whole numbers past `i64`. None of those is wrong any more, so a module
whose entire content was that policy had nothing left to say. Its `தரை`/`மேல்`/
`வட்டமிடு` table went with it: all three are ordinary builtins now, along with
the other fifty-six.

`codegen.rs` also lost `ArrayInfo`, the arrays and records maps, the
double-versus-integer duality, the hand-rolled floor division with its sign
correction, the division-by-zero guard blocks, and the whole
print-the-pieces-of-a-concatenation machinery. `&` is a real operation now, so
`அச்சு "விலை: " & 2.05` is one call. The file is shorter than the version it
replaced.

## The cost, stated plainly

**`output.ll` is no longer self-contained.**

```bash
etamil --llvm myprogram.qmz
clang output.ll -o myprogram \
      -L etamil_compiler/target/release -letamil_compiler \
      -Wl,-rpath,etamil_compiler/target/release -lm
```

`Cargo.toml` already built the `cdylib`, so there is no new artefact — only new
exported symbols in it. But a compiled program now ships with
`libetamil_compiler.so` beside it, and `-rpath` is what lets it find the library
again at run time. `scripts/run_parity.sh` does this, and prints clang's own
error if the link fails, because "clang rejected the IR" on its own is not
something anyone can act on.

**Nothing in the arena is ever freed.** A compiled program runs and exits, and
an arena that only grows cannot dangle, cannot double-free, and needs no
reference counting in the emitted IR. A long-running program compiled this way
would grow without bound. That is the honest limit of the approach and the
reason the VM remains the way to run a server.

## What is still refused

Statements, not expressions. Files, databases, HTTP, routes, scheduling, and
`இறக்கு` at the point codegen sees one. Each is named individually by
`stmt_label` so that `run_parity.sh` can rank them, and each needs the VM's own
machinery rather than a value representation — a route is not a value problem.

Refusing remains the discipline. IR that dropped a statement, or evaluated an
expression as a placeholder, would make a compiled program quietly disagree
with the same source run on the VM, and that is the one failure this project
does not accept. `main.rs` refuses to emit when the list is non-empty.

## Measured, on Ubuntu

`scripts/run_parity.sh`, LLVM 18, clang present so every accepted program was
compiled *and run*:

```
51 match, 2 mismatch, 11 refused, 0 compiled-only, 4 skipped
all 68 accounted for
```

The previous design — values in `i64` registers — scored `7 match, 0 mismatch,
57 refused`. So the runtime bought 44 programs, and no module was rejected by
clang's verifier in either run.

What matches now is most of the library: `nUlakam/kAcu.qmz` and its suite,
accounting, tax, insurance, customs, banking, UPI addresses, Redis, JSON, the
document builder, and the test framework they are all written against. Money to
the paisa, compiled, agreeing with the interpreter line for line.

### The two that disagreed

A refusal is expected. A disagreement is the one failure this project does not
accept, so both are worth stating precisely.

**`examples/basic_samples/example.qmz` — mine, and fixed.** `உள்ளிடு வருவாய்1;`
reads a line on the VM and stores it. This backend printed the variable's
current value as a "prompt" and never read at all, so the compiled program
emitted `nil` and left its input unconsumed. The prompt-printing was invented
here rather than read out of the VM's bytecode, which prints no prompt and
stores only when the operand is a bare name. `codegen.rs` now does exactly
that, and `etamil_prompt` is gone from the runtime because nothing calls it.

Worth keeping in view: `உள்ளிடு "prompt" & பெயர்` reads a line and **throws it
away** on both backends, because the VM stores only for a bare name. That is a
language wart, not a backend one, and it is now faithfully reproduced rather
than papered over on one side.

**`nUlakam/upi/upi_cOqaZY.qmz` — open.** The suite passes 45 of 45 on the VM, so
exactly one assertion or one printed line differs. Reading the code did not find
it: the constructs it uses that the eight *matching* suites do not are a
`திரும்பு` inside a `ஒவ்வொரு`, a record indexed by a string key, and
`அல்லது` chains returned directly — and each of those looks right on inspection.
Inspection is how the `உள்ளிடு` bug got written in the first place, so the next
step is to look rather than reason:

```bash
./scripts/run_parity.sh --diff nUlakam/upi/upi_cOqaZY.qmz
```

That runs one program under both backends and prints where their output parts
company. The summary form says only *that* two backends disagree, which made
every mismatch cost a round trip.

## Re-measuring

```bash
cd etamil_compiler && cargo build --release --features llvm && cd ..
./scripts/run_parity.sh
```

The last line must read `all 68 accounted for`. If it does not, the run stopped
early and nothing above it is a measurement.

## Type-checking on a machine that cannot build it

`codegen.rs` cannot be built without LLVM 18, but it can be *type-checked*
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
IR, and it cannot catch a wrong answer.

**The runtime, though, is testable here** — and that is the main reason the value
semantics live in `src/runtime.rs` rather than in the emitted IR. The part most
likely to be wrong is now the part that runs under `cargo test` on any machine:
`0.1 + 0.2` is `"0.3"` and `1 / 3` is all twenty-eight digits, asserted rather
than argued.
