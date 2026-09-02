#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (C) 2026 Mohammed Maruff (Esan Maruff) <esan@etamil.in>
"""Tamil -> ezuqqu romanization, and a checker for the lexer's keyword table.

The ezuqqu scheme assigns one Latin letter per Tamil letter, using case to
separate letters English collapses together:

    ண N    ந n    ன Z          ள L    ல l          ற R    ர r
    ழ z    ங w    ஞ W          த q    ட t          ச c

Usage:
    python scripts/transliterate.py <tamil-word> [...]   # transliterate
    python scripts/transliterate.py --check              # audit lexer.rs
"""
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
LEXER = ROOT / "etamil_compiler" / "src" / "lexer.rs"

# Independent vowels
VOWELS = {
    "அ": "a", "ஆ": "A", "இ": "i", "ஈ": "I", "உ": "u", "ஊ": "U",
    "எ": "e", "ஏ": "E", "ஐ": "Y", "ஒ": "o", "ஓ": "O", "ஔ": "V",
}

# Dependent vowel signs (matras)
SIGNS = {
    "ா": "A", "ி": "i", "ீ": "I", "ு": "u", "ூ": "U",
    "ெ": "e", "ே": "E", "ை": "Y", "ொ": "o", "ோ": "O",
    "ௌ": "V",
}

CONSONANTS = {
    "க": "k", "ங": "w", "ச": "c", "ஞ": "W", "ட": "t", "ண": "N",
    "த": "q", "ந": "n", "ப": "p", "ம": "m", "ய": "y", "ர": "r",
    "ல": "l", "வ": "v", "ழ": "z", "ள": "L", "ற": "R", "ன": "Z",
    # Grantha letters for borrowed sounds
    "ஜ": "j", "ஷ": "S", "ஸ": "s", "ஹ": "H",
}

PULLI = "்"   # ் — strips the inherent vowel
AYTHAM = "ஃ"


def transliterate(word: str) -> str:
    out = []
    i = 0
    while i < len(word):
        ch = word[i]

        # க்ஷ is a single letter in the scheme
        if word.startswith("க" + PULLI + "ஷ", i):
            i += 3
            if i < len(word) and word[i] == PULLI:
                out.append("x")
                i += 1
            elif i < len(word) and word[i] in SIGNS:
                out.append("x" + SIGNS[word[i]])
                i += 1
            else:
                out.append("xa")
            continue

        if ch in VOWELS:
            out.append(VOWELS[ch])
            i += 1
        elif ch == AYTHAM:
            out.append("h")
            i += 1
        elif ch in CONSONANTS:
            base = CONSONANTS[ch]
            i += 1
            if i < len(word) and word[i] == PULLI:
                out.append(base)          # bare consonant
                i += 1
            elif i < len(word) and word[i] in SIGNS:
                out.append(base + SIGNS[word[i]])
                i += 1
            else:
                out.append(base + "a")    # inherent vowel
        elif ch == "_":
            out.append("_")
            i += 1
        else:
            out.append(ch)                # digits, ASCII, punctuation
            i += 1
    return "".join(out)


# The scheme is one Latin letter per Tamil letter, so it inverts. The reverse
# tables are derived from the forward ones rather than written out again: a
# second copy of the mapping is a second thing to keep in step, which is the
# drift this file exists to catch.
_R_CONSONANTS = {v: k for k, v in CONSONANTS.items()} | {"x": "க" + PULLI + "ஷ"}
_R_VOWELS = {v: k for k, v in VOWELS.items()}
_R_SIGNS = {v: k for k, v in SIGNS.items()} | {"a": ""}


def untransliterate(word: str) -> str:
    """ezuqqu -> Tamil. The inverse of transliterate().

    A letter the scheme does not assign — `b`, `d`, `f`, `g`, `D` — has no
    Tamil to become and is passed through as itself. That is the point: run a
    romanized name back through this and a stray Latin letter sitting inside
    Tamil output is a letter that was never in the scheme.
    """
    out = []
    i = 0
    while i < len(word):
        ch = word[i]
        if ch in _R_CONSONANTS:
            base = _R_CONSONANTS[ch]
            if i + 1 < len(word) and word[i + 1] in _R_SIGNS:
                out.append(base + _R_SIGNS[word[i + 1]])
                i += 2
            else:
                out.append(base + PULLI)
                i += 1
        elif ch in _R_VOWELS:
            out.append(_R_VOWELS[ch])
            i += 1
        elif ch == "h":
            out.append(AYTHAM)
            i += 1
        else:
            out.append(ch)
            i += 1
    return "".join(out)


# Keywords that keep an off-scheme romanization, and why. An exception with a
# stated reason is a decision; a count of nineteen with no reasons was a chore
# nobody could evaluate.
#
# The scheme's spelling can be *added* to a keyword without breaking anything —
# the lexer accepts several, and this audit reads the second alternative as the
# canonical one. What it cannot do is take a spelling that working code already
# uses as a name.
EXCEPTIONS: dict[str, str] = {}


def check() -> int:
    """Compare every keyword's stored romanization against the scheme.

    Returns the number of *unexplained* mismatches, so a clean run means the
    lexer is on-scheme except where EXCEPTIONS says otherwise and gives a
    reason. That is what makes this safe to run as a gating CI step.
    """
    text = LEXER.read_text(encoding="utf-8")
    rows = re.findall(r'#\[regex\("([^"]+)"\)\]\s*(\w+)', text)
    mismatches = []
    explained = []
    for alts, token in rows:
        parts = alts.split("|")
        if len(parts) < 2:
            continue
        tamil, stored = parts[0], parts[1]
        expected = transliterate(tamil)
        if expected == stored:
            continue
        if tamil in EXCEPTIONS:
            explained.append((tamil, stored, expected, token))
        else:
            mismatches.append((tamil, stored, expected, token))

    print(
        f"checked {len(rows)} keywords, {len(mismatches)} off-scheme, "
        f"{len(explained)} off-scheme on purpose\n"
    )
    if mismatches:
        print(f"{'Tamil':<22} {'in lexer':<22} {'per scheme':<22} token")
        print("-" * 88)
        for tamil, stored, expected, token in mismatches:
            print(f"{tamil:<22} {stored:<22} {expected:<22} {token}")
        print()
    for tamil, stored, expected, token in explained:
        print(f"{tamil} keeps {stored} rather than {expected} ({token}):")
        print(f"    {EXCEPTIONS[tamil]}")
    return len(mismatches)


if __name__ == "__main__":
    args = sys.argv[1:]
    if not args:
        print(__doc__)
    elif args[0] == "--check":
        sys.exit(0 if check() == 0 else 1)
    else:
        for word in args:
            print(f"{word}\t{transliterate(word)}")
