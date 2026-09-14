# The eTamil fonts

Two faces ship, and they are not alternatives to each other.

`ican_qamiz-Regular-2.1.1.ttf` — `ican qamiz`, the font in which the ASCII
letters carry Tamil glyphs. `c` draws ச, `q` draws த, `Z` draws ன: the ezuqqu
mapping, one glyph per letter, so that Tamil can be typed on an ASCII keyboard
and read back as Tamil.

`ican_qamiz_Smart-Regular-2.1.0.ttf` — `ican qamiz Smart`, the same mapping with
the Tamil block added and three contextual rules in OpenType, so that one face
sets both a program and the prose about it.

`ican_qamiz-Regular.ttf`, version 2.1.0, is kept beside them. It and the 2.1.1
plain face differ in the `name` table and nowhere else — every outline, the cmap
and all sixteen other tables are byte-identical. `src/bundle.ts` holds the
`FACES` table naming which files ship, and `test/bundle.test.js` reads those
names rather than repeating them, so only the named files are installed on a
machine.

**eTamil: Install the eTamil font** copies both to the per-user font directory
of the machine — `~/Library/Fonts`, `~/.local/share/fonts`, or
`%LOCALAPPDATA%\Microsoft\Windows\Fonts` with the matching `HKCU` registration —
and offers to set `etamil.eTamilFont` to `ican qamiz Smart`. No administrator
rights, and nothing is installed without being asked for.

VS Code cannot load a font out of an extension. The editor is not a webview and
there is no API that registers one, so the font has to reach the operating
system's own font list before any `font-family` naming it resolves. That is why
there is an install command at all, and why VS Code has to be restarted after
it runs.

## What is in the files

Read from the fonts' own tables, not from anywhere else:

| | ican qamiz | ican qamiz Smart |
|---|---|---|
| file | `ican_qamiz-Regular-2.1.1.ttf` | `ican_qamiz_Smart-Regular-2.1.0.ttf` |
| version | 2.1.1 | 2.1.0 |
| glyphs | 132 | 239 |
| code points mapped | 129 | 201 |
| printable ASCII | 95 of 95 | 95 of 95 |
| Latin-1 supplement | 32 | 32 |
| Tamil block | **0** | **72** |
| GSUB features | none applied | `calt`, `rlig` |
| units per em | 1000 | 1000 |
| `isFixedPitch` | 0 — **proportional** | 0 — **proportional** |

Both: copyright `Copyright (c) 2026, eTamil.in.`, licensed SIL Open Font License
1.1 (`OFL.txt` beside this file), designer Esan Maruff, manufacturer eTamil
India, eTamil.in.

## Counting the Tamil block: resolve the glyph, do not trust the span

The Tamil figure above is the number of code points in U+0B80–U+0BFF that
resolve to a **non-zero glyph id**. It is not the number of code points the cmap
segments span, and the two differ.

`ican qamiz Smart` has been reported as covering 87 Tamil characters. It does
not. A format 4 cmap subtable stores ranges, and a range may contain code points
whose `idRangeOffset` lookup yields glyph 0 — mapped by the segment, drawn by
nothing. Fifteen do here:

    U+0B91  U+0B96  U+0B97  U+0B98  U+0B9B  U+0B9D  U+0BA0  U+0BA1
    U+0BA2  U+0BA5  U+0BA6  U+0BA7  U+0BC9  U+0BCE  U+0BCF

Count segment spans and you get 87; resolve each code point and you get 72. The
cross-check that settles it: 201 mapped code points minus 72 Tamil leaves 129,
which is exactly what the plain face maps. Any tool reading these files should
follow `idDelta` and `idRangeOffset` through to the glyph id.

## The plain face has no Tamil, and that is deliberate

Three cmap subtables — (0,3) format 4, (1,0) format 0, (3,1) format 4 — were
read in full for `ican qamiz`. The two Unicode subtables map 129 code points
each, the same 129 both times, all of them ASCII and Latin-1; the Mac subtable
maps 123 byte values, which are MacRoman rather than Unicode and so cannot be
added to that 129. Not one character of U+0B80–U+0BFF has a glyph.

That is the design and not an export accident. The face exists to give the ASCII
letters Tamil shapes, and Unicode Tamil is drawn by whatever face the editor is
already using. The Smart face is where the Tamil block was wanted, and it has
it.

## Which way round the extension paints, and why the Smart face does not change it

`etamil.eTamilFont` is applied to the ASCII that is eTamil script and to nothing
else. Unicode Tamil is never painted and keeps whatever Tamil face the editor's
own font stack provides.

The Smart face covering the Tamil block does **not** make the other direction
viable. The reason for the direction is English, not Tamil: an English word
drawn in either face is unreadable — `sum` draws as ஸும் — and nothing in a file
says which ASCII letters were English. Paint the whole editor in an eTamil face
and every English run in every file becomes unreadable, whether or not that face
has Tamil in it. See `docs/reference/SCRIPT_RULES.md`.

## The Smart face's three rules are the font's, not the language's

1. A consonant that no vowel follows takes the pulli.
2. A vowel that no consonant precedes is drawn at full size upon the baseline,
   in its own advance, rather than as the attached sign.
3. The inherent `a` after a consonant draws nothing, so `k` reads as க.

Under these, `vaNakkam` sets as வணக்கம் while the eight bytes on disk are
unchanged.

**The compiler does not accept them** in keywords, function names or variables.
It requires the canonical form, in which every vowel is written, because that is
what makes the encoding reversible: if `k` could mean either the bare consonant
or the consonant with `a`, the map would be one-to-many. The rules are for a
person setting Tamil in a document, not for source.

They are implemented as contextual substitutions under `calt`, with `rlig` as a
fallback. Every encoded glyph keeps the outline the plain face gives it — the
alternative forms live in unencoded glyphs that only the substitutions reach —
so a renderer that does not apply `calt` shows exactly the plain rendering.
`src/bundle.ts` carries `FONT_FEATURES`, and the extension injects
`font-feature-settings` into the decoration CSS for eTamil spans alone, so
turning the rules on does not touch `editor.fontLigatures` or anything outside
those spans.

## Both are proportional

A run painted in either does not occupy the same width as the monospace grid
underneath, so text after a painted run on the same line shifts. A monospaced
build of the face would make that exact, and nothing in the extension would have
to change.

## Checking a replacement

`test/bundle.test.js` reads these files and asserts that the families they
declare are the families `src/bundle.ts` names, that a font declaring the OFL
has `OFL.txt` beside it, and that `calt` is present in the Smart face and absent
from the plain one. All three failures are otherwise silent: a family name wrong
by one character resolves to nothing, the editor draws in its own font, and the
feature appears simply not to work.
