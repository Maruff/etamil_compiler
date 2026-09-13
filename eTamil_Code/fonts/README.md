# The eTamil font

`ican_qamiz-Regular.ttf` — `ican qamiz`, the font in which the ASCII letters
carry Tamil glyphs. `c` draws ச, `q` draws த, `Z` draws ன: the ezuqqu mapping,
one glyph per letter, so that Tamil can be typed on an ASCII keyboard and read
back as Tamil.

It ships inside the VSIX. **eTamil: Install the eTamil font** copies it to the
per-user font directory of the machine — `~/Library/Fonts`, `~/.local/share/
fonts`, or `%LOCALAPPDATA%\Microsoft\Windows\Fonts` with the matching
`HKCU` registration — and offers to set `etamil.eTamilFont` to its family name.
No administrator rights, and nothing is installed without being asked for.

VS Code cannot load a font out of an extension. The editor is not a webview and
there is no API that registers one, so the font has to reach the operating
system's own font list before any `font-family` naming it resolves. That is why
there is an install command at all, and why VS Code has to be restarted after
it runs.

## What is in the file

Read from the font's own tables, not from anywhere else:

| | |
|---|---|
| family | `ican qamiz` |
| subfamily | Regular |
| version | 2.1.0 |
| units per em | 1000 |
| glyphs | 132 |
| `isFixedPitch` | 0 — **proportional, not monospaced** |
| copyright | Copyright (c) 2026, eTamil.in. All rights reserved. |
| licence | SIL Open Font License 1.1 — `OFL.txt` beside this file |
| designer | Esan Maruff |
| manufacturer | eTamil India, eTamil.in |

## The Tamil block is empty, and that is worth knowing

Three cmap subtables — (0,3) format 4, (1,0) format 0, (3,1) format 4 — were
read in full. Between them they map 143 codepoints, all of them ASCII and
Latin-1. **Not one character of U+0B80–U+0BFF has a glyph.**

The FontForge source has them. `ican_qmz.sfd` holds 201 glyphs, of which some
seventy are `uniXXXX.glyph` files encoded at Tamil codepoints — உ at 2953, ஊ at
2954, and so on, in glyph order 129 upwards. They are in the design and they
are not in this export. Whoever regenerates the `.ttf` should check the export
selection; nothing in this repository can add them.

It is not blocking, because of which way round the extension paints.
`etamil.eTamilFont` is applied to the ASCII that is eTamil script and to
nothing else, so Unicode Tamil is never painted and keeps whatever Tamil face
the editor's own font stack provides. Had the extension worked the other way —
the whole editor in `ican qamiz`, with the English regions painted back to ISO
— every Tamil letter in every file would have fallen back to an arbitrary face.
See `docs/reference/SCRIPT_RULES.md`.

## It is proportional

A run painted in it does not occupy the same width as the monospace grid
underneath, so text after a painted run on the same line shifts. A monospaced
build of the same face would make that exact, and nothing in the extension
would have to change.

## Checking a replacement

`test/bundle.test.js` reads this file and asserts that the family it declares
is the family `src/bundle.ts` names, and that a font declaring the OFL has
`OFL.txt` beside it. Both failures are otherwise silent: a family name wrong by
one character resolves to nothing, the editor draws in its own font, and the
feature appears simply not to work.
