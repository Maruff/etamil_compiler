# eTamil Documentation - Usage Guide

**Updated**: January 31, 2026  
**Status**: ✅ All corrections applied

---

## 📚 Which Documentation to Use

### For General Users
**→ Start Here**: [README.md](README.md)
- Overview of eTamil language
- Correct syntax examples
- How to run programs
- Keyword reference

### For Developers
**→ Complete Reference**: [ACTUAL_KEYWORDS.md](ACTUAL_KEYWORDS.md)
- All 90+ keywords listed
- Verified against lexer/parser
- What works vs what doesn't
- Correct code examples
- Database keywords explained

### For Understanding Corrections
**→ See What Changed**: [DOCUMENTATION_CORRECTIONS.md](DOCUMENTATION_CORRECTIONS.md)
- All false claims listed
- Corrections applied
- Verification against source
- Impact analysis

---

## 🔑 Essential Keywords to Know

### Most Important (5 keywords)
```
| Tamil | English | Purpose |
|-------|---------|---------|
| எனில் | enil | If statement |
| இன்றேல் | inREl | Else clause |
| சுற்று | cuRRu | Loop |
| அச்சு | accu | Print output |
| உள்ளிடு | uLLitu | Read input |
```

### Quick Examples
```etamil
// Print
அச்சு "Hello World";

// Input
எண் age;
உள்ளிடு age;

// If-Else
(age > 18) எனில् {
    அச்சு "Adult";
}
இன்றேல् {
    அச்சு "Minor";
}

// Loop
சுற்று i = 1; i <= 5; {
    அச்சு i;
}
```

---

## ❌ What Does NOT Work

These are **NOT** valid eTamil keywords (even though documentation might have said they were):

```etamil
// ❌ WRONG - Don't use these
print "Hello";          // Use அச்சு instead
input x;                // Use உள்ளிடு instead
if (x > 5) { }          // Use எனில் instead
loop { }                // Use சுற்று instead
else { }                // Use இன்றேல् instead
fun myFunc() { }        // Functions not supported
file_open("x.txt");     // Use கோப்பு_திற instead
20%;                    // May not work as expected
```

---

## ✅ What DOES Work

### Verified Keywords
- ✅ Control flow: `எனில்`, `இன்றேல்`, `சுற்று`
- ✅ I/O: `அச்சு`, `உள்ளிடு`
- ✅ File ops: `கோப்பு_திற`, `கோப்பு_மூடு`, `கோப்பு_படி`, `கோப்பு_எழுது`
- ✅ CSV: `தரவுரை_படி`, `தரவுரை_எழுது`
- ✅ Database: `தளம்_இணை`, `தளம்_பிரி`, `தளம்_வினா`, etc.
- ✅ Data types: `எண்`, `பின்னம்`, `சொல்`, `பொது`, etc.

### Verified NOT Working
- ❌ Execution modes: `--async`, `--server`, `--llvm` (only `--vm` works)
- ❌ Package managers: `choco install`, `pip install`, `brew install`
- ❌ Functions: No `fun` keyword available
- ❌ Parameters in file operations: `file_open("x.txt", "mode")` doesn't work

---

## 📖 Reading Order

### New to eTamil?
1. Read [README.md](README.md) - Get overview
2. Look at examples in Quick Start section
3. Try running a simple program
4. Use [ACTUAL_KEYWORDS.md](ACTUAL_KEYWORDS.md) when you need a keyword

### Intermediate Level?
1. Study [ACTUAL_KEYWORDS.md](ACTUAL_KEYWORDS.md) - All keywords explained
2. Review database operations section
3. Try file I/O examples
4. Experiment with loops and conditions

### Advanced / Debugging?
1. Read [DOCUMENTATION_CORRECTIONS.md](DOCUMENTATION_CORRECTIONS.md)
2. Check what was changed from false documentation
3. Understand why certain features don't work
4. See verification against lexer/parser source

---

## 🎯 Common Tasks

### Task: Print Text
**Keyword**: `அச்சு` (accu)
```etamil
அச்சு "Hello";
அச்சு "வணக்கம்";
```

### Task: Get User Input
**Keyword**: `உள்ளிடு` (uLLitu)
```etamil
எண் age;
அச்சு "Enter age: ";
உள்ளிடு age;
```

### Task: Check Condition
**Keywords**: `எனில்` (if) / `இன்றேல்` (else)
```etamil
(age > 18) எனில् {
    அச்சு "Adult";
}
இன்றேல் {
    அச்சு "Minor";
}
```

### Task: Loop Numbers
**Keyword**: `சுற்று` (cuRRu)
```etamil
சுற்று i = 1; i <= 10; {
    அச்சு i;
}
```

### Task: Read/Write Files
**Keywords**: `கோப்பு_திற`, `கோப்பு_எழுது`, `கோப்பு_மூடு`
```etamil
கோப்பு_திற "data.txt";
கோப்பு_எழுது "data.txt", "content";
கோப்பு_மூடு "data.txt";
```

---

## 🔗 File Navigation

| Document | Purpose | When to Use |
|----------|---------|-------------|
| [README.md](README.md) | Main guide | First time setup |
| [ACTUAL_KEYWORDS.md](ACTUAL_KEYWORDS.md) | Complete keyword list | Looking up keywords |
| [DOCUMENTATION_CORRECTIONS.md](DOCUMENTATION_CORRECTIONS.md) | What was fixed | Understanding changes |
| [DOCUMENTATION_INDEX.md](DOCUMENTATION_INDEX.md) | Doc index | Finding things |
| This file | Usage guide | Right now! |

---

## 🐛 Troubleshooting

### Problem: "keyword not found" error
**Solution**: Check [ACTUAL_KEYWORDS.md](ACTUAL_KEYWORDS.md) for correct spelling
- Make sure you're using Tamil keywords, not English
- Example: Use `அச்சு` not `print`

### Problem: "unexpected token" error
**Solution**: Check syntax in [README.md](README.md) examples
- Verify parentheses/brackets are balanced
- Check that keywords are spelled correctly

### Problem: "File not found" when opening file
**Solution**: Make sure the file exists in the correct path
- Use full path: `/home/user/data.txt` or `C:\Users\user\data.txt`
- Check file permissions

### Problem: "Compiler not found"
**Solution**: Build from source
```bash
cd etamil_compiler
cargo build --release
# Then use: target/release/etamil_compiler or etamil_compiler.exe
```

---

## 📝 Documentation Files in eTamil_Code/

```
eTamil_Code/
├── README.md                          ← START HERE
├── ACTUAL_KEYWORDS.md                ← All keywords reference
├── DOCUMENTATION_CORRECTIONS.md       ← What was fixed
├── DOCUMENTATION_INDEX.md             ← Index of all docs
└── HOW_TO_USE.md (this file)         ← You are here
```

---

## ✨ Quick Reference Card

### Run a Program
```bash
etamil --vm myprogram.etamil
echo "input" | etamil --vm myprogram.etamil
```

### Basic Syntax
```etamil
// Variable declaration
எண் x = 10;
சொல் name = "text";

// Print output
அச்சு x;

// Read input
உள்ளிடु x;

// If-Else
(x > 5) எனில् { அச்சு "big"; } இன்றேல् { அச்சு "small"; }

// Loop
சுற்று i = 1; i <= 10; { அச்சு i; }

// File I/O
கோப்பு_திற "file.txt";
கோப்பு_எழுது "file.txt", "data";
கோப்பு_மூடு "file.txt";
```

### Data Types
- `எண்` = Number/Integer
- `சொல்` = String/Text
- `பொது` = Boolean
- `பின்னம்` = Float

---

## 🚀 Getting Started in 5 Minutes

1. **Create a file**: `hello.etamil`
2. **Write code**:
   ```etamil
   அச்சு "வணக்கம் உலகம்";
   ```
3. **Run it**:
   ```bash
   etamil --vm hello.etamil
   ```
4. **See output**: `வணக்கம் உலகம்`

Done! Now explore [ACTUAL_KEYWORDS.md](ACTUAL_KEYWORDS.md) for more keywords.

---

**Version**: 0.2.0  
**Last Updated**: January 31, 2026  
**Status**: ✅ All documentation corrected and verified
