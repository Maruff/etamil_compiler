# eTamil Roadmap

What is not built yet, why it matters, and what finishing it involves. Items are ordered by how much they block real use.

## A note on "phases"

Two unrelated numbering schemes existed in this project, both called Phase 1-N. They are now kept apart:

| Term | Meaning | Where it appears |
|---|---|---|
| **Paper Phase 1-5** | The research roadmap: compiler core → domain modules → tooling/REPL → pilot projects → policy engagement | *eTamil: An Indian FinTech DSL* (Maruff & Valli), p.13 |
| **Backend milestone 1-4** | This repository's HTTP work: sync server → async → logging → auth | source comments in `src/http/` |

They mean entirely different things. Backend milestones 1-4 being "complete" says nothing about paper Phase 2, which has not started. Against the paper's scheme, the project is mid-**Phase 1**: the compiler core exists, the domain modules do not.

---

## 1. ~~Decimal arithmetic for currency~~ — RESOLVED

Every number is now a fixed-point `Decimal`, from the lexer through the AST to `Value::Number`. `0.1 + 0.2` is exactly `0.3`; `99.99 * 3` is exactly `299.97`; `18%` is exactly `0.18`.

Equality is exact. The previous `f64` value type compared numbers equal within `1e-10`, so two amounts a fraction of a paisa apart were indistinguishable — a real hazard in reconciliation.

**Division policy:** division keeps the decimal type's full precision and does *not* round at each step. Indian tax computation rounds once at the end, and rounding intermediates compounds error through a chained calculation. An explicit rounding builtin is still needed — see item 3.

**Still open:** `எண்` is the only numeric type. A separate money type carrying a currency, and integer/decimal distinction, would let the type checker (item 7) reject nonsense like adding rupees to a count.

---

## 2. ~~Keep the source text of tokens~~ — RESOLVED

`tokenize()` returns `Vec<Spanned>` — the token, its line and column, and the text it matched. That one capability fixed both problems it was blocking.

**Names are the author's own.** `வங்கி = 5` creates `வங்கி`, not `Bank`; `{வரி: 100}` produces the field `வரி`. Two positions deliberately keep the canonical English name, because what they name belongs to the host rather than the author: the database type in `தளம்_இணை`, which `db::open` matches on, and the HTTP method in `வழி`, which the router matches on.

**Parse errors carry a position**, and the parser returns `Result<_, ParseError>` rather than panicking:

```
வரி 3, நெடுவரிசை 1: ';' எதிர்பார்க்கப்பட்டது, 'அச்சு' கிடைத்தது
(line 3, column 1: expected ';', found 'அச்சு')
```

Columns count written letters rather than bytes, for the same reason string length does.

**The tradeoff, taken deliberately:** `{வரி: 1}` and `{vari: 1}` are now *different* fields, and `வருவாய்` and `varuvAy` different variables. A name is data — what the author typed — not a language construct. Nothing in `nUlakam/` or `examples/` needed changing, because the workaround at the time was compound names, which were always stored as written; a program mixing the two spellings of one keyword-backed name is the case that breaks.

**Still open:** the declared type on an assignment is now kept rather than discarded, but nothing checks it yet — see item 7.

---

## 3. Database execution

**Done for SQLite.** `src/db/` holds a `Database` trait with no driver dependency, and `src/db/sqlite.rs` implements it with the blocking `rusqlite` driver behind the `sqlite` feature (on by default). The VM keeps a connection per database type.

Queries are **always parameterised** — there is deliberately no way to splice a value into SQL text from eTamil:

```etamil
தளம்_வினா "SELECT peyar, qokY FROM pativukaL WHERE vakY = ?", ["வரவு"], வரவுகள்;
```

Rows return as an array of records, so a result set iterates like any other table. Decimals cross the boundary as text rather than `REAL`, so no precision is lost — using an inexact SQL type would defeat the point of decimal arithmetic.

**Still to do:** PostgreSQL and MySQL (the trait is the only thing they need to implement); transactions; more than one connection open at a time, which the VM currently refuses rather than guessing.

---

## 4. The async HTTP server

**Today:** `--async` prints a warning and runs the synchronous server. `src/http/async_mod.rs` and `src/http/async_handler.rs` are **not declared in `src/http/mod.rs`**, so they are never compiled — and they would not compile as written:

- `use futures::stream::StreamExt;` — `futures` is not a dependency
- `use signal_hook::consts::signal::*;` — `signal-hook` is unix-only but imported unconditionally
- `use crate::vm::{VM, bytecode::compiler::Compiler};` — the type is `BytecodeCompiler`

**What it involves:** fix those three, declare the modules, add `axum` and `futures` back to `Cargo.toml`, gate the signal handling behind `#[cfg(unix)]`, and give the VM a per-request execution model. Also needs item 5, since a server without route statements can only serve one handler.

---

## 5. ~~Routes and responses as language statements~~ — RESOLVED

`வழி` statements are lifted out of the program at startup and registered with the router; everything else becomes a prelude compiled into every handler, so a route can call the file's imports and functions. `பதில்` records status and body for the server to send. Handlers compile once at registration rather than on every request.

Request data is injected as variables — `request_method`, `request_path`, `query_params`, `headers`, `request_body` — and map indexing makes `query_params["id"]` readable from eTamil.

**Still to do:** `Stmt::SendResponse` discards its headers; `ஜேசான்_உரை` (JSON responses) needs a serializer; path parameters (`/kaNakku/:id`) are not matched; and each request builds a fresh VM, so a database connection opened in the prelude is reopened per request rather than pooled.

---

## 6. ~~Resolving ந vs ன in the romanization~~ — RESOLVED

The three nasals now have distinct letters: **ண = `N`, ந = `n`, ன = `Z`.**

`Z` was unassigned (ழ is lowercase `z`), so it takes ன and leaves `N` unambiguously ண. Thirty keyword spellings changed, including four that spelled ந as `N` (`nikara`, `nirY`, `kOppu_nirY`, `vAwkunar`), one that used `N` for ங் (`pawku`, formerly `paNgu`), and one that spelled நிலை two different ways (`nilY_ceyqi`, formerly `nilai_ceyqi`).

This is a **breaking change to romanized source**; Tamil-script source is unaffected.

### Follow-up: 19 keywords are still off-scheme for other letters

`scripts/transliterate.py` implements the scheme and `--check` audits the lexer against it. It reproduces 177 of 196 keywords exactly; the other 19 use letters the scheme does not assign, or assigns elsewhere:

| Problem | Examples |
|---|---|
| `t`/`q` swapped (ட is `t`, த is `q`) | `soqqu`→`coqqu`, `toqai`→`qokY`, `uqal`→`utal`, `talY`→`qalY` |
| Letters not in the scheme at all | `matippIDu` (`D`), `toguippu` (`g`), `vazhi` (`zh`), `paDil` (`D`) |
| ச written `s` instead of `c` | `soqqu`→`coqqu` |
| Doubled consonants dropped | `iraqi_pulli`→`iRuqi_puLLi` |
| Compound convention | `varumAZ_aRikkY` vs `varumAZa_aRikkY` — the lexer drops the inherent vowel before `_`; decide and document this |

Sweeping these is one more breaking change to romanized source, so it should land in a single commit with lexer tests asserting every letter round-trips. CI runs `--check` today as a non-gating step; make it gating once the sweep is done.

---

## 7. Type checking

**Today:** type keywords (`எண்`, `சொல்`, `ஈர்ம`…) are parsed and then discarded. `சொல் x = 5;` is accepted.

**What it involves:** keep the declared type in `Stmt::Assign`, check assignments against it, and report mismatches with the diagnostics from item 2.

---

## Smaller known issues

- **No short-circuiting.** `மற்றும்` and `அல்லது` evaluate both sides. Harmless today because expressions have no side effects, but it must change when function calls arrive.
- **Chained comparisons parse oddly.** `a > b > c` becomes `(a > b) > c`.
- **Functions are not implemented.** `Call` and `Return` instructions exist but nothing emits them.
- **`src/finance/` is empty.** `calculator.rs` and `mod.rs` are zero-byte files.
- **`src/api/` duplicates parser and codegen logic** and is not reachable from `main.rs`.
- **`CommandExecutor` in `src/commands.rs` is never called** from either binary.
- **Encryption is XOR, not AES.** `src/fileio/crypto.rs` uses a repeating-key XOR cipher with a default key. It should not be described as encryption in user-facing docs until it uses a real AEAD.
- **`rustfmt` and `clippy` are not clean.** CI runs both with `continue-on-error: true`; remove that once the backlog is cleared.
