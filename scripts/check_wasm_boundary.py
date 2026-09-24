#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (C) 2026 Mohammed Maruff (Esan Maruff) <esan@etamil.in>
"""Audit what `lib.rs` and `wasm.rs` say about the wasm boundary.

The boundary is one `#[cfg(not(target_family = "wasm"))]` per module and it is
described in prose in three places: the crate doc's `# wasm builds` section,
the section headers in the module list, and `wasm.rs`'s own module comment.
Prose does not compile, and all three had drifted:

  - the crate doc said everything below the front end "needs an operating
    system ... and is gated out of a wasm build". `vm` is not gated, which is
    the whole reason the editor on etamil.in runs a program rather than only
    checking one.
  - a header read `Everything below needs an OS` over a list in which
    `crypt`, `signing`, `runtime`, `net` and `stdlib` are ungated.
  - `stdlib` carried a comment claiming wasm needed it most, when `module` --
    its only caller -- is gated out of wasm, so nothing there can reach it.
  - `wasm.rs` said only `lexer -> parser -> check` was reachable, after `run`
    and `run_with_input` had made that untrue, and called six exports "both
    entry points".

None of that is catchable by a compiler, and each one misled a reader. What is
derivable is derived, exactly as `generate_editor_support.py` derives the
editor's keyword list from `lexer.rs` rather than trusting a copy.

Run from the repository root:

    python scripts/check_wasm_boundary.py
    python scripts/check_wasm_boundary.py --check   # CI: exit 1 on any finding

Three things are checked.

**The crate doc's two lists.** The `# wasm builds` section names the modules it
says are gated out and the modules it says are portable. Every name in either
list is classified against the attributes actually on its declaration, and a
contradiction is a finding. The lists are found by the marker phrases below; if
a rewrite removes them, that is itself reported rather than silently passing.

**A note on every module past the front end.** `redis` and `signing` had their
comments off by one -- the note describing HMAC sat above the Redis driver --
so the gated socket module read as portable pure Rust and the portable one had
nothing at all. Requiring a comment on each declaration is what would have
caught it.

**Every browser export named in `wasm.rs`'s module comment.** That comment is
the only index of the surface a browser sees, and it went stale twice.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
LIB = ROOT / "etamil_compiler" / "src" / "lib.rs"
WASM = ROOT / "etamil_compiler" / "src" / "wasm.rs"

# Where the crate doc's two lists start. Each is read to the end of its
# sentence, and every `backticked` name in it is taken as a claim.
GATED_MARKER = "What is gated out"
PORTABLE_MARKER = "left ungated"

# Section headers in the module list. Everything before the *second* one is
# the portable front end -- `check`, `lexer`, `parser` -- which is named in the
# crate doc and needs no note of its own. The rule applies from there on.
SECTION = "// ---"

DECL = re.compile(r"^pub mod (\w+);")
ATTR = re.compile(r"^\s*#\[")
BACKTICKED = re.compile(r"`(\w+)`")


class Module:
    """One `pub mod` declaration, and what its attributes say about wasm."""

    def __init__(self, name: str, line: int, attrs: list[str], comments: list[str]):
        self.name = name
        self.line = line
        self.attrs = attrs
        self.comments = comments

    @property
    def cfg(self) -> str:
        return " ".join(self.attrs)

    @property
    def gated_out(self) -> bool:
        """Absent from a wasm build."""
        return 'not(target_family = "wasm")' in self.cfg

    @property
    def wasm_only(self) -> bool:
        return 'cfg(target_family = "wasm")' in self.cfg

    @property
    def portable(self) -> bool:
        return not self.gated_out and not self.wasm_only


def modules() -> tuple[list[Module], int]:
    """Every top-level `pub mod`, and the line the module list opens on.

    Attributes are collected between one declaration and the next rather than
    by scanning backwards: a backwards scan stops at the first blank or comment
    line and mis-attributes the `#[cfg]` above it, which reported `net` and
    `mongo` as portable when both are gated. Getting this wrong is the reason
    the check exists, so it is worth doing the boring way.
    """
    found: list[Module] = []
    headers: list[int] = []
    previous = -1
    lines = LIB.read_text(encoding="utf-8").split("\n")

    for index, line in enumerate(lines):
        if line.startswith(SECTION):
            headers.append(index)
        match = DECL.match(line)
        if not match:
            continue
        between = lines[previous + 1 : index]
        found.append(
            Module(
                name=match.group(1),
                line=index + 1,
                attrs=[l.strip() for l in between if ATTR.match(l)],
                comments=[
                    l.strip()
                    for l in between
                    if l.strip().startswith("//") and not l.strip().startswith("//!")
                ],
            )
        )
        previous = index

    # The second header, or the first if a section was removed.
    boundary = headers[1] if len(headers) > 1 else (headers[0] if headers else 0)
    return found, boundary


def claimed(marker: str) -> tuple[list[str], str | None]:
    """The names a crate-doc sentence claims, or why they could not be read."""
    doc = "\n".join(
        line for line in LIB.read_text(encoding="utf-8").split("\n") if line.startswith("//!")
    )
    position = doc.find(marker)
    if position < 0:
        return [], f"the crate doc no longer contains {marker!r}"
    # To the end of the sentence: a full stop that is not inside a `path.rs`.
    tail = doc[position:]
    end = re.search(r"\.(?:\s|$)", tail.replace("//!", "   "))
    sentence = tail[: end.end()] if end else tail
    return BACKTICKED.findall(sentence), None


def exports() -> list[str]:
    """Every `#[wasm_bindgen] pub fn` in wasm.rs."""
    text = WASM.read_text(encoding="utf-8")
    return re.findall(r"#\[wasm_bindgen\]\s*\npub fn (\w+)", text)


def module_doc(path: Path) -> str:
    return "\n".join(
        line for line in path.read_text(encoding="utf-8").split("\n") if line.startswith("//!")
    )


def check() -> list[str]:
    findings: list[str] = []
    found, header_line = modules()
    by_name = {module.name: module for module in found}

    # 1. The crate doc's two lists have to match the attributes.
    for marker, expect_gated in ((GATED_MARKER, True), (PORTABLE_MARKER, False)):
        names, problem = claimed(marker)
        if problem:
            findings.append(f"lib.rs  {problem} -- the check cannot read its claims")
            continue
        if not names:
            findings.append(f"lib.rs  the sentence at {marker!r} names no module")
        for name in names:
            module = by_name.get(name)
            if module is None:
                findings.append(
                    f"lib.rs  the crate doc names `{name}`, which is not a module"
                )
                continue
            if expect_gated and not module.gated_out:
                findings.append(
                    f"lib.rs:{module.line}  doc says `{name}` is gated out of wasm; "
                    f"its declaration is not -- {module.cfg or 'no attributes'}"
                )
            if not expect_gated and not module.portable:
                findings.append(
                    f"lib.rs:{module.line}  doc says `{name}` is portable; "
                    f"its declaration says {module.cfg}"
                )

    # 2. Every module past the front end explains itself.
    for module in found:
        if module.line <= header_line or module.name == "wasm":
            continue
        if not module.comments:
            findings.append(
                f"lib.rs:{module.line}  `{module.name}` has no comment of its own. "
                "A reader takes the note above it as describing it, which is how "
                "the redis/signing pair came to be off by one"
            )

    # 3. Every browser export is named in wasm.rs's module comment.
    doc = module_doc(WASM)
    for name in exports():
        if name not in doc:
            findings.append(
                f"wasm.rs  `{name}` is exported to the browser but its module "
                "comment does not mention it"
            )

    return findings


def main() -> int:
    findings = check()
    found, _ = modules()
    gated = sum(1 for module in found if module.gated_out)
    print(
        f"checked {len(found)} modules ({gated} gated out of wasm) "
        f"and {len(exports())} browser exports: {len(findings)} findings\n"
    )
    for finding in findings:
        print(finding)
    return len(findings)


if __name__ == "__main__":
    if sys.argv[1:2] == ["--check"]:
        sys.exit(0 if main() == 0 else 1)
    main()
