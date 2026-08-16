# eTamil Documentation Index

```
docs/
├── getting-started/   # installation and first steps
├── architecture/      # how the compiler is put together
├── backend/           # HTTP server and database guides
├── reference/         # language and letter-scheme reference
├── phases/            # development phase reports
└── archive/           # superseded historical documents
```

> **Read [ROADMAP.md](ROADMAP.md) first if you plan to build on eTamil.** Several documents below describe features as complete that are not yet wired up — the roadmap and the status table in the [README](../README.md) are the authoritative account of what runs today.

---

## Start here

- **[README](../README.md)** — what eTamil is, how to build it, and the current status of every subsystem
- **[ROADMAP](ROADMAP.md)** — what is unfinished, why it matters, and what it takes to finish
- **[Installation Guide](getting-started/INSTALLATION.md)** — setup instructions
- **[Quick Start](getting-started/QUICKSTART.md)** — short tutorial

---

## Language reference

- **[Tamil Letter Equivalents](reference/COMPILER_TAMIL_LETTER_EQUIVALENTS.md)** — the ezuqqu romanization scheme
- **[eTamil Standard](reference/ETAMIL_STANDARD.md)** — language standard
- **[Quick Reference](reference/QUICK_REFERENCE.md)** — syntax at a glance
- **[File I/O Features](reference/FILE_IO_FEATURES.md)** — file and CSV statements
- **[VM Quick Start](reference/QUICK_START_VM.md)** — running programs on the VM

---

## Architecture

- **[System Overview](architecture/OVERVIEW.md)** — high-level design
- **[DSL Design](architecture/DSL.md)** — language design rationale
- **[VM Implementation](architecture/VM_IMPLEMENTATION_SUMMARY.md)** — bytecode and interpreter

The pipeline is: `lexer.rs` (logos) → `parser.rs` (recursive descent) → `vm/bytecode/compiler.rs` → `vm/interpreter.rs`. The optional LLVM backend in `codegen.rs` is an alternative to the last two stages.

---

## Backend development

⚠️ These guides describe the intended design. Today only the **synchronous** server runs, it serves a single handler on every route, and database statements fail with "not implemented". See [ROADMAP](ROADMAP.md) items 3–5.

- **[HTTP Server Implementation](backend/HTTP_SERVER_IMPLEMENTATION.md)**
- **[HTTP Server Quick Reference](backend/HTTP_SERVER_QUICKREF.md)**
- **[Database Commands](backend/DATABASE_COMMANDS_GUIDE.md)** — syntax reference; execution is not implemented
- **[Deployment Guide](backend/DEPLOYMENT_GUIDE.md)**
- **[Production Hardening](backend/PRODUCTION_HARDENING_GUIDE.md)**

---

## Development phases

Historical reports on how the code was built. They record intent at the time of writing and have **not** been reconciled with the current state of the code — Phase 2 in particular is described as complete, but its modules are not compiled. Treat the roadmap as authoritative.

- Phase 1 — [complete](phases/PHASE_1_COMPLETE.md): synchronous HTTP server
- Phase 2 — [status](phases/PHASE_2_STATUS.md): async server, **not wired up**
- Phase 3 — [logging](phases/PHASE_3_LOGGING_IMPLEMENTATION.md): logging, errors, metrics
- Phase 4 — [module status](phases/PHASE_4_MODULE_STATUS.md): auth, cache, resilience — built and unit-tested, not reachable from the DSL

---

## Examples

Sample programs live in [`../examples/`](../examples/), not under `docs/`. Basic and file I/O samples run today; database samples fail by design until database execution exists.

---

## Archive

`docs/archive/` holds superseded status reports, completion manifests, and refactoring summaries kept for history. Nothing there should be treated as current.
