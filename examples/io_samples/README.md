# File I/O Samples Directory

This folder contains file input/output example programs and sample data files for the eTamil compiler.

## Contents Overview

### eTamil Example Programs
- `simple_fileio.qmz` - Basic file operations (read, write, open, close)
- `fileio_example.qmz` - CSV file processing and data handling

### Sample Input/Output Files

**Text Files:**
- `input.txt` - Sample text input file
- `sample.txt` - Sample text data
- `tamil_sample.txt` - Tamil language sample text
- `numbers.txt` - Numeric data sample

**Data Files (CSV):**
- `data.csv` - Sample tabular data
- `products.csv` - Product list with pricing

## Quick Start

### Run eTamil File I/O Examples
```bash
cd /home/esan/ஆவணங்கள்/eTamil/etamil_compiler
cargo run --bin etamil_compiler examples/io_samples/simple_fileio.qmz
cargo run --bin etamil_compiler examples/io_samples/fileio_example.qmz
```

### View Generated Output
```bash
cat output.ll  # LLVM IR output
```

## Example Program Details

### simple_fileio.qmz
Demonstrates fundamental file operations:
- Opening files in read/write modes
- Reading content from files
- Writing data to files
- Closing file handles

**Input:** input.txt  
**Output:** LLVM IR to output.ll

### fileio_example.qmz
Demonstrates CSV data handling:
- Reading CSV format data
- Parsing tabular information
- Writing CSV output
- Data processing

**Input:** data.csv, products.csv  
**Output:** LLVM IR to output.ll

## File Format Specifications

### Text Files (.txt)
- UTF-8 encoded
- Line-based format
- Support for Tamil characters

### CSV Files (.csv)
- Comma-separated values
- Standard RFC 4180 format
- Can contain Tamil text

## Integration with eTamil Compiler

These examples demonstrate:
1. **Basic File Operations** - Reading and writing
2. **Data Processing** - CSV parsing and handling
3. **Text Handling** - Tamil and English text support
4. **I/O Control** - File open, read, write, close operations

## Encryption Support

These files can be encrypted using the eTamil encryption system:
- Text files → `.ani` (encrypted text)
- CSV files → `.qrv` (encrypted CSV)

Example:
```bash
cargo run --example crypto_demo
# Demonstrates encryption of text and CSV files
```

## See Also

- `../README_EXAMPLES.md` - Complete guide to all examples
- `../../DATA_FILES_GUIDE.md` - Detailed data specifications
- `../../ENCRYPTION_SYSTEM.md` - Encryption details

---

**Total Files:** 8 (2 Programs + 6 Data Files)  
**Total Size:** ~32 KB  
**Format:** UTF-8 with full Tamil character support
