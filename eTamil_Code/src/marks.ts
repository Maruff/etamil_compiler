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
// Two more things stay in the ISO font without being marked, because they are
// data rather than language: a string literal, and a name reached through `.`
// — a field name, or the extension in `paNam.qmz`. Those are view conditions
// and nothing more. Nothing is added to the file for them, and
// `scripts/check_script_rules.py` asks nothing of them.
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

/** Rule 2's mark. One opens an English region and the next one closes it. */
const MARK = '__';

/** What precedes a field name, an extension, a dotted name. */
const DOT = '.';

/**
 * Every span of ASCII that should be drawn in the eTamil font.
 *
 * Left out, and so drawn in the editor's own ISO font:
 *
 *   - anything inside a string literal, which is data and carries no marks
 *   - an identifier beginning with `_`, in whole
 *   - a name immediately preceded by `.` — a field name, an extension
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
  // So does an English comment. A sentence is longer than a line and an author
  // writes it as one — `__` on the first line and `__` on the last, which is
  // the form SCRIPT_RULES.md's own Rule 2 example uses. Scanning each line on
  // its own left both of those lines holding a single unmatched mark, so
  // neither counted and every English word between them was painted as Tamil.
  // That is the failure this module exists to prevent, not a missed span.
  let inEnglish = false;

  lines.forEach((line, lineNumber) => {
    let index = 0;
    let commented = false;

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
        inEnglish = comment(spans, lineNumber, line, index + 2, inEnglish);
        commented = true;
        break;
      }

      if (IDENT_START.test(character)) {
        let end = index;
        while (end < line.length && IDENT_PART.test(line[end])) {
          end += 1;
        }
        // Rule 1: the mark is on the identifier, so the whole of a marked name
        // stays in the ISO font — including `_மொத்த_cgst`, where only part of
        // it is Latin.
        //
        // A name reached through `.` stays in the ISO font too, and for the
        // reason strings do: a field name is data, not a language construct —
        // parser.rs says so where it reads one. No mark is added to the file
        // for this and none is required; it decides rendering and nothing
        // else. `line[-1]` is undefined at the start of a line, not `.`.
        if (line[index] !== '_' && line[index - 1] !== DOT) {
          latinRuns(spans, lineNumber, line.slice(index, end), index);
        }
        index = end;
        continue;
      }

      index += 1;
    }

    // A block is contiguous `//` lines, so the region ends at the first line
    // without a comment on it. That keeps one unclosed `__` from making the
    // remainder of the file English — and were it carried instead, the file
    // would merely render eTamil as Latin, so the failure stays harmless
    // either way.
    if (!commented) {
      inEnglish = false;
    }
  });

  return spans;
}

/**
 * The eTamil-script part of one comment, and whether English is still open.
 *
 * `inEnglish` comes in as the state the previous comment line left and goes
 * out as the state this one leaves, so `__` opens a region that survives to
 * the line carrying the closing mark. A line that opens and closes is the
 * ordinary single-line case and needs no special handling: the same toggle
 * covers both.
 */
function comment(
  spans: Span[],
  lineNumber: number,
  line: string,
  from: number,
  inEnglish: boolean
): boolean {
  const body = line.slice(from);
  // Exempt, and deliberately state-neutral: the header sits above everything
  // and must not open or close a region for the code below it.
  if (HEADER.test(body.trim())) {
    return inEnglish;
  }

  // Where English runs on this line. An open region that this line does not
  // close reaches the end of it and continues on the next.
  const english: Array<[number, number]> = [];
  let inside = inEnglish;
  let openedAt = inside ? 0 : -1;

  for (let index = 0; index < body.length; ) {
    if (body.startsWith(MARK, index)) {
      if (inside) {
        english.push([openedAt, index + MARK.length]);
        inside = false;
      } else {
        openedAt = index;
        inside = true;
      }
      index += MARK.length;
      continue;
    }
    index += 1;
  }
  if (inside) {
    english.push([openedAt, body.length]);
  }

  LATIN.lastIndex = 0;
  for (let match = LATIN.exec(body); match; match = LATIN.exec(body)) {
    const runStart = match.index;
    const runEnd = runStart + match[0].length;
    const isEnglish = english.some(([open, close]) => runStart >= open && runEnd <= close);
    // The same, and in a comment it is what the condition is mostly for: an
    // extension or a dotted name — `.qmz`, `.gitignore`, `paNam.qmz` — is
    // English however the words around it are spelled.
    const afterDot = body[runStart - 1] === DOT;
    if (!isEnglish && !afterDot) {
      spans.push({ line: lineNumber, start: from + runStart, end: from + runEnd });
    }
  }

  return inside;
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
