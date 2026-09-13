// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Mohammed Maruff (Esan Maruff) <esan@etamil.in>
//
// Drawing eTamil-script ASCII in the eTamil font.
//
// The eTamil font maps the ASCII letters onto Tamil glyphs — `c` draws ச, `q`
// draws த — so that Tamil can be typed on an ASCII keyboard and read back as
// Tamil. It is optional, and it cannot be the whole editor's font, because a
// file also contains English: names marked with a leading `_`, comments
// wrapped in `__ … __`, string literals, the licence header. Those are the
// places `docs/reference/SCRIPT_RULES.md` calls ISO, and they have to keep
// drawing as Latin.
//
// **The editor's own font stays the ISO one.** This module paints the other
// direction: it finds the ASCII that is eTamil script and draws only that in
// the eTamil face. Two reasons, and the second is the important one.
//
// VS Code cannot set a font family from a theme — `editor.tokenColorCustom-
// izations` carries `fontStyle`, which is bold, italic and underline, and
// nothing else. A decoration can, because its `textDecoration` is injected as
// CSS and a font family can ride along behind it. That is the only lever there
// is, and it is worth knowing it is a lever and not an API.
//
// And painting the eTamil side rather than the ISO side means a span this
// misses renders eTamil as plain Latin — the ordinary view of the file, no
// worse than not having the font. Painted the other way, a miss would render
// English in Tamil glyphs, which is unreadable. Given a mechanism this
// informal, the failure has to fall on the harmless side.

import * as vscode from 'vscode';

import { scanETamilScript } from './marks';

const LANGUAGE = 'etamil';

/**
 * What may appear in the font setting.
 *
 * The value is interpolated into CSS, so it is checked rather than trusted:
 * letters, digits, spaces, hyphens, underscores, quotes and commas, which is a
 * CSS font stack and nothing else. A `;` or a `}` would let a font name close
 * the declaration and open another, and although the setting is machine-scoped
 * — a workspace cannot set it — the check costs one regex and removes the
 * question entirely.
 */
const FONT_STACK = /^[A-Za-z0-9 _'",-]+$/;

/** How long to wait after a keystroke before repainting. */
const REPAINT_DEBOUNCE_MS = 120;

export function registerScriptFont(
  context: vscode.ExtensionContext,
  output: vscode.LogOutputChannel
): void {
  let decoration: vscode.TextEditorDecorationType | undefined;
  let family: string | undefined;
  let timer: NodeJS.Timeout | undefined;

  const configure = () => {
    const setting = vscode.workspace
      .getConfiguration('etamil')
      .get<string>('eTamilFont', '')
      .trim();

    if (setting === family) {
      return;
    }

    decoration?.dispose();
    decoration = undefined;
    family = setting;

    if (!setting) {
      return;
    }
    if (!FONT_STACK.test(setting)) {
      output.warn(
        `etamil.eTamilFont: '${setting}' is not a font stack — ` +
          "letters, digits, spaces, quotes, commas, '-' and '_' only. Ignored."
      );
      family = '';
      return;
    }

    // `textDecoration` is emitted into the decoration's CSS verbatim. The
    // leading `none` closes the property it was meant for and leaves the font
    // family as a second declaration. Unofficial, and the only way there is.
    decoration = vscode.window.createTextEditorDecorationType({
      textDecoration: `none; font-family: ${setting}`,
    });
    output.info(`eTamil script font: ${setting}`);
  };

  const paint = (editor: vscode.TextEditor | undefined) => {
    if (!editor || editor.document.languageId !== LANGUAGE) {
      return;
    }
    if (!decoration) {
      return;
    }
    const ranges = scanETamilScript(editor.document.getText()).map(
      (span) =>
        new vscode.Range(
          new vscode.Position(span.line, span.start),
          new vscode.Position(span.line, span.end)
        )
    );
    editor.setDecorations(decoration, ranges);
  };

  const paintAll = () => {
    for (const editor of vscode.window.visibleTextEditors) {
      paint(editor);
    }
  };

  const schedule = () => {
    if (timer) {
      clearTimeout(timer);
    }
    timer = setTimeout(() => {
      timer = undefined;
      paintAll();
    }, REPAINT_DEBOUNCE_MS);
  };

  configure();
  paintAll();

  context.subscriptions.push(
    vscode.workspace.onDidChangeConfiguration((event) => {
      if (event.affectsConfiguration('etamil.eTamilFont')) {
        configure();
        paintAll();
      }
    }),
    vscode.window.onDidChangeVisibleTextEditors(() => paintAll()),
    vscode.workspace.onDidChangeTextDocument((event) => {
      if (event.document.languageId === LANGUAGE) {
        schedule();
      }
    }),
    new vscode.Disposable(() => {
      if (timer) {
        clearTimeout(timer);
      }
      decoration?.dispose();
    })
  );
}
