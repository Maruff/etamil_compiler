# Architecture and decisions

Why the project is built the way it is. The README says what works; the
roadmap says what does not; this says **why**, so a decision is not
relitigated or accidentally reversed.

---

## The layering rule

| Layer | Written in | Contents |
|---|---|---|
| **0 — Runtime primitives** | Rust | sockets, drivers, file I/O, decimal and text operations, the clock |
| **1 — Language capability** | Rust (compiler) | functions, collections, iteration, results, modules |
| **2 — Frameworks** | **eTamil** | standard library, accounting, and anything built on them |

**Keep Layer 0 as thin as possible.** Every capability implemented in Rust
instead of eTamil is one the users cannot read, extend or fix — and one that
argues the language was not sufficient. Layer 0 should hold only what a
language genuinely cannot express: syscalls, drivers, and operations on
representations the language does not own.

This is also the project's research claim. "The accounting framework is
written in eTamil" is a far stronger statement than "eTamil calls a Rust
accounting library."

**Applied test:** when the standard library needed rounding, `வட்டமிடு` went
into the host, because rounding needs the decimal representation. When it
needed Indian digit grouping, that stayed in eTamil, because string
manipulation is expressible. Same for date arithmetic (host — the calendar
is not derivable from string ops) versus date comparison (eTamil — ISO dates
sort chronologically).

---

## Decisions, and why

### Numbers are fixed-point decimals, never `f64`

This is a language for tax and accounting. `0.1 + 0.2` must be exactly `0.3`,
and a ledger has to balance to the paisa. Every value flows through
`rust_decimal::Decimal` — lexer, AST, VM.

It is not decoration. The double-entry balance check in `pErEtu.qmz` is an
equality between two sums of money; under `f64` a transaction could be a
fraction of a paisa out and either fail spuriously or silently pass.

**Never introduce a float into the value path.** If a host primitive needs
one internally (the LLVM backend does), convert at the boundary.

### Failures are loud

Unsupported statements, undefined variables, unlexable input, division by
zero, out-of-range indices, missing record fields — all raise errors.

The project began by removing the opposite behaviour: `==` compiled to
nothing, file I/O silently did nothing, undefined variables read as `0.0`.
Programs exited 0 with wrong numbers. In a tax calculator that is worse than
a crash, because nobody investigates a success.

The same reasoning drove making the LLVM backend refuse constructs it cannot
compile rather than emitting `0.0` for them.

### Errors are values, following Rust

`சரி(v)` and `தவறு(e)`, with `?` to propagate. Not exceptions.

For a compliance language, "this transaction failed validation" is ordinary
control flow, not an exceptional event. Returned values make the failure path
visible in the signature and impossible to ignore silently.

### Concurrency is a thread per request, not an async VM

The HTTP server runs a fixed worker pool; each request gets its own VM.

Making the VM async would mean yield points at every I/O instruction — a
rewrite. A pool handles the hundreds-of-concurrent-requests range that Indian
SME fintech backends actually need, and critically it allows **blocking**
database drivers (`rusqlite`), which are far simpler to bind into a
synchronous interpreter than async ones.

Revisit only if measured load demands it.

### Database queries are always parameterised

`தளம்_வினா "… WHERE x = ?", [அளவுரு], முடிவு;` — the parameter array is
required even when empty, and there is deliberately no way to splice a value
into SQL text from eTamil.

For a language aimed at financial systems, the first example anyone copies
must not contain an injection. A test drives a `DROP TABLE` payload through a
query and asserts it reaches the driver as an inert bound parameter.

### Decimals cross the SQL boundary as TEXT

SQLite has no exact decimal type. Storing money as `REAL` would undo the
entire reason for decimal arithmetic. Text that parses as a decimal is read
back as a number, so the round trip is lossless.

### Strings are measured in written letters, not code points

`நீளம்("வணக்கம்")` is 5. A Tamil letter is frequently a consonant plus a
vowel sign or pulli, so counting code points gives 7 and every string helper
is wrong on the text the language exists for. Length, indexing and iteration
use grapheme clusters.

### Drivers sit behind traits

`src/db/mod.rs` defines `Database` with no driver dependency; `src/db/sqlite.rs`
implements it behind a feature. This keeps the VM wiring, parameter binding
and row conversion testable without a driver, and means PostgreSQL and MySQL
need only implement the trait.

---

## The pipeline

```
source (.etamil / .qmz)
  ↓  module.rs        இறக்கு resolution, splicing imports ahead of the importer
  ↓  lexer.rs         logos; 201 keywords; errors carry line and column
  ↓  parser.rs        hand-written recursive descent → Expr / Stmt
  ↓  vm/bytecode/compiler.rs
  ↓  vm/interpreter.rs   stack machine, call frames, builtins
```

`codegen.rs` is an optional LLVM backend replacing the last two stages. It
supports **much less** of the language than the VM and refuses what it cannot
compile.

Builtins are dispatched by name in `interpreter.rs::call_builtin`, accepting
Tamil, romanized and `_english` spellings, and user functions shadow them.
That is the extension point for new host primitives.

---

## Names are the author's own — resolved

A name is now stored exactly as written, keyword or not: `வங்கி = 5` creates
`வங்கி`, and `{வரி: 100}` produces the field `வரி`.

Until [roadmap item 2](ROADMAP.md) landed, the lexer discarded the text it
matched and kept only the token, so both spellings of a keyword collapsed onto
its English name — `Bank`, `Tax`. That made Tamil and romanized source
interchangeable, at the price of three things:

1. A Tamil author's chosen name was silently anglicised.
2. Printing such a record emitted English field names into Tamil output.
3. String-keyed lookup needed the token name, not the written one.

It bit **six times** while writing the standard library and the accounting
framework, because the natural Tamil word for a thing is very often already a
keyword: `சொல்`, `வரிசை`, `நிலுவை`, `எண்`, `நடப்பு`, `பற்று`, `வரவு`,
`வங்கி`, `சொத்து`. For a language whose whole purpose is letting people write
in their own language, that was the sharpest remaining contradiction.

**The tradeoff, taken deliberately:** `{வரி: 1}` and `{vari: 1}` are now
*different* fields, and `வருவாய்` and `varuvAy` different variables. A name is
data — what the author typed — not a language construct, so this is the right
way round; but it is a change in meaning, and a program should pick one
spelling and keep to it. Nothing in `nUlakam/` or `examples/` needed touching,
because the workaround at the time was compound names, which were always
stored as written.

Two positions deliberately keep the canonical English name, because what they
name belongs to the host rather than the author: the database type in
`தளம்_இணை`, which `db::open` matches on, and the HTTP method in `வழி`, which
the router matches on.

Separately, **type keywords and SQL clause keywords are hard reserved** and
cannot be names at all: `எண்`, `சொல்`, `அணி`, `வரிசை`, `விதி`, `இடம்`,
`உள்`, `வெளி`, `குழு`, `சேர்`. Financial keywords are *not* reserved. The
Token column in [KEYWORDS.md](reference/KEYWORDS.md) lists every one.

### The same change gave parse errors their positions

`tokenize` returns `Vec<Spanned>` — token, line, column, and the matched text
— so the parser knows where it is. It returns `Result<_, ParseError>` rather
than panicking:

```
வரி 3, நெடுவரிசை 1: ';' எதிர்பார்க்கப்பட்டது, 'அச்சு' கிடைத்தது
(line 3, column 1: expected ';', found 'அச்சு')
```

Columns count written letters rather than bytes, for the same reason string
length does. Positions are handed out by a cursor that walks the source once,
because logos yields tokens in order — rescanning from the start for each
token would make tokenizing quadratic.

Separately, **type keywords and SQL clause keywords are hard reserved** and
cannot be names at all: `எண்`, `சொல்`, `அணி`, `வரிசை`, `விதி`, `இடம்`,
`உள்`, `வெளி`, `குழு`, `சேர்`. Financial keywords are *not* reserved.

---

## Working on this project

**Build and test**

```bash
cd etamil_compiler && cargo test          # 134 tests
cd .. && ./scripts/run_examples.sh        # every example, with expectations
python3 scripts/transliterate.py --check  # romanization audit
```

See [TESTING.md](../TESTING.md) for the full procedure including the server,
database and LLVM paths.

**Adding a keyword**

1. `#[regex("தமிழ்|romanized|_english")] TokenName,` in `lexer.rs`, in the
   right section — get the romanization from `scripts/transliterate.py`,
   never by hand.
2. Nothing else is needed for it to be usable as a name; tokens fall through
   `is_identifier_like`.
3. `python3 scripts/generate_keywords.py` to refresh the reference.

**Adding an instruction**

1. Variant in `vm/bytecode/mod.rs`
2. Emit it in `vm/bytecode/compiler.rs`
3. Execute it in `vm/interpreter.rs` — the match is exhaustive, so a missing
   arm is a compile error rather than a silent no-op
4. Test in `etamil_compiler/tests/language_tests.rs`

**Adding a host builtin**

Add an arm to `call_builtin` with all three spellings. Ask first whether it
belongs in Layer 0 at all.

**Tests assert on results, not exit codes.** The suite this replaced checked
only that examples exited 0, which is why several wrong-answer bugs survived
in it for months.

---

## Terminology

Two unrelated "Phase 1-N" numbering schemes existed and are now kept apart:

- **Paper Phase 1-5** — the research roadmap in *eTamil: An Indian FinTech
  DSL* (Maruff & Valli): compiler core → domain modules → tooling → pilots →
  policy. Against this scheme the project is mid-Phase 1.
- **Backend milestone 1-4** — this repository's HTTP work: sync server →
  async → logging → auth.

The four milestones referred to in recent commits are a third thing again:
language capability → standard library → backend primitives → accounting
framework. All four are complete.
