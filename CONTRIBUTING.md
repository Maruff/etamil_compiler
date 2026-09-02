# Contributing to eTamil

Contributions are welcome. This file covers the one piece of paperwork there is,
and the checks your change has to pass.

## The sign-off

Every commit needs a `Signed-off-by` line:

```bash
git commit -s -m "your message"
```

`-s` adds it for you, using your `user.name` and `user.email`. It looks like
this:

    Signed-off-by: Your Name <your@email>

That line is the [Developer Certificate of Origin](https://developercertificate.org)
1.1, and by adding it you are stating that you wrote the change, or have the
right to submit it under the project's licence, and that you understand the
contribution and its record are public and permanent.

**You keep the copyright in what you write.** There is no copyright assignment
here and no contributor licence agreement to sign. You license your contribution
under AGPL-3.0-or-later, the same terms as the rest of the project, and you are
added to `AUTHORS`. The DCO exists so that the licence can be honoured towards
everyone it is owed to — which means the record of who wrote what has to be
right from the first outside commit rather than reconstructed later.

One consequence worth stating plainly: because copyright stays with each author,
the project cannot be relicensed without every contributor's agreement. That is
deliberate. It means the openness of eTamil does not depend on anyone's
continued goodwill, including the original author's.

If your employer owns your work, get their sign-off before you send it, not
after.

## Licence headers

Every source file starts with two lines:

```rust
// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Mohammed Maruff (Esan Maruff) <esan@etamil.in>
```

A new file you write gets the same two lines with your own copyright line. If
you make substantial changes to an existing file you may add your copyright line
below the existing one — do not replace it. Removing a copyright line is a
licence violation under AGPL section 5(b), not a formatting preference.

`NOTICE` explains what the licence asks of you in full.

## Before you open a pull request

These all gate CI, so a failure here is a failure there.

```bash
cd etamil_compiler && cargo test && cd ..
./scripts/run_examples.sh
python3 scripts/generate_editor_support.py --check
python3 scripts/transliterate.py --check
python3 scripts/check_names.py --check
cd etamil_compiler && cargo check --lib --target wasm32-unknown-unknown --no-default-features
```

The last one catches things a native build compiles straight through, and it is
the one people forget.

Two of these deserve a word, because they are unusual:

- **`transliterate.py --check`** holds every keyword's romanization to the
  ezuqqu scheme. Adding a keyword with an off-scheme spelling fails it.
- **`check_names.py --check`** does the same for everything that is not a
  keyword — module and file names, SQL tables and columns, record keys. A new
  English name containing `b`, `d`, `f` or `g` will trip it and belongs in that
  file's `ALLOW` list.

Neither drifts loudly. Both fail silently in the sense that nothing else in the
build notices, which is exactly why they gate.

## Naming things

Names in `nUlakam/` and in the language itself are Tamil, romanized under the
ezuqqu scheme — one Latin letter per Tamil letter, documented in
`docs/reference/COMPILER_TAMIL_LETTER_EQUIVALENTS.md`. It is deliberately not
ISO 15919: no diacritics, no digraphs. `த` is `q` and not `th`, `ழ` is `z` and
not `zh`, `ஐ` is `Y` and not `ai`.

To check a name, run it back through the scheme:

```bash
python3 scripts/transliterate.py விகிதம்      # -> vikiqam
```

## Commit messages

A short declarative subject saying what changed, then prose explaining what was
wrong and why — not a bullet list. Look at `git log` for the register. The
subject should read as a statement, not a category tag.

## Reporting a security problem

Do not open a public issue. Email <esan@etamil.in>.
