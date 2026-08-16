# eTamil Programming Language

**A programming language whose vocabulary is Tamil, aimed at Indian FinTech.**

[![CI](https://github.com/Maruff/etamil_compiler/actions/workflows/ci.yml/badge.svg)](https://github.com/Maruff/etamil_compiler/actions/workflows/ci.yml)

---

## What is eTamil?

eTamil lets you write programs in Tamil. It is not an English language with translated keywords: finance is built into the vocabulary, so `வரவு` (credit), `பற்று` (debit), `வரி` (tax) and `இருப்புநிலை` (balance sheet) are part of the language itself.

Every keyword accepts three spellings that mean exactly the same thing:

```etamil
எண் வருவாய் = 100000;     // Tamil script
eN varuvAy = 100000;       // romanized (ezuqqu scheme)
```

That is the core idea: Tamil semantics you can type on a plain keyboard.

### Real example

```etamil
// Income tax calculator
எண் வருவாய்;
அச்சு "Enter income: ";
உள்ளிடு வருவாய்;
வரி = 20%;

(வருவாய் > 800000) எனில் {
    அச்சு "High Tax Bracket";
    அச்சு (வருவாய் - 800000) * வரி;
}
இன்றேல் {
    அச்சு "Low Tax Bracket (No Tax)";
}
```

```bash
echo "950000" | etamil --vm examples/basic_samples/example.qmz
```

---

## Project status

eTamil is **usable for scripts and calculations, and under construction everywhere else.** This table is the honest state of the code — please read it before planning work on top of it.

| Area | Status | Notes |
|---|---|---|
| Lexer (Tamil / romanized / English keywords) | ✅ Working | ~200 tokens, reports errors with line and column |
| Variables, arithmetic, percentages, strings | ✅ Working | |
| Comparisons, `எனில்` / `இன்றேல்`, `சுற்று` loops | ✅ Working | |
| Logical `மற்றும்` / `அல்லது` / `இல்லை` | ✅ Working | both sides always evaluated — no short-circuiting |
| File I/O and CSV row counting | ✅ Working | in the VM (`--vm`) |
| VM bytecode executor | ✅ Working | |
| Sync HTTP server (`--server`) | 🟡 Minimal | single-threaded; serves one handler on all routes |
| Async HTTP server (`--async`) | ❌ Not implemented | falls back to the sync server; see [ROADMAP](docs/ROADMAP.md) |
| Databases (`தளம்_இணை` etc.) | ❌ Not implemented | statements parse, then fail with a clear error |
| Routes / responses as DSL statements | ❌ Not implemented | same |
| LLVM backend (`--llvm`) | 🟡 Optional | off by default, Linux/macOS only, `--features llvm` |
| Decimal currency arithmetic | ✅ Working | fixed-point decimals, not `f64` |

Anything marked "not implemented" **fails with an explicit message** rather than quietly doing nothing. That is deliberate: silent no-ops in a tax calculator are worse than errors.

### Money is exact

Every number in eTamil is a fixed-point decimal, so the arithmetic a tax program actually performs comes out right:

```etamil
அச்சு 0.1 + 0.2;      // 0.3        — not 0.30000000000000004
அச்சு 99.99 * 3;      // 299.97     — not 299.96999999999997
அச்சு 18%;            // 0.18       exactly
```

Equality is exact too. Division keeps full precision rather than rounding at each step, because Indian tax computation rounds once at the end — round explicitly when you need to.

---

## Installation

### Build from source (all platforms)

Requires **Rust 1.85+** (edition 2024) and a C toolchain for linking.

```bash
git clone https://github.com/Maruff/etamil_compiler.git
cd etamil_compiler/etamil_compiler
cargo build --release
```

The binary is `target/release/etamil` (`etamil.exe` on Windows). Put it on your `PATH`:

**Linux / macOS**
```bash
sudo cp target/release/etamil /usr/local/bin/etamil
```

**Windows (PowerShell)**
```powershell
Copy-Item "target\release\etamil.exe" "$env:USERPROFILE\bin\etamil.exe"
```

Verify:

```bash
etamil --version
```

### Optional: LLVM backend

Only needed for `--llvm`, and only available on Linux/macOS:

```bash
cargo build --release --features llvm
```

---

## Quick start

```bash
echo 'அச்சு "வணக்கம் உலகம்!";' > hello.etamil
etamil --vm hello.etamil
```

On Windows, write the file as UTF-8:

```powershell
'அச்சு "வணக்கம் உலகம்!";' | Out-File hello.etamil -Encoding UTF8
etamil --vm hello.etamil
```

---

## Language reference

### Variables and types

```etamil
எண் age = 25;          // number
எண் price = 99.99;     // fixed-point decimal; no separate int/float yet
எண் rate = 15%;        // percentage literal -> exactly 0.15
சொல் name = "Ravi";    // string
```

Type keywords are currently **accepted but not enforced** — nothing stops `சொல் x = 5;`.

### Input and output

```etamil
எண் வருவாய்;
அச்சு "Enter income: ";
உள்ளிடு வருவாய்;
அச்சு "Income: " & வருவாய்;   // & concatenates
```

Input always arrives as text and is converted when compared or used in arithmetic.

### Conditionals and loops

```etamil
(வருவாய் > 800000) எனில் {
    அச்சு "High";
}
இன்றேல் {
    அச்சு "Low";
}

எண் i = 0;
(i < 3) சுற்று {
    அச்சு i;
    i = i + 1;
}
```

### Operators

| Kind | Operators |
|---|---|
| Arithmetic | `+` `-` `*` `/` (and unary `-`) |
| Comparison | `==` `!=` `<` `<=` `>` `>=` |
| Logical | `மற்றும்` / `maRRum` / `_and`, `அல்லது` / `allaqu` / `_or`, `இல்லை` / `illY` / `_not` |
| String | `&` |

Precedence, loosest first: `or` → `and` → `not` → comparison → `+ -` → `* /`.

```etamil
(வருவாய் > 800000 மற்றும் வயது < 60) எனில் {
    அச்சு "Taxable";
}
```

### File I/O

```etamil
கோப்பு_திற "output.txt", "write";     // opening for write truncates
கோப்பு_எழுது "output.txt", "வணக்கம்";  // subsequent writes append
கோப்பு_மூடு "output.txt";

கோப்பு_படி "output.txt", data;        // read whole file into a variable
அச்சு data;
```

CSV row counting (excludes the header):

```etamil
தரவுரை_படி "students.csv", total;
அச்சு total;
```

---

## Tamil letter mapping (ezuqqu scheme)

12 vowels + 18 consonants + ஃ + 5 borrowed letters.

| Tamil | eTamil | Transliteration | ISO 15919 |
|-------|--------|-----------------|-----------|
| அ | a | a | a |
| ஆ | A | aa | ā |
| இ | i | i | i |
| ஈ | I | ii | ī |
| உ | u | u | u |
| ஊ | U | uu | ū |
| எ | e | e | e |
| ஏ | E | ee | ē |
| ஐ | Y | ai | ai |
| ஒ | o | o | o |
| ஓ | O | oo | ō |
| ஔ | V | au | au |
| க | k | k | k |
| ங | w | ng | ṅ |
| ச | c | ch | c |
| ஞ | W | nj | ñ |
| ட | t | t | ṭ |
| ண | N | nn | ṇ |
| த | q | th | t |
| ந | n | n | n |
| ப | p | p | p |
| ம | m | m | m |
| ய | y | y | y |
| ர | r | r | r |
| ல | l | l | l |
| வ | v | v | v |
| ழ | z | zh | ḻ |
| ள | L | ll | ḷ |
| ற | R | rr | ṟ |
| ன | Z | n | ṉ |
| ஃ | h | h | ḵ |
| ஹ | H | h | h |
| ஜ | j | j | j |
| ஷ | S | sh | ṣ |
| ஸ | s | s | s |
| க்ஷ | x | ksh | kṣ |

### The n-family letters

Tamil has three distinct nasals that English collapses into one `n`. eTamil keeps them apart:

| Tamil | eTamil | ISO 15919 | Example |
|---|---|---|---|
| ண | `N` | ṇ | `எண்` → `eN` |
| ந | `n` | n | `நிதி` → `niqi` |
| ன | `Z` | ṉ | `பயன்` → `payaZ` |

`Z` was free — ழ is lowercase `z` — so it takes ன, leaving `N` unambiguously ண and `n` unambiguously ந. Words containing more than one show the distinction clearly: `நாணயம்` → `nANayam` (ந-ண), `பின்னம்` → `piZZam` (two ன), `வருமானம்` → `varumAZam`.

This replaces an earlier scheme in which ந and ன both used `n` and a few keywords spelled ந as `N`. **Romanized source written before this change needs updating** — Tamil-script source is unaffected.

See the [Tamil Letter Equivalents Guide](docs/reference/COMPILER_TAMIL_LETTER_EQUIVALENTS.md) for the full derivation.

---

## Commands

```
etamil [FLAGS] [OPTIONS] <FILE>
```

| Flag | Effect |
|---|---|
| `--vm` | Run on the bytecode VM (default) |
| `--server` | Start the synchronous HTTP server |
| `--async` | Currently an alias for `--server` |
| `--llvm` | LLVM backend (requires `--features llvm`; Linux/macOS) |
| `--port <PORT>` | Server port (default 8080) |
| `--host <HOST>` | Server host (default 127.0.0.1) |

---

## Examples

All examples live in [`examples/`](examples/):

| Path | What it shows |
|---|---|
| `basic_samples/example.qmz` | Income tax calculator |
| `io_samples/simple_fileio.qmz` | File read/write |
| `io_samples/fileio_example.qmz` | Longer file handling |
| `db_samples/*.qmz` | Database syntax — **these currently fail**, since DB execution is not implemented |
| `backend/*.qmz` | HTTP server samples for `--server` |

```bash
etamil --vm examples/basic_samples/example.qmz
etamil --server --port 8080 examples/backend/hello_server.qmz
```

---

## Testing

```bash
cd etamil_compiler
cargo test
```

`tests/language_tests.rs` covers the front end end-to-end — operators, control flow, file I/O, and diagnostics — by asserting on **program results**, not exit codes. Unit tests for the HTTP and file modules live beside their source.

CI runs the build and the full test suite on Linux and Windows for every push and pull request.

---

## Repository layout

```
etamil_compiler/
├── etamil_compiler/        # Rust crate
│   ├── src/
│   │   ├── lexer.rs        # bilingual tokenizer (logos)
│   │   ├── parser.rs       # recursive-descent parser -> AST
│   │   ├── vm/             # bytecode compiler + stack VM
│   │   ├── codegen.rs      # LLVM IR backend (optional feature)
│   │   ├── http/           # sync HTTP server, auth, cache, logging
│   │   └── fileio/         # file and CSV handling
│   └── tests/              # end-to-end language tests
├── examples/               # sample eTamil programs
├── scripts/                # keyword generation, romanization checker
└── docs/                   # guides, reference, roadmap
```

---

## Contributing

The most useful contributions right now are on the [roadmap](docs/ROADMAP.md): decimal arithmetic, parser error positions, and wiring the database layer to real drivers.

Please add a test to `etamil_compiler/tests/language_tests.rs` for any language behaviour you change, and make sure `cargo test` passes on both Linux and Windows.

---

## License

Not yet specified.

---

**Version**: 0.2.0 · **Authors**: Esan Maruff
