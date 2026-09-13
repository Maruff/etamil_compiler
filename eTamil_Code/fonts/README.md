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
| version | 2.538 |
| units per em | 1000 |
| `isFixedPitch` | 0 — **proportional, not monospaced** |
| copyright | Copyright (c) 2026, eTamil.in. All rights reserved. |
| licence | SIL Open Font License 1.1 — `OFL.txt` beside this file |
| designers | Aadarsh Rajan, Girish Dalvi, Yashodeep Gholap |
| manufacturer | Ek Type, www.ektype.in |

## Two things the file says that are worth knowing

**It covers ASCII and nothing else.** 129 codepoints are mapped: the 52 Latin
letters, the digits, and punctuation. **The Tamil block is empty** — not one of
U+0B80–U+0BFF has a glyph.

That is not a defect and it does not need fixing, because of which way round
the extension paints. `etamil.eTamilFont` is applied to the ASCII that is
eTamil script and to nothing else; Unicode Tamil is never painted, so it keeps
whatever Tamil face the editor's own font stack provides. Had the extension
worked the other way — the whole editor in `ican qamiz`, with the English
regions painted back to ISO — every Tamil letter in every file would have
fallen back to some arbitrary face. See `docs/reference/SCRIPT_RULES.md`.

**It is proportional.** A run painted in it does not occupy the same width as
the monospace grid underneath, so text after a painted run on the same line
shifts. A monospaced build of the same face would make that exact, and nothing
in the extension would have to change.

## For whoever maintains this

The font is a derivative: the designers and the foundry named above are Ek
Type's, and the name table's copyright names only eTamil.in. OFL 1.1 asks a
derivative to carry the original copyright notices as well as the new one.
Worth checking against the upstream face before publishing, and correcting in
the font rather than here — `OFL.txt` deliberately repeats only what the font
itself declares.
