# Changelog

Releases of the eTamil compiler, standard library and editor support. The
extension's version tracks the language's, so both are 1.0.0 here.

Every release is tagged `vN.N.N`, which is what builds the platform packages.
GitHub's generated notes list the commits; this file says what they add up to.

---

## 1.0.0 — 2026-09-13

Sixty commits since 0.4.0. The headline is not a feature: it is that the
standard library is now large enough to do a real job, and that several things
the repository used to claim about itself are now checked rather than asserted.

### The standard library nearly tripled

**254 functions in 41 files became 691 in 82**, across seven new directories.
All of it is written in eTamil and tested in eTamil — 881 assertions in the new
suites alone, run by `scripts/run_examples.sh` like any other program.

| directory | what it is |
|---|---|
| `celavu/` | cost accounting — marginal and absorption costing, overhead apportionment including the reciprocal case, process costing with equivalent units and normal/abnormal loss, activity-based costing |
| `qittam/` | project costing — earned value, the critical path, contract pricing types with the point of total assumption, the four dependency types with lag |
| `nErativari/` | direct tax — the five heads, Chapter VI-A, the slab → rebate → surcharge → marginal relief → cess ladder, TDS, advance tax with 234A/B/C, deferred tax under Ind AS 12 |
| `varuvAy/` | revenue — the Ind AS 115 five-step model, contract assets and liabilities, modifications, principal versus agent, onerous contracts under Ind AS 37 |
| `nAtkAtti/` | the calendar — date arithmetic, working days, and the validation the host's own date handling does not do |
| `vaLam/` | resources, timesheets, and posting actual cost to the ledger |
| `itar/` | risk — expected monetary value, and three-point estimation with the variance that a single-point estimate cannot carry |

`examples/qittam/` threads thirteen of these modules across five directories
through one worked project.

### The language says which of its ASCII is English

eTamil is written three ways and two of them are the same bytes: `ceyal` is
`செயல்` spelled under the ezuqqu scheme, `sum` is an English word. There is an
optional **eTamil font** in which the ASCII letters carry Tamil glyphs, and
under it an unmarked English name is unreadable — `sum` draws as ஸும்.

Two marks fix that, and they are now rules of the language:

- an identifier containing English ASCII begins with `_`
- a comment containing English ASCII is wrapped in `__ … __`

Applied across the tree: 38 names and 4,704 comments. `scripts/check_script_rules.py`
enforces it and gates CI. See [docs/reference/SCRIPT_RULES.md](docs/reference/SCRIPT_RULES.md).

### Seventeen keywords that had no English spelling

Half the keyword table could be written in English and half could not, and the
half that could was the plumbing — `_get`, `_select`, `_encrypt`. There was no
English eTamil program, not even a hello-world: the first conditional dropped
you back into Tamil. The types, the literals, `_if`, `_else`, `_loop`, `_print`
and `_input` close it.

**No reserved keyword is now without an English spelling.** 118 of 202 keywords
have one; the 84 that do not are, without exception, words the parser accepts as
ordinary names — the accounting vocabulary, which stays Tamil on purpose.

### The LLVM backend stopped trading the language for a register

It computed in `f64`, then in `i64`, and both were the same bargain: the IR held
the value, so the language shrank to what fits in a register. Now every value is
a handle into an arena in `src/runtime.rs` and every operation is a call into
it — the same `rust_decimal` and the same `Value::to_string` the VM uses, so
`0.1 + 0.2` is `0.3` and `1 / 3` keeps all twenty-eight digits on both sides
because it is the same code rather than two that agree.

`தளம்_இணை`, `தளம்_செய்` and `தளம்_வினா` compile, so a compiled program can open
a database and read and write it. **99 of 111 programs would compile**, and
`scripts/llvm_gap_report.py` makes what is refused countable on a machine with
no LLVM at all.

### The VM stopped copying what it was about to throw away

Two measured fixes:

- Every instruction was deep-copied before dispatch, which meant a heap
  allocation for a variable's name on every access. That was most of the ~75 ns
  per instruction the benchmarks recorded.
- `x = இணை(x, v)` copied the whole array per call, so building a ledger of *n*
  rows copied ~n²/2 elements. It appends in place now, which is
  indistinguishable because every other binding already owns its own copy.

Appending 64,000 items now costs what 4,000 did.

### The editor is the whole installation

The VS Code extension carries the compiler, the standard library, the examples
and the eTamil font, and installs the font for you. It renders both marks —
under `etamil.eTamilFont` the eTamil face is painted over the ASCII that is
eTamil script, leaving the ISO font everywhere else. Its keyword, builtin and
stdlib counts are pinned to the generated language data by a test, because that
README was wrong twice.

### Audits that now gate rather than being remembered

`transliterate.py` checked only the keyword table, which is how `viziqam` — a
spelling that means nothing — reached a module name, a table and two columns.
Four scripts gate CI now: the scheme on keywords, the scheme everywhere else,
the two script marks, and duplicate function names across `nUlakam`.

### Breaking changes

- **`உள்ளதா` is now only the array function.** `AvaNam.qmz` defined a second one
  for substrings; two modules sharing a name is a silent redefinition, and the
  document one is `ஆவணத்தில்_உள்ளதா`.
- **`xml_ஆக்கு` and `pdf_ஆக்கு` are `_xml_ஆக்கு` and `_pdf_ஆக்கு`**, along with
  the document format constants and the z-score functions in `itar/`. Rule 1
  applies to the library first.
- **`கணக்கு_ஆக்கு` in `qittam/` is `கட்டுப்பாட்டுக்_கணக்கு_ஆக்கு`**, for the
  same reason `உள்ளதா` moved.
- **Seventeen new reserved words** in the `_` namespace. The type names are the
  ones to watch: `_int`, `_float`, `_string`, `_text`, `_bool`, `_array`,
  `_data`, `_object` and `_date` cannot be variable names, where most languages
  would allow them.

---

## 0.4.0 and earlier

See the [releases](https://github.com/Maruff/etamil_compiler/releases) and the
git history. `eTamil_Code/CHANGELOG.md` carries the extension's own record back
to 0.1.0.
