// Does the generated highlight.js definition actually highlight eTamil?
//
//   node --test test/
//
// The definition is generated from the compiler's token table, so what needs
// testing is not the keyword list — `generate_editor_support.py --check`
// guards that. What needs testing is that highlight.js can *see* those
// keywords at all, and one thing in particular: highlight.js decides what
// counts as a word with `$pattern`, and its default is ASCII. Get that wrong
// and every Tamil keyword in the language is silently invisible while the
// romanized ones still work — which would look like a partial success rather
// than a broken file.

import assert from 'node:assert/strict';
import { test, describe } from 'node:test';

import hljs from 'highlight.js/lib/core';
import etamil from '../src/languages/etamil.js';

hljs.registerLanguage('etamil', etamil);

/** The set of hljs class names applied anywhere in `code`. */
function classesIn(code) {
  const html = hljs.highlight(code, { language: 'etamil' }).value;
  return new Set([...html.matchAll(/class="hljs-([a-z_.]+)"/g)].map((m) => m[1]));
}

/** The class applied to `word` in `code`, or undefined. */
function classOf(code, word) {
  const html = hljs.highlight(code, { language: 'etamil' }).value;
  const match = html.match(
    new RegExp(`<span class="hljs-([a-z_.]+)">${word}</span>`)
  );
  return match?.[1];
}

describe('the language registers and highlights', () => {
  test('it is a registered language', () => {
    assert.ok(hljs.getLanguage('etamil'), 'etamil did not register');
    assert.deepEqual(hljs.getLanguage('etamil').aliases, ['etamil', 'qmz']);
  });

  test('Tamil-script keywords are seen as words', () => {
    // The assertion the $pattern exists for. Without a Unicode-aware pattern
    // this returns undefined and the whole language is dead for Tamil source.
    assert.equal(classOf('(அ > 1) எனில் {', 'எனில்'), 'keyword');
    assert.equal(classOf('சுற்று', 'சுற்று'), 'keyword');
    assert.equal(classOf('செயல் f() {', 'செயல்'), 'keyword');
    assert.equal(classOf('இறக்கு "x.qmz";', 'இறக்கு'), 'keyword');
  });

  test('romanized keywords use the current Z scheme', () => {
    assert.equal(classOf('(a > 1) eZil {', 'eZil'), 'keyword');
    // `enil` is the superseded spelling and the compiler rejects it, so it
    // must not be highlighted as though it were valid.
    assert.notEqual(classOf('(a > 1) enil {', 'enil'), 'keyword');
  });

  test('types and literals are distinguished from keywords', () => {
    assert.equal(classOf('எண் x = 1;', 'எண்'), 'type');
    assert.equal(classOf('ஈர்ம b = மெய்;', 'ஈர்ம'), 'type');
    assert.equal(classOf('அச்சு மெய்;', 'மெய்'), 'literal');
    assert.equal(classOf('அச்சு இன்மை;', 'இன்மை'), 'literal');
  });

  test('builtins and library functions are marked as builtins', () => {
    assert.equal(classOf('அச்சு நீளம்("abc");', 'நீளம்'), 'built_in');
    assert.equal(classOf('அச்சு ரூபாய்(1);', 'ரூபாய்'), 'built_in');
  });

  test('domain vocabulary is not syntax', () => {
    // README is explicit that தொகை is a perfectly good name for an amount, so
    // it must not read as a reserved word.
    const kind = classOf('தொகை = 1500;', 'தொகை');
    assert.notEqual(kind, 'keyword');
    assert.notEqual(kind, 'type');
  });

  test('a percentage literal is a number', () => {
    // 18% is exactly 0.18 in this language, which is the point of it.
    assert.ok(classesIn('வரி = 18%;').has('number'));
    assert.ok(classesIn('அச்சு 99.99;').has('number'));
  });

  test('comments and strings', () => {
    assert.ok(classesIn('// வணக்கம்').has('comment'));
    assert.ok(classesIn('அச்சு "வணக்கம்";').has('string'));
    // There is no block comment in the language, so /* */ must not become one.
    assert.ok(!classesIn('/* not a comment */').has('comment'));
  });

  test('a realistic program highlights without swallowing itself', () => {
    const program = [
      '// வருமான வரி',
      'எண் வருவாய் = 950000;',
      'வரி_வீதம் = 20%;',
      '(வருவாய் > 800000) எனில் {',
      '    அச்சு "Tax: " & வட்டமிடு((வருவாய் - 800000) * வரி_வீதம், 2);',
      '}',
      'இன்றேல் {',
      '    அச்சு "No tax";',
      '}',
    ].join('\n');

    const result = hljs.highlight(program, { language: 'etamil' });
    const classes = new Set(
      [...result.value.matchAll(/class="hljs-([a-z_.]+)"/g)].map((m) => m[1])
    );

    for (const expected of ['comment', 'type', 'keyword', 'number', 'string', 'built_in']) {
      assert.ok(classes.has(expected), `no ${expected} found; got ${[...classes]}`);
    }
    // Auto-detection should also pick it over other languages for this text.
    assert.ok(result.relevance > 0, 'the language scored zero relevance');
  });
});
