# eTamil VS Code Extension - Quick Reference Guide

## Installation

### For End Users
1. Open VS Code
2. Go to Extensions (Ctrl+Shift+X)
3. Search for "eTamil Support"
4. Click Install
5. Extension will auto-detect and install compiler (or prompt if needed)

### For Developers
```bash
cd eTamil_Code
npm install
npm run build
# Press F5 to launch Extension Development Host
```

---

## Auto-Install Compiler

When you activate the extension for the first time:

```
✓ Extension checks: Is eTamil compiler installed?
  ↓
  YES → Extension ready, no action needed
  NO → Shows dialog: "eTamil compiler not found. Would you like to install it now?"
    ├─ [Install] → Choose installation method → Install runs in terminal
    ├─ [Remind Later] → Don't ask again this session
    └─ [Skip] → Never ask again
```

### Installation Methods

Build from source (requires Rust):
```bash
git clone https://github.com/Maruff/etamil_compiler.git
cd etamil_compiler/etamil_compiler
cargo build --release
```
- Custom command (optional)

---

## IntelliSense & Autocomplete

### Type to Autocomplete
Start typing any eTamil keyword and suggestions appear:

```
Type: "acc"     → Suggests: accu, அச்சு
Type: "eni"     → Suggests: enil, எனில்
Type: "kOppu"   → Suggests: kOppu_qiRa, கோப்பு_திற, கோப்பு_எழுது, கோப்பு_படி
Type: "+"       → Suggests: + operator with description
```

### Hover for Help
Hover your mouse over any keyword to see:
- **Keyword Name**: Tamil or romanized
- **Description**: What it does
- **Example**: Brief usage info

```
Hover over "அச்சு" → Shows:
  **அச்சு**
  
  print output
```

### Snippet Expansion
Type snippet prefix and press Tab to expand:

```
Type: "enil" + Tab
Result:
  ((condition) எனில் {
    |
  })

Type: "kOppu_ezuqu" + Tab
Result:
  கோப்பு_திற "${1:filename}";
  கோப்பு_எழுது "${1:filename}", "${2:data}";
  கோப்பு_மூடு "${1:filename}";
  |
```

---

## Available Snippets

### Control Flow
| Trigger | Tamil | Expands To |
|---------|-------|-----------|
| `enil` | `எனில்` | (condition) எனில் { } இன்றேல் { } |
| `if_only` | `எனில்_மட்டும்` | (condition) எனில் { } |
| `cuRRu` | `சுற்று` | சுற்று i = 0; i < 10; i = i + 1; { } |

### I/O
| Trigger | Tamil | Expands To |
|---------|-------|-----------|
| `accu` | `அச்சு` | அச்சு "value"; |
| `uLLitu` | `உள்ளிடு` | அச்சு "Question: "; உள்ளிடு var; |

### Variables
| Trigger | Tamil | Expands To |
|---------|-------|-----------|
| `eN` | `எண்` | எண் var = 0; |

### File Operations
| Trigger | Tamil | Expands To |
|---------|-------|-----------|
| `kOppu_ezuqu` | `கோப்பு_எழுது` | Open, write, close file |
| `kOppu_pati` | `கோப்பு_படி` | Open, read, close file |
| `qaravurY_ezuqu` | `தரவுரை_எழுது` | Write CSV headers & rows |
| `qaravurY_pati` | `தரவுரை_படி` | Read CSV data |

### Templates
| Trigger | Purpose |
|---------|---------|
| `tax_calc` | Income tax calculator (complete working example) |
| `comment` | Multi-line comment block `/* */` |

---

## Syntax Highlighting

Keywords are automatically highlighted with colors:

### Color Coding
- **Purple**: Keywords (எனில், இன்றேல், சுற்று, etc.)
- **Orange**: Data types (எண், உரை, அணி, etc.)
- **Green**: Strings ("text in quotes")
- **Red**: Numbers (123, 45.67, 15%)
- **Blue**: Functions and operations (அச்சு, கோப்பு_திற, etc.)
- **Cyan**: Comments (// and /* */)
- **Gray**: Operators (+, -, *, /, ==, etc.)

### Bilingual Highlighting
Type: "enil" + Tab
Result:
  ((condition) எனில் {
    |
  })
Type: "kOppu_ezuqu" + Tab
Result:
  கோப்பு_திற "${1:filename}";
  கோப்பு_எழுது "${1:filename}", "${2:data}";
  கோப்பு_மூடு "${1:filename}";
  |
(income > 800000) எனில் {     // "எனில்" highlighted as keyword
  அச்சு "High income";      // "அச்சு" highlighted as function
}
```

---

## Smart Bracket Support

### Auto-Closing
Type any opening bracket → auto-closes:

```
Type: {     → Result: {|}  (cursor between)
Type: [     → Result: [|]
Type: (     → Result: (|)
Type: "     → Result: "|"
Type: '     → Result: '|'
```

### Auto-Indenting
When you press Enter inside brackets, indentation auto-adjusts:

```
if (condition) {|      (press Enter)
  |                    (cursor indented automatically)
}
```

### Bracket Matching
Click on opening bracket → closing bracket highlighted:

```
if (condition) {        ← Click here
  statement;
}                       ← Highlights in background
```

---

## Configuration Options

Edit VS Code settings to customize:

```json
{
  "etamil.autoInstallOnActivation": true,    // Auto-check/install compiler
  "etamil.syntaxHighlight": true,            // Enable syntax colors
  "etamil.showIntelliSense": true,           // Show suggestions
  "etamil.installCommand": "..."             // Custom install command
}
```

### How to Change Settings
1. Press `Ctrl+,` to open Settings
2. Search for "etamil"
3. Edit any option as desired

---

## File Types Supported

- **`.etamil`** - eTamil source code files
- **`.qmz`** - Encrypted eTamil files

Both automatically open with eTamil language mode.

---

## Keyboard Shortcuts

| Action | Windows/Linux | macOS |
|--------|---------------|-------|
| Trigger IntelliSense | Ctrl+Space | Cmd+Space |
| Trigger Snippets | Tab (after typing prefix) | Tab |
| Show Hover Info | Hover mouse | Hover mouse |
| Format Document | Shift+Alt+F | Shift+Opt+F |
| Comment Line | Ctrl+/ | Cmd+/ |
| Block Comment | Shift+Alt+A | Shift+Opt+A |

---

## Troubleshooting

### Problem: "etamil: command not found"
**Solution**: 
1. Open extension command palette (Ctrl+Shift+P)
2. Type "eTamil: Install Compiler"
3. Choose installation method
4. Wait for installation to complete

### Problem: Auto-install dialog not showing
**Solution**: 
1. Check if compiler is already installed: `etamil --version`
2. If not found, try manual installation via command palette
3. Check `etamil.autoInstallOnActivation` is `true` in settings

### Problem: Autocomplete not working
**Solution**: 
1. Check `etamil.showIntelliSense` is `true` in settings
2. Try pressing Ctrl+Space to manually trigger suggestions
3. Ensure you're in an `.etamil` or `.qmz` file

### Problem: Syntax highlighting looks wrong
**Solution**: 
1. Check if file is recognized as `etamil` language (see bottom-right corner)
2. If not, click and select "eTamil" from language dropdown
3. Check `etamil.syntaxHighlight` is `true` in settings

### Problem: Bracket highlighting not working
**Solution**: 
1. This is automatic - click on opening bracket to see match
2. If no color appears, theme might not support it
3. Try changing color theme in VS Code

---

## Common Workflows

### Creating a New eTamil Script

1. Create new file: `.etamil` extension
2. Type first few characters and press Ctrl+Space
3. Select from autocomplete suggestions
4. Let snippets expand code structure
5. Edit template values
6. Save and run with compiler

**Example**:
```
Create: calculation.etamil
Type: "num" + Tab → Expands number variable
Type: "if" + Tab → Expands if-else block
Type: "print" + Tab → Expands print statement
Fill in values and save
Run: etamil --vm calculation.etamil
```

### Using File I/O

1. Type "file_write" + Tab → Expands file writing template
2. Edit filename and data
3. Type "file_read" + Tab → Expands file reading template
4. Edit filename and variable
5. Hover over keywords for help

### Working with CSV

1. Type "csv_write" + Tab → Expands CSV template
2. Edit headers and rows
3. Type "csv_read" + Tab → Expands read template
4. Edit filename and variable
5. Run program to verify

---

## Tips & Tricks

1. **Multi-line Selection**: Select code and use Shift+Alt+A for block comments
2. **Quick Format**: While typing, auto-indentation happens automatically
3. **Fast Navigation**: Use Ctrl+G to jump to line number
4. **Search Keywords**: Ctrl+F to search for keywords in file
5. **IntelliSense**: Start typing any word to see completions
6. **Snippets**: Check all available snippets by pressing Ctrl+K, Ctrl+X

---

## Reporting Issues

Found a bug or have a feature request?

1. Go to: https://github.com/Maruff/etamil_compiler/issues
2. Click "New Issue"
3. Describe the problem clearly
4. Include:
   - Your OS (Windows/Linux/macOS)
   - eTamil compiler version (`etamil --version`)
   - VS Code version
   - Steps to reproduce

---

## Further Learning

- **Official Documentation**: https://github.com/Maruff/etamil_compiler
- **Example Programs**: Check `examples/` in the repository
- **Language Guide**: See full eTamil language reference
- **Video Tutorials**: (Coming soon)

---

**Version**: 0.2.0  
**Last Updated**: January 31, 2026  
**Status**: Production Ready
