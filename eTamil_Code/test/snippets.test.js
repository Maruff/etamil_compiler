// Every snippet body is fed to the real compiler.
//
// This is the check that was missing. The previous snippet set offered a
// C-style `சுற்று i = 0; i < 10; ...` loop, a /* */ comment block, and a
// `கோப்பு_திற` with no mode — the first two are parse errors and the third
// silently opened for reading and then appended, so the CSV template
// duplicated its header on every run. All three would have failed here.
//
//   node --test test/snippets.test.js
//
// Skipped with a printed notice when the compiler has not been built, so a
// checkout without a Rust toolchain can still run the grammar tests.

const assert = require('node:assert/strict');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const { spawnSync } = require('node:child_process');
const { test, describe } = require('node:test');

const EXT_ROOT = path.join(__dirname, '..');
const REPO_ROOT = path.join(EXT_ROOT, '..');
const SNIPPETS = path.join(EXT_ROOT, 'snippets', 'etamil.code-snippets');

/**
 * A built compiler that understands `--check`.
 *
 * The capability probe is not paranoia. A stale `target/release` binary from
 * before `--check` existed reports "Unknown option" and exits 2, which looks
 * identical to every snippet being broken — 36 failures with no compiler
 * message in any of them. Checking once, up front, turns that into one clear
 * skip.
 */
function findCompiler() {
  const exe = process.platform === 'win32' ? 'etamil.exe' : 'etamil';
  const rejected = [];

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
      return { path: candidate, profile };
    }
    rejected.push(`${profile}: ${(probe.stderr || probe.stdout || '').trim().split('\n')[0]}`);
  }

  return { path: null, rejected };
}

/**
 * Turn a snippet body into source by resolving its tabstops the way VS Code
 * would on first insertion: a placeholder becomes its default text, a choice
 * becomes its first option, and a repeated tabstop mirrors whatever the first
 * occurrence of that index supplied.
 *
 * Mirroring matters. `தளம்_பிரி ${1};` reuses the backend chosen on line 1,
 * so dropping it would leave `தளம்_பிரி ;` and fail a snippet that is
 * actually correct.
 */
function expand(body) {
  let text = Array.isArray(body) ? body.join('\n') : body;

  // What each index resolves to, taken from its first defining occurrence.
  const values = new Map();
  for (const [, index, choices] of text.matchAll(/\$\{(\d+)\|([^|]*)\|\}/g)) {
    if (!values.has(index)) {
      values.set(index, choices.split(',')[0]);
    }
  }
  for (const [, index, fallback] of text.matchAll(/\$\{(\d+):([^{}]*)\}/g)) {
    if (!values.has(index)) {
      values.set(index, fallback);
    }
  }

  // ${1|a,b,c|} -> a
  text = text.replace(/\$\{(\d+)\|[^|]*\|\}/g, (_, index) => values.get(index) ?? '');
  // ${1:default} -> default, innermost first so nesting resolves
  let previous;
  do {
    previous = text;
    text = text.replace(/\$\{\d+:([^{}]*)\}/g, '$1');
  } while (text !== previous);
  // ${1} and $1 -> whatever index 1 resolved to
  text = text.replace(/\$\{(\d+)\}/g, (_, index) => values.get(index) ?? '');
  text = text.replace(/\$(\d+)/g, (_, index) => values.get(index) ?? '');

  return text;
}

/**
 * Run the front end over `source`.
 *
 * `--check` lexes, parses and type checks and then stops, which is exactly
 * what a snippet needs to be held to. What a template does at *run* time is
 * not its fault: it calls functions that do not exist yet, opens files that
 * are not there, and `வழி` is a server statement the VM refuses by design.
 *
 * It also has to be `--check` rather than `--vm` for a duller reason. Running
 * these templates runs them: an earlier version of this test used `--vm` and
 * the file and CSV snippets wrote kOppu.txt, qaravu.csv and kaNakku.db into
 * the extension directory, where `vsce package` duly picked them up.
 */
function frontEnd(compiler, source) {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'etamil-snippet-'));
  const file = path.join(dir, 'snippet.qmz');
  fs.writeFileSync(file, source, 'utf8');

  try {
    const result = spawnSync(compiler, ['--check', file], {
      encoding: 'utf8',
      input: '',
      timeout: 30000,
      cwd: dir,
      // Imports in a snippet are written as "nUlakam/..." — a path relative to
      // the importing file. The file lives in a temp directory, so the search
      // path is what makes them resolve without copying the library around.
      env: { ...process.env, ETAMIL_PATH: REPO_ROOT },
    });
    return {
      // --check writes nothing to stdout and exits 0 when the program is
      // accepted, so the exit code is the whole answer.
      accepted: result.status === 0,
      output: `${result.stdout || ''}${result.stderr || ''}`,
    };
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
}

const compiler = findCompiler();

describe('snippets compile', () => {
  const raw = JSON.parse(fs.readFileSync(SNIPPETS, 'utf8'));
  const entries = Object.entries(raw).filter(([name]) => name !== '//');

  test('the file defines snippets', () => {
    assert.ok(entries.length >= 20, `only ${entries.length} snippets found`);
  });

  test('every prefix is a real spelling, and Tamil and romanized are paired', () => {
    for (const [name, snippet] of entries) {
      const prefixes = [].concat(snippet.prefix);
      assert.ok(prefixes.length >= 2, `${name}: needs a Tamil and a romanized prefix`);
      assert.ok(
        prefixes.some((p) => /[஀-௿]/.test(p)),
        `${name}: no Tamil prefix`
      );
      assert.ok(
        prefixes.some((p) => /^[a-zA-Z]/.test(p)),
        `${name}: no romanized prefix`
      );
      // The pre-`Z` romanization is what made the old set unusable.
      for (const prefix of prefixes) {
        assert.ok(
          !/^(enil|inREl|pinnam|poqu|poi|inmY)$/.test(prefix),
          `${name}: "${prefix}" is the superseded romanization`
        );
      }
    }
  });

  test('no snippet emits a block comment or a single-quoted string', () => {
    for (const [name, snippet] of entries) {
      const source = expand(snippet.body);
      assert.ok(!source.includes('/*'), `${name}: eTamil has no block comment`);
      assert.ok(
        !/'[^']*'/.test(source),
        `${name}: eTamil has no single-quoted string`
      );
    }
  });

  test('a file opened for writing says so', () => {
    // `கோப்பு_திற "f"` with no mode defaults to "read", and because the mode
    // is not enforced the write then succeeds as an append — so a template
    // that omitted it duplicated its output on every run.
    for (const [name, snippet] of entries) {
      const source = expand(snippet.body);
      for (const line of source.split('\n')) {
        if (/கோப்பு_திற|kOppu_qiRa/.test(line)) {
          assert.match(
            line,
            /"(read|write|append)"/,
            `${name}: கோப்பு_திற needs an explicit mode`
          );
        }
      }
    }
  });

  if (!compiler.path) {
    const reason = compiler.rejected?.length
      ? `no compiler with --check support. Run: cargo build --release. (${compiler.rejected.join('; ')})`
      : 'etamil binary not built. Run: cargo build --release';
    test('compiler checks', { skip: reason }, () => {});
  } else {
    for (const [name, snippet] of entries) {
      test(name, () => {
        const source = expand(snippet.body);
        const { accepted, output } = frontEnd(compiler.path, source);
        assert.ok(
          accepted,
          `snippet "${name}" does not compile.\n\n--- source ---\n${source}\n\n--- compiler ---\n${output}`
        );
      });
    }
  }
});
