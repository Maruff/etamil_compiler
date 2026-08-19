// Reading real compiler output.
//
// The error format is the one thing in this extension that the extension does
// not control, so these tests do not assert against a format written down from
// memory — they run the compiler on programs that are wrong in specific ways
// and check that what comes back is parsed.
//
//   node --test test/errors.test.js
//
// Requires `npm run build`, since this loads the compiled out/errors.js. That
// module deliberately imports nothing from the vscode API so it can be loaded
// here at all.

const assert = require('node:assert/strict');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const { spawnSync } = require('node:child_process');
const { test, describe } = require('node:test');

const EXT_ROOT = path.join(__dirname, '..');
const REPO_ROOT = path.join(EXT_ROOT, '..');
const BUILT = path.join(EXT_ROOT, 'out', 'errors.js');

const available = fs.existsSync(BUILT);
const { parseErrors } = available ? require(BUILT) : { parseErrors: null };

function findCompiler() {
  const exe = process.platform === 'win32' ? 'etamil.exe' : 'etamil';
  for (const profile of ['release', 'debug']) {
    const candidate = path.join(REPO_ROOT, 'etamil_compiler', 'target', profile, exe);
    if (!fs.existsSync(candidate)) {
      continue;
    }
    const probe = spawnSync(candidate, ['--check'], {
      encoding: 'utf8',
      input: 'அச்சு 1;\n',
      timeout: 30000,
    });
    if (probe.status === 0) {
      return candidate;
    }
  }
  return null;
}

const compiler = findCompiler();

/** Real stderr from `etamil --check` over `source`. */
function stderrFor(source) {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'etamil-errors-'));
  try {
    const result = spawnSync(compiler, ['--check'], {
      encoding: 'utf8',
      input: source,
      cwd: dir,
      timeout: 30000,
    });
    return result.stderr || '';
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
}

describe('parsing compiler errors', () => {
  if (!available) {
    test('needs a build', { skip: 'run npm run build first' }, () => {});
    return;
  }

  test('an empty run yields no errors', () => {
    assert.deepEqual(parseErrors(''), []);
    assert.deepEqual(parseErrors('\n\n  \n'), []);
  });

  test('a line without the error marker is ignored', () => {
    // Warnings and banners must not become diagnostics.
    assert.deepEqual(parseErrors('⚠️  ETAMIL_JWT_SECRET is not set\n'), []);
  });

  if (!compiler) {
    test('against real output', { skip: 'no compiler with --check support' }, () => {});
    return;
  }

  test('a parse error keeps its line, column and message', () => {
    const errors = parseErrors(stderrFor('அச்சு "a" அச்சு;\n'));

    assert.equal(errors.length, 1);
    assert.equal(errors[0].line, 1);
    assert.equal(errors[0].column, 11);
    assert.match(errors[0].message, /எதிர்பார்க்கப்பட்டது/);
    // The message keeps both spellings, which is how the compiler writes it.
    assert.match(errors[0].message, /expected/);
    // The marker is not part of the message.
    assert.ok(!errors[0].message.startsWith('✗'));
  });

  test('a position on a later line is read correctly', () => {
    const errors = parseErrors(stderrFor('அச்சு 1;\nஅச்சு 2;\nஅச்சு "a" அச்சு;\n'));

    assert.equal(errors.length, 1);
    assert.equal(errors[0].line, 3);
  });

  test('every type error becomes its own diagnostic', () => {
    const errors = parseErrors(stderrFor('ஈர்ம கொடி = 5;\nஅணி பட்டியல் = 1;\n'));

    assert.equal(errors.length, 2);
    assert.deepEqual(
      errors.map((error) => error.line),
      [1, 2]
    );
    assert.ok(errors.every((error) => error.column > 0));
  });

  test('a lexical error is positioned', () => {
    const errors = parseErrors(stderrFor('அச்சு 1 @;\n'));

    assert.equal(errors.length, 1);
    assert.equal(errors[0].line, 1);
    assert.match(errors[0].message, /அறியப்படாத உள்ளீடு/);
  });

  test('a missing module is reported, anchored to line 1', () => {
    // This one has no position of its own. Dropping it would leave the editor
    // showing a clean file that does not build.
    const errors = parseErrors(stderrFor('இறக்கு "illY/kOppu.qmz";\n'));

    assert.equal(errors.length, 1);
    assert.equal(errors[0].line, 1);
    assert.equal(errors[0].column, 1);
    assert.match(errors[0].message, /illY\/kOppu\.qmz/);
  });

  test('an accepted program produces nothing to report', () => {
    assert.deepEqual(parseErrors(stderrFor('எண் வருவாய் = 100000;\nஅச்சு வருவாய்;\n')), []);
  });

  // The column the compiler reports counts code points, so a position after
  // Tamil text is larger than the letter the reader would point at. The
  // extension converts code points to UTF-16 rather than assuming they match;
  // this pins the input side of that conversion.
  test('columns after Tamil text are code-point counts', () => {
    const errors = parseErrors(stderrFor('குகுகு @;\n'));

    assert.equal(errors.length, 1);
    // குகுகு is 3 written letters but 6 code points, so the @ lands at 8.
    assert.equal(errors[0].column, 8);
  });
});
