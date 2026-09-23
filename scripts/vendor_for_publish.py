#!/usr/bin/env python3
"""Stage the files `cargo publish` needs but cannot reach.

    python scripts/vendor_for_publish.py            # stage
    python scripts/vendor_for_publish.py --check    # is the staged copy current?
    python scripts/vendor_for_publish.py --clean    # remove it again

Cargo packages only what lives under the package root, and two things the crate
needs live above it:

  nUlakam/   the standard library, which is eTamil source, one level up because
             it belongs to the *language* rather than to this Rust crate.
             `build.rs` compiles it into the binary, so without it a published
             crate would build a compiler whose first import fails.
  LICENSE    the AGPL text, so the published crate carries its own licence
             rather than only naming one in metadata.

Both are copied into `etamil_compiler/` and both are gitignored. That is the
point: the copy exists for the length of a publish and is regenerated from the
original every time, so it cannot drift the way a committed second copy would.
`--check` exists for CI, which should fail if a stale copy was left behind and
committed by accident.
"""

from __future__ import annotations

import argparse
import filecmp
import os
import shutil
import stat
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CRATE = ROOT / "etamil_compiler"

# (source, destination-inside-the-crate)
VENDORED = [
    (ROOT / "nUlakam", CRATE / "nUlakam"),
    (ROOT / "LICENSE", CRATE / "LICENSE"),
]


def _force_remove(func, path, _exc_info) -> None:
    """Clear the read-only bit and retry.

    `copy2` preserves permissions, so on Windows a read-only source file
    produces a read-only copy, and `rmtree` then fails with "Access is denied"
    on the directory containing it. Since this whole tree is a staging copy
    that is meant to be disposable, forcing it writable is the right answer
    rather than asking the user to fix attributes by hand.
    """
    os.chmod(path, stat.S_IWRITE)
    func(path)


def remove(destination: Path) -> None:
    """Delete a staged copy, whatever its file attributes."""
    if destination.is_dir():
        shutil.rmtree(destination, onexc=_force_remove)
    elif destination.exists():
        os.chmod(destination, stat.S_IWRITE)
        destination.unlink()


def stage() -> int:
    for source, destination in VENDORED:
        if not source.exists():
            print(f"error: {source} does not exist", file=sys.stderr)
            return 2
        remove(destination)
        if source.is_dir():
            # Only the language sources. Anything a run left behind — a
            # SQLite file an example wrote, a __pycache__ — must not be
            # published inside the crate.
            shutil.copytree(
                source,
                destination,
                ignore=shutil.ignore_patterns(
                    "*.db", "*.db-journal", "*.sqlite", "__pycache__", ".*"
                ),
            )
            count = sum(1 for _ in destination.rglob("*.qmz"))
            print(f"staged {destination.relative_to(ROOT).as_posix()} ({count} modules)")
        else:
            shutil.copy2(source, destination)
            print(f"staged {destination.relative_to(ROOT).as_posix()}")
    return 0


def check() -> int:
    stale: list[str] = []
    for source, destination in VENDORED:
        if not destination.exists():
            continue  # not staged at all is the normal committed state
        if source.is_dir():
            comparison = filecmp.dircmp(source, destination)
            if comparison.left_only or comparison.diff_files or comparison.funny_files:
                stale.append(destination.relative_to(ROOT).as_posix())
        elif not filecmp.cmp(source, destination, shallow=False):
            stale.append(destination.relative_to(ROOT).as_posix())

    if stale:
        print(
            "a staged copy differs from its original:\n  " + "\n  ".join(stale) +
            "\n\nrun: python scripts/vendor_for_publish.py",
            file=sys.stderr,
        )
        return 1

    print("no stale staged copies")
    return 0


def clean() -> int:
    for _source, destination in VENDORED:
        if destination.exists():
            remove(destination)
            print(f"removed {destination.relative_to(ROOT).as_posix()}")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    group = parser.add_mutually_exclusive_group()
    group.add_argument("--check", action="store_true")
    group.add_argument("--clean", action="store_true")
    args = parser.parse_args()

    if args.check:
        return check()
    if args.clean:
        return clean()
    return stage()


if __name__ == "__main__":
    sys.exit(main())
