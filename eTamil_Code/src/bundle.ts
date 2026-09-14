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
 * The OpenType features the eTamil face is drawn with.
 *
 * A smart build of the font carries contextual rules — a consonant drops its
 * pulli before a vowel, a vowel after a consonant shrinks to its sign, the
 * inherent `a` draws nothing — and those rules live in `calt`, with `rlig` as
 * the fallback for shapers that will not apply `calt`.
 *
 * They have to be asked for. VS Code's `editor.fontLigatures` is `false` by
 * default, and when it is false the editor emits
 * `font-feature-settings: "liga" 0, "calt" 0`, which switches the rules off.
 * Setting that option would turn ligatures on for every font in every file the
 * user opens, which is not this extension's business.
 *
 * So the features are declared on the decoration instead. `src/fonts.ts`
 * already injects a font family through the decoration's CSS; the same
 * declaration carries the features, and a declared value beats an inherited
 * one, so the editor's global setting does not have to change and nothing
 * outside the eTamil-script spans is affected.
 */
export const FONT_FEATURES = '"calt" 1, "rlig" 1';

/**
 * Whether a string is a `font-feature-settings` value this may emit.
 *
 * The value is interpolated into CSS, so it is checked rather than trusted,
 * exactly as the font stack is: a `;` or a `}` would let it close the
 * declaration and open another. Accepted is `normal`, or a comma-separated
 * list of four-character tags in quotes, each optionally followed by `on`,
 * `off` or a number — which is the grammar CSS defines and nothing else.
 */
export function isFontFeatureSettings(value: string): boolean {
  const text = value.trim();
  if (!text || text === 'normal') {
    return true;
  }
  const feature = /^["'][A-Za-z0-9]{4}["'](\s+(on|off|\d+))?$/;
  return text.split(',').every((part) => feature.test(part.trim()));
}

/**
 * The faces the extension carries.
 *
 * Two, and they are not interchangeable. `ican qamiz` maps the 95 printable
 * ASCII characters onto Tamil glyphs and nothing else; `ican qamiz Smart` maps
 * the same 95, adds 87 characters of the Tamil block, and carries the
 * contextual rules of `calt` — the pulli appears on a consonant that no vowel
 * follows, a vowel not preceded by a consonant is drawn at full size on the
 * baseline, and the inherent `a` draws nothing.
 *
 * They are separate families rather than two styles of one, so that both can
 * be installed at once and a user who has already set `ican qamiz` keeps
 * exactly the rendering they chose.
 *
 * The version is in each file name because `fonts/` keeps earlier builds
 * beside the current one, and a name that does not say which is which makes
 * the directory unreadable.
 */
export interface Face {
  /** What the file's `name` table calls itself; what a font stack must ask for. */
  readonly family: string;
  /** The file, inside `fonts/`. */
  readonly file: string;
  /** Whether it carries the contextual rules. */
  readonly smart: boolean;
}

export const FACES: readonly Face[] = [
  { family: 'ican qamiz', file: 'ican_qamiz-Regular-2.1.1.ttf', smart: false },
  {
    family: 'ican qamiz Smart',
    file: 'ican_qamiz_Smart-Regular-2.1.0.ttf',
    smart: true,
  },
];

/** The face whose rules the editor is meant to show, where one is wanted. */
export const SMART_FACE: Face = FACES[1];

/**
 * The file of the plain face.
 *
 * Kept as its own export because it is what earlier versions installed, and
 * what `fontInstalled()` looks for when deciding whether the setting names
 * something the machine actually has.
 */
export const FONT_FILE = FACES[0].file;

/** Where a face sits inside the extension. */
export function fontSource(extensionPath: string, file: string = FONT_FILE): string {
  return path.join(extensionPath, 'fonts', file);
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
export function fontRegistryName(family: string = FONT_FAMILY): string {
  return `${family} (TrueType)`;
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
