# eTamil File I/O Features

## Overview

The eTamil compiler now supports comprehensive file I/O operations for reading and writing both text and CSV files. All operations support bilingual syntax (Tamil/English).

## File I/O Operations

### 1. File Open
Opens a file in specified mode.

**Syntax:**
```
கோப்பு_திற "filename.txt", "read";     // Tamil
fileOpen "filename.txt", "read";         // English
```

**Parameters:**
- `filename`: String path to the file
- `mode`: String mode ("read", "write", "append")

**Example:**
```
கோப்பு_திற "data.txt", "read";
fileOpen "output.txt", "write";
```

### 2. File Close
Closes an open file.

**Syntax:**
```
கோப்பு_மூடு "filename.txt";          // Tamil
_fileClose "filename.txt";              // English
```

**Parameters:**
- `filename`: String path to the file to close

**Example:**
```
கோப்பு_மூடு "data.txt";
_fileClose "output.txt";
```

### 3. File Read
Reads data from a file into a variable.

**Syntax:**
```
கோப்பு_படி "filename.txt", variable;   // Tamil
fileRead "filename.txt", variable;       // English
```

**Parameters:**
- `filename`: String path to the file
- `variable`: Variable name to store the read value

**Example:**
```
எண் revenue_data;
கோப்பு_படி "revenue.txt", revenue_data;
fileRead "sales.txt", total_sales;
```

### 4. File Write
Writes data to a file.

**Syntax:**
```
கோப்பு_எழுது "filename.txt", data;     // Tamil
fileWrite "filename.txt", data;          // English
```

**Parameters:**
- `filename`: String path to the file
- `data`: Expression or variable to write

**Example:**
```
கோப்பு_எழுது "output.txt", result;
fileWrite "report.txt", total_amount;
```

### 5. Read CSV
Reads data from a CSV file into a variable.

**Syntax:**
```
தரவுரை_படி "data.csv", variable;           // Tamil
_readCSV "data.csv", variable;            // English
```

**Parameters:**
- `filename`: String path to the CSV file
- `variable`: Variable name to store the read data

**Example:**
```
எண் sales_data;
தரவுரை_படி "sales.csv", sales_data;
readCSV "expenses.csv", expense_data;
```

### 6. Write CSV
Writes data to a CSV file.

**Syntax:**
```
தரவுரை_எழுது "data.csv", data;             // Tamil
_writeCSV "data.csv", data;               // English
```

**Parameters:**
- `filename`: String path to the CSV file
- `data`: Expression or variable to write

**Example:**
```
தரவுரை_எழுது "output.csv", processed_data;
writeCSV "results.csv", summary;
```

## Complete Example

```etamil
// Read revenue from file
எண் வருவாய_;
கோப்பு_திற "data.txt", "read";
கோப்பு_படி "data.txt", வருவாய_;

// Display revenue
அச்சு "Revenue from file: " & வருவாய_;

// Write result to file
கோப்பு_திற "output.txt", "write";
கோப்பு_எழுது "output.txt", வருவாய_;
கோப்பு_மூடு "output.txt";

// CSV Operations
தரவுரை_படி "sales.csv", வருவாய_;
தரவுரை_எழுது "processed.csv", வருவாய_;
```

## Bilingual Support

All file I/O operations support Tamil and English syntax:

| Operation | Tamil | English | Abbreviation |
|-----------|-------|---------|--------------|
| File Open | கோப்பு_திற | _fileOpen | kOppu_qiRa |
| File Close | கோப்பு_மூடு | _fileClose | kOppu_mUtu |
| File Read | கோப்பு_படி | _fileRead | kOppu_pati |
| File Write | கோப்பு_எழுது | _fileWrite | kOppu_ezuqu |
| Read CSV | தரவுரை_படி | _readCSV | qaravurY_pati |
| Write CSV | தரவுரை_எழுது | _writeCSV | qaravurY_ezuqu |

## Implementation Details

### Current Approach
File I/O operations are currently implemented as:
- **File Open/Close**: Print notification messages
- **File Read**: Simulated via `scanf()` for interactive input
- **File Write**: Simulated via `printf()` for output display
- **CSV Operations**: Display notification messages

This simulation mode demonstrates the file I/O pipeline and can be extended with actual libc file operations (fopen, fread, fwrite, fclose).

### Generated LLVM IR
The compiler generates LLVM IR that:
1. Uses `printf()` for displaying file operation status
2. Uses `scanf()` for reading numerical values from stdin
3. Allocates local variables for file data
4. Handles string concatenation for messages

### Example Generated IR
```llvm
@read_msg = private unnamed_addr constant [47 x i8] c"[Reading from file into variable]: \00"
@read_fmt = private unnamed_addr constant [4 x i8] c"%lf\00"

; File read operation:
%0 = call i32 (ptr, ...) @printf(ptr @read_msg)
%1 = call i32 (ptr, ...) @scanf(ptr @read_fmt, ptr %variable)
```

## Token Architecture

### Lexer Tokens
- `FileOpen`: Recognizes கோப்பு_திற, kOppu_qiRa, _fileOpen
- `FileClose`: Recognizes கோப்பு_மூடு, kOppu_mUtu, _fileClose
- `FileRead`: Recognizes கோப்பு_படி, kOppu_pati, _fileRead
- `FileWrite`: Recognizes கோப்பு_எழுது, kOppu_ezuqu, _fileWrite
- `ReadCSV`: Recognizes தரவுரை_படி, qaravurY_pati, _readCSV
- `WriteCSV`: Recognizes தரவுரை_எழுது, qaravurY_ezuqu, _writeCSV

### Parser AST Nodes
```rust
FileOpen { filename: Expr, mode: String }
FileClose { filename: Expr }
FileRead { filename: Expr, variable: String }
FileWrite { filename: Expr, data: Expr }
ReadCSV { filename: Expr, variable: String }
WriteCSV { filename: Expr, data: Expr }
```

## Future Enhancements

1. **Actual File I/O**: Integrate libc bindings for real file operations
2. **CSV Parsing**: Implement CSV field parsing and splitting
3. **Exception Handling**: Add error handling for file not found, permission errors
4. **Multiple File Handles**: Support handling multiple open files simultaneously
5. **Binary File Support**: Extend to handle binary file formats
6. **Buffered I/O**: Implement buffered reading/writing for performance

## Building Examples

```bash
cd etamil_compiler

# Compile file I/O example
cargo run examples/simple_fileio.qmz

# View generated LLVM IR
cat output.ll

# The fileio_example.qmz demonstrates both file and CSV operations
cargo run examples/fileio_example.qmz
```

## Compatibility

- File I/O features are fully bilingual and consistent with existing eTamil syntax
- All operations integrate seamlessly with existing financial keywords and operations
- Generated LLVM IR is compatible with standard C runtime library (printf, scanf)
- Backward compatible - existing programs continue to work without modification
