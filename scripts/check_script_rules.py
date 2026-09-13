#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (C) 2026 Mohammed Maruff (Esan Maruff) <esan@etamil.in>
"""Audit the `_` and `__` marks that say which ASCII is English.

eTamil is written three ways and two of them are ASCII: `ceyal` is செயல்
spelled under the ezuqqu scheme, `_length` is the English word. Nothing in the
bytes separates them, and there is an eTamil font that draws ASCII as Tamil —
`c` as ச, `q` as த — under which an unmarked English name renders as nonsense.
`sum` draws as ஸும்.

So the file says which is which. The rules are in
`docs/reference/SCRIPT_RULES.md`:

    Rule 1  an identifier containing English ASCII begins with `_`
    Rule 2  a comment containing English ASCII is wrapped in `__ … __`

## What is checked, and how well

**Rule 2 is checked exactly.** A comment is a line, the marks go at both ends
of it, and a Latin letter outside the marks is a violation with no judgement
involved. The licence header is exempt — licence scanners read the SPDX
expression to the end of the line, and a trailing `__` becomes part of it.

**Rule 1 cannot be checked exactly**, for the reason `check_names.py` sets out
at length: a name spelled only in scheme letters produces Tamil whether or not
Tamil was meant. `sum` reverses to ஸும், which is on-scheme and is not a word.
Two things are done instead, and between them they catch what has actually gone
wrong here:

  - **Latin inside a Tamil name.** Unicode Tamil and ezuqqu are two spellings
    of one thing and nobody mixes them within a name, so the Latin in
    `z_மதிப்பு` or `XLSX_வடிவம்` is English by construction.

  - **Off-scheme letters.** `b`, `d`, `f`, `g`, `D` and the rest are assigned
    nothing, so a name containing one cannot be eTamil. `cgst`, `XLSX` and
    `pdf` are caught this way, mechanically and with no list.

  - **A list of common English words.** `sum`, `status`, `port`, `counter` —
    every letter in the scheme, every one of them English. There is no way to
    derive this, so it is written down. The list is short on purpose: it holds
    the words that turn up in code, not a dictionary.

A name that is English, is spelled only in scheme letters, and is not in the
list goes through unmarked. That is a known hole and the reason the rule is
also a rule for people, not only for this file.

Only binding positions are examined — a `செயல்` name and its parameters, an
assignment target, a loop variable. Record keys and SQL columns are data rather
than identifiers; `check_names.py` audits those.

Usage:

    python3 scripts/check_script_rules.py            # report
    python3 scripts/check_script_rules.py --check    # CI: exit 1 on any finding
"""
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from transliterate import untransliterate  # noqa: E402

ROOT = Path(__file__).resolve().parent.parent
SCANNED = ("nUlakam", "examples", "android", "scripts")

TAMIL = re.compile(r"[஀-௿]")
LATIN = re.compile(r"[A-Za-z]")

WORD = r"[஀-௿A-Za-z_][஀-௿A-Za-z0-9_]*"
DEFINE = re.compile(r"செயல்\s+(" + WORD + r")\s*\(([^)]*)\)")
ASSIGN = re.compile(r"^\s*(" + WORD + r")\s*=(?!=)")
FOREACH = re.compile(r"ஒவ்வொரு\s+(" + WORD + r")\s+இல்")

# Lines a licence scanner reads. The SPDX expression runs to the end of the
# line by definition, so a closing `__` would be parsed as part of the licence
# name. Exempt, and named here rather than guessed at by position.
HEADER = re.compile(r"^\s*(SPDX-[A-Za-z-]+:|Copyright\s*\(C\))")

# English words whose every letter happens to be in the ezuqqu scheme, so
# nothing mechanical can tell them from Tamil. Written down because they cannot
# be derived. Add a word when you meet one; do not add a Tamil name.
ENGLISH = {
    "active", "amount", "api", "app", "arg", "args", "array", "auth",
    "cache", "cart", "case", "cell", "chart", "check", "class", "client",
    "code", "col", "column", "config", "content", "count", "counter", "cpu",
    "csv", "currency", "current", "cursor", "customer", "cycle", "data",
    "date", "day", "delta", "disk", "duration", "email", "entry", "error",
    "event", "expense", "file", "filter", "first", "flag", "format", "hour",
    "html", "http", "income", "index", "input", "inactive", "interest",
    "item", "items", "json", "key", "label", "last", "layer", "length",
    "level", "limit", "line", "list", "log", "loss", "main", "map", "mask",
    "master", "match", "max", "memory", "message", "meta", "method", "min",
    "minute", "mode", "module", "month", "name", "net", "next", "node",
    "note", "null", "number", "offset", "order", "output", "page", "param",
    "parent", "path", "pattern", "payment", "period", "phase", "plan",
    "port", "premium", "price", "printer", "profit", "query", "queue",
    "quote", "range", "rate", "ratio", "record", "result", "return",
    "revenue", "root", "route", "row", "rule", "salary", "sale", "sales",
    "score", "second", "sector", "server", "service", "session", "set",
    "size", "sort", "source", "start", "state", "status", "step", "stock",
    "store", "string", "style", "sum", "summary", "table", "tag", "tax",
    "temp", "test", "text", "time", "title", "token", "total", "type",
    "unit", "uptime", "url", "user", "users", "value", "version", "view",
    "week", "year", "zone",
}


def strip_strings(line: str) -> tuple[str, str | None]:
    """Return the line with string contents blanked, and any comment body.

    `//` inside a string is not a comment — `"https://etamil.in"` would
    otherwise be read as one and its own contents audited as prose. Walking the
    line once is enough: eTamil has only double-quoted strings, and only `\\"`
    escapes a quote inside one.
    """
    out, in_string, i, comment = [], False, 0, None
    while i < len(line):
        ch = line[i]
        if in_string:
            out.append(" ")
            if ch == "\\" and i + 1 < len(line):
                out.append(" ")
                i += 2
                continue
            if ch == '"':
                out[-1] = '"'
                in_string = False
        elif ch == '"':
            out.append(ch)
            in_string = True
        elif ch == "/" and line[i : i + 2] == "//":
            comment = line[i + 2 :]
            break
        else:
            out.append(ch)
        i += 1
    return "".join(out), comment


def english(name: str) -> str | None:
    """Why this name is English, or None if nothing here can tell."""
    if TAMIL.search(name):
        # Unicode Tamil and ezuqqu are two spellings of the same thing, and
        # nobody switches between them inside one name. So Latin sitting in a
        # Tamil name is English — `z_மதிப்பு` is a z-score, not ழ், and
        # `நிகழ்தகவு_Z` is not நிகழ்தகவு_ன். No list needed for these.
        return "Latin in a Tamil name is English"
    back = untransliterate(name)
    if LATIN.search(back):
        stray = "".join(sorted({c for c in back if c.isascii() and c.isalpha()}))
        return f"{stray} not in the ezuqqu scheme"
    for part in re.split(r"[_0-9]+", name.lower()):
        if part in ENGLISH:
            return f"'{part}' is an English word"
    return None


def bindings(code: str):
    """Every name this line binds."""
    for m in DEFINE.finditer(code):
        yield m.group(1)
        for param in m.group(2).split(","):
            if param.strip():
                yield param.strip()
    m = ASSIGN.match(code)
    if m:
        yield m.group(1)
    for m in FOREACH.finditer(code):
        yield m.group(1)


def check() -> list[tuple[str, int, str, str]]:
    findings = []
    for top in SCANNED:
        for path in sorted((ROOT / top).rglob("*.qmz"), key=lambda p: p.as_posix()):
            rel = path.relative_to(ROOT).as_posix()
            for number, line in enumerate(
                path.read_text(encoding="utf-8").split("\n"), 1
            ):
                code, comment = strip_strings(line)

                if comment is not None:
                    body = comment.strip()
                    if LATIN.search(body) and not HEADER.match(body):
                        if not (
                            body.startswith("__")
                            and body.endswith("__")
                            and len(body) > 4
                        ):
                            findings.append(
                                (rel, number, "rule 2", f"// {body[:60]}")
                            )

                for name in bindings(code):
                    if name.startswith("_") or not LATIN.search(name):
                        continue
                    why = english(name)
                    if why:
                        findings.append((rel, number, "rule 1", f"{name} — {why}"))
    return findings


def main() -> int:
    findings = check()
    rule1 = sum(1 for f in findings if f[2] == "rule 1")
    rule2 = len(findings) - rule1
    print(
        f"checked {len(SCANNED)} trees: "
        f"{rule1} unmarked English names, {rule2} unmarked English comments\n"
    )
    for rel, number, rule, detail in findings:
        print(f"{rel}:{number}  {rule}  {detail}")
    return len(findings)


if __name__ == "__main__":
    if sys.argv[1:2] == ["--check"]:
        sys.exit(0 if main() == 0 else 1)
    main()
