// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Mohammed Maruff (Esan Maruff) <esan@etamil.in>
//
// The switch that turns eTamil script rendering on and off.
//
// `src/fonts.ts` paints the ASCII that is eTamil script in the eTamil font and
// leaves everything the marks call ISO in the editor's own face. Whether it
// paints at all is decided by one setting, `etamil.eTamilFont`, which is empty
// by default. This module is the button for it: a status bar item that both
// shows which way the switch is set and flips it when clicked.
//
// Nothing here repaints anything. `registerScriptFont` already listens for
// `etamil.eTamilFont` changing and rebuilds its decoration, so writing the
// setting is the whole of the work — and it means the button, the command
// palette and hand-editing settings.json all go down the same path.
//
// Two decisions worth knowing.
//
// **The previous font is remembered.** Turning the switch off writes `''`,
// which would otherwise discard a font stack the author had chosen by hand;
// turning it back on would then silently substitute the bundled face. So the
// outgoing value is kept in global state and restored on the way back.
//
// **The write is `Global`.** `etamil.eTamilFont` is declared `scope: machine`
// — deliberately, because its value is interpolated into the decoration's CSS
// and a workspace should not be able to choose it — so `Workspace` is refused.

import * as vscode from 'vscode';

import { FONT_FAMILY } from './bundle';

const LANGUAGE = 'etamil';

/** Where the font stack that was switched off is kept. */
const REMEMBERED = 'etamil.previousScriptFont';

/**
 * To the left of the language indicator, which is the neighbourhood VS Code
 * uses for things that describe how the current file is being read.
 */
const PRIORITY = 100;

/** The font this extension carries, or whichever one was switched off last. */
function fontToRestore(context: vscode.ExtensionContext): string {
  const remembered = context.globalState.get<string>(REMEMBERED, '').trim();
  return remembered || FONT_FAMILY;
}

export function registerScriptMode(
  context: vscode.ExtensionContext,
  output: vscode.LogOutputChannel
): void {
  const item = vscode.window.createStatusBarItem(
    'etamil.scriptMode',
    vscode.StatusBarAlignment.Right,
    PRIORITY
  );
  item.name = 'eTamil script';
  item.command = 'etamil.toggleScriptFont';

  const current = () =>
    vscode.workspace.getConfiguration('etamil').get<string>('eTamilFont', '').trim();

  const render = () => {
    const family = current();
    // The label says which script the ASCII is being drawn as, not whether a
    // setting is set — "ISO" is the state a reader can see on screen, and
    // `etamil.eTamilFont: ""` is not.
    //
    // No codicon. The word *is* the information, the status bar's own
    // neighbours are plain text — line and column, language mode, encoding —
    // and an icon name that does not exist renders as an empty box with no
    // error anywhere.
    item.text = family ? 'eTamil' : 'ISO';
    item.tooltip = family
      ? `eTamil script drawn in ${family}. Click to read it as ISO.`
      : 'ASCII drawn in the editor’s own font. Click to read it as eTamil script.';
  };

  const reveal = (editor: vscode.TextEditor | undefined) => {
    if (editor?.document.languageId === LANGUAGE) {
      item.show();
    } else {
      item.hide();
    }
  };

  const toggle = async () => {
    const family = current();
    const config = vscode.workspace.getConfiguration('etamil');
    try {
      if (family) {
        await context.globalState.update(REMEMBERED, family);
        await config.update('eTamilFont', '', vscode.ConfigurationTarget.Global);
        output.info('eTamil script font off — ASCII draws as ISO');
      } else {
        const restoring = fontToRestore(context);
        await config.update('eTamilFont', restoring, vscode.ConfigurationTarget.Global);
        output.info(`eTamil script font on — ${restoring}`);
      }
    } catch (error) {
      // A machine-scoped setting cannot be written when VS Code is running
      // with user settings it does not own, which is worth saying rather than
      // leaving a button that appears to do nothing.
      const message = error instanceof Error ? error.message : String(error);
      output.error(`etamil.eTamilFont could not be written: ${message}`);
      void vscode.window.showErrorMessage(
        `eTamil: could not change etamil.eTamilFont — ${message}`
      );
    }
    // Not strictly needed: the configuration listener below repaints too. It
    // is here so the label changes with the click rather than with the event.
    render();
  };

  render();
  reveal(vscode.window.activeTextEditor);

  context.subscriptions.push(
    item,
    vscode.commands.registerCommand('etamil.toggleScriptFont', toggle),
    vscode.window.onDidChangeActiveTextEditor((editor) => reveal(editor)),
    vscode.workspace.onDidChangeConfiguration((event) => {
      // Hand-editing settings.json has to move the label too.
      if (event.affectsConfiguration('etamil.eTamilFont')) {
        render();
      }
    })
  );
}
