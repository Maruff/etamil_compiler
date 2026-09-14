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

/** The GSUB feature tags a font declares, read out of the table directory. */
function gsubFeatures(file) {
  const data = fs.readFileSync(file);
  const tables = data.readUInt16BE(4);
  let gsub = null;
  for (let i = 0; i < tables; i += 1) {
    const rec = 12 + i * 16;
    if (data.toString('latin1', rec, rec + 4) === 'GSUB') {
      gsub = data.readUInt32BE(rec + 8);
    }
  }
  const found = new Set();
  if (gsub === null) {
    return found;
  }
  const list = gsub + data.readUInt16BE(gsub + 6);
  const count = data.readUInt16BE(list);
  for (let i = 0; i < count; i += 1) {
    found.add(data.toString('latin1', list + 2 + i * 6, list + 6 + i * 6));
  }
  return found;
}

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

  test('the examples sit beside the library, under the same runtime', () => {
    const layout = bundle.bundleLayout('/ext', 'linux', 'x64');
    assert.equal(path.dirname(layout.examples), layout.library);
    assert.equal(path.basename(layout.examples), 'examples');
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

/**
 * The `name` table of a TrueType file, as {nameId: string}.
 *
 * Forty lines of struct reading rather than a dependency, because the one
 * question worth asking of the shipped font is what family it calls itself,
 * and the answer has to come out of the file rather than out of a constant
 * that agrees with another constant.
 */
function readNames(file) {
  const data = fs.readFileSync(file);
  const tableCount = data.readUInt16BE(4);
  let nameTable = -1;
  for (let i = 0; i < tableCount; i += 1) {
    const entry = 12 + i * 16;
    if (data.toString('latin1', entry, entry + 4) === 'name') {
      nameTable = data.readUInt32BE(entry + 8);
    }
  }
  assert.notEqual(nameTable, -1, 'no name table');

  const records = data.readUInt16BE(nameTable + 2);
  const strings = nameTable + data.readUInt16BE(nameTable + 4);
  const names = {};
  for (let i = 0; i < records; i += 1) {
    const record = nameTable + 6 + i * 12;
    const platform = data.readUInt16BE(record);
    const nameId = data.readUInt16BE(record + 6);
    const length = data.readUInt16BE(record + 8);
    const offset = data.readUInt16BE(record + 10);
    const raw = data.subarray(strings + offset, strings + offset + length);
    const text = platform === 0 || platform === 3 ? raw.swap16().toString('utf16le') : raw.toString('latin1');
    if (names[nameId] === undefined) {
      names[nameId] = text;
    }
  }
  return names;
}

describe('the carried font', { skip: available ? false : 'run npm run build' }, () => {
  // Read from the extension rather than restated, for the same reason the
  // packaging table is: a rename that updates one of the two and not the other
  // is the failure this file exists to catch.
  const FONT = path.join(__dirname, '..', 'fonts', bundle.FONT_FILE);

  test('it ships', () => {
    assert.ok(fs.existsSync(FONT), `${FONT} is missing`);
  });

  test('the family the extension names is the family the file declares', () => {
    // The failure this catches is silent from end to end. Replace the font
    // with a build that calls itself something else and everything still
    // installs, the setting still takes the value, the decoration is still
    // created — and the family resolves to nothing, so the editor draws in its
    // own font and the feature appears to do nothing at all.
    const names = readNames(FONT);
    assert.equal(names[1], bundle.FONT_FAMILY);
  });

  test('it says it is under a licence we ship', () => {
    const names = readNames(FONT);
    assert.match(names[13] || '', /SIL Open Font License/);
    assert.ok(
      fs.existsSync(path.join(__dirname, '..', 'fonts', 'OFL.txt')),
      'the font declares the OFL and OFL.txt is not beside it'
    );
  });

  test('the registry value name is built from the family', () => {
    assert.equal(bundle.fontRegistryName(), `${bundle.FONT_FAMILY} (TrueType)`);
  });

  test('both carried faces exist and declare the family the table claims', () => {
    // The failure this catches is the same silent one as for the single face,
    // twice over: a file renamed, or a face rebuilt under a different family,
    // and the setting resolves to nothing while everything still installs.
    for (const face of bundle.FACES) {
      const file = path.join(__dirname, '..', 'fonts', face.file);
      assert.ok(fs.existsSync(file), `fonts/${face.file} is not there`);
      assert.equal(readNames(file)[1], face.family);
      assert.equal(
        bundle.fontRegistryName(face.family),
        `${face.family} (TrueType)`
      );
    }
  });

  test('only the smart face carries the contextual rules', () => {
    // calt is what makes the smart face smart. If a rebuild drops it the font
    // still installs, still resolves, and quietly renders like the plain one.
    for (const face of bundle.FACES) {
      const features = gsubFeatures(path.join(__dirname, '..', 'fonts', face.file));
      if (face.smart) {
        assert.ok(features.has('calt'), `${face.file} has lost calt`);
      } else {
        assert.ok(!features.has('calt'), `${face.file} should not carry calt`);
      }
    }
  });

  test('the smart face is the one the installer offers', () => {
    assert.equal(bundle.SMART_FACE.smart, true);
    assert.equal(bundle.FONT_FILE, bundle.FACES[0].file);
  });

  test('the default features ask for the two the smart font uses', () => {
    // A smart build puts its contextual rules in calt, with rlig for the
    // shapers that will not apply calt. Asking for neither would leave the
    // rules dormant and look exactly like a font that has none.
    assert.match(bundle.FONT_FEATURES, /"calt"/);
    assert.match(bundle.FONT_FEATURES, /"rlig"/);
    assert.ok(bundle.isFontFeatureSettings(bundle.FONT_FEATURES));
  });

  test('a feature setting that could break out of the CSS is refused', () => {
    // The value is interpolated into the decoration's CSS beside the font
    // family, so anything that could close the declaration has to be rejected
    // rather than trusted, exactly as the font stack is.
    for (const bad of [
      '"calt" 1; color: red',
      '"calt" 1 } body {',
      'url(x)',
      '"toolongatag" 1',
      '"cal" 1',
      'calt 1',
      '"calt" maybe',
    ]) {
      assert.equal(
        bundle.isFontFeatureSettings(bad),
        false,
        `should have been refused: ${bad}`
      );
    }
  });

  test('the shapes CSS allows are accepted', () => {
    for (const good of [
      '',
      'normal',
      '"calt"',
      '"calt" 1',
      '"calt" on',
      '"calt" off',
      "'calt' 1, 'rlig' 1",
      '"calt" 1, "rlig" 1, "liga" 0',
      '  "calt" 1 ,  "rlig" 1  ',
    ]) {
      assert.ok(
        bundle.isFontFeatureSettings(good),
        `should have been accepted: ${good}`
      );
    }
  });

  test('each platform gets its own per-user font directory', () => {
    assert.equal(
      bundle.fontInstallDir('darwin', '/Users/ada'),
      path.join('/Users/ada', 'Library', 'Fonts')
    );
    assert.equal(
      bundle.fontInstallDir('linux', '/home/ada'),
      path.join('/home/ada', '.local', 'share', 'fonts')
    );
    const windows = bundle.fontInstallDir('win32', 'C:\\Users\\Ada', 'C:\\Users\\Ada\\AppData\\Local');
    assert.ok(windows.endsWith(path.join('Microsoft', 'Windows', 'Fonts')), windows);
  });

  test('the file it installs is the file it carries', () => {
    assert.equal(path.basename(bundle.fontSource('/ext')), bundle.FONT_FILE);
    assert.equal(path.basename(FONT), bundle.FONT_FILE);
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
