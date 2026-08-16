# eTamil Documentation

Start with the [README](../README.md) for what eTamil is and which parts of it run today, then pick a page below.

## Getting started

| Page | What it covers |
|---|---|
| [Installation](getting-started/INSTALLATION.md) | Prerequisites, building from source, verifying, troubleshooting |
| [Quick Start](getting-started/QUICKSTART.md) | Your first program, through to files and a server |

## Libraries

Both are written in eTamil, not Rust.

| Page | What it covers |
|---|---|
| [Standard library](../nUlakam/README.md) | `nUlakam/` — strings, math, arrays, money with Indian digit grouping |
| Accounting framework | `nUlakam/kaNakkiyal/` — chart of accounts, double-entry ledger, GST, and the three statements. See `examples/finance/kaNakkiyal.qmz` for a full cycle |

## Language reference

| Page | What it covers |
|---|---|
| [Keywords](reference/KEYWORDS.md) | Every keyword in all three spellings, with its token name — generated from the lexer |
| [Commands](reference/COMMANDS.md) | CLI flags, exit codes, environment variables |
| [Tamil Letter Equivalents](reference/COMPILER_TAMIL_LETTER_EQUIVALENTS.md) | How the ezuqqu romanization is derived |
| [File I/O](reference/FILE_IO_FEATURES.md) | File and CSV statements |
| [VM Quick Start](reference/QUICK_START_VM.md) | Running programs on the bytecode VM |
| [Quick Reference](reference/QUICK_REFERENCE.md) | Syntax at a glance — partly outdated |
| [eTamil Standard](reference/ETAMIL_STANDARD.md) | The language standard, with PDFs alongside |
| [VS Code Extension](reference/VSCODE_README.md) | Editor support (the extension itself lives in another repo) |

## Architecture

| Page | What it covers |
|---|---|
| [Module Overview](architecture/OVERVIEW.md) | How the crate is organized |
| [VM Implementation](architecture/VM_IMPLEMENTATION_SUMMARY.md) | Bytecode format and interpreter |

The pipeline is `lexer.rs` → `parser.rs` → `vm/bytecode/compiler.rs` → `vm/interpreter.rs`. `codegen.rs` is an optional LLVM backend replacing the last two stages.

## Backend

Both pages carry a status banner; read it first.

| Page | What it covers |
|---|---|
| [HTTP Server](backend/HTTP_SERVER_QUICKREF.md) | Server usage — sync only |
| [Database Commands](backend/DATABASE_COMMANDS_GUIDE.md) | Database syntax — not executable yet |

## Planning

| Page | What it covers |
|---|---|
| [Roadmap](ROADMAP.md) | What is unfinished, why it matters, what finishing it takes |

## Examples

Sample programs live in [`../examples/`](../examples/). The basic and file I/O samples run; the database samples fail by design.

---

## A note on older documents

Superseded documents — phase completion reports, backend planning drafts, older installation guides, VS Code write-ups — were removed rather than kept in the tree. Several described features as complete that were never wired up, which made them actively misleading. They remain in git history if you need them:

```bash
git log --diff-filter=D --name-only -- 'docs/archive/*'
```

When a page here and the **code** disagree, the code wins — please open an issue or fix the page.
