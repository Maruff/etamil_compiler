// Language services built on the generated data.
//
// Everything here reads KEYWORDS and FUNCTIONS from src/generated, which are
// produced from lexer.rs, parser.rs, interpreter.rs and nUlakam by
// scripts/generate_editor_support.py. Nothing in this file restates a keyword
// or a spelling — that is what drifted last time, until a third of the
// language was missing from the editor and the romanized spellings it did
// offer were ones the compiler rejects.

import * as path from 'path';
import * as vscode from 'vscode';

import {
  FUNCTIONS,
  IDENTIFIER_SOURCE,
  KEYWORDS,
  type FunctionEntry,
  type KeywordEntry,
} from './generated/language-data';

const IDENTIFIER = new RegExp(IDENTIFIER_SOURCE, 'u');

/** Does this spelling use Tamil script? */
function isTamil(text: string): boolean {
  return /[஀-௿]/u.test(text);
}

/**
 * Variables the HTTP server binds into every route handler.
 *
 * These are not keywords — `handler::bind_request` sets them as plain
 * variables, deliberately, so a handler reads the same in either spelling.
 * They are not in the generated data because they are not in the lexer, but an
 * author writing a route needs them more than almost anything else.
 */
const REQUEST_BINDINGS: ReadonlyArray<{ name: string; detail: string }> = [
  { name: 'request_method', detail: 'GET, POST, … as sent' },
  { name: 'request_path', detail: 'the path, without the query string' },
  { name: 'request_body', detail: 'the body as text — parse it with ஜேசான்_படி' },
  { name: 'query_params', detail: 'record of query parameters, percent-decoded' },
  { name: 'headers', detail: 'record of request headers, names lower-cased' },
  { name: 'path_params', detail: 'record of :name segments from the route pattern' },
  { name: 'response_status', detail: 'set by பதில் — the status to send' },
  { name: 'response_body', detail: 'set by பதில் — the body to send' },
  { name: 'response_headers', detail: 'set by பதில் — a record of headers' },
];

// ---------------------------------------------------------------------------
// Documentation
// ---------------------------------------------------------------------------

/** How a keyword's spellings read in documentation: Tamil · romanized · English. */
function spellings(entry: KeywordEntry): string {
  return entry.forms.map((form) => `\`${form}\``).join(' · ');
}

function keywordDocs(entry: KeywordEntry): vscode.MarkdownString {
  const docs = new vscode.MarkdownString();
  docs.appendMarkdown(`**${entry.forms[0]}** — ${entry.group.toLowerCase()}\n\n`);
  docs.appendMarkdown(`${spellings(entry)}\n\n`);

  if (entry.noSyntax) {
    // Told plainly, because the alternative is an author spending an afternoon
    // on a word that can only ever produce "expected a statement".
    docs.appendMarkdown(
      `⚠️ Reserved, but no statement uses it yet — writing it is always an error. ` +
        `It cannot be used as a variable name either.\n\n`
    );
  } else if (entry.reserved) {
    docs.appendMarkdown(`Reserved — this word cannot be used as a name.\n\n`);
  } else {
    docs.appendMarkdown(
      `Domain vocabulary: this word is **not** reserved, so it is also a ` +
        `perfectly good variable or field name.\n\n`
    );
  }

  return docs;
}

function signature(entry: FunctionEntry): string {
  if (entry.params) {
    return `${entry.name}(${entry.params.join(', ')})`;
  }
  if (entry.arity === 0) {
    return `${entry.name}()`;
  }
  if (entry.arity !== null) {
    const slots = Array.from({ length: entry.arity }, (_, i) => `அளவு${i + 1}`);
    return `${entry.name}(${slots.join(', ')})`;
  }
  return `${entry.name}(…)`;
}

function functionDocs(entry: FunctionEntry): vscode.MarkdownString {
  const docs = new vscode.MarkdownString();
  docs.appendCodeblock(signature(entry), 'etamil');

  if (entry.doc) {
    docs.appendMarkdown(`\n${entry.doc}\n`);
  }

  if (entry.kind === 'builtin') {
    docs.appendMarkdown(
      `\n\nProvided by the host — the decimal arithmetic, text measurement and ` +
        `crypto a language cannot express in itself.`
    );
    if (entry.forms.length > 1) {
      docs.appendMarkdown(`\n\nAlso spelled ${spellings(entry as never)}`);
    }
  } else if (entry.module) {
    docs.appendMarkdown(`\n\nWritten in eTamil, in \`${entry.module}\`.`);
    docs.appendMarkdown(`\n\n\`\`\`etamil\nஇறக்கு "${entry.module}";\n\`\`\``);
  }

  return docs;
}

// ---------------------------------------------------------------------------
// Completion
// ---------------------------------------------------------------------------

/**
 * Every completion the language offers, built once.
 *
 * The old provider rebuilt ~50 items on every keystroke and returned all of
 * them unfiltered, including bare `+` and `-` as completion items. VS Code
 * does the filtering; the provider's job is to offer a correct, complete set.
 */
function buildCompletions(): vscode.CompletionItem[] {
  const items: vscode.CompletionItem[] = [];

  for (const entry of KEYWORDS) {
    for (const form of entry.forms) {
      const item = new vscode.CompletionItem(form);
      item.detail = entry.group;
      item.documentation = keywordDocs(entry);

      if (entry.noSyntax) {
        // Sorted last and marked, so it is available to look up but never the
        // first thing suggested.
        item.kind = vscode.CompletionItemKind.Text;
        item.sortText = `zzz${form}`;
        item.tags = [vscode.CompletionItemTag.Deprecated];
      } else if (entry.scope.startsWith('storage.type')) {
        item.kind = vscode.CompletionItemKind.TypeParameter;
      } else if (entry.scope.startsWith('constant')) {
        item.kind = vscode.CompletionItemKind.Constant;
      } else if (entry.scope.startsWith('support.type.domain')) {
        item.kind = vscode.CompletionItemKind.Value;
      } else if (entry.scope.startsWith('support.function')) {
        item.kind = vscode.CompletionItemKind.Function;
      } else {
        item.kind = vscode.CompletionItemKind.Keyword;
      }

      // A romanized author gets romanized placeholders. Half of "Tamil
      // semantics you can type on a plain keyboard" is this path, and the
      // previous extension only ever produced Tamil bodies.
      const template = isTamil(form) ? entry.snippetTamil : entry.snippetLatin;
      if (template && !entry.noSyntax) {
        item.insertText = new vscode.SnippetString(template.replace(/\{kw\}/g, form));
      }

      items.push(item);
    }
  }

  for (const entry of FUNCTIONS) {
    for (const form of entry.forms) {
      const item = new vscode.CompletionItem(form, vscode.CompletionItemKind.Function);
      item.detail = entry.kind === 'builtin' ? 'host builtin' : `nUlakam — ${entry.module}`;
      item.documentation = functionDocs(entry);

      const argc = entry.params?.length ?? entry.arity ?? 0;
      if (argc === 0) {
        item.insertText = new vscode.SnippetString(`${form}()`);
      } else {
        const slots = (entry.params ?? Array.from({ length: argc }, () => 'அளவு'))
          .map((name, index) => `\${${index + 1}:${name}}`)
          .join(', ');
        item.insertText = new vscode.SnippetString(`${form}(${slots})`);
      }

      // Builtins before library functions, both before domain nouns.
      item.sortText = entry.kind === 'builtin' ? `1${form}` : `2${form}`;
      items.push(item);
    }
  }

  return items;
}

let cachedCompletions: vscode.CompletionItem[] | undefined;

function allCompletions(): vscode.CompletionItem[] {
  if (!cachedCompletions) {
    cachedCompletions = buildCompletions();
  }
  return cachedCompletions;
}

/** Completions for the request variables, offered only inside a route handler. */
function requestCompletions(): vscode.CompletionItem[] {
  return REQUEST_BINDINGS.map(({ name, detail }) => {
    const item = new vscode.CompletionItem(name, vscode.CompletionItemKind.Variable);
    item.detail = detail;
    item.documentation = new vscode.MarkdownString(
      'Bound into every route handler by the server. Not a keyword — an ordinary ' +
        'variable, so it reads the same in Tamil or romanized source.'
    );
    item.sortText = `0${name}`;
    return item;
  });
}

/** Is this position inside a `வழி` route handler? */
function insideRoute(document: vscode.TextDocument, position: vscode.Position): boolean {
  const routeForms = KEYWORDS.find((entry) => entry.token === 'Route')?.forms ?? [];
  const pattern = new RegExp(`(?:${routeForms.join('|')})\\b`, 'u');

  let depth = 0;
  for (let line = position.line; line >= 0; line -= 1) {
    const text = line === position.line
      ? document.lineAt(line).text.slice(0, position.character)
      : document.lineAt(line).text;

    for (const character of [...text].reverse()) {
      if (character === '}') {
        depth += 1;
      } else if (character === '{') {
        if (depth === 0) {
          // Found the brace that opens the enclosing block.
          return pattern.test(document.lineAt(line).text);
        }
        depth -= 1;
      }
    }
  }
  return false;
}

export function completionProvider(): vscode.CompletionItemProvider {
  return {
    provideCompletionItems(document, position) {
      const items = [...allCompletions()];
      if (insideRoute(document, position)) {
        items.push(...requestCompletions());
      }
      // Functions defined in this file, so a program's own vocabulary
      // completes alongside the language's.
      items.push(...localFunctionCompletions(document));
      return items;
    },
  };
}

const DEFINITION = new RegExp(
  `^\\s*(?:${(KEYWORDS.find((e) => e.token === 'Function')?.forms ?? ['செயல்'])
    .map((f) => f.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'))
    .join('|')})\\s+(${IDENTIFIER_SOURCE})\\s*\\(([^)]*)`,
  'u'
);

interface LocalFunction {
  name: string;
  params: string[];
  line: number;
}

/** The செயல் definitions in one document. */
export function localFunctions(document: vscode.TextDocument): LocalFunction[] {
  const found: LocalFunction[] = [];

  for (let line = 0; line < document.lineCount; line += 1) {
    const match = DEFINITION.exec(document.lineAt(line).text);
    if (match) {
      found.push({
        name: match[1],
        params: match[2]
          .split(',')
          .map((param) => param.trim())
          .filter((param) => param.length > 0),
        line,
      });
    }
  }

  return found;
}

function localFunctionCompletions(document: vscode.TextDocument): vscode.CompletionItem[] {
  return localFunctions(document).map(({ name, params }) => {
    const item = new vscode.CompletionItem(name, vscode.CompletionItemKind.Function);
    item.detail = 'in this file';
    const slots = params.map((param, index) => `\${${index + 1}:${param}}`).join(', ');
    item.insertText = new vscode.SnippetString(`${name}(${slots})`);
    item.sortText = `0${name}`;
    return item;
  });
}

// ---------------------------------------------------------------------------
// Hover
// ---------------------------------------------------------------------------

export function hoverProvider(): vscode.HoverProvider {
  return {
    provideHover(document, position) {
      const range = document.getWordRangeAtPosition(position, IDENTIFIER);
      if (!range) {
        return undefined;
      }
      const word = document.getText(range);

      const keyword = KEYWORDS.find((entry) => entry.forms.includes(word));
      if (keyword) {
        return new vscode.Hover(keywordDocs(keyword), range);
      }

      const fn = FUNCTIONS.find((entry) => entry.forms.includes(word));
      if (fn) {
        return new vscode.Hover(functionDocs(fn), range);
      }

      const binding = REQUEST_BINDINGS.find((entry) => entry.name === word);
      if (binding) {
        const docs = new vscode.MarkdownString();
        docs.appendMarkdown(`**${binding.name}** — ${binding.detail}\n\n`);
        docs.appendMarkdown('Bound into every route handler by the server.');
        return new vscode.Hover(docs, range);
      }

      const local = localFunctions(document).find((entry) => entry.name === word);
      if (local) {
        const docs = new vscode.MarkdownString();
        docs.appendCodeblock(`${local.name}(${local.params.join(', ')})`, 'etamil');
        docs.appendMarkdown('\nDefined in this file.');
        return new vscode.Hover(docs, range);
      }

      return undefined;
    },
  };
}

// ---------------------------------------------------------------------------
// Signature help
// ---------------------------------------------------------------------------

export function signatureHelpProvider(): vscode.SignatureHelpProvider {
  return {
    provideSignatureHelp(document, position) {
      const before = document.getText(
        new vscode.Range(new vscode.Position(Math.max(0, position.line - 2), 0), position)
      );

      // Walk back to the open paren of the innermost unclosed call.
      let depth = 0;
      let index = before.length - 1;
      let commas = 0;
      while (index >= 0) {
        const character = before[index];
        if (character === ')') {
          depth += 1;
        } else if (character === '(') {
          if (depth === 0) {
            break;
          }
          depth -= 1;
        } else if (character === ',' && depth === 0) {
          commas += 1;
        }
        index -= 1;
      }
      if (index < 0) {
        return undefined;
      }

      const name = /([a-zA-Z_஀-௿][a-zA-Z0-9_஀-௿]*)\s*$/u.exec(before.slice(0, index))?.[1];
      if (!name) {
        return undefined;
      }

      const fn = FUNCTIONS.find((entry) => entry.forms.includes(name));
      const local = localFunctions(document).find((entry) => entry.name === name);
      if (!fn && !local) {
        return undefined;
      }

      const label = fn ? signature(fn) : `${local!.name}(${local!.params.join(', ')})`;
      const params = fn?.params ?? local?.params ?? [];

      const information = new vscode.SignatureInformation(
        label,
        fn ? functionDocs(fn) : undefined
      );
      information.parameters = params.map((param) => new vscode.ParameterInformation(param));

      const help = new vscode.SignatureHelp();
      help.signatures = [information];
      help.activeSignature = 0;
      help.activeParameter = Math.min(commas, Math.max(0, params.length - 1));
      return help;
    },
  };
}

// ---------------------------------------------------------------------------
// Symbols and definitions
// ---------------------------------------------------------------------------

export function documentSymbolProvider(): vscode.DocumentSymbolProvider {
  return {
    provideDocumentSymbols(document) {
      return localFunctions(document).map(({ name, params, line }) => {
        const range = document.lineAt(line).range;
        return new vscode.DocumentSymbol(
          name,
          `(${params.join(', ')})`,
          vscode.SymbolKind.Function,
          range,
          range
        );
      });
    },
  };
}

/**
 * Go to Definition, for the standard library and for this file.
 *
 * The library's location comes from the generated data, which recorded the
 * file and line of every செயல் in nUlakam. Resolving it needs the repository
 * root, which is only knowable when the workspace contains the compiler
 * checkout — so this degrades to nothing rather than guessing.
 */
export function definitionProvider(): vscode.DefinitionProvider {
  return {
    async provideDefinition(document, position) {
      const range = document.getWordRangeAtPosition(position, IDENTIFIER);
      if (!range) {
        return undefined;
      }
      const word = document.getText(range);

      const local = localFunctions(document).find((entry) => entry.name === word);
      if (local) {
        return new vscode.Location(document.uri, new vscode.Position(local.line, 0));
      }

      const fn = FUNCTIONS.find(
        (entry) => entry.kind === 'stdlib' && entry.forms.includes(word)
      );
      if (!fn?.module || fn.line === null) {
        return undefined;
      }

      for (const folder of vscode.workspace.workspaceFolders ?? []) {
        const candidate = vscode.Uri.joinPath(folder.uri, ...fn.module.split('/'));
        try {
          await vscode.workspace.fs.stat(candidate);
          return new vscode.Location(candidate, new vscode.Position(fn.line - 1, 0));
        } catch {
          // Not this folder; try the next.
        }
      }

      // Also try beside the document, for a program that sits inside the
      // compiler checkout but is opened as a single file.
      const guess = path.resolve(path.dirname(document.uri.fsPath), '..', fn.module);
      try {
        const uri = vscode.Uri.file(guess);
        await vscode.workspace.fs.stat(uri);
        return new vscode.Location(uri, new vscode.Position(fn.line - 1, 0));
      } catch {
        return undefined;
      }
    },
  };
}

/** Counts, for the extension's status message. */
export const COVERAGE = {
  keywords: KEYWORDS.length,
  spellings: KEYWORDS.reduce((total, entry) => total + entry.forms.length, 0),
  builtins: FUNCTIONS.filter((entry) => entry.kind === 'builtin').length,
  stdlib: FUNCTIONS.filter((entry) => entry.kind === 'stdlib').length,
};

export { IDENTIFIER };
