# Changelog

## 0.2.0 (January 31, 2026)

### ✨ Major Features
- **Auto-Compiler Installation**: Automatically detects and installs eTamil compiler on extension activation with platform-aware installation methods
- **Advanced Syntax Highlighting**: Comprehensive bilingual (Tamil/English) keyword support with proper color coding (20+ patterns)
- **IntelliSense & Autocomplete**: 25+ keyword completions with context-aware suggestions, hover documentation, and code snippets
- **Enhanced Code Snippets**: Expanded from 5 to 13 templates including file I/O, CSV operations, and complete working examples
- **Smart Language Configuration**: Auto-closing brackets, intelligent indentation, code folding, and comment recognition

### 📝 Code Changes
- **src/extension.ts**: Expanded from 125 to 263 lines with auto-install and IntelliSense functions
- **syntaxes/etamil.tmLanguage.json**: Complete rewrite with 20+ keyword patterns, fixed duplicate structure
- **snippets/etamil.code-snippets**: Expanded from 5 to 13 templates with multi-line support
- **package.json**: Version 0.0.1 → 0.2.0, added new configuration options and keywords
- **language-configuration.json**: Enhanced with folding markers and indentation rules

### ⚙️ Configuration
- Added `etamil.autoInstallOnActivation` (bool, default: true)
- Added `etamil.syntaxHighlight` (bool, default: true)
- Added `etamil.showIntelliSense` (bool, default: true)

### 🔧 Installation Methods
- **Windows**: pip, Chocolatey, custom command
- **Linux**: pip, apt, GitHub clone + build, custom command
- **macOS**: Homebrew, pip, GitHub clone + build, custom command

### 📚 Documentation
- Added EXTENSION_UPDATES.md (technical details)
- Added QUICK_REFERENCE.md (user guide with examples)
- Added IMPLEMENTATION_COMPLETE.md (comprehensive summary)
- Updated README.md with new features

### ✅ Build Status
- TypeScript compilation: SUCCESS (no errors)
- Output: out/extension.js (11,807 bytes)
- All 25+ keywords supported
- All 13 snippets functional

### 🔄 Backward Compatibility
- ✅ All existing .etamil files work unchanged
- ✅ All existing .qmz files work unchanged
- ✅ Language ID remains unchanged
- ✅ Manual install command still functional

## 0.0.1
- Initial release: language id, syntax highlighting, install command.
