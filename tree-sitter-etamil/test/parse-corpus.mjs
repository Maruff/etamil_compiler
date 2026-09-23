// Parse every real eTamil program in the repository.
//
//   node test/parse-corpus.mjs
//
// The corpus tests in test/corpus/ pin the shape of individual constructs.
// This one asks a blunter question: can the grammar parse the language as it
// is actually written? It found four real gaps the corpus tests did not —
// multi-line SQL string literals, the database statements, `ஜேசான்_உரை` and
// `இடைவெளி` — because none of those appear in a hand-written example until
// someone writes one.
//
// Run as part of `npm test` in CI, so adding a statement to the compiler
// without adding it here fails the build rather than silently degrading every
// editor that uses this grammar.

import { execFileSync } from 'node:child_process';
import { existsSync, readdirSync, statSync } from 'node:fs';
import { dirname, join, relative } from 'node:path';
import { fileURLToPath } from 'node:url';

const HERE = dirname(fileURLToPath(import.meta.url));
const GRAMMAR = join(HERE, '..');
const REPO = join(GRAMMAR, '..');

const CLI = join(
  GRAMMAR,
  'node_modules',
  '.bin',
  process.platform === 'win32' ? 'tree-sitter.cmd' : 'tree-sitter'
);

/** Every .qmz under `directory`, recursively. */
function collect(directory) {
  if (!existsSync(directory)) {
    return [];
  }
  const found = [];
  for (const entry of readdirSync(directory)) {
    const path = join(directory, entry);
    if (statSync(path).isDirectory()) {
      found.push(...collect(path));
    } else if (entry.endsWith('.qmz')) {
      found.push(path);
    }
  }
  return found;
}

const sources = [
  ...collect(join(REPO, 'nUlakam')),
  ...collect(join(REPO, 'examples')),
];

if (sources.length === 0) {
  console.error('no .qmz files found — is this running inside the repository?');
  process.exit(2);
}

if (!existsSync(CLI)) {
  console.error('tree-sitter CLI not installed; run: npm install');
  process.exit(2);
}

let output = '';
try {
  output = execFileSync(CLI, ['parse', '--quiet', '--stat', ...sources], {
    cwd: GRAMMAR,
    encoding: 'utf8',
    stdio: ['ignore', 'pipe', 'pipe'],
    maxBuffer: 32 * 1024 * 1024,
  });
} catch (error) {
  // A non-zero exit means at least one file failed; the report is still on
  // stdout and is the useful part.
  output = `${error.stdout ?? ''}${error.stderr ?? ''}`;
}

// Any line naming a file and an ERROR or MISSING node is a parse failure.
const failures = output
  .split(/\r?\n/)
  .filter((line) => /\((?:ERROR|MISSING)/.test(line))
  .map((line) => line.trim());

const summary = output.match(/Total parses: (\d+); successful parses: (\d+)/);

if (failures.length > 0) {
  console.error(`\n${failures.length} file(s) the grammar cannot parse:\n`);
  for (const failure of failures) {
    console.error(`  ${failure}`);
  }
  console.error(
    '\nEach of these is a construct the compiler accepts and this grammar does not.\n'
  );
  process.exit(1);
}

const [, total, ok] = summary ?? [];
console.log(
  `parsed ${ok ?? sources.length}/${total ?? sources.length} eTamil files ` +
    `from ${relative(REPO, join(REPO, 'nUlakam'))} and ` +
    `${relative(REPO, join(REPO, 'examples'))} with no errors`
);
