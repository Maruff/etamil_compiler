#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (C) 2026 Mohammed Maruff (Esan Maruff) <esan@etamil.in>
"""Rank what the LLVM backend still refuses, over the whole corpus.

`scripts/run_parity.sh` is the measurement that counts: it compiles every
program, runs it, and compares the answer against the VM. It needs LLVM 18 and
clang, so it runs on the Ubuntu box and nowhere else.

This is the cheaper question asked anywhere: *what would it refuse, and which
of those refusals is worth building next.* It shells out to `etamil
--llvm-gaps`, which walks the AST with the backend's own list and needs no LLVM
at all.

## The number to steer by

Not "how many times does `வழி` appear" — a construct used forty times in one
program is one program. And not "how many programs contain it" either, because
a program blocked by four different things does not compile when you fix one of
them.

The column that moves the total is **sole blocker**: programs where this
construct is the only thing standing between the source and a compiled binary.
Build the one with the highest sole-blocker count and that many programs move
from refused to compiled the same day.

## What it does not count

Statement-level refusals only. The backend also refuses a name nothing in its
own scope defines, and a handful of operators today's parser does not build. So
"would compile" is an **upper bound** — `run_parity.sh` on a machine with LLVM
is still the number that settles it, and this is the one that tells you what to
work on before you get there.

    python3 scripts/llvm_gap_report.py
    python3 scripts/llvm_gap_report.py --binary path/to/etamil
"""
import argparse
import subprocess
import sys
from collections import Counter, defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
TREES = ("examples", "nUlakam")


def binary(given: str | None) -> Path:
    if given:
        return Path(given)
    for build in ("release", "debug"):
        for name in ("etamil.exe", "etamil"):
            candidate = ROOT / "etamil_compiler" / "target" / build / name
            if candidate.exists():
                return candidate
    sys.exit("no etamil binary found — build one, or pass --binary")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary")
    options = parser.parse_args()
    etamil = binary(options.binary)

    files = sorted(
        path
        for tree in TREES
        for path in (ROOT / tree).rglob("*.qmz")
    )

    clean, blocked, unreadable = [], {}, []
    for path in files:
        rel = path.relative_to(ROOT).as_posix()
        done = subprocess.run(
            [str(etamil), "--llvm-gaps", str(path)],
            capture_output=True, text=True, encoding="utf-8", errors="replace",
        )
        if done.returncode == 0:
            clean.append(rel)
        elif done.returncode == 1:
            labels = [
                line.split("\t", 1)[1]
                for line in done.stdout.strip().split("\n")
                if "\t" in line
            ]
            blocked[rel] = labels
        else:
            # Parse or module-resolution failure: not a backend gap, and
            # counting it as one would hide a broken file.
            unreadable.append(rel)

    programs = Counter()
    sole = Counter()
    for labels in blocked.values():
        for label in set(labels):
            programs[label] += 1
        if len(set(labels)) == 1:
            sole[next(iter(set(labels)))] += 1

    total = len(files)
    print(f"corpus: {total} programs under {', '.join(TREES)}")
    print(f"  would compile   {len(clean)}  (upper bound: statements only)")
    print(f"  refused         {len(blocked)}")
    if unreadable:
        print(f"  did not load    {len(unreadable)}  (not a backend gap)")
        for rel in unreadable:
            print(f"      {rel}")
    print()

    if not blocked:
        print("nothing is refused. The gap is zero.")
        return 0

    width = max(len(label) for label in programs)
    print(f"{'construct':<{width}}  programs  sole blocker")
    print("-" * (width + 24))
    for label, count in sorted(
        programs.items(), key=lambda kv: (-sole[kv[0]], -kv[1], kv[0])
    ):
        print(f"{label:<{width}}  {count:>8}  {sole[label]:>12}")

    print()
    print("Refused programs, and what each is waiting on:")
    by_blockers = defaultdict(list)
    for rel, labels in blocked.items():
        by_blockers[", ".join(sorted(set(labels)))].append(rel)
    for blockers, programs_here in sorted(
        by_blockers.items(), key=lambda kv: (len(kv[0].split(", ")), kv[0])
    ):
        print(f"  {blockers}")
        for rel in sorted(programs_here):
            print(f"      {rel}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
