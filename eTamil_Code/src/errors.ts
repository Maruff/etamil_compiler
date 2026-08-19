// Reading the compiler's error output.
//
// Kept free of the vscode API so it can be tested against real compiler output
// by plain node — which matters, because the format is the one thing here that
// this extension does not control.

/** One error the front end reported, with the position it carried. */
export interface CompilerError {
  /** 1-based, as the compiler reports it. */
  line: number;
  /** 1-based, counted in Unicode code points. See toPosition in compiler.ts. */
  column: number;
  message: string;
}

/**
 * A positioned error line, in either spelling.
 *
 * The compiler prints each error twice on one line — Tamil first, then the same
 * thing in English in parentheses. Both openings are matched, in case the
 * bilingual pairing ever changes, and the whole remainder is kept as the
 * message: an author reading their own diagnostics should see them in the
 * language they are writing in.
 */
const POSITIONED =
  /^(?:வரி|line)\s+(\d+),\s*(?:நெடுவரிசை|column)\s+(\d+):\s*(.+)$/u;

/** Strip the leading marker the compiler puts on every error line. */
function withoutMarker(line: string): string {
  return line.replace(/^✗\s*/u, '');
}

/**
 * Every error in a run of the compiler's stderr.
 *
 * Lines without a position — a module that cannot be found, a lexical failure
 * reported for the file as a whole — are anchored to line 1 rather than
 * dropped. A missing import is the single most common reason a file will not
 * compile at all, and silently discarding it would leave the editor showing no
 * errors for a file that does not build.
 */
export function parseErrors(stderr: string): CompilerError[] {
  const errors: CompilerError[] = [];

  for (const raw of stderr.split(/\r?\n/)) {
    const line = raw.trim();
    if (line.length === 0) {
      continue;
    }

    const positioned = POSITIONED.exec(withoutMarker(line));
    if (positioned) {
      errors.push({
        line: Number(positioned[1]),
        column: Number(positioned[2]),
        message: positioned[3].trim(),
      });
      continue;
    }

    if (line.startsWith('✗')) {
      errors.push({ line: 1, column: 1, message: withoutMarker(line).trim() });
    }
  }

  return errors;
}
