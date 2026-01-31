# eTamil Compiler - AI Coding Agent Instructions

## Project Overview

**eTamil** is a Tamil-based programming language with a complete compiler toolchain. The project has two main components:

- **eTamil Language** (`eTamil_Code/`): VS Code extension providing syntax highlighting for `.etamil` and `.qmz` files
- **eTamil Compiler** (`etamil_compiler/`): Rust-based compiler with 4 execution backends

---

## Architecture Essentials

### The Four Execution Paths

The compiler supports **4 different execution models**, selectable via CLI flags:

1. **VM Executor** (default, `--vm`): AST → Bytecode → Stack-based VM
   - Fastest startup, <100ms execution
   - No external dependencies
   - Used for: scripts, CLI tools, quick prototyping

2. **HTTP Server - Sync** (`--server`): Handles one request sequentially
   - 1-10 req/sec throughput
   - Simple code paths, good for MVP/testing
   - Uses `tiny_http` crate

3. **HTTP Server - Async** (`--async`): High-concurrency backend with Tokio
   - 100-1000 req/sec (100x improvement over sync)
   - Production-ready for backend APIs
   - All `async`/`await` logic in `src/http/async_handler.rs`

4. **LLVM Backend** (`--llvm`): AST → LLVM IR → native code (legacy)
   - Complex, rarely used in new work
   - See `src/codegen.rs` for implementation

**Key File**: [etamil_compiler/src/main.rs](etamil_compiler/src/main.rs#L23) - Contains the flag-dispatch logic that selects which path to execute.

### Compilation Pipeline

```
Source (.etamil/.qmz)
  ↓ [Lexer: src/lexer.rs]
Tokens (100+ token types)
  ↓ [Parser: src/parser.rs]
Abstract Syntax Tree (Expr/Stmt enums)
  ↓ [Execute Path - choose above]
```

- **Lexer** (`src/lexer.rs`): Uses `logos` crate, handles bilingual Tamil/English tokens
- **Parser** (`src/parser.rs`): Recursive descent parser, builds Expr/Stmt trees
- **VM Codegen** (`src/vm/bytecode/compiler.rs`): ~170 lines, converts AST→bytecode
- **VM Interpreter** (`src/vm/interpreter.rs`): Stack-based execution, ~500 lines

---

## Critical Module Organization

### Core Modules (`etamil_compiler/src/`)

| Module | Purpose | Key Files |
|--------|---------|-----------|
| `lexer.rs` | Tokenization | Bilingual token enums |
| `parser.rs` | AST construction | Expr/Stmt enums, recursive descent |
| `vm/` | Bytecode execution | `bytecode/mod.rs`, `interpreter.rs`, `value.rs` |
| `http/` | HTTP request handling | `mod.rs`, `async_handler.rs` (Phase 2 async) |
| `db/` | Database connections | `relational.rs` (SQL), `nosql.rs` (MongoDB/Redis) |
| `fileio/` | File I/O & CSV | `csv_handler.rs`, `crypto.rs` (encryption) |
| `commands.rs` | CLI command dispatch | Database commands, variable management |

### Entry Points

- **Standalone execution**: `etamil_compiler/src/main.rs` (async main with Tokio)
- **Library usage**: `etamil_compiler/src/lib.rs` (exports public APIs)

---

## Phase Structure & Status

| Phase | Focus | Status | Key Files |
|-------|-------|--------|-----------|
| **1** | HTTP Server MVP | ✅ Complete | `src/http/mod.rs`, `src/http/request.rs` |
| **2** | Async Server (100x throughput) | ✅ Complete | `src/http/async_handler.rs`, `load_test_async.sh` |
| **3** | Production Features (logging, errors) | ✅ Complete | Documentation in `docs/phases/` |
| **4** | Advanced Modules (JWT, caching, circuit breakers) | 🔄 Ready | Module files in `src/` (awaiting integration) |

**Note**: Phase 4 modules exist but aren't wired into the async handler yet. Integration is planned.

---

## Build & Test Workflow

### Development Workflow

```bash
# Build
cd etamil_compiler
cargo build              # Debug build (~7s)
cargo build --release   # Release (~2-5 min)

# Test
cargo test              # Run all unit tests
./test_all_examples.sh  # Run all eTamil examples
./test_fileio.sh        # Test file I/O operations
./test_http_backend.sh  # Test HTTP server modes

# Run
etamil myprogram.etamil                    # VM execution
etamil --server --port 8080 api.etamil     # Sync HTTP server
etamil --async --port 8080 api.etamil      # Async HTTP server
```

### Test Files Location

- **Unit tests**: Inline in `src/*.rs` files (use `#[test]` attribute)
- **Integration tests**: `etamil_compiler/tests/`
- **Example programs**: `etamil_compiler/examples/`
- **Test scripts**: `*.sh` files in root and `etamil_compiler/`

---

## Project-Specific Patterns & Conventions

### 1. Bilingual Support (Tamil + English)

The language accepts both Tamil and English keywords. Both forms compile identically:

```
Tamil:   அச்சு("வணக்கம்");
English: print("வணக்கம்");
```

**Implementation**: `src/lexer.rs` maps both token variants to the same enum value. When adding new keywords, register both forms in the token lexer.

### 2. Stack-Based VM Architecture

Values flow through a stack, not a traditional call frame system:

```rust
// src/vm/interpreter.rs structure:
pub struct VM {
    pub stack: Vec<Value>,
    pub variables: HashMap<String, Value>,
    pub instruction_pointer: usize,
}
```

**Operations**: Push values, perform operations (ops pop from stack), push results. See `src/vm/bytecode/mod.rs` for all instruction types (80+).

### 3. Async Server Pattern (Phase 2)

The async server uses Tokio + Axum framework, NOT `tiny_http`. When modifying HTTP handling:

- **Sync server**: Single-threaded, see [src/http/mod.rs](etamil_compiler/src/http/mod.rs)
- **Async server**: Multi-threaded, see [src/http/async_handler.rs](etamil_compiler/src/http/async_handler.rs)

Request handlers inject request data as variables into the VM interpreter.

### 4. Database Module Pattern

Database modules support **6 types**: SQLite, MySQL, PostgreSQL, MongoDB, Redis, JSON.

- **Relational** (`src/db/relational.rs`): SQL databases
- **NoSQL** (`src/db/nosql.rs`): MongoDB, Redis, JSON

Connection pooling is built-in. See [DATABASE_COMMANDS_GUIDE.md](docs/backend/DATABASE_COMMANDS_GUIDE.md) for usage patterns.

### 5. File I/O with Encryption

File operations are in `src/fileio/csv_handler.rs` and support encryption (`crypto.rs`):

- `.txt` files → encrypt to `.ani`
- `.csv` files → encrypt to `.qrv`

Custom encryption keys are supported. See examples in `examples/` directory.

---

## Critical Integration Points

### HTTP → VM Execution

Request data flows from HTTP handler into VM via variable injection:

```rust
// In async_handler.rs
interpreter.variables.insert("request_path".to_string(), Value::String(path));
interpreter.variables.insert("request_body".to_string(), Value::String(body));
// Then execute bytecode with these variables in scope
```

### Database Connectivity

Database queries are issued through `CommandExecutor`:

```rust
pub struct CommandExecutor {
    relational_connection: Option<RelationalDB>,
    nosql_connection: Option<NoSQLDB>,
}
```

Not all DB operations are wired into the async handler yet (Phase 4 integration pending).

---

## Common Development Tasks

### Adding a New Instruction Type

1. Add variant to `enum Instruction` in `src/vm/bytecode/mod.rs`
2. Add compilation case in `src/vm/bytecode/compiler.rs`
3. Add execution case in `src/vm/interpreter.rs`
4. Test with example program in `examples/`

### Adding a New HTTP Route

**For Sync Server**: Register in [src/http/mod.rs](etamil_compiler/src/http/mod.rs#L50) using `register_route(method, path, handler)`

**For Async Server**: Add route handler in [src/http/async_handler.rs](etamil_compiler/src/http/async_handler.rs), using Axum routing.

### Running Load Tests

```bash
cd etamil_compiler
chmod +x ../load_test_async.sh
../load_test_async.sh
# Verifies 100x throughput improvement target
```

---

## Key Workarounds & Known Issues

- **Phase 4 Modules**: JWT, caching, circuit breakers, metrics exist in `src/` but aren't integrated with the async HTTP handler. Wire them by modifying `async_handler.rs`.
- **LLVM Path**: Rarely used; if maintaining, see `src/codegen.rs` (~1100 lines). LLVM dependencies are optional in `Cargo.toml`.
- **Language Design**: eTamil is expression-based. All statements (assign, print, loops) are expressions that may return values.

---

## Documentation Hierarchy (Read in Order)

1. **[START_HERE.md](START_HERE.md)** - Quick overview for new users
2. **[README.md](README.md)** - Feature list and quick start
3. **[docs/architecture/OVERVIEW.md](docs/architecture/OVERVIEW.md)** - Detailed module breakdown
4. **[docs/phases/PHASE_2_IMPLEMENTATION.md](docs/phases/PHASE_2_IMPLEMENTATION.md)** - Async server integration details
5. **[etamil_compiler/src/main.rs](etamil_compiler/src/main.rs)** - Flag dispatch and entry point logic

---

## Testing Requirements

Before submitting changes:

- [ ] `cargo build` succeeds without errors
- [ ] `cargo test` passes all unit tests
- [ ] Run relevant `./test_*.sh` script
- [ ] No new compiler warnings introduced (54 existing warnings are acceptable)
- [ ] New code follows module organization patterns (see above)

---

**Last Updated**: January 31, 2026 | **Status**: Production Ready
