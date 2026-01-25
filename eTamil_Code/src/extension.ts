import * as vscode from 'vscode';
import { exec } from 'child_process';

export function activate(context: vscode.ExtensionContext) {
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
}

export function deactivate() {}

async function guidedInstall() {
  const platform = process.platform;
  const picks: { label: string; command: string }[] = [];

  if (platform === 'linux') {
    picks.push(
      { label: 'Install via pip (Linux)', command: 'python3 -m pip install -U etamil' },
      { label: 'Install via apt (Linux)', command: 'sudo apt update && sudo apt install -y etamil' },
      { label: 'Install from GitHub (install.sh)', command: 'git clone https://github.com/Maruff/etamil_compiler.git && cd etamil_compiler && chmod +x install.sh && ./install.sh' }
    );
  } else if (platform === 'darwin') {
    picks.push(
      { label: 'Install via pip (macOS)', command: 'python3 -m pip install -U etamil' },
      { label: 'Install via Homebrew (macOS)', command: 'brew install etamil' }
    );
  } else if (platform === 'win32') {
    picks.push(
      { label: 'Install via pip (Windows)', command: 'pip install --upgrade etamil' },
      { label: 'Install via Chocolatey (Windows)', command: 'choco install etamil -y' }
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
