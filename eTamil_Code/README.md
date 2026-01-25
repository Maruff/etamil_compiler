# eTamil Support (VS Code Extension)

Provides basic language support for eTamil:
- Language id: `etamil`
 - File extensions: `.etamil`, `.qmz`
- Syntax highlighting via TextMate grammar
- Comments and brackets via language configuration
- Command: "eTamil: Install Compiler" (`etamil.install`) to run a configurable installation command in the integrated terminal.

## Syntax & Comments
// single-line comments (primary) and /* */ block comments are supported.

## Snippets
Enabled snippets for common constructs:
- Function: trigger `fun` or `முறை`
- If-Else: trigger `if` or `என்றால்`
- Loop: trigger `loop` or `சுழறு`
- Print: trigger `print` or `அச்சிடு`
- Input: trigger `input` or `உள்ளிடு`

## Install eTamil
Guided options:
- Install from GitHub: choose "Install from GitHub (install.sh)" to clone and run the installer from Maruff/etamil_compiler.
- Install via package managers: pip, apt/Homebrew/Chocolatey.
- Custom: provide your own command.

The extension opens a terminal to run the installer and then verifies with `etamil --version` (and alternatives).

Defaults:
- The setting `etamil.installCommand` defaults to cloning and running the installer script:
```bash
git clone https://github.com/Maruff/etamil_compiler.git && cd etamil_compiler && chmod +x install.sh && ./install.sh
```

Other examples (adjust to your environment):
```bash
# Linux (pip)
python3 -m pip install -U etamil

# Linux (apt) — example placeholder
sudo apt update && sudo apt install -y etamil

# macOS (brew)
brew install etamil

# Windows (choco)
choco install etamil -y
```

## Development
- Run `npm install` then `npm run build`.
 - Press F5 to launch an Extension Development Host.
 - Open an `.etamil` or `.qmz` file to activate language features.

## Notes
Keywords and syntax are placeholders. Please replace with the canonical eTamil keywords and patterns.
