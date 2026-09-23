# tree-sitter-etamil

A [tree-sitter](https://tree-sitter.github.io) grammar for
**[eTamil](https://etamil.in)** — a programming language whose vocabulary is
Tamil, aimed at Indian accounting, tax and FinTech.

This is what Neovim, Helix and Zed parse with, and what GitHub uses for code
navigation.

```etamil
எண் வருவாய் = 950000;
வரி_வீதம் = 20%;

(வருவாய் > 800000) எனில் {
    அச்சு "Tax: " & வட்டமிடு((வருவாய் - 800000) * வரி_வீதம், 2);
}
```

## Status

Parses **69 of 69** real eTamil programs in the compiler repository — all 41
standard library modules and all 28 examples — with no errors, plus 24 corpus
tests pinning individual constructs.

```bash
npm install
npm test              # corpus tests
npm run parse:corpus  # every real program in the repository
```

## The vocabulary is generated

`keywords.js` holds every spelling the compiler accepts for each keyword —
Tamil script, the romanized form, and where one exists an English alias — and
is generated from the compiler's own token table:

```bash
python ../scripts/generate_editor_support.py
```

So the grammar cannot accept a spelling the compiler rejects, or miss one it
accepts. `queries/highlights.scm` is generated the same way, and CI fails if
either drifts.

`grammar.js` is **hand-written**, because the shape of a statement is not
recoverable from a list of tokens. Two shapes are worth knowing:

- **The condition comes before the keyword.** `(வருவாய் > 800000) எனில் {` —
  not `எனில் (…)`. Editor integrations that assume the C ordering silently
  never fire.
- **A name is stored exactly as written.** `{வரி: 1}` and `{vari: 1}` are
  different fields. Field names are data, not syntax, so `queries/` captures
  them as `@property` rather than as keywords.

One more, in the queries: the financial vocabulary — `வரவு`, `பற்று`, `தொகை` —
is deliberately **not** reserved. The parser accepts those as ordinary names,
so they are captured as `@variable.member`, not `@keyword`. Colouring them as
syntax would tell the reader the opposite of the truth.

## Ambiguity

`வரிசை[0]` begins an index assignment if a `=` follows and is an index
expression otherwise, and one token of lookahead cannot tell. The compiler's
recursive-descent parser decides by peeking; tree-sitter is GLR, so the grammar
declares the ambiguity in `conflicts` and lets the parser carry both readings
until the `=` settles it.

## Editor setup

**Neovim** (nvim-treesitter), until this is upstreamed:

```lua
require('nvim-treesitter.parsers').get_parser_configs().etamil = {
  install_info = {
    url = 'https://github.com/Maruff/tree-sitter-etamil',
    files = { 'src/parser.c' },
    branch = 'main',
  },
  filetype = 'etamil',
}
vim.filetype.add({ extension = { qmz = 'etamil', etamil = 'etamil' } })
```

Then `:TSInstall etamil`, and copy `queries/highlights.scm` to
`queries/etamil/highlights.scm` in your config.

## Licence

MIT — deliberately, and separately from the compiler's AGPL. Grammar consumers
(nvim-treesitter, Helix, Zed, GitHub Linguist) all expect a permissive licence,
and the same reasoning put the TextMate grammar in its own MIT repository.
