# eTamil Roadmap

What is not built yet, why it matters, and what finishing it involves. Items are ordered by how much they block real use.

## A note on "phases"

Two unrelated numbering schemes existed in this project, both called Phase 1-N. They are now kept apart:

| Term | Meaning | Where it appears |
|---|---|---|
| **Paper Phase 1-5** | The research roadmap: compiler core → domain modules → tooling/REPL → pilot projects → policy engagement | *eTamil: An Indian FinTech DSL* (Maruff & Valli), p.13 |
| **Backend milestone 1-4** | This repository's HTTP work: sync server → async → logging → auth | source comments, `docs/archive/phases/` |

They mean entirely different things. Backend milestones 1-4 being "complete" says nothing about paper Phase 2, which has not started. Against the paper's scheme, the project is mid-**Phase 1**: the compiler core exists, the domain modules do not.

---

## 1. Decimal arithmetic for currency

**Today:** every number is `f64`, from the lexer (`Token::Number(f64)`) through the AST to `Value::Number(f64)`. `0.1 + 0.2` prints `0.30000000000000004`.

**Why it matters:** eTamil is aimed at tax and accounting work, where totals must balance exactly. Floating point cannot promise that.

**What it involves:** introduce a decimal value type (`rust_decimal` was chosen originally and is a good fit), thread it through `Value`, the arithmetic instructions, and the lexer's number and percentage literals. Decide the rounding rule for division and document it. Add `rust_decimal` back to `Cargo.toml` in that commit.

---

## 2. Source positions in parser errors

**Today:** the lexer reports line and column, but `tokenize()` returns a bare `Vec<Token>`, so the parser has no positions. Parse failures are `panic!` with a message like `Expected Semicolon` and no location.

**What it involves:** return `Vec<(Token, Span)>` from the lexer, carry the span through `Parser`, and convert the parser from `panic!` to a `Result` with a diagnostic type shared with the lexer. Messages should be bilingual, like the lexer's already are.

This is the single biggest usability gap for learners.

---

## 3. Database execution

**Today:** `தளம்_இணை`, `தளம்_வினா` and friends parse into AST nodes, then compile to an `Unsupported` instruction that fails at runtime with a clear message. `src/db/` contains an in-memory `HashMap` simulation that prints `[DB] ...` lines and is not connected to the VM.

**What it involves:** decide whether `src/db/` becomes a real backend or is deleted. For real database work, add `sqlx` back and give the VM an async execution path — the VM is currently synchronous, so this depends on item 4.

---

## 4. The async HTTP server

**Today:** `--async` prints a warning and runs the synchronous server. `src/http/async_mod.rs` and `src/http/async_handler.rs` are **not declared in `src/http/mod.rs`**, so they are never compiled — and they would not compile as written:

- `use futures::stream::StreamExt;` — `futures` is not a dependency
- `use signal_hook::consts::signal::*;` — `signal-hook` is unix-only but imported unconditionally
- `use crate::vm::{VM, bytecode::compiler::Compiler};` — the type is `BytecodeCompiler`

**What it involves:** fix those three, declare the modules, add `axum` and `futures` back to `Cargo.toml`, gate the signal handling behind `#[cfg(unix)]`, and give the VM a per-request execution model. Also needs item 5, since a server without route statements can only serve one handler.

---

## 5. Routes and responses as language statements

**Today:** `வழி` (route), `பதில்` (response) and the request accessors parse, but compile to `Unsupported`. `main.rs` registers the *entire program* as the handler for every method and path, so the server can only do one thing.

**What it involves:** execute `DefineRoute` at load time to populate the router, and give the VM a request context so `உடல்`, `அளவுரு` and `தலைப்பு` can read from it. Note that `Stmt::SendResponse` currently discards its headers.

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
