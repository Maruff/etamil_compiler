// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Mohammed Maruff (Esan Maruff) <esan@etamil.in>
//
// Where the carried compiler is looked for.
//
// This is the part of carrying a compiler that breaks silently. If the path
// the extension computes and the path the packaging script writes ever stop
// agreeing, nothing fails: the extension simply does not find a binary, falls
// back to the PATH, and the user is told to install a compiler by an extension
// that is carrying one. So both sides are asserted here, and the packaging
// script's table is read rather than restated.
//
//   node --test test/bundle.test.js
//
// Requires `npm run build`; out/bundle.js imports nothing from the vscode API.

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { test, describe } = require('node:test');

const BUILT = path.join(__dirname, '..', 'out', 'bundle.js');
const available = fs.existsSync(BUILT);
const bundle = available ? require(BUILT) : null;

const PACKAGER = path.join(__dirname, '..', '..', 'scripts', 'package_extension.py');

describe('the carried toolchain', { skip: available ? false : 'run npm run build' }, () => {
  test('the binary is named for the platform that runs it', () => {
    assert.equal(
      path.basename(bundle.bundleLayout('/ext', 'win32', 'x64').compiler),
      'etamil.exe'
    );
    assert.equal(
      path.basename(bundle.bundleLayout('/ext', 'linux', 'x64').compiler),
      'etamil'
    );
  });

  test('macOS keeps its architecture, because it is two binaries', () => {
    const intel = bundle.bundleLayout('/ext', 'darwin', 'x64').compiler;
    const silicon = bundle.bundleLayout('/ext', 'darwin', 'arm64').compiler;
    assert.notEqual(intel, silicon);
  });

  test('the library is one directory, shared by every platform', () => {
    const windows = bundle.bundleLayout('/ext', 'win32', 'x64').library;
    const mac = bundle.bundleLayout('/ext', 'darwin', 'arm64').library;
    assert.equal(windows, mac);
    assert.equal(path.basename(windows), 'runtime');
  });

  test('every directory the packaging script writes is one the extension looks in', () => {
    // The script's TARGETS table maps a vsce --target to a bin/ directory.
    // Reading it here is the point: a table copied into this file would agree
    // with itself forever and with nothing else.
    const source = fs.readFileSync(PACKAGER, 'utf8');
    const table = source.slice(source.indexOf('TARGETS = {'));
    const keys = [...table.matchAll(/^\s+"[\w-]+": \("([\w-]+)"/gm)].map((m) => m[1]);
    assert.ok(keys.length >= 4, `found only ${keys.length} targets`);

    for (const key of keys) {
      const [platform, arch] = [key.slice(0, key.lastIndexOf('-')), key.slice(key.lastIndexOf('-') + 1)];
      assert.equal(
        bundle.platformKey(platform, arch),
        key,
        `packaging writes bin/${key}, which the extension would not look in`
      );
    }
  });
});

describe('installing it', { skip: available ? false : 'run npm run build' }, () => {
  test('Unix installs beside what packaging/install.sh installs', () => {
    const layout = bundle.installLayout('linux', '/home/ada');
    assert.equal(layout.bin, path.join('/home/ada', '.local', 'bin'));
    assert.equal(layout.lib, path.join('/home/ada', '.local', 'lib', 'etamil'));
  });

  test('Windows uses LOCALAPPDATA when it is set', () => {
    const layout = bundle.installLayout('win32', 'C:\\Users\\Ada', 'C:\\Users\\Ada\\AppData\\Local');
    assert.ok(layout.bin.includes('AppData'), layout.bin);
    assert.ok(layout.bin.endsWith('bin'), layout.bin);
  });

  test('Windows falls back to the conventional place when it is not', () => {
    const layout = bundle.installLayout('win32', 'C:\\Users\\Ada');
    assert.ok(layout.bin.includes('AppData'), layout.bin);
  });

  test('the PATH advice names both the binary and the library', () => {
    for (const platform of ['linux', 'darwin', 'win32']) {
      const layout = bundle.installLayout(platform, '/home/ada', 'C:\\x');
      const advice = bundle.pathAdvice(platform, layout);
      assert.ok(advice.includes(layout.bin), `${platform}: no bin in ${advice}`);
      assert.ok(advice.includes('ETAMIL_PATH'), `${platform}: no ETAMIL_PATH`);
      assert.ok(advice.includes(layout.lib), `${platform}: no lib in ${advice}`);
    }
  });
});
