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

import { FONT_FAMILY, FONT_FEATURES, isFontFeatureSettings } from './bundle';
import { fontInstalled } from './fontinstall';
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

// Why the OpenType features are declared here and not in `editor.fontLigatures`.
//
// A smart build of the eTamil font carries contextual rules in `calt` — the
// pulli appears on a consonant that no vowel follows, a vowel after a consonant
// shrinks to its sign, the inherent `a` draws nothing. VS Code's
// `editor.fontLigatures` is `false` by default, and while it is false the
// editor emits `font-feature-settings: "liga" 0, "calt" 0`, which turns those
// rules off.
//
// Setting that option would switch ligatures on for every font in every file
// the user opens, which is not this extension's business and is not something
// it should do to a machine-wide setting on their behalf. The font here is
// already applied through a decoration whose CSS this module writes, so the
// features ride in the same declaration: a declared value beats an inherited
// one, the editor's own setting is left alone, and nothing outside the
// eTamil-script spans is touched.

/** How long to wait after a keystroke before repainting. */
const REPAINT_DEBOUNCE_MS = 120;

/** Say so, once, when the setting names a font that is not on the machine. */
let offered = false;

async function offerToInstall(output: vscode.LogOutputChannel): Promise<void> {
  if (offered) {
    return;
  }
  offered = true;
  output.warn(`${FONT_FAMILY} is not installed on this machine`);
  const install = 'Install it';
  const picked = await vscode.window.showWarningMessage(
    `eTamil: etamil.eTamilFont names ${FONT_FAMILY}, which is not installed, ` +
      'so nothing will look any different.',
    install
  );
  if (picked === install) {
    await vscode.commands.executeCommand('etamil.installFont');
  }
}

export function registerScriptFont(
  context: vscode.ExtensionContext,
  output: vscode.LogOutputChannel
): void {
  let decoration: vscode.TextEditorDecorationType | undefined;
  let family: string | undefined;
  let applied: string | undefined;
  let timer: NodeJS.Timeout | undefined;

  const configure = () => {
    const config = vscode.workspace.getConfiguration('etamil');
    const setting = config.get<string>('eTamilFont', '').trim();
    const requested = config.get<string>('eTamilFontFeatures', FONT_FEATURES).trim();

    // Either half changing means the decoration has to be rebuilt.
    let features = requested;
    if (!isFontFeatureSettings(features)) {
      output.warn(
        `etamil.eTamilFontFeatures: '${features}' is not a ` +
          "font-feature-settings value — 'normal', or quoted four-letter tags " +
          "each optionally followed by on, off or a number. Ignored."
      );
      features = '';
    }

    if (setting === family && features === applied) {
      return;
    }

    decoration?.dispose();
    decoration = undefined;
    family = setting;
    applied = features;

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
    // The features follow as a third, for the reason given above FONT_STACK.
    const css =
      `none; font-family: ${setting}` +
      (features && features !== 'normal' ? `; font-feature-settings: ${features}` : '');
    decoration = vscode.window.createTextEditorDecorationType({
      textDecoration: css,
    });
    output.info(
      `eTamil script font: ${setting}` +
        (features && features !== 'normal' ? ` (features ${features})` : '')
    );

    // A font family naming a font the machine does not have resolves to
    // nothing and the decoration draws in the editor's font — which looks
    // exactly like the setting having no effect, with no error anywhere. The
    // one case that can be checked is the font this extension carries.
    if (setting === FONT_FAMILY && !fontInstalled()) {
      void offerToInstall(output);
    }
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
      if (
        event.affectsConfiguration('etamil.eTamilFont') ||
        event.affectsConfiguration('etamil.eTamilFontFeatures')
      ) {
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
