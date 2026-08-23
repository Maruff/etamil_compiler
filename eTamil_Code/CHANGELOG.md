# Changelog

## 0.4.0

### Added

- **eTamil: Install the compiler** now offers the prebuilt package first. The
  old dialog offered only `cargo build --release`, which asks for Rust and a C
  toolchain before the author has run a single line of eTamil. The package needs
  neither.

  It opens the release URL in a browser and, if asked, copies the two
  extract-and-install commands to the clipboard. It deliberately does not fetch
  the archive or pipe anything remote into a shell: the author downloads it,
  sees what they have, and runs the installer themselves.

  On a platform with no prebuilt package — macOS today — the same entry opens
  the releases page and points at the source build.

### Changed

- The generated keyword data now carries **523 spellings across 202 tokens**, up
  from 505. Eighteen keywords gained their on-scheme romanization — `utal`,
  `paqil`, `vazi` and the rest — while keeping the spelling they had, so both
  highlight and complete. Nothing that lexed before stops lexing.

- Completions show a `செயல்`'s declared signature where it has one, because
  parameters and return types can now be declared:
  `செயல் வரி(எண் தொகை) எண் { … }`.

## 0.3.0

The keyword data is now generated from the compiler instead of maintained by
hand, and the extension gained the features that needed it.

### Fixed — the extension described a language two revisions old

- **67 of the lexer's 201 keywords were missing from the grammar**, including
  `செயல்` and `திரும்பு` (functions), `ஒவ்வொரு`/`இல்` (iteration), `இறக்கு`
  (modules), `மற்றும்`/`அல்லது`/`இல்லை` (logical operators), `ஈர்ம` (the
  boolean type), and every database type and table operation. Because the
  grammar ended in a catch-all Tamil rule, all 67 rendered as variable names —
  so `nUlakam/kaNakkiyal/`, the accounting framework, opened with its control
  flow uncoloured.
- **28 romanized spellings were the pre-`Z` scheme.** `eZil` got no
  highlighting; `enil`, which the compiler rejects, got highlighted as a
  keyword. A third set of spellings (`paNgu`, `vazhi`) belonged to neither
  scheme.
- `பொது` was offered as the boolean type. It is not a keyword at all — the
  completion inserted a parse error. The real type, `ஈர்ம`, appeared nowhere.
- The `சுற்று` snippet and completion emitted a C-style three-clause `for`
  loop. eTamil writes `(cond) சுற்று { }`.
- `/* */` block comments were declared in the grammar, the language
  configuration and a snippet. The lexer has no block comment, so commenting a
  region greyed it out and then failed to lex.
- `'...'` was treated as a string. eTamil has no single-quoted string, and a
  stray apostrophe started a phantom string that swallowed the rest of the file.
- The file snippets opened without a mode, which defaults to `read`. Because the
  mode is not enforced, the write then succeeded as an *append* — so the CSV
  template added a fresh header row on every run.
- `மாறி` and `நிலை` were highlighted as declarations. The parser treats both as
  ordinary names, and neither has any statement syntax.
- Financial nouns were scoped as keywords, though the compiler deliberately
  allows them as names. They now use a `support.type.domain` scope, and hover
  says they are free to reuse.
- Percentage literals never highlighted: the pattern ended in `\b` after `%`,
  which cannot match before `;` or end of line.
- `1_000` was highlighted as a number. The lexer rejects digit separators.
- `indentationRules` looked for `எனில்` followed by `(`, but eTamil puts the
  condition first, so they never fired.
- The declared language icon pointed at an `icons/` directory that does not
  exist.
- `license` said MIT. The repository is AGPL-3.0-or-later, deliberately.
- `.eslintrc.json` had no TypeScript parser, so `eslint src --ext .ts` failed on
  the first type annotation. It had never linted this code.

### Fixed — security and behaviour

- `etamil.installCommand` was a resource-scoped string executed in a terminal,
  so a cloned repository's own `.vscode/settings.json` could choose the command.
  Removed. The install command is now fixed in code, or typed by the author in
  the moment.
- `capabilities.untrustedWorkspaces` is declared unsupported. The extension runs
  a compiler binary, so it must not be active in an untrusted workspace.
- Activation is `onLanguage:etamil` only. `onStartupFinished` meant the
  extension woke in every window and immediately offered to run
  `git clone && cargo build`.
- `etamil.autoInstallOnActivation`, `etamil.syntaxHighlight` and
  `etamil.showIntelliSense` were declared and never read. Removed the first two;
  `etamil.intelliSense` now actually gates the providers.
- "Skip" wrote a `workspaceState` flag that was never read, so the prompt came
  back on the next startup. "Don't ask again" now means it.
- The 60-second verification poll was not cancellable. Removed.
- Completions no longer offer bare operators (`+`, `-`, `*`) as items, and are
  no longer rebuilt on every keystroke.

### Added

- **Errors as you type**, from `etamil --check` — a new compiler mode that stops
  after the type checker, so an editor never runs the program it is checking.
  Positions convert from the compiler's code-point columns to UTF-16 exactly.
- **Completions** for all 23 host builtins and all 122 `செயல்` functions in
  `nUlakam`, with parameter names and doc comments read from their source.
- **Spelling-aware templates**: a romanized prefix inserts romanized
  placeholders.
- **Request-variable completions** inside a `வழி` handler — `request_body`,
  `query_params`, `headers`, `path_params` and the rest.
- **Signature help**, **Go to Definition** into `nUlakam`, **document outline**
  of every `செயல்`, and **eTamil: Run this file** / **Serve this file**.
- A Tamil-first font stack for eTamil files, because most monospace faces carry
  no Tamil glyphs and fall back mid-line.
- `test/grammar.test.js` — scope assertions via vscode-textmate, including that
  `\b` behaves correctly around Tamil combining marks, and a no-holes sweep over
  the standard library.
- `test/snippets.test.js` — every snippet body is compiled by the real compiler.

## 0.2.0 and earlier

Syntax highlighting, snippets and a keyword completion list, maintained by hand.
