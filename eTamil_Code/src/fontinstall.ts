// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Mohammed Maruff (Esan Maruff) <esan@etamil.in>
//
// Installing the eTamil font.
//
// `ican qamiz` is the font in which the ASCII letters carry Tamil glyphs — `c`
// draws ச, `q` draws த, `Z` draws ன — so that Tamil can be typed on an ASCII
// keyboard and read back as Tamil. It ships inside the VSIX, in `fonts/`.
//
// **Shipping it is not installing it.** VS Code's editor is not a webview,
// there is no API that registers a font with it, and `editor.fontFamily` and a
// decoration's `font-family` both resolve against the fonts the operating
// system knows about. A font sitting in an extension directory is a file; it
// has to reach the OS font list before anything can name it. So there is an
// install command, and the editor has to be restarted afterwards, because the
// font list is read when the process starts.
//
// Everything here is per-user. No administrator rights on any of the three:
//
//   macOS    copy into ~/Library/Fonts — that is the whole of it
//   Linux    copy into ~/.local/share/fonts, then fc-cache
//   Windows  copy into %LOCALAPPDATA%\Microsoft\Windows\Fonts **and** add a
//            value under HKCU\...\Fonts. The copy alone is the failure that
//            looks like success: the file is in the right directory and no
//            application can see it.
//
// Nothing runs without being asked. The command is invoked by a person, the
// dialog says which files and which registry value before anything is written,
// and declining leaves the machine as it was.

import { spawn } from 'child_process';
import { existsSync, promises as fs } from 'fs';
import * as os from 'os';
import * as path from 'path';
import * as vscode from 'vscode';

import {
  FACES,
  FONT_FILE,
  SMART_FACE,
  fontInstallDir,
  fontRegistryName,
  fontSource,
} from './bundle';

/** Run a command and resolve with what happened, never rejecting. */
function run(command: string, args: string[]): Promise<string | undefined> {
  return new Promise((resolve) => {
    // `shell: false`: every argument here is a constant or a path this
    // extension computed, and none of it should be parsed by a shell.
    const child = spawn(command, args, { shell: false });
    let stderr = '';
    child.stderr.on('data', (chunk) => {
      stderr += chunk.toString();
    });
    child.on('error', (error) => resolve(error.message));
    child.on('close', (code) =>
      resolve(code === 0 ? undefined : stderr.trim() || `exit ${code}`)
    );
  });
}

/** What the confirmation dialog says will happen, before it happens. */
function plan(platform: string, directory: string): string {
  const files = FACES.map((face) => face.file).join(' and ');
  const copy = `${files} will be copied to ${directory}.`;
  if (platform === 'win32') {
    const values = FACES.map((face) => `"${fontRegistryName(face.family)}"`).join(
      ' and '
    );
    return (
      `${copy}\n\nValues named ${values} will be added under ` +
      'HKEY_CURRENT_USER\\Software\\Microsoft\\Windows NT\\CurrentVersion\\Fonts, ' +
      'which is how Windows installs a font for one user without administrator ' +
      'rights. Nothing else on the machine is changed.'
    );
  }
  if (platform === 'linux') {
    return `${copy}\n\nThen fc-cache is run so applications notice them.`;
  }
  return copy;
}

/**
 * Copy the font into the user's font directory and register it.
 *
 * Returns true when the font is there afterwards, whether this call put it
 * there or a previous one did.
 */
export async function installFont(
  output: vscode.LogOutputChannel,
  extensionPath: string
): Promise<boolean> {
  const absent = FACES.filter(
    (face) => !existsSync(fontSource(extensionPath, face.file))
  );
  if (absent.length === FACES.length) {
    void vscode.window.showErrorMessage(
      `eTamil: this build does not carry ${FONT_FILE}.`
    );
    return false;
  }
  for (const face of absent) {
    output.warn(`font: this build does not carry ${face.file}; skipping it`);
  }
  const carried = FACES.filter((face) => !absent.includes(face));

  const directory = fontInstallDir(
    process.platform,
    os.homedir(),
    process.env.LOCALAPPDATA
  );

  const proceed = await vscode.window.showInformationMessage(
    `Install the eTamil fonts, ${carried.map((f) => f.family).join(' and ')}?`,
    {
      modal: true,
      detail: plan(process.platform, directory),
    },
    'Install'
  );
  if (proceed !== 'Install') {
    return false;
  }

  return vscode.window.withProgress(
    { location: vscode.ProgressLocation.Notification, title: 'Installing the eTamil font' },
    async () => {
      for (const face of carried) {
        const destination = path.join(directory, face.file);
        try {
          await fs.mkdir(directory, { recursive: true });
          await fs.copyFile(fontSource(extensionPath, face.file), destination);
          output.info(`font: wrote ${destination}`);
        } catch (error) {
          const message = error instanceof Error ? error.message : String(error);
          output.error(`font: ${message}`);
          void vscode.window.showErrorMessage(
            `eTamil: could not write ${face.file} — ${message}`
          );
          return false;
        }

        if (process.platform === 'win32') {
          // The copy is not the install. Without this value the file sits in
          // the correct directory and no application lists the family.
          const failure = await run('reg', [
            'add',
            'HKCU\\Software\\Microsoft\\Windows NT\\CurrentVersion\\Fonts',
            '/v',
            fontRegistryName(face.family),
            '/t',
            'REG_SZ',
            '/d',
            destination,
            '/f',
          ]);
          if (failure) {
            output.error(`font: registration failed — ${failure}`);
            void vscode.window.showErrorMessage(
              `eTamil: ${face.file} was copied but could not be registered — ` +
                `${failure}. Double-click ${destination} to install it by hand.`
            );
            return false;
          }
          output.info(`font: registered "${fontRegistryName(face.family)}"`);
        }
      }

      if (process.platform === 'linux') {
        // Best effort. fontconfig picks the directory up on its own eventually,
        // and a machine without fc-cache is not a machine that failed to
        // install the font.
        const failure = await run('fc-cache', ['-f', directory]);
        if (failure) {
          output.warn(`font: fc-cache did not run — ${failure}`);
        }
      }

      await offerToUseIt(output);
      return true;
    }
  );
}

/**
 * Ask whether to switch eTamil files over to it, and say a restart is needed.
 *
 * Setting `etamil.eTamilFont` is what turns the two-font rendering on, and it
 * is a separate decision from having the font on the machine — someone may
 * want the face available and their files drawn the way they were. So it is
 * offered rather than assumed, and it is written at global scope because the
 * setting is machine-scoped and a workspace cannot hold it.
 */
async function offerToUseIt(output: vscode.LogOutputChannel): Promise<void> {
  const configuration = vscode.workspace.getConfiguration('etamil');
  const current = configuration.get<string>('eTamilFont', '').trim();

  if (current === SMART_FACE.family) {
    void vscode.window.showInformationMessage(
      `eTamil: ${SMART_FACE.family} is installed. Restart VS Code to see it — the ` +
        'font list is read when the window starts.'
    );
    return;
  }

  const use = 'Use it for eTamil files';
  const picked = await vscode.window.showInformationMessage(
    `eTamil: ${SMART_FACE.family} is installed. Use it to draw the ASCII that is ` +
      'eTamil rather than English?',
    use,
    'Not now'
  );
  if (picked !== use) {
    return;
  }

  await configuration.update(
    'eTamilFont',
    SMART_FACE.family,
    vscode.ConfigurationTarget.Global
  );
  output.info(`font: etamil.eTamilFont set to ${SMART_FACE.family}`);
  void vscode.window.showInformationMessage(
    'eTamil: restart VS Code to pick the font up. The font list is read when ' +
      'the window starts, so it will not appear until then.'
  );
}

/** Whether the font is already in the user's font directory. */
export function fontInstalled(file: string = FONT_FILE): boolean {
  return existsSync(
    path.join(
      fontInstallDir(process.platform, os.homedir(), process.env.LOCALAPPDATA),
      file
    )
  );
}

/** The face a family name refers to, if the extension carries it. */
export function faceFor(family: string): (typeof FACES)[number] | undefined {
  return FACES.find((face) => face.family === family);
}
