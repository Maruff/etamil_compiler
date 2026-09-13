// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Mohammed Maruff (Esan Maruff) <esan@etamil.in>
//
// Finding the ASCII that is eTamil rather than English.
//
// eTamil is written three ways and two of them are the same bytes. `செயல்` is
// Tamil; `ceyal` is the same word spelled under the ezuqqu scheme, where one
// Latin letter stands for one Tamil letter; `_length` is English. There is an
// eTamil font in which the ASCII letters carry Tamil glyphs — `c` draws ச, `q`
// draws த — so under it `ceyal` reads as செயல் and `sum` reads as ஸும், which
// is nonsense.
//
// Two marks in the file say which is which, and this module finds the text
// they leave unmarked. `docs/reference/SCRIPT_RULES.md` is the specification:
//
//     Rule 1  an identifier containing English ASCII begins with `_`
//     Rule 2  a comment containing English ASCII is wrapped in `__ … __`
//
// **What is returned is the eTamil-script ASCII, not the English.** The editor
// keeps its ordinary ISO font as the base and paints only these spans in the
// eTamil face. Painting the other way round would mean that any span this
// scanner missed rendered English as Tamil gibberish; this way a miss renders
// eTamil as Latin, which is merely the ordinary view of the file.
//
// Nothing here imports the vscode API, so test/marks.test.js can load it.

/** A run of characters on one line, in UTF-16 offsets, as VS Code counts. */
export interface Span {
  line: number;
  start: number;
  end: number;
}

/** Latin letters. Digits are not remapped by the font and are left alone. */
const LATIN = /[A-Za-z]+/g;

/** The identifier shape the lexer accepts. */
const IDENT_START = /[A-Za-z_஀-௿]/;
const IDENT_PART = /[A-Za-z0-9_஀-௿]/;

/**
 * The two licence header lines, which Rule 2 exempts.
 *
 * A licence scanner reads the SPDX expression to the end of the line, so the
 * closing `__` would become part of the licence name. They carry no marks and
 * are English regardless.
 */
const HEADER = /^(?:SPDX-[A-Za-z-]+:|Copyright\s*\(C\))/;

/** `__ … __` inside a comment. Non-greedy: two marked runs on one line are two. */
const MARKED = /__.+?__/g;

/**
 * Every span of ASCII that should be drawn in the eTamil font.
 *
 * Left out, and so drawn in the editor's own ISO font:
 *
 *   - anything inside a string literal, which is data and carries no marks
 *   - an identifier beginning with `_`, in whole
 *   - comment text between `__` and `__`
 *   - the licence header
 *   - Unicode Tamil, which both fonts draw as Tamil, so the choice is moot
 */
export function scanETamilScript(text: string): Span[] {
  const spans: Span[] = [];
  const lines = text.split('\n');
  // Strings may span lines: the lexer's `"([^"\\]|\\.)*"` does not exclude a
  // newline. Carrying the state means an unterminated quote does not start
  // painting the rest of the file.
  let inString = false;

  lines.forEach((line, lineNumber) => {
    let index = 0;

    while (index < line.length) {
      if (inString) {
        if (line[index] === '\\') {
          index += 2;
          continue;
        }
        if (line[index] === '"') {
          inString = false;
        }
        index += 1;
        continue;
      }

      const character = line[index];

      if (character === '"') {
        inString = true;
        index += 1;
        continue;
      }

      if (character === '/' && line[index + 1] === '/') {
        comment(spans, lineNumber, line, index + 2);
        return;
      }

      if (IDENT_START.test(character)) {
        let end = index;
        while (end < line.length && IDENT_PART.test(line[end])) {
          end += 1;
        }
        // Rule 1: the mark is on the identifier, so the whole of a marked name
        // stays in the ISO font — including `_மொத்த_cgst`, where only part of
        // it is Latin.
        if (line[index] !== '_') {
          latinRuns(spans, lineNumber, line.slice(index, end), index);
        }
        index = end;
        continue;
      }

      index += 1;
    }
  });

  return spans;
}

/** The eTamil-script part of one comment, which runs to the end of the line. */
function comment(spans: Span[], lineNumber: number, line: string, from: number): void {
  const body = line.slice(from);
  if (HEADER.test(body.trim())) {
    return;
  }

  const marked: Array<[number, number]> = [];
  MARKED.lastIndex = 0;
  for (let match = MARKED.exec(body); match; match = MARKED.exec(body)) {
    marked.push([match.index, match.index + match[0].length]);
  }

  LATIN.lastIndex = 0;
  for (let match = LATIN.exec(body); match; match = LATIN.exec(body)) {
    const start = match.index;
    const end = start + match[0].length;
    const inside = marked.some(([open, close]) => start >= open && end <= close);
    if (!inside) {
      spans.push({ line: lineNumber, start: from + start, end: from + end });
    }
  }
}

/** The Latin letters inside one unmarked identifier. */
function latinRuns(
  spans: Span[],
  lineNumber: number,
  token: string,
  offset: number
): void {
  LATIN.lastIndex = 0;
  for (let match = LATIN.exec(token); match; match = LATIN.exec(token)) {
    spans.push({
      line: lineNumber,
      start: offset + match.index,
      end: offset + match.index + match[0].length,
    });
  }
}
