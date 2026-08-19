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

eTamil runs backend programs today: functions, collections, error handling, modules, a SQLite database layer, a concurrent HTTP server with routing, and an accounting framework written in the language itself. This table is the honest state of the code.

| Area | Status | Notes |
|---|---|---|
| Lexer (Tamil / romanized / English keywords) | ✅ Working | 201 tokens across ~500 spellings; errors carry line and column |
| Variables, arithmetic, percentages, strings | ✅ Working | |
| Comparisons, `எனில்` / `இன்றேல்`, `சுற்று` loops | ✅ Working | |
| Logical `மற்றும்` / `அல்லது` / `இல்லை` | ✅ Working | both sides always evaluated — no short-circuiting |
| File I/O and CSV row counting | ✅ Working | in the VM (`--vm`) |
| VM bytecode executor | ✅ Working | |
| Functions (`செயல்` / `திரும்பு`) | ✅ Working | parameters, returns, local scope, recursion |
| Arrays (`[…]`) and records (`{…}`) | ✅ Working | indexing, field access, assignment |
| Iteration (`ஒவ்வொரு … இல்`) | ✅ Working | arrays, records, strings |
| Results (`சரி` / `தவறு` / `?`) | ✅ Working | Rust semantics; failure is a value, not an exception |
| Modules (`இறக்கு`) | ✅ Working | resolves beside the file, then `ETAMIL_PATH` |
| Decimal arithmetic | ✅ Working | fixed point, not `f64` |
| Standard library (`nUlakam/`) | ✅ Working | strings, math, arrays, money — **written in eTamil** |
| Accounting framework | ✅ Working | double entry, GST, three statements — **written in eTamil** |
| SQLite (`தளம்_இணை` etc.) | ✅ Working | parameterised queries only; rows return as an array of records |
| PostgreSQL | ✅ Working | `--features postgres`; money as native `NUMERIC`, so a text column stays text — unlike SQLite, where decimals are stored as text |
| MySQL / MariaDB | 🟡 Untested | `--features mysql`; compiles and is complete, but has not yet been run against a live server |
| HTTP server (`--server`) | ✅ Working | worker pool; `வழி` routes with `:id` path parameters, query params, headers and request bodies; `பதில்` responses |
| LLVM backend (`--llvm`) | 🟡 Subset | Linux/macOS, `--features llvm`. Compiles far less than the VM — no functions, iteration, collections or modules — and refuses what it cannot build rather than emitting IR that computes something else |
| Response headers | ✅ Working | `பதில் 200, உடல், {"Content-Type": "text/html"}` — an ordinary record; defaults to JSON when omitted |
| JSON (`nUlakam/jEcAZ.qmz`) | ✅ Working | `ஜேசான்_ஆக்கு` / `ஜேசான்_படி` — **written in eTamil**; `\uXXXX` escapes are not decoded |
| Authentication | ✅ Working | bcrypt and JWT in the host; `கடவுச்சொல்_மறை` `கடவுச்சொல்_சரியா` `சீட்டு_ஆக்கு` `சீட்டு_சரிபார்`. Set `ETAMIL_JWT_SECRET` |
| String escapes | ✅ Working | `\n` `\t` `\r` `\"` `\\`; an unknown escape keeps both characters |
| `ஜேசான்_உரை` statement | ❌ Not implemented | parses but the VM refuses it — build the body with `ஜேசான்_ஆக்கு` and send it with `பதில்` |
| MongoDB, Redis | ❌ Not implemented | they say so explicitly; neither fits the SQL-shaped `Database` trait, so both need a design first |
| Async HTTP server (`--async`) | ✅ Working | tokio accept loop, handlers on the blocking pool; the VM stays synchronous |
| Parse error positions | ✅ Working | every error carries a line and column, bilingually |
| Type checking | ✅ Working | a declared type is enforced, with a position; deliberately narrow — no rule the rest of the language does not follow |
| VS Code extension | ✅ Working | `eTamil_Code/` — highlighting for all 201 keywords in every spelling, completions for 23 builtins and 122 `nUlakam` functions, and errors from `--check` as you type. Grammar and completion data are **generated** from `lexer.rs`; CI fails if they drift |

Anything marked "not implemented" **fails with an explicit message** rather than quietly doing nothing. That is deliberate: silent no-ops in a tax calculator are worse than errors.

### One thing to know before you write much

**A name is stored exactly as you wrote it**, including when the word you chose is also a keyword: `வங்கி = 5` creates a variable called `வங்கி`, and `{வரி: 100}` produces the field `வரி`. Names used to be filed under their English token name — `Bank`, `Tax` — which anglicised a Tamil author's chosen words and put English field names into Tamil output.

The consequence, which is a real change in meaning: `{வரி: 1}` and `{vari: 1}` are now **different** fields, and `வருவாய்` and `varuvAy` are different variables. Pick one spelling per program. A field name is data — what you typed — not a language construct.

Type keywords and SQL clause keywords remain **hard reserved** and cannot be names at all: `எண்`, `சொல்`, `அணி`, `வரிசை`, `விதி`, `இடம்`, `உள்`, `வெளி`, `குழு`, `சேர்`. Financial keywords are *not* reserved — `தொகை` is a perfectly good name for an amount. [KEYWORDS.md](docs/reference/KEYWORDS.md) lists every one.

### Errors say where

```
✗ வரி 3, நெடுவரிசை 1: ';' எதிர்பார்க்கப்பட்டது, 'அச்சு' கிடைத்தது
  (line 3, column 1: expected ';', found 'அச்சு')
```

Columns count written letters, not bytes, so the position is the one you would point at on the screen.

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

### Download a package (no Rust, no C toolchain)

**[Windows x64](https://github.com/Maruff/etamil_compiler/releases/latest/download/etamil-windows-x64.zip)**
&middot;
**[Linux x64](https://github.com/Maruff/etamil_compiler/releases/latest/download/etamil-linux-x64.tar.gz)**
&middot;
[all releases](https://github.com/Maruff/etamil_compiler/releases/latest)

The archive holds the compiler, `nUlakam/` (the eTamil standard library) and the
examples. The install script copies them into place, puts `etamil` on your `PATH`
and sets `ETAMIL_PATH` so `இறக்கு "nUlakam/paNam.qmz"` resolves from any
directory. Neither installer needs administrator rights.

**Windows (PowerShell)**
```powershell
Expand-Archive etamil-windows-x64.zip -DestinationPath .
.\etamil-windows-x64\install.ps1
```

**Linux**
```bash
tar -xzf etamil-linux-x64.tar.gz
./etamil-linux-x64/install.sh
```

Open a new terminal afterwards — a shell that is already running does not see a
`PATH` change — then `etamil --version`.

There is nothing else to install. The Windows binary links the C runtime
statically, so it does not need the Visual C++ Redistributable; the Linux binary
is built against musl, so it is one static ELF that does not depend on the build
machine's glibc. Uninstalling is deleting a directory:
`%LOCALAPPDATA%\Programs\eTamil` on Windows, `~/.local/{bin/etamil,lib/etamil}` on
Linux.

macOS has no prebuilt package yet — build from source.

To build the packages yourself, see [`packaging/`](packaging/).

### Build from source (all platforms)

Needed for the optional database drivers, the LLVM backend, or work on the
compiler itself.

Requires **Rust 1.85+** (edition 2024) and a C toolchain, since the bundled
SQLite and the crypto crates compile C:

- **Windows** — Visual Studio Build Tools with the "Desktop development with
  C++" workload. The MSVC linker is not optional: without it even
  `cargo check` fails, because proc-macro crates link as DLLs.
- **Linux / macOS** — a working `cc` (build-essential, or the Xcode command
  line tools).

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

### Optional: PostgreSQL and MySQL

SQLite is built in. The others are behind features, so a default build does
not carry their dependencies:

```bash
cargo build --release --features postgres,mysql
```

```etamil
தளம்_இணை போச்குரசீகுல், "postgres://user:pass@localhost:5432/kaNakku";
தளம்_வினா "SELECT peyar, qokY FROM pativukaL WHERE vakY = $1", ["வரவு"], வரவுகள்;
```

PostgreSQL placeholders are `$1, $2, …`; SQLite and MySQL use `?`. Both of the
server backends keep money in the database's own exact decimal type, so a text
column stays text on the way back — where the SQLite backend, which stores
decimals as text, hands back a number. PostgreSQL also folds unquoted
identifiers to lower case: write `"qokY"` if you want that column name back as
you spelled it.

```etamil
தளம்_இணை மைசீகுல், "mysql://root@localhost:3306/kaNakku";
தளம்_வினா "SELECT peyar, qokY FROM pativukaL WHERE vakY = ?", ["வரவு"], வரவுகள்;
```

`examples/db_samples/mYcIkul_qaLam.qmz` checks the things worth checking on a
real server — exact `DECIMAL` sums, an integer key bound from a number, dates
as ISO text, `NULL` as `இன்மை`, and an injection payload staying inert. It
needs a database, so the example runner skips it unless you opt in:

```bash
ETAMIL_TEST_MYSQL=1 ./scripts/run_examples.sh
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

A declared type is **enforced**, and a later assignment is held to it too:

```
✗ வரி 2, நெடுவரிசை 6: 'கொடியா' ஈர்ம (Irma, a boolean) என அறிவிக்கப்பட்டது,
  ஆனால் ஒரு அணி (an array) வழங்கப்பட்டது
  (line 2, column 6: 'கொடியா' is declared a boolean, but was given an array)
```

The checker is deliberately narrow: it holds you to what you declared and
states no rule the rest of the language does not follow. A number satisfies
`சொல்`, because every value renders as text and `உள்ளிடு` hands back text that
is routinely compared with numbers. A call, an index and a field access make no
claim, because functions have no declared signatures yet — silence there is the
absence of a claim, not approval.

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
| `--check` | Lex, parse and type check only, then stop — reports every error and **never runs the program** |
| `--server` | Start the synchronous HTTP server |
| `--async` | Concurrent server: async accept, blocking handlers, Ctrl-C to stop |
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
| `db_samples/*.qmz` | SQLite — parameterised queries, rows returned as records |
| `finance/*.qmz` | The accounting framework: GST, payroll, a full cycle, receivables ageing |
| `backend/*.qmz`, `api/*.qmz` | HTTP handlers — run these with `--server`, not `--vm` |

```bash
etamil --vm examples/basic_samples/example.qmz
etamil --server --port 8080 examples/backend/hello_server.qmz
```

A route reads its request through plain variables, so the same handler is
readable in either spelling:

```etamil
வழி பெறு, "/kaNakku/:id" {
    பதில் 200, "id=" & param_id & " vakY=" & query_params["vakY"];
}

வழி பதி, "/pativu" {
    பதில் 201, "got body: " & request_body;
}

வழி பெறு, "/aRikkY.csv" {
    பதில் 200, வரிசைகள், {"Content-Type": "text/csv"};
}
```

A handler reads its request through `request_method` · `request_path` ·
`request_body` · `query_params` · `headers` · `path_params`, and each path
parameter also arrives as `param_<name>`. Response headers are an ordinary
record; without one the server answers `application/json`.

---

## Testing

```bash
cd etamil_compiler
cargo test          # 176 language tests + 51 unit tests
```

`tests/language_tests.rs` covers the front end end-to-end — operators, control flow, file I/O, the standard library and the accounting framework — by asserting on **program results**, not exit codes. Every bug those cover exited 0 while producing the wrong answer. Unit tests for the HTTP, request and file modules live beside their source.

Every example also runs with its expected outcome checked, including the ones that are *supposed* to fail:

```bash
./scripts/run_examples.sh
python3 scripts/transliterate.py --check   # romanization audit
```

CI runs the build and the full test suite on Linux and Windows for every push and pull request.

---

## Repository layout

```
etamil_compiler/
├── etamil_compiler/        # Rust crate — the host
│   ├── src/
│   │   ├── lexer.rs        # bilingual tokenizer (logos)
│   │   ├── parser.rs       # recursive-descent parser -> AST
│   │   ├── module.rs       # இறக்கு resolution
│   │   ├── vm/             # bytecode compiler + stack VM
│   │   ├── db/             # Database trait + SQLite driver
│   │   ├── http/           # HTTP server, auth, cache, logging
│   │   ├── fileio/         # file and CSV handling
│   │   └── codegen.rs      # LLVM IR backend (optional feature)
│   └── tests/              # end-to-end language tests
├── nUlakam/                # standard library — written in eTamil
│   ├── col.qmz  kaNiqam.qmz  aNi.qmz  paNam.qmz
│   └── kaNakkiyal/         # accounting framework — written in eTamil
├── eTamil_Code/            # VS Code extension — grammar and completions generated from the lexer
├── examples/               # sample eTamil programs
├── scripts/                # keyword generation, romanization audit, runner
└── docs/                   # guides, reference, roadmap
```

## Libraries, written in eTamil

The standard library and the accounting framework are eTamil source, not Rust. The host provides only what a language cannot express — arithmetic on decimals, text measurement, file and socket access — and everything above that is readable and editable by the people who use it.

```etamil
இறக்கு "nUlakam/paNam.qmz";
அச்சு ரூபாய்(12345678.5);            // ₹1,23,45,678.50 — Indian grouping
```

```etamil
இறக்கு "nUlakam/kaNakkiyal/pErEtu.qmz";

த = பரிவர்த்தனை_ஆக்கு("JV1", "2026-04-01", "மூலதனம்", [
    பற்று_வரிசை("1000", 500000),
    வரவு_வரிசை("3000", 500000)
]);
பேரேடு = மதிப்பு(பதிவிடு(பேரேடு, த));   // refuses anything unbalanced
```

See [`nUlakam/README.md`](nUlakam/README.md) for the full list, and `examples/finance/kaNakkiyal.qmz` for a complete accounting cycle with GST and the three statements.

---

## Contributing

The most useful contributions right now are on the [roadmap](docs/ROADMAP.md): keeping the source text of tokens (which also gives parse errors their positions), the async server, type checking, and the remaining database drivers.

Please add a test to `etamil_compiler/tests/language_tests.rs` for any language behaviour you change, and make sure `cargo test` passes on both Linux and Windows.

---

## License

[GNU Affero General Public License v3.0 or later](LICENSE) (AGPL-3.0-or-later).

The AGPL matters here because eTamil is a backend language: running a modified
compiler or runtime as a network service counts as distribution, so the changes
have to be offered back to the people using that service.

---

**Version**: 0.2.0 · **Authors**: Esan Maruff
