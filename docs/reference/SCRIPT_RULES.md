# Script rules — telling eTamil ASCII from English ASCII

**Status**: normative. Enforced by `scripts/check_script_rules.py --check`.

---

## The problem

eTamil is written in three ways, and two of them use the same bytes.

| | example | what it is |
|---|---|---|
| Tamil | `செயல்`, `மொத்தம்` | Unicode Tamil, U+0B80–U+0BFF |
| eTamil | `ceyal`, `moqqam` | Tamil spelled in ASCII under the ezuqqu scheme |
| English | `_length`, `_today` | English words, in ASCII |

The first is unambiguous: a Tamil codepoint is Tamil wherever it appears. The
other two are the same character range. `col` is `சொல்`. `sum` is not a Tamil
word at all. Nothing in the bytes says which was meant, and a reader who does
not already know the scheme cannot tell.

This is not only a reading problem. There is an **eTamil font**, optional, in
which the ASCII letters carry Tamil glyphs — `c` draws ச, `q` draws த, `Z`
draws ன, one glyph per letter, exactly the ezuqqu mapping. A developer who
turns it on sees `ceyal` as செயல் and writes Tamil on an ASCII keyboard.

Such a font need not have any Tamil of its own. Of the two shipped with the VS
Code extension, `ican qamiz` has none — it maps 129 codepoints, all of them
ASCII and Latin-1, and not one character of U+0B80–U+0BFF — while `ican qamiz
Smart` maps 72 characters of that block. Either way the eTamil face is only ever
**part** of the font a file is drawn in, over a base that carries the rest.

An English word under that font is unreadable. `sum` draws as ஸும். `status`
draws as ஸ்டடுஸ். The font has no way to know that those particular ASCII
letters were English, because nothing in the file says so.

So the file has to say so.

## Rule 1 — an English identifier carries a leading `_`

> Any keyword or variable written in English takes `_` as its first character.

```etamil
மொத்தம் = 0;           // Tamil
moqqam  = 0;           // eTamil — the same name, ASCII
_sum    = 0;           // English
```

The leading underscore is a mark on the identifier, not part of the word. It
says: *the ASCII in this name is English, read it as English.*

The keyword table already works this way. Every keyword has up to three
spellings, and the English one always begins with `_`:

```
நீளம்  |  nILam  |  _length
இன்று   |  iZRu   |  _today
```

Rule 1 extends the same convention from the keywords to the names you write.

**Mixed names take the underscore too, at the front.** `cgst` and `XLSX` and
the `Z` of a Z-score are English no matter what surrounds them, so the whole
identifier is marked once at the start rather than at each boundary:

```etamil
_மொத்த_cgst        // not மொத்த_cgst
_z_மதிப்பு           // not z_மதிப்பு — bare `z` is ழ
_XLSX_வடிவம்        // not XLSX_வடிவம்
```

A single leading mark is used rather than one per segment because `_` doubled
mid-name would collide with Rule 2's `__`, and because the Tamil parts of a
mixed name need no mark: Unicode Tamil draws as Tamil under either font.

**Marked names share a namespace with the English keywords**, which is the
point of the mark and also its one cost. `_port` is how you write துறை in
English, so `_port` is reserved and a variable cannot be called that — exactly
as a variable cannot be called `வரி`. The keyword list is in
[KEYWORDS.md](KEYWORDS.md); a clash reads as `expected a statement, found
'_port'` and the fix is a longer name, `_server_port`.

The ones most likely to catch you are the type names, which eTamil reserves
where most languages do not: **`_int`, `_float`, `_string`, `_text`, `_bool`,
`_array`, `_data`, `_object` and `_date`** are all spellings of a type, so
`_date = ...` is a parse error while `_date _when = ...` is a typed
declaration. `_if`, `_else`, `_loop`, `_true`, `_false`, `_null`, `_print` and
`_input` are reserved too, and surprise nobody.

## Rule 2 — an English comment is wrapped in `__`

> Any comment written in English starts and ends with `__`.

```etamil
// __Rounded to paise once, at the end. Rounding each share as it is
// computed adds money that was never there.__
```

eTamil has one comment form, `//` to the end of the line. **The marks go at
the ends of the sentence, not of each line.** The opening `__` starts an
English region and the next `__` ends it, however many comment lines lie
between them — which is what the example above does, and what an author writing
a sentence does without thinking about it. A sentence that fits on one line
opens and closes on that line; nothing about the rule changes.

The region ends at the first line that is not a comment, so one unclosed `__`
cannot make the remainder of a file English. Two marked runs on one line are
still two: `// kaZakku __two words__ mudivu` marks only the middle.

A comment with no ASCII letters needs no marks — a rule separator, a Tamil
sentence, a line of Tamil identifiers:

```etamil
// -----------------------------------------------------------------
// வட்டி ஆண்டுக்கு ஒரு முறை கூட்டப்படுகிறது.
```

**Unicode Tamil is legal inside `__ … __`, and is the normal case.** English
prose in this codebase names Tamil functions constantly. Wrapping the whole
line is correct: the marks switch the ASCII to English, and the Tamil in the
middle is unaffected, because Unicode Tamil draws as Tamil in both fonts.

```etamil
// __`விளிம்பு_நிவாரணம்` runs the whole ladder twice — there is no closed
// form for it.__
```

This is the sense in which ISO Tamil belongs in both places: in a bare `//`
comment and inside `__ … __`. The marks say nothing about the Tamil. They say
only what to do with the ASCII.

## What the marks mean to an editor

An editor keeps the **standard ISO font** as the base and draws in the eTamil
font only:

- ASCII identifiers with no leading `_`, and not reached through `.`
- ASCII in comments outside `__ … __`, and not preceded by `.`

Everything else stays in the base font:

- any identifier beginning with `_`, including keywords
- any name immediately preceded by `.` — a field name, an extension
- any comment text between `__` and `__`
- the licence header lines (below)
- string literals, always — see below
- **Unicode Tamil**, which the eTamil font may not have at all

Two entries in that list are not rules and add nothing to a file. A string
literal is data, and so is a name reached through `.` — a field name, or the
extension in `paNam.qmz`, which is English whatever the words around it are
spelled in. Neither carries a mark, neither is required to, and the checker
asks nothing of them. They are here because an editor still has to decide what
to draw them in, and the answer is the ISO font, in either direction of the
`etamil.eTamilFont` switch.

The Unicode Tamil entry decides the direction, and it is not a detail. The eTamil face is
painted over the ASCII that wants it, never the other way round. The reason is
English: a word like `sum` drawn in either eTamil face is unreadable, and
nothing outside the two marks says which ASCII letters were English, so painting
the whole editor in an eTamil face would make every English run in every file
unreadable. That `ican qamiz Smart` covers the Tamil block does not change it —
the difficulty is Latin text, not Tamil. Painting the other way would also leave
every Tamil letter in a file that used the plain face falling back to whatever
the machine offered.

An editor using only standard fonts renders everything in them and ignores the
marks, which then do nothing but tell the reader the same thing. Nothing about
the rules depends on which font is in use; the marks are in the file either
way. That is deliberate — a file must not read differently on a machine that
does not have the eTamil font installed.

### What the extension does

Set `etamil.eTamilFont` to the eTamil font's family name and the VS Code
extension renders exactly the division above, from version 1.0.0. The
extension carries two such fonts, `ican qamiz` and `ican qamiz Smart`, and
**eTamil: Install the eTamil font** puts both where the operating system can see
them.

The grammar names both regions, which is what any highlighter needs to act on
them, and a theme can colour them:

    meta.english.comment.etamil      the text between `__` and `__`
    variable.other.english.etamil    a name marked by Rule 1

A theme cannot change the *font*, though — VS Code's token colours carry
`fontStyle`, which is bold, italic and underline, and nothing else. So the
extension does it with a decoration, whose `textDecoration` is injected as CSS
and can carry a font family behind it. That is unofficial, and it is the only
lever there is.

Which way round it paints follows from how informal that lever is.
`eTamil_Code/src/marks.ts` finds the ASCII that is **eTamil script** and paints
only that, leaving the editor's own ISO font everywhere else. A span it fails to
recognise therefore renders eTamil as plain Latin — the ordinary view of the
file — instead of rendering English in Tamil glyphs, which would be unreadable.
Given a mechanism this informal, the failure has to fall on the harmless side.

## What the rules do not cover

**String literals.** A string is data. `"Total"` is what the program prints,
and adding a mark to it would change the output, so no mark is added and the
rules stay out. Editors render string literals in the standard ISO font
unconditionally.

**File and directory names.** `basic_samples` and `fabric` are English; `kAcu`
and `nErativari` are ezuqqu. Filenames are not eTamil source and carry no
marks. That surface is audited separately by `scripts/check_names.py`, whose
`ALLOW` list names the English ones.

**The licence header.** The two lines

```etamil
// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Mohammed Maruff (Esan Maruff) <esan@etamil.in>
```

are read by licence scanners, which expect the SPDX expression to run to the
end of the line. A trailing `__` becomes part of it and breaks the parse. Both
lines are exempt from Rule 2, and editors render them in the ISO font by
recognising them.

## Checking

```bash
python3 scripts/check_script_rules.py --check
```

It reads every `.qmz` file and reports, for each violation, the file, the line,
and which rule. It gates CI alongside `transliterate.py --check` and
`check_names.py --check`.

Rule 2 is checked mechanically and exactly: a comment containing a Latin letter
either is wrapped or is a violation. Rule 1 cannot be checked that exactly,
because an ASCII name spelled only in scheme letters is valid eTamil whether or
not eTamil was meant — the same limit `check_names.py` documents. What the
checker can do, and does, is catch the names that are English on their face:
they contain a letter the ezuqqu scheme does not assign (`b`, `d`, `f`, `g`,
`D`, and the rest), so they cannot be eTamil, and they must be marked.

---

See also: [Tamil Letter Equivalents](COMPILER_TAMIL_LETTER_EQUIVALENTS.md) for
the ezuqqu scheme itself, and [Keywords](KEYWORDS.md) for all three spellings of
every keyword.
