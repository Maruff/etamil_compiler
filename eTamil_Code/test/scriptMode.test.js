// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Mohammed Maruff (Esan Maruff) <esan@etamil.in>
//
// The eTamil/ISO switch, exercised against a stub VS Code.
//
// `src/scriptMode.ts` is the one module here that has to import `vscode`, and
// `node --test` cannot load that module — it exists only inside the editor's
// extension host. So the import is intercepted and answered with the small
// part of the API this module actually touches. What is tested is the
// behaviour that is easy to get wrong and invisible until someone complains:
// which way the switch writes, where it writes, and whether a font stack the
// author chose by hand survives being switched off and on.

const assert = require('node:assert/strict');
const Module = require('node:module');
const path = require('node:path');
const { test } = require('node:test');

const { FONT_FAMILY } = require('../out/bundle');

const MODULE = path.join(__dirname, '..', 'out', 'scriptMode.js');

/** Enough of the VS Code API for this module, and a way to look at it after. */
function stubVSCode({ languageId = 'etamil', settings = {}, failWrites = false } = {}) {
  const item = {
    text: '',
    tooltip: '',
    name: '',
    command: '',
    visible: false,
    show() {
      this.visible = true;
    },
    hide() {
      this.visible = false;
    },
    dispose() {},
  };

  const state = new Map();
  const commands = new Map();
  const listeners = { editor: [], config: [] };
  const errors = [];
  const values = { ...settings };

  const vscode = {
    StatusBarAlignment: { Left: 1, Right: 2 },
    ConfigurationTarget: { Global: 1, Workspace: 2, WorkspaceFolder: 3 },
    Disposable: class {
      constructor(fn) {
        this.dispose = fn ?? (() => {});
      }
    },
    workspace: {
      getConfiguration: () => ({
        get: (key, fallback) => (key in values ? values[key] : fallback),
        update: async (key, value, target) => {
          if (failWrites) {
            throw new Error('Unable to write to User Settings');
          }
          assert.equal(target, vscode.ConfigurationTarget.Global, 'must write Global');
          values[key] = value;
          // The editor fires this; so does hand-editing settings.json.
          for (const fn of listeners.config) {
            fn({ affectsConfiguration: (section) => section === `etamil.${key}` });
          }
        },
      }),
      onDidChangeConfiguration: (fn) => {
        listeners.config.push(fn);
        return new vscode.Disposable();
      },
    },
    window: {
      activeTextEditor: languageId ? { document: { languageId } } : undefined,
      createStatusBarItem: () => item,
      onDidChangeActiveTextEditor: (fn) => {
        listeners.editor.push(fn);
        return new vscode.Disposable();
      },
      showErrorMessage: (message) => {
        errors.push(message);
        return Promise.resolve(undefined);
      },
    },
    commands: {
      registerCommand: (id, fn) => {
        commands.set(id, fn);
        return new vscode.Disposable();
      },
    },
  };

  const context = {
    subscriptions: [],
    globalState: {
      get: (key, fallback) => (state.has(key) ? state.get(key) : fallback),
      update: async (key, value) => void state.set(key, value),
    },
  };

  const output = { info() {}, warn() {}, error() {} };

  return { vscode, context, output, item, commands, listeners, values, errors, state };
}

/** Load scriptMode.js fresh, with `require('vscode')` answered by the stub. */
function load(stub) {
  const original = Module._load;
  Module._load = function (request, ...rest) {
    if (request === 'vscode') {
      return stub.vscode;
    }
    return original.call(this, request, ...rest);
  };
  try {
    delete require.cache[require.resolve(MODULE)];
    const { registerScriptMode } = require(MODULE);
    registerScriptMode(stub.context, stub.output);
  } finally {
    Module._load = original;
  }
  return stub;
}

const toggle = (stub) => stub.commands.get('etamil.toggleScriptFont')();

test('the command is registered under the id package.json contributes', () => {
  const stub = load(stubVSCode());
  assert.ok(stub.commands.has('etamil.toggleScriptFont'));
});

test('reads ISO when the setting is empty, and eTamil when it is not', () => {
  assert.equal(load(stubVSCode()).item.text, 'ISO');
  const on = load(stubVSCode({ settings: { eTamilFont: FONT_FAMILY } }));
  assert.equal(on.item.text, 'eTamil');
});

test('shown for an eTamil file, hidden for anything else', () => {
  assert.equal(load(stubVSCode()).item.visible, true);
  assert.equal(load(stubVSCode({ languageId: 'python' })).item.visible, false);
});

test('hides and shows as the active editor changes', () => {
  const stub = load(stubVSCode());
  const [onEditor] = stub.listeners.editor;

  onEditor({ document: { languageId: 'markdown' } });
  assert.equal(stub.item.visible, false);
  onEditor({ document: { languageId: 'etamil' } });
  assert.equal(stub.item.visible, true);
  // No editor at all, which VS Code reports as undefined.
  onEditor(undefined);
  assert.equal(stub.item.visible, false);
});

test('switching on writes the carried font, switching off writes empty', async () => {
  const stub = load(stubVSCode());

  await toggle(stub);
  assert.equal(stub.values.eTamilFont, FONT_FAMILY);
  assert.equal(stub.item.text, 'eTamil');

  await toggle(stub);
  assert.equal(stub.values.eTamilFont, '');
  assert.equal(stub.item.text, 'ISO');
});

test('a hand-chosen font stack survives being switched off and on', async () => {
  const mine = 'ican qamiz Smart, Cascadia Code';
  const stub = load(stubVSCode({ settings: { eTamilFont: mine } }));

  await toggle(stub);
  assert.equal(stub.values.eTamilFont, '');
  await toggle(stub);
  // Not FONT_FAMILY: substituting the carried face here would quietly discard
  // the author's choice, which is the whole reason for the remembered value.
  assert.equal(stub.values.eTamilFont, mine);
});

test('editing settings.json by hand moves the label too', () => {
  const stub = load(stubVSCode());
  const [onConfig] = stub.listeners.config;

  stub.values.eTamilFont = FONT_FAMILY;
  onConfig({ affectsConfiguration: (section) => section === 'etamil.eTamilFont' });
  assert.equal(stub.item.text, 'eTamil');

  // An unrelated setting changing must not be mistaken for this one.
  stub.values.eTamilFont = '';
  onConfig({ affectsConfiguration: () => false });
  assert.equal(stub.item.text, 'eTamil');
});

test('a refused write is reported rather than silently doing nothing', async () => {
  const stub = load(stubVSCode({ failWrites: true }));

  await toggle(stub);
  assert.equal(stub.values.eTamilFont, undefined, 'nothing was written');
  assert.equal(stub.errors.length, 1);
  assert.match(stub.errors[0], /could not change etamil\.eTamilFont/);
  assert.equal(stub.item.text, 'ISO', 'the label still reflects the real setting');
});
