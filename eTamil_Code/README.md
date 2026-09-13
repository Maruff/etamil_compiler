> 📖 New to eTamil? Start with the
> **[user manual at etamil.in/manual](https://etamil.in/manual/)**
> ([தமிழ்](https://etamil.in/ta/manual/)), or the
> [keyword reference](https://etamil.in/keywords/).

# eTamil for VS Code

Language support for [eTamil](https://etamil.in) — a programming language whose
vocabulary is Tamil, aimed at Indian FinTech.

```etamil
எண் வருவாய்;
அச்சு "Enter income: ";
உள்ளிடு வருவாய்;
வரி_வீதம் = 20%;

(வருவாய் > 800000) எனில் {
    அச்சு "Tax payable: " & வட்டமிடு((வருவாய் - 800000) * வரி_வீதம், 2);
}
இன்றேல் {
    அச்சு "No tax payable";
}
```

## What you get

| | |
|---|---|
| **Syntax highlighting** | All 202 keywords across 541 spellings — Tamil script, romanized, and the English aliases |
| **Errors as you type** | From the compiler's own front end, so they are the errors you will actually get |
| **Completions** | Keywords with correct statement templates, 62 host builtins, and all 691 `செயல்` functions in the `nUlakam` standard library |
| **Hover** | Every spelling of a word, whether it is reserved, and the doc comment from its definition |
| **Signature help** | Parameter names, read from the library's own source |
| **Go to Definition** | Jumps into `nUlakam`, and to functions in the current file |
| **Outline** | Every `செயல்` in the file |
| **Run** | Run or serve the current file in a terminal |
| **Examples** | **eTamil: Open an example** — twenty-nine working programs, carried in the extension |
| **Documentation** | **eTamil: Documentation…** — the manual, the playground and the reference on [etamil.in](https://etamil.in) |

Both spellings are first-class. Type `eZil` and you get a romanized template;
type `எனில்` and you get a Tamil one.

## Learning eTamil

The extension knows the language. It does not teach it, and everything that
does is on **[etamil.in](https://etamil.in)**:

| | |
|---|---|
| [User manual](https://etamil.in/language/manual/) | Installation through to database-backed HTTP services, in order |
| [Playground](https://etamil.in/start/) | Run eTamil in the browser — the compiler, built to WebAssembly |
| [Language tour](https://etamil.in/language/) | The language in editable, runnable examples |
| [Keyword reference](https://etamil.in/language/keywords/) | Every keyword in all three spellings |
| [Finance and accounting](https://etamil.in/finance/) | Ledgers, GST, tax |
| [Databases and HTTP](https://etamil.in/server/) | Building services |
| [Status and roadmap](https://etamil.in/status/) | What works today, and what does not yet |
| [Source](https://github.com/Maruff/etamil_compiler) | The compiler, the library and this extension |

**eTamil: Documentation…** in the Command Palette opens any of them, and every
keyword hover links the reference for that word.

**eTamil: Open an example** is the shorter route. The extension carries the
repository's twenty-nine example programs — the accounting framework, the HTTP
server, the GST invoice, the project-costing worked example — and opens a copy
you can edit and run. A copy, not the original: the extension directory is
replaced on every update.

## Errors never run your program

Diagnostics come from `etamil --check`, which lexes, parses and type checks and
then stops. That distinction matters for this language: an eTamil program writes
files, issues database queries and starts an HTTP server. Opening one in an
editor should not do any of that.

## Setup

There isn't any. **The extension carries the compiler.**

Inside it are the `etamil` binary for your platform and the whole nUlakam
standard library. Error checking, **eTamil: Run this file** and Go to Definition
into the library all work the moment the extension finishes installing — no
Rust, no download, no `PATH`. The Marketplace sends you the build for your
machine; installing a `.vsix` by hand means picking the one whose name ends in
your platform, `etamil-support-1.0.0-win32-x64.vsix`.

Three things are worth knowing about it.

**Your own compiler wins.** Set `etamil.compilerPath` and that is what runs — a
`cargo build` of the repository, say, or a release you installed yourself. The
carried one is only the default.

**A terminal cannot see inside an extension.** The editor runs the carried
binary where it lies, but `etamil` at a shell prompt is a different question,
and the extension directory is replaced wholesale on every update, so pointing
at it would break. **eTamil: Install the compiler for use outside the editor**
copies the binary and the library to `~/.local` — `%LOCALAPPDATA%\Programs\eTamil`
on Windows — and shows you the two lines that put them on `PATH` and
`ETAMIL_PATH`. It shows them rather than writing them: how every program on your
machine starts is not something an extension should change on its own.

**A platform with no build falls back.** If this VSIX carries nothing for your
platform and architecture, the extension looks for `etamil` on the `PATH` and
the install command offers the release package and the source build, as before.

By hand, the prebuilt package needs no Rust and no C toolchain:

```powershell
# Windows — https://github.com/Maruff/etamil_compiler/releases/latest
Expand-Archive etamil-windows-x64.zip -DestinationPath .
.\etamil-windows-x64\install.ps1
```

```bash
# Linux
tar -xzf etamil-linux-x64.tar.gz
./etamil-linux-x64/install.sh
```

```bash
# macOS — arm64 for Apple Silicon, x64 for Intel; `uname -m` tells you which.
tar -xzf etamil-macos-arm64.tar.gz
./etamil-macos-arm64/install.sh
# These builds are not notarized, so clear the quarantine flag once or
# Gatekeeper refuses to run them.
xattr -dr com.apple.quarantine ~/.local/lib/etamil
```

The packages carry the PostgreSQL and MySQL drivers. Build from source for the
LLVM backend — Rust 1.85+ and a C toolchain, MSVC Build Tools with "Desktop
development with C++" on Windows and `cc` elsewhere:

```bash
git clone https://github.com/Maruff/etamil_compiler.git
cd etamil_compiler/etamil_compiler
cargo build --release
```

Either way the binary has to be on your `PATH` — the installers do that — or in
`etamil.compilerPath`.

## Settings

| Setting | Default | |
|---|---|---|
| `etamil.compilerPath` | *(carried, then PATH)* | Path to the `etamil` binary. Machine-scoped, because it names an executable the extension runs |
| `etamil.eTamilFont` | *(off)* | Font for ASCII that is eTamil rather than English — `ican qamiz` ships with the extension, see below |
| `etamil.checkOnType` | `true` | Report errors while you type |
| `etamil.intelliSense` | `true` | Completions and signature help |

## Two scripts in one file

eTamil is written three ways and two of them are the same bytes. `செயல்` is
Tamil. `ceyal` is the same word spelled under the ezuqqu scheme, where one Latin
letter stands for one Tamil letter. `_length` is English. The last two are both
ASCII, and nothing in the bytes separates them.

There is an **eTamil font** in which the ASCII letters carry Tamil glyphs — `c`
draws ச, `q` draws த, `Z` draws ன. It lets you write Tamil on an ASCII keyboard
and read it back as Tamil. Under it an English word is nonsense: `sum` draws as
ஸும்.

So the file says which is which, and the extension reads what it says:

```etamil
// __Rounded to paise once, at the end.__
_sum = moqqam;
```

**The extension carries such a font**, `ican qamiz`, under the SIL Open Font
License. Run **eTamil: Install the eTamil font** and it copies the file to your
own font directory — `~/Library/Fonts`, `~/.local/share/fonts`, or
`%LOCALAPPDATA%\Microsoft\Windows\Fonts` with the registry value Windows needs
— then offers to set `etamil.eTamilFont` for you. No administrator rights, a
dialog that says what it will write before it writes it, and a restart of VS
Code afterwards, because the font list is read when the window starts.

The install command exists because shipping a font is not installing one. VS
Code's editor is not a webview and has no API that registers a font, so a
`font-family` only resolves against fonts the operating system already knows.

With the setting in place, `moqqam` is drawn in the eTamil face while `_sum`,
the comment between its `__` marks, every string literal and the licence header
stay in your ordinary editor font. **The editor's font is not changed.** It
stays the ISO stack and only the eTamil-script ASCII is painted over the top,
which matters more than it sounds: `ican qamiz` has no Tamil glyphs at all — it
maps ASCII and nothing else — so the base font is the one that has to carry
every `மொத்தம்` in the file. A span the extension fails to recognise then
renders eTamil as plain Latin, the usual view of the file, rather than English
in Tamil glyphs.

One caveat. `ican qamiz` is proportional, not monospaced, so a painted run does
not sit on the same grid as the code around it and text after it on the same
line shifts. A monospaced build of the face would make that exact and nothing
in the extension would change.

The two marks are the language's, not the extension's:
[SCRIPT_RULES.md](https://github.com/Maruff/etamil_compiler/blob/main/docs/reference/SCRIPT_RULES.md) specifies them,
`scripts/check_script_rules.py` gates them, and `fonts/README.md` records what
is in the font file.

## Tamil rendering

The extension sets a font stack for eTamil files that names Tamil faces before
the monospace fallback. Most programming fonts carry no Tamil glyphs at all and
fall back mid-line, which makes a line of source jump around as you read it. If
your editor still renders Tamil unevenly, install
[Noto Sans Tamil](https://fonts.google.com/noto/specimen/Noto+Sans+Tamil) and
put it first in `editor.fontFamily` for `[etamil]`.

## Romanized spelling

eTamil's romanization keeps Tamil's three nasals apart: **ண = `N`, ந = `n`,
ன = `Z`**. So the conditional is `eZil`, not `enil`, and the float type is
`piZZam`, not `pinnam`.

Versions of this extension before 0.3.0 shipped the older scheme, where ந and
ன both used `n`. The effect was backwards: the spellings the editor highlighted
were the ones the compiler rejected, and the ones it accepted got no
highlighting. If you have romanized eTamil written against a pre-0.3.0
extension, it needs updating — see the [letter equivalents
guide](https://github.com/Maruff/etamil_compiler/blob/main/docs/reference/COMPILER_TAMIL_LETTER_EQUIVALENTS.md).

## Contributing

**The keyword lists are generated. Do not edit them by hand.**

`syntaxes/etamil.tmLanguage.json` and `src/generated/language-data.ts` are
produced from the compiler:

```bash
python ../scripts/generate_editor_support.py
```

It reads the keywords and their spellings from `etamil_compiler/src/lexer.rs`,
whether each one is reserved from `src/parser.rs`, the builtins from
`src/vm/interpreter.rs`, and the standard library from `nUlakam/**/*.qmz`.
Nothing is restated here, which is the point — a second hand-kept copy is what
drifted before.

CI runs `--check`, which fails if the committed files no longer match the
compiler. Add a keyword to the lexer, and the build tells you to regenerate.

What is *not* generated is the shape of each statement — `(cond) எனில் { }`
cannot be inferred from a token name. Those templates live in the `SNIPPETS`
table in the generator and in `snippets/etamil.code-snippets`, and every one of
them is fed to the real compiler by `test/snippets.test.js`.

```bash
npm install
npm run verify     # generated files current, tsc, eslint, tests
```

The test suite has two parts. `test/grammar.test.js` loads the grammar with
vscode-textmate and asserts scopes — including that `\b` behaves correctly
around Tamil combining marks, which is what stops `எண்` matching inside
`எண்ணி`. `test/snippets.test.js` compiles every snippet.

## License

[GNU Affero General Public License v3.0 or later](LICENSE), matching the
compiler.
