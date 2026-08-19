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
| **Syntax highlighting** | All 201 keywords, in every spelling the compiler accepts — Tamil script, romanized, and the English aliases |
| **Errors as you type** | From the compiler's own front end, so they are the errors you will actually get |
| **Completions** | Keywords with correct statement templates, 23 host builtins, and all 122 `செயல்` functions in the `nUlakam` standard library |
| **Hover** | Every spelling of a word, whether it is reserved, and the doc comment from its definition |
| **Signature help** | Parameter names, read from the library's own source |
| **Go to Definition** | Jumps into `nUlakam`, and to functions in the current file |
| **Outline** | Every `செயல்` in the file |
| **Run** | Run or serve the current file in a terminal |

Both spellings are first-class. Type `eZil` and you get a romanized template;
type `எனில்` and you get a Tamil one.

## Errors never run your program

Diagnostics come from `etamil --check`, which lexes, parses and type checks and
then stops. That distinction matters for this language: an eTamil program writes
files, issues database queries and starts an HTTP server. Opening one in an
editor should not do any of that.

## Setup

The extension needs the `etamil` binary for error checking. Highlighting,
completions and hover work without it.

**eTamil: Install the compiler** in the Command Palette is the short way — it
offers the prebuilt package for your platform, or the source build. It opens the
download in a browser rather than fetching and running anything itself.

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

Or build it, which is what you want for the optional database drivers and the
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
| `etamil.compilerPath` | *(PATH)* | Path to the `etamil` binary. Machine-scoped, because it names an executable the extension runs |
| `etamil.checkOnType` | `true` | Report errors while you type |
| `etamil.intelliSense` | `true` | Completions and signature help |

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
guide](../docs/reference/COMPILER_TAMIL_LETTER_EQUIVALENTS.md).

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
