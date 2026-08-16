# eTamil Compiler — AI Coding Agent Instructions

**eTamil** is a programming language whose vocabulary is Tamil, aimed at Indian
FinTech. Keywords accept three interchangeable spellings: Tamil script, ezuqqu
romanization, and (for many) an `_english` alias.

Be accurate about status. This project's documentation was previously full of
features described as complete that were never wired up; do not reintroduce
that. If something is not implemented, say so.

---

## Pipeline

```
Source (.etamil / .qmz)
  ↓  src/lexer.rs        logos, 196 keyword tokens, reports line/column errors
Tokens
  ↓  src/parser.rs       hand-written recursive descent → Expr / Stmt
AST
  ↓  src/vm/bytecode/compiler.rs
Bytecode (39 instructions)
  ↓  src/vm/interpreter.rs   stack machine
```

`src/codegen.rs` is an optional LLVM backend replacing the last two stages,
behind `--features llvm` (Linux/macOS only, needs LLVM 18).

## Modules

| Path | Purpose |
|---|---|
| `src/lexer.rs` | Bilingual tokenizer |
| `src/parser.rs` | AST construction |
| `src/vm/` | Bytecode compiler, stack interpreter, `Value` |
| `src/http/` | Synchronous HTTP server, plus auth/cache/logging/resilience modules |
| `src/fileio/` | CSV handling and a (weak) XOR file cipher |
| `src/codegen.rs` | LLVM IR backend, optional |
| `scripts/` | `generate_keywords.py`, `transliterate.py` |

## Execution modes

- `--vm` (default) — bytecode VM. This is the only path with test coverage.
- `--server` — synchronous, single-threaded `std::net::TcpListener`. Registers
  the **whole program** as the handler for every route plus `/health`.
- `--async` — **not implemented**; prints a warning and runs the sync server.
- `--llvm` — emits `output.ll`. No native binary or WebAssembly output.

## What is NOT implemented

Do not write code or docs that assume these exist:

- **Functions.** The language has no call syntax. `Instruction::Call`/`Return`
  are defined but nothing emits them.
- **Type checking.** Type keywords are parsed and discarded; `col x = 5;` is
  accepted.
- **Semantic / regulatory validation.** There is no semantic pass.
- **Databases.** `தளம்_இணை` and friends parse, then compile to
  `Instruction::Unsupported` and fail at runtime. There is no driver and no
  `src/db/` module.
- **Routes as statements.** `வழி`, `பதில்` etc. parse, then fail the same way.
- **Blockchain, UPI, KYC, GSTN/NPCI integration.** Nothing exists.

See `docs/ROADMAP.md` for what finishing each of these involves.

## Conventions that matter

**Numbers are `rust_decimal::Decimal`, never `f64`.** This is a tax and
accounting language; `0.1 + 0.2` must be exactly `0.3`. Never introduce a
float into the value path.

**Failures must be loud.** Unsupported statements, undefined variables,
unlexable input and division by zero all raise errors. Historically these
were silent no-ops that produced wrong numbers while exiting 0 — never
restore that behaviour.

**Error messages are bilingual**, Tamil first then English in parentheses.

**The romanization scheme distinguishes all three Tamil nasals:**
ண = `N`, ந = `n`, ன = `Z`. Also ழ = `z`, ள = `L`, ற = `R`, த = `q`, ட = `t`,
ச = `c`, ங = `w`, ஞ = `W`. Never hand-write a romanization — run
`python scripts/transliterate.py <தமிழ்>`, and audit with `--check`.

**Keywords used as variables are stored under their token name.** `varuvAy`
is the `Revenue` keyword, so `eN varuvAy = 5;` creates a variable named
`Revenue`. That is what makes the Tamil and romanized spellings refer to the
same variable.

**"Backend milestone N"** refers to this repo's HTTP work. It is unrelated to
the Phase 1-5 in the eTamil paper (compiler core → domain modules → tooling →
pilots → policy). Keep them distinct.

## Adding a keyword

1. Add `#[regex("தமிழ்|romanized|_english")] TokenName,` to the right section
   of `src/lexer.rs` — get the romanization from `scripts/transliterate.py`.
2. If it should be usable as a variable name, nothing else is needed; tokens
   fall through `is_identifier_like`.
3. Run `python scripts/generate_keywords.py` to refresh
   `docs/reference/KEYWORDS.md`.

## Adding an instruction

1. Variant in `src/vm/bytecode/mod.rs`
2. Emit it in `src/vm/bytecode/compiler.rs`
3. Execute it in `src/vm/interpreter.rs` — the match is exhaustive, so a
   missing arm is a compile error rather than a silent no-op
4. Add a test to `etamil_compiler/tests/language_tests.rs`

## Build and test

```bash
cd etamil_compiler
cargo build --release      # produces target/release/etamil
cargo test                 # 28 end-to-end language tests + unit tests
cargo build --release --features llvm   # Linux/macOS only
python ../scripts/transliterate.py --check   # romanization audit
```

Tests assert on **program results**, not exit codes. The previous suite
checked only that examples exited 0, which is why several wrong-answer bugs
survived in it for months.

## Documentation

1. `README.md` — status table of every subsystem
2. `docs/ROADMAP.md` — what is unfinished and why
3. `docs/reference/KEYWORDS.md` — generated; do not edit by hand
4. `docs/getting-started/` — installation and quick start
