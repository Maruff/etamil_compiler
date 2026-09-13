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
//
// The library is shared rather than repeated under each target: it is eTamil
// source, identical everywhere, and `ETAMIL_PATH` is what points the compiler
// at it.
//
// Nothing here imports the vscode API or touches the filesystem — it computes
// paths and nothing else, so test/bundle.test.js can load it and assert the
// layout on every platform rather than only on the one running the tests.

import * as path from 'path';

/** Where the packaged compiler and library sit inside the extension. */
export interface BundleLayout {
  /** The binary for this platform. May not exist: see `platformKey`. */
  compiler: string;
  /** The directory to put on `ETAMIL_PATH`; `nUlakam` sits inside it. */
  library: string;
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

/**
 * The line that puts the install on `PATH`, for the user to paste.
 *
 * The extension does not edit shell profiles or the registry. `install.sh`
 * does, because a person ran it on purpose and can read it first; an editor
 * extension changing the login environment of a machine is a different thing,
 * and it would be doing it on behalf of a VSIX rather than a person.
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
