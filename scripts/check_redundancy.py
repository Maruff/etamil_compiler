#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (C) 2026 Mohammed Maruff (Esan Maruff) <esan@etamil.in>
"""Find nUlakam functions that compute the same thing twice.

`dups.py`-style name collisions are a different problem and already zero: two
functions cannot share a name. This is the quieter one — two *differently
named* functions with the same body, which is how a library ends up with four
ways to pro-rate an amount by days and no agreement about which to call.

## How a body is compared

Comments out, whitespace out, then parameters renamed to their position, so
that

    செயல் கழிவுக்குப்_பின்(தொகை, கழிவு) { ... அ - ஆ ... }
    செயல் விலக்கிய_ஆதாயம்(ஆதாயம், விலக்கு) { ... அ - ஆ ... }

compare equal despite naming their arguments after their own domain. Numeric
and string literals are kept: a different rate is a different function, and an
error message is part of what a function does.

**Record keys are kept literal**, which is the one rule that stops this being
noise. Nine constructors in nUlakam have the body `திரும்பு {அ: அ, ஆ: ஆ, இ: இ}`
and are not duplicates of each other — they build nine different records. Slot
the keys and they collide; keep them and they do not.

## What a finding means

Not always "delete one". Standard costing names six variances that are all
`rate × (actual − standard)`, and a cost accountant looks for
`பொருள்_வீத_வேறுபாடு` by name. The vocabulary is the point of the library. What
a finding means is that **the formula should exist once** and the six names
should call it — so that a correction lands in one place.

A group that is deliberate goes in `ALLOW` with the reason, which is also how
this file stays readable as a list of decisions rather than of exceptions.

    python3 scripts/check_redundancy.py            # report
    python3 scripts/check_redundancy.py --check    # exit 1 on anything new
"""
import re
import subprocess
import sys
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

WORD = r"[஀-௿A-Za-z_][஀-௿A-Za-z0-9_]*"
DEFINE = re.compile(r"^\s*(?:செயல்|ceyal|_fn)\s+(" + WORD + r")\s*\((.*?)\)\s*\{")
TOKEN = re.compile(WORD + r'|\d+(?:\.\d+)?%?|"[^"]*"|[^\s]')

# A body this short says nothing — `திரும்பு அ + ஆ;` is four tokens and two
# functions sharing it is a coincidence, not a decision.
FLOOR = 8

# A body that is one call and nothing else is the *fix*, not the finding. Six
# domain names delegating to one primitive is what "the formula exists once"
# looks like, and reporting those six as a duplicate group would mean the audit
# went red the moment anybody acted on it.
DELEGATION = re.compile(r"^திரும்பு \S+ \( (?:@\d+ , )*@\d+ \) ; \}$")

# Groups that are meant to be what they are. Keyed by the normalised body, so
# a group stops being allowed the moment anybody edits one of its members.
ALLOW: dict[str, str] = {
    "திரும்பு @0 * ( @1 - @2 ) ; }":
        "The standard-costing variances in celavu/niyamam.qmz. The shape is the "
        "same and the arguments are not: பொருள்_வீத_வேறுபாடு weights a price "
        "difference by actual quantity, பொருள்_பயன்பாட்டு_வேறுபாடு weights a "
        "quantity difference by standard price, and விற்பனை_வீத_வேறுபாடு "
        "deliberately reverses the operands because selling above standard is "
        "favourable. Collapsing six onto one `a * (b - c)` would need a name that "
        "is not a domain concept, and would hide the two things the module's "
        "comments work hardest to make visible: which quantity weights which "
        "difference, and where the sign turns over.",

    "( @1 < = 0 ) எனில் { திரும்பு 0 ; } திரும்பு வட்டப்_பங்கு ( @0 , @1 ) ; }":
        "மேல்வரி_தொகை and தேக்கத்_தொகை already share the formula — வட்டப்_பங்கு "
        "is the shared part. What is left is a two-line guard, and folding that "
        "into வட்டப்_பங்கு would change what its other three callers do with a "
        "negative rate. 'No rate, no charge' is these two callers' rule, not the "
        "primitive's.",

    "திரும்பு நீளம் ( @0 ) = = 0 ; }":
        "காலியா asks it of an array and காலியா_பதிவேடு of a record. The bodies "
        "match because நீளம் answers both, and the two names exist so that "
        "aNi.qmz and poruL.qmz each answer the obvious question about their own "
        "kind. Making one call the other would make the record module depend on "
        "the array module to ask whether a record is empty.",
}


def functions():
    """Every nUlakam function outside the test suites."""
    listed = subprocess.run(
        ["git", "ls-files", "nUlakam/*.qmz"],
        capture_output=True, text=True, encoding="utf-8", cwd=ROOT,
    ).stdout.split("\n")
    for name in listed:
        if not name or name.endswith("_cOqaZY.qmz"):
            continue
        lines = (ROOT / name).read_text(encoding="utf-8").split("\n")
        for index, line in enumerate(lines):
            define = DEFINE.match(line)
            if not define:
                continue
            # Brace-balanced from here, with strings and comments blanked so a
            # `{` inside either does not move the count.
            depth, body, cursor = 0, [], index
            while cursor < len(lines):
                bare = re.sub(r"//.*$", "", re.sub(r'"[^"]*"', '""', lines[cursor]))
                depth += bare.count("{") - bare.count("}")
                body.append(lines[cursor])
                if cursor > index and depth <= 0:
                    break
                if cursor == index and depth == 0:
                    break
                cursor += 1
            params = [
                p.strip().split()[-1] for p in define.group(2).split(",") if p.strip()
            ]
            yield {
                "name": define.group(1),
                "module": name.replace("\\", "/"),
                "line": index + 1,
                "params": params,
                "body": "\n".join(body[1:]),
            }


def normalise(function) -> list[str]:
    """The body, with parameter names replaced by their position."""
    text = re.sub(r"//.*$", "", function["body"], flags=re.M)
    slot = {name: "@%d" % i for i, name in enumerate(function["params"])}
    tokens = TOKEN.findall(text)
    out = []
    for index, token in enumerate(tokens):
        # A record key is data the author chose, not a name that happens to be
        # in scope: `{விலை: விலை}` is a field called விலை. Keep it.
        is_key = index + 1 < len(tokens) and tokens[index + 1] == ":"
        out.append(token if is_key else slot.get(token, token))
    return out


def groups():
    same = defaultdict(list)
    for function in functions():
        tokens = normalise(function)
        body = " ".join(tokens)
        if len(tokens) >= FLOOR and not DELEGATION.match(body):
            same[body].append(function)
    return {body: members for body, members in same.items() if len(members) > 1}


def main() -> int:
    found = groups()
    new = {body: members for body, members in found.items() if body not in ALLOW}
    total = sum(len(members) for members in new.values())

    print(
        f"{len(found)} groups of functions share a body "
        f"({len(found) - len(new)} recorded as deliberate)\n"
    )
    for body, members in sorted(new.items(), key=lambda kv: (-len(kv[1]), kv[0])):
        print(f"{len(members)} functions, {len(body.split())} tokens:")
        for function in sorted(members, key=lambda f: (f["module"], f["line"])):
            print(f"    {function['name']:<36} {function['module']}:{function['line']}")
        print(f"    {body[:120]}\n")
    return total


if __name__ == "__main__":
    count = main()
    if sys.argv[1:2] == ["--check"]:
        sys.exit(0 if count == 0 else 1)
