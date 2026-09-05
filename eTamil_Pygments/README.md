# pygments-etamil

A [Pygments](https://pygments.org) lexer for eTamil, a programming language
whose keywords are Tamil words.

Installing it gives you highlighted eTamil in everything built on Pygments —
Sphinx, MkDocs, Jekyll, Read the Docs, `pygmentize`, and most static site
generators — without waiting on a Pygments release.

```sh
pip install pygments-etamil
pygmentize -l etamil examples/finance/kaNakkiyal.qmz
```

Files ending `.qmz` are detected automatically; in Markdown or reST, tag a
fenced block `etamil`.

## Why the licence differs from the compiler

Pygments is BSD-licensed and will not take copyleft code, so this package is
BSD-2-Clause while the compiler stays AGPL-3.0-or-later. Both licences are
granted by the same copyright holder. The same reasoning applies to the
TextMate grammar in `Maruff/etamil-tmlanguage`.

## The word lists are generated

`_etamil_builtins.py` is emitted by `scripts/generate_editor_support.py` in the
compiler repository, read out of the compiler's own lexer. Do not edit it by
hand — regenerate:

```sh
python scripts/generate_editor_support.py
```

`etamil.py` is hand-written and holds every decision about how a word is
coloured.

## A note on word boundaries

`\b` is useless here. `எனில்` ends in U+0BCD, category `Mn`, which `\w` does not
match, so `\bஎனில்\b` never fires in Python's `re`. The lexer anchors on the
identifier class the compiler accepts instead. TextMate's Oniguruma engine does
not have this problem, which is why the grammar and this lexer spell boundaries
differently.

## Tests

```sh
python -m unittest discover -s tests
```

The cases mirror `eTamil_Code/test/grammar.test.js`, so the TextMate grammar and
this lexer cannot disagree about what a word is without a test failing. The
suite also lexes every program under `examples/` and fails on any `Error` token.
