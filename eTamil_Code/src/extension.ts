import * as vscode from 'vscode';
import { exec } from 'child_process';
import * as os from 'os';
import * as path from 'path';

export async function activate(context: vscode.ExtensionContext) {
  // Auto-install eTamil on extension activation
  await autoInstallEtamil(context);

  // Register install command
  const installCommand = vscode.commands.registerCommand('etamil.install', async () => {
    const config = vscode.workspace.getConfiguration('etamil');
    let cmd = config.get<string>('installCommand');

    if (!cmd || cmd.trim().length === 0) {
      await guidedInstall();
      return;
    }

    const term = vscode.window.createTerminal({ name: 'eTamil Install' });
    term.show(true);
    term.sendText(cmd, true);
    await verifyWithProgress();
  });

  context.subscriptions.push(installCommand);

  // Register autocomplete provider
  registerCompletionProvider(context);
}

export function deactivate() {}

async function guidedInstall() {
  const platform = process.platform;
  const picks: { label: string; command: string }[] = [];

  if (platform === 'linux') {
    picks.push(
      { label: 'Build from source (Linux)', command: 'git clone https://github.com/Maruff/etamil_compiler.git && cd etamil_compiler/etamil_compiler && cargo build --release' }
    );
  } else if (platform === 'darwin') {
    picks.push(
      { label: 'Build from source (macOS)', command: 'git clone https://github.com/Maruff/etamil_compiler.git && cd etamil_compiler/etamil_compiler && cargo build --release' }
    );
  } else if (platform === 'win32') {
    picks.push(
      { label: 'Build from source (Windows)', command: 'git clone https://github.com/Maruff/etamil_compiler.git && cd etamil_compiler\\etamil_compiler && cargo build --release' }
    );
  }

  picks.push({ label: 'Custom command…', command: '' });

  const choice = await vscode.window.showQuickPick(picks, {
    placeHolder: 'Choose how to install eTamil'
  });

  if (!choice) {
    return;
  }

  let commandToRun = choice.command;
  if (choice.label.startsWith('Custom')) {
    const input = await vscode.window.showInputBox({
      prompt: 'Enter the install command to run in the terminal',
      placeHolder: 'e.g., git clone <repo> && cd etamil_compiler && ./install.sh'
    });
    if (!input) {
      return;
    }
    commandToRun = input;
  }

  const term = vscode.window.createTerminal({ name: 'eTamil Install' });
  term.show(true);
  term.sendText(commandToRun, true);
  await verifyWithProgress();
}

function verifyEtamil(): Promise<string | null> {
  return new Promise((resolve) => {
    // Try multiple possible binaries
    const candidates = ['etamil --version', 'etamilc --version'];
    let idx = 0;
    const tryNext = () => {
      if (idx >= candidates.length) {
        resolve(null);
        return;
      }
      const cmd = candidates[idx++];
      exec(cmd, (error: Error | null, stdout: string, stderr: string) => {
        if (error) {
          tryNext();
          return;
        }
        const out = (stdout || stderr).trim();
        resolve(out.length > 0 ? out : '');
      });
    };
    tryNext();
  });
}

async function verifyWithProgress() {
  await vscode.window.withProgress(
    {
      location: vscode.ProgressLocation.Notification,
      title: 'Verifying eTamil installation…',
      cancellable: false
    },
    async (progress) => {
      const timeoutMs = 60000;
      const start = Date.now();
      let verified: string | null = null;
      while (Date.now() - start < timeoutMs) {
        verified = await verifyEtamil();
        if (verified) {
          break;
        }
        await new Promise((r) => setTimeout(r, 2000));
      }
      if (verified) {
        vscode.window.showInformationMessage(`eTamil is installed: ${verified}`);
      } else {
        vscode.window.showWarningMessage('Verification timed out. If installation requires elevation or user input, complete it in the terminal and retry.');
      }
    }
  );
}

// Auto-install eTamil on extension activation for Windows and Linux
async function autoInstallEtamil(context: vscode.ExtensionContext) {
  const platform = process.platform;
  
  // Skip auto-install for macOS or if already installed
  if (platform === 'darwin') {
    return;
  }

  const verified = await verifyEtamil();
  if (verified) {
    // eTamil is already installed
    console.log('eTamil compiler already installed:', verified);
    return;
  }

  // Not installed - prompt user for installation
  const installChoice = await vscode.window.showInformationMessage(
    'eTamil compiler not found. Would you like to install it now?',
    'Install',
    'Remind Later',
    'Skip'
  );

  if (installChoice === 'Install') {
    await guidedInstall();
  } else if (installChoice === 'Skip') {
    // Mark as skipped in workspace state
    context.workspaceState.update('etamil.skipAutoInstall', true);
  }
}

// Register autocomplete and syntax hint provider
function registerCompletionProvider(context: vscode.ExtensionContext) {
  const etamilKeywords = [
    // Control flow
    { label: 'எனில்', kind: vscode.CompletionItemKind.Keyword, detail: 'if statement', snippet: '(${1:condition}) எனில் {\n  ${0}\n}' },
    { label: 'enil', kind: vscode.CompletionItemKind.Keyword, detail: 'if statement', snippet: '(${1:condition}) enil {\n  ${0}\n}' },
    { label: 'இன்றேல்', kind: vscode.CompletionItemKind.Keyword, detail: 'else statement', snippet: 'இன்றேல் {\n  ${0}\n}' },
    { label: 'inREl', kind: vscode.CompletionItemKind.Keyword, detail: 'else statement', snippet: 'inREl {\n  ${0}\n}' },

    // I/O
    { label: 'அச்சு', kind: vscode.CompletionItemKind.Function, detail: 'print output', snippet: 'அச்சு ${1:value};' },
    { label: 'accu', kind: vscode.CompletionItemKind.Function, detail: 'print output', snippet: 'accu ${1:value};' },
    { label: 'உள்ளிடு', kind: vscode.CompletionItemKind.Function, detail: 'read input', snippet: 'உள்ளிடு ${1:variable};' },
    { label: 'uLLitu', kind: vscode.CompletionItemKind.Function, detail: 'read input', snippet: 'uLLitu ${1:variable};' },

    // Variables & types
    { label: 'எண்', kind: vscode.CompletionItemKind.Keyword, detail: 'number type', snippet: 'எண் ${1:var} = ${2:0};' },
    { label: 'eN', kind: vscode.CompletionItemKind.Keyword, detail: 'number type', snippet: 'eN ${1:var} = ${2:0};' },
    { label: 'பின்னம்', kind: vscode.CompletionItemKind.Keyword, detail: 'float type', snippet: 'பின்னம் ${1:var} = ${2:0.0};' },
    { label: 'pinnam', kind: vscode.CompletionItemKind.Keyword, detail: 'float type', snippet: 'pinnam ${1:var} = ${2:0.0};' },
    { label: 'சொல்', kind: vscode.CompletionItemKind.Keyword, detail: 'string type', snippet: 'சொல் ${1:var} = "${2:text}";' },
    { label: 'col', kind: vscode.CompletionItemKind.Keyword, detail: 'string type', snippet: 'col ${1:var} = "${2:text}";' },
    { label: 'பொது', kind: vscode.CompletionItemKind.Keyword, detail: 'boolean type', snippet: 'பொது ${1:var} = ${2:மெய்};' },
    { label: 'poqu', kind: vscode.CompletionItemKind.Keyword, detail: 'boolean type', snippet: 'poqu ${1:var} = ${2:mey};' },
    { label: 'உரை', kind: vscode.CompletionItemKind.Keyword, detail: 'text type', snippet: 'உரை ${1:var} = "${2:text}";' },
    { label: 'urY', kind: vscode.CompletionItemKind.Keyword, detail: 'text type', snippet: 'urY ${1:var} = "${2:text}";' },
    { label: 'அணி', kind: vscode.CompletionItemKind.Keyword, detail: 'array type', snippet: 'அணி ${1:var};' },
    { label: 'aNi', kind: vscode.CompletionItemKind.Keyword, detail: 'array type', snippet: 'aNi ${1:var};' },
    { label: 'தேதி', kind: vscode.CompletionItemKind.Keyword, detail: 'date type', snippet: 'தேதி ${1:var};' },
    { label: 'qEqi', kind: vscode.CompletionItemKind.Keyword, detail: 'date type', snippet: 'qEqi ${1:var};' },
    { label: 'மாறி', kind: vscode.CompletionItemKind.Keyword, detail: 'variable declaration', snippet: 'மாறி ${1:var} = ${2:value};' },
    { label: 'mARi', kind: vscode.CompletionItemKind.Keyword, detail: 'variable declaration', snippet: 'mARi ${1:var} = ${2:value};' },
    { label: 'நிலை', kind: vscode.CompletionItemKind.Keyword, detail: 'constant declaration', snippet: 'நிலை ${1:var} = ${2:value};' },
    { label: 'nilY', kind: vscode.CompletionItemKind.Keyword, detail: 'constant declaration', snippet: 'nilY ${1:var} = ${2:value};' },

    // Loops
    { label: 'சுற்று', kind: vscode.CompletionItemKind.Keyword, detail: 'loop construct', snippet: 'சுற்று ${1:i} = ${2:0}; ${1:i} < ${3:10}; ${1:i} = ${1:i} + 1; {\n  ${0}\n}' },
    { label: 'cuRRu', kind: vscode.CompletionItemKind.Keyword, detail: 'loop construct', snippet: 'cuRRu ${1:i} = ${2:0}; ${1:i} < ${3:10}; ${1:i} = ${1:i} + 1; {\n  ${0}\n}' },

    // File I/O
    { label: 'கோப்பு_திற', kind: vscode.CompletionItemKind.Function, detail: 'open file', snippet: 'கோப்பு_திற "${1:filename}";' },
    { label: 'kOppu_qiRa', kind: vscode.CompletionItemKind.Function, detail: 'open file', snippet: 'kOppu_qiRa "${1:filename}";' },
    { label: 'கோப்பு_மூடு', kind: vscode.CompletionItemKind.Function, detail: 'close file', snippet: 'கோப்பு_மூடு "${1:filename}";' },
    { label: 'kOppu_mUtu', kind: vscode.CompletionItemKind.Function, detail: 'close file', snippet: 'kOppu_mUtu "${1:filename}";' },
    { label: 'கோப்பு_எழுது', kind: vscode.CompletionItemKind.Function, detail: 'write to file', snippet: 'கோப்பு_எழுது "${1:filename}", "${2:data}";' },
    { label: 'kOppu_ezuqu', kind: vscode.CompletionItemKind.Function, detail: 'write to file', snippet: 'kOppu_ezuqu "${1:filename}", "${2:data}";' },
    { label: 'கோப்பு_படி', kind: vscode.CompletionItemKind.Function, detail: 'read from file', snippet: 'கோப்பு_படி "${1:filename}", ${2:variable};' },
    { label: 'kOppu_pati', kind: vscode.CompletionItemKind.Function, detail: 'read from file', snippet: 'kOppu_pati "${1:filename}", ${2:variable};' },

    // CSV
    { label: 'தரவுரை_எழுது', kind: vscode.CompletionItemKind.Function, detail: 'write CSV', snippet: 'தரவுரை_எழுது "${1:filename}", "${2:data}";' },
    { label: 'qaravurY_ezuqu', kind: vscode.CompletionItemKind.Function, detail: 'write CSV', snippet: 'qaravurY_ezuqu "${1:filename}", "${2:data}";' },
    { label: 'தரவுரை_படி', kind: vscode.CompletionItemKind.Function, detail: 'read CSV', snippet: 'தரவுரை_படி "${1:filename}", ${2:variable};' },
    { label: 'qaravurY_pati', kind: vscode.CompletionItemKind.Function, detail: 'read CSV', snippet: 'qaravurY_pati "${1:filename}", ${2:variable};' },
    
    // Operators
    { label: '+', kind: vscode.CompletionItemKind.Operator, detail: 'addition' },
    { label: '-', kind: vscode.CompletionItemKind.Operator, detail: 'subtraction' },
    { label: '*', kind: vscode.CompletionItemKind.Operator, detail: 'multiplication' },
    { label: '/', kind: vscode.CompletionItemKind.Operator, detail: 'division' },
    { label: '==', kind: vscode.CompletionItemKind.Operator, detail: 'equal to' },
    { label: '!=', kind: vscode.CompletionItemKind.Operator, detail: 'not equal to' },
    { label: '>', kind: vscode.CompletionItemKind.Operator, detail: 'greater than' },
    { label: '<', kind: vscode.CompletionItemKind.Operator, detail: 'less than' },
    { label: '>=', kind: vscode.CompletionItemKind.Operator, detail: 'greater than or equal' },
    { label: '<=', kind: vscode.CompletionItemKind.Operator, detail: 'less than or equal' },
  ];

  const provider = vscode.languages.registerCompletionItemProvider(
    'etamil',
    {
      provideCompletionItems(document: vscode.TextDocument, position: vscode.Position) {
        const items: vscode.CompletionItem[] = [];
        
        etamilKeywords.forEach((kw) => {
          const item = new vscode.CompletionItem(kw.label, kw.kind);
          item.detail = kw.detail;
          if (kw.snippet) {
            item.insertText = new vscode.SnippetString(kw.snippet);
          }
          item.documentation = new vscode.MarkdownString(`**${kw.label}** - ${kw.detail}`);
          items.push(item);
        });
        
        return items;
      }
    },
    // Trigger on any character
    ...Array.from('abcdefghijklmnopqrstuvwxyz஀஁ஂஃ஄அஆஇஈஉஊ஋஌஍எஏஐ஑ஒஓஔ')
  );

  context.subscriptions.push(provider);

  // Register hover provider for helpful information
  const hoverProvider = vscode.languages.registerHoverProvider('etamil', {
    provideHover(document: vscode.TextDocument, position: vscode.Position) {
      const range = document.getWordRangeAtPosition(position);
      if (!range) {
        return;
      }
      
      const word = document.getText(range);
      const keyword = etamilKeywords.find(k => k.label === word);
      
      if (keyword) {
        return new vscode.Hover(new vscode.MarkdownString(`**${keyword.label}**\n\n${keyword.detail}`));
      }
    }
  });

  context.subscriptions.push(hoverProvider);
}
