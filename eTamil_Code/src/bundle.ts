// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Mohammed Maruff (Esan Maruff) <esan@etamil.in>
//
// Where the compiler the extension carries lives, and where it installs to.
//
// The extension ships the `etamil` binary and the nUlakam standard library
// inside the VSIX, so that installing the extension is the whole installation:
// no Rust, no release download, no PATH. One VSIX per platform, built by
// `scripts/package_extension.py` with `vsce package --target`, so a Windows
// user never downloads a macOS binary.
//
// Two directories, staged at package time and ignored by git because they are
// build output:
//
//     bin/<platform>-<arch>/etamil[.exe]
//     runtime/nUlakam/**
//     runtime/examples/**
//
// The library and the examples are shared rather than repeated under each
// target: they are eTamil source, identical everywhere, and `ETAMIL_PATH` is
// what points the compiler at the library. The examples are there so that
// somebody who has just installed the extension has twenty-nine working
// programs to open, rather than an empty file and a language they have never
// seen.
//
// A third thing travels with them and is committed rather than staged, because
// it is a source asset and not a build product:
//
//     fonts/ican_qamiz-Regular.ttf
//
// That is the eTamil font, in which the ASCII letters carry Tamil glyphs. It
// cannot be loaded from here — VS Code's editor is not a webview and no API
// registers a font — so it has to be copied into the operating system's own
// per-user font directory before any `font-family` naming it resolves.
//
// Nothing here imports the vscode API or touches the filesystem — it computes
// paths and nothing else, so test/bundle.test.js can load it and assert the
// layout on every platform rather than only on the one running the tests.

import * as path from 'path';

/** Where the packaged compiler, library and examples sit inside the extension. */
export interface BundleLayout {
  /** The binary for this platform. May not exist: see `platformKey`. */
  compiler: string;
  /** The directory to put on `ETAMIL_PATH`; `nUlakam` sits inside it. */
  library: string;
  /** The repository's `examples/`, for **eTamil: Open an example**. */
  examples: string;
}

/** Where an install puts them, outside the extension. */
export interface InstallLayout {
  /** Directory for the binary, to be added to `PATH`. */
  bin: string;
  /** Directory for the standard library, to be named by `ETAMIL_PATH`. */
  lib: string;
}

/**
 * The `bin/` subdirectory for a platform.
 *
 * `process.platform` and `process.arch` spellings, joined — the same pair VS
 * Code's own `--target` names are derived from, and the same pair the
 * extension's `downloadFor` already keys on. macOS is two genuinely different
 * binaries, so the architecture is never dropped.
 */
export function platformKey(platform: string, arch: string): string {
  return `${platform}-${arch}`;
}

/** Paths to the carried toolchain, whether or not this VSIX was built with it. */
export function bundleLayout(
  extensionPath: string,
  platform: string,
  arch: string
): BundleLayout {
  const exe = platform === 'win32' ? 'etamil.exe' : 'etamil';
  return {
    compiler: path.join(extensionPath, 'bin', platformKey(platform, arch), exe),
    library: path.join(extensionPath, 'runtime'),
    examples: path.join(extensionPath, 'runtime', 'examples'),
  };
}

/**
 * Where `eTamil: Install the compiler` copies the toolchain to.
 *
 * A per-user location on both sides, so nothing asks for administrator rights.
 * `~/.local` on Unix is the same prefix `packaging/install.sh` uses, so the
 * extension and the standalone installer do not fight over two copies.
 * `%LOCALAPPDATA%\Programs` is the Windows equivalent — the directory a
 * per-user MSI installs into, and already on the ignore list of every
 * sensible antivirus policy.
 */
export function installLayout(
  platform: string,
  home: string,
  localAppData?: string
): InstallLayout {
  if (platform === 'win32') {
    const base = path.join(localAppData || path.join(home, 'AppData', 'Local'), 'Programs', 'eTamil');
    return { bin: path.join(base, 'bin'), lib: path.join(base, 'lib') };
  }
  return {
    bin: path.join(home, '.local', 'bin'),
    lib: path.join(home, '.local', 'lib', 'etamil'),
  };
}

// ---------------------------------------------------------------------------
// The font
// ---------------------------------------------------------------------------

/**
 * The family name the font declares, read from its own `name` table.
 *
 * Lower case, with a space: that is what is in the file, and a font family is
 * matched by what the file says rather than by what the file is called.
 * `test/bundle.test.js` reads the shipped `.ttf` and asserts this, so replacing
 * the font with one that calls itself something else fails a test instead of
 * silently naming a family no machine has.
 */
export const FONT_FAMILY = 'ican qamiz';

/**
 * The file, inside `fonts/`.
 *
 * Carries the version, because `fonts/` keeps the earlier build beside it and
 * a name that does not say which is which would make the pair unreadable.
 */
export const FONT_FILE = 'ican_qamiz-Regular-2.1.1.ttf';

/** Where the font sits inside the extension. */
export function fontSource(extensionPath: string): string {
  return path.join(extensionPath, 'fonts', FONT_FILE);
}

/**
 * The per-user font directory of this machine.
 *
 * Per-user on all three, so nothing asks for administrator rights. macOS and
 * Linux read these directories directly — Linux after `fc-cache`. Windows
 * needs the file *and* a registry value under HKCU; the directory alone
 * installs nothing, which is the part that looks like it worked and has not.
 */
export function fontInstallDir(
  platform: string,
  home: string,
  localAppData?: string
): string {
  if (platform === 'win32') {
    return path.join(
      localAppData || path.join(home, 'AppData', 'Local'),
      'Microsoft',
      'Windows',
      'Fonts'
    );
  }
  if (platform === 'darwin') {
    return path.join(home, 'Library', 'Fonts');
  }
  return path.join(home, '.local', 'share', 'fonts');
}

/**
 * The value name Windows lists a font under.
 *
 * The convention is the face's full name followed by the format in
 * parentheses, and the family is what a `font-family` will later ask for, so
 * the two are derived from the same constant rather than typed twice.
 */
export function fontRegistryName(): string {
  return `${FONT_FAMILY} (TrueType)`;
}

/**
 * The line that puts the install on `PATH`, for the user to paste.
 *
 * The extension does not edit shell profiles, and does not touch the
 * environment: changing how every program on a machine starts is not a thing
 * to do on a VSIX's behalf. `install.sh` may, because a person ran it on
 * purpose and could read it first.
 *
 * The font install is the one place a registry value is written, and it is not
 * an exception to this. It adds a face to the user's own font list, which is
 * the only mechanism Windows has for installing a font without administrator
 * rights, and it happens behind a dialog that says so.
 */
export function pathAdvice(platform: string, layout: InstallLayout): string {
  if (platform === 'win32') {
    return (
      `setx PATH "%PATH%;${layout.bin}"\n` +
      `setx ETAMIL_PATH "${layout.lib}"`
    );
  }
  return (
    `export PATH="$PATH:${layout.bin}"\n` +
    `export ETAMIL_PATH="${layout.lib}"`
  );
}
