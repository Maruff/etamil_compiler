# Basic Examples Directory

This folder contains basic eTamil compiler examples demonstrating fundamental language features.

## Contents Overview

### eTamil Example Programs
- `example.qmz` - Income tax calculator with financial calculations

## Quick Start

### Run Basic Example
```bash
cd /home/esan/ஆவணங்கள்/eTamil/etamil_compiler
cargo run --bin etamil_compiler examples/basic_samples/example.qmz
```

## Example Program Details

### example.qmz
Demonstrates financial calculation using eTamil:
- **Purpose:** Calculate income tax
- **Features:**
  - Variable initialization
  - Numeric calculations
  - Conditional logic
  - Output/display operations
  - Bilingual support (Tamil + English transliteration)

**Keywords Used:**
- `எண்` - Number declaration
- `கணக்கீடு` - Calculate/Computation
- `அச்சு` - Print/Display

**Output:** LLVM IR to output.ll

## Language Features Demonstrated

✅ **Variables** - Declaring and initializing numeric variables  
✅ **Arithmetic** - Basic mathematical operations  
✅ **Logic** - Conditional calculations  
✅ **Output** - Displaying results  
✅ **Bilingual Support** - Tamil keywords + English transliteration  

## Key Concepts

### Tax Calculation
```tamil
எண் வருவாய् = 50000;
எண் வரி = வருவாய् * 0.1;
எண் நல_வருவாய் = வருவாய् - வரி;
```

### Token System
The eTamil compiler uses 103 unique tokens:
- 50+ Tamil language keywords
- 50+ English transliteration equivalents
- Special operators and symbols

## Output Generation

The program generates LLVM IR (Intermediate Representation):
- **File:** output.ll
- **Format:** LLVM Assembly
- **Optimization:** Ready for further compilation

## Integration with Other Examples

This basic example leads to more complex examples:

```
example.qmz (Basic)
    ↓
io_samples/simple_fileio.qmz (File Operations)
    ↓
io_samples/fileio_example.qmz (CSV Processing)
    ↓
db_samples/payroll_system.qmz (Financial + Database)
    ↓
db_samples/inventory_system.qmz (Advanced)
```

## Educational Value

Perfect for learning:
1. **eTamil Syntax** - Basic language structure
2. **Bilingual Programming** - Tamil + English
3. **LLVM Compilation** - IR generation
4. **Financial Logic** - Tax/income calculations

## See Also

- `../io_samples/README.md` - File I/O examples
- `../db_samples/README.md` - Database examples
- `../crypto_samples/README.md` - Encryption examples
- `../README_EXAMPLES.md` - Complete guide to all examples
- `../../ARCHITECTURE.md` - System architecture

---

**Total Files:** 1 Program  
**Total Size:** 358 bytes  
**Complexity:** Beginner
