# eTamil Compiler - Test Data Files

## Overview
This directory contains sample text and CSV files for testing the eTamil compiler's file I/O operations.

---

## Text Files

### 1. sample.txt
**Purpose:** Basic text file for general file I/O testing  
**Content:** Simple multi-line text with English and Tamil characters  
**Use Case:** Test basic fileread and filewrite operations

### 2. input.txt
**Purpose:** Structured input data file  
**Content:** Formatted text with headers and various data types  
**Use Case:** Test reading formatted text data

### 3. numbers.txt
**Purpose:** Numeric data file  
**Content:** List of numbers (one per line)  
**Use Case:** Test reading and processing numeric data

### 4. tamil_sample.txt
**Purpose:** Tamil language text file  
**Content:** Text written in Tamil script with mixed English  
**Use Case:** Test Unicode/Tamil character handling

---

## CSV Files

### 1. data.csv
**Purpose:** Employee data  
**Columns:** Name, Age, City, Salary  
**Rows:** 8 employee records  
**Use Case:** Test basic CSV parsing and data extraction

### 2. products.csv
**Purpose:** Product inventory data  
**Columns:** ProductID, ProductName, Category, Price, Stock  
**Rows:** 8 product records  
**Use Case:** Test CSV reading with numeric IDs and categorical data

### 3. students.csv
**Purpose:** Student grades data  
**Columns:** StudentID, Name, Grade, Subject, Marks  
**Rows:** 8 student records  
**Use Case:** Test CSV operations with student performance data

---

## Usage Examples

### Reading a Text File
```etamil
விரி "r"              // fileopen "r"
படி data             // fileread data
மூடு                 // fileclose
```

### Writing to a Text File
```etamil
விரி "w"              // fileopen "w"
எழுது value          // filewrite value
மூடு                 // fileclose
```

### Reading CSV File
```etamil
கோப்பு_திற "r"              // _fileOpen "r"
தரவுரை_படி record       // _readCSV record
கோப்பு_மூடு                 // _fileClose
```

### Writing CSV File
```etamil
கோப்பு_திற "w"              // _fileOpen "w"
தரவுரை_எழுது data       // _writeCSV data
கோப்பு_மூடு                 // _fileClose
```

---

## Testing Workflow

### Step 1: Compile Example
```bash
cd etamil_compiler
cargo run examples/fileio_example.qmz
```

### Step 2: View Generated LLVM IR
```bash
cat output.ll
```

### Step 3: Test with Different Files
Modify your `.qmz` programs to reference these test files.

---

## File Specifications

| File | Type | Size | Lines | Encoding |
|------|------|------|-------|----------|
| sample.txt | Text | ~200 bytes | 6 | UTF-8 |
| input.txt | Text | ~300 bytes | 14 | UTF-8 |
| numbers.txt | Text | ~50 bytes | 10 | ASCII |
| tamil_sample.txt | Text | ~400 bytes | 15 | UTF-8 |
| data.csv | CSV | ~350 bytes | 9 | UTF-8 |
| products.csv | CSV | ~400 bytes | 9 | UTF-8 |
| students.csv | CSV | ~400 bytes | 9 | UTF-8 |

---

## Data Validation

### CSV Structure
All CSV files follow standard format:
- First row: Headers
- Subsequent rows: Data records
- Delimiter: Comma (,)
- No quoted fields (for simplicity)

### Text Files
- UTF-8 encoding for Tamil characters
- Unix-style line endings (LF)
- No trailing whitespace

---

## Test Scenarios

### Scenario 1: Basic File Read
**File:** sample.txt  
**Operations:** Open → Read → Close  
**Expected:** Successfully read content

### Scenario 2: CSV Data Processing
**File:** data.csv  
**Operations:** Open → ReadCSV → Process → Close  
**Expected:** Parse employee records correctly

### Scenario 3: Numeric Data
**File:** numbers.txt  
**Operations:** Open → Read → Sum/Average → Close  
**Expected:** Calculate statistics correctly

### Scenario 4: Tamil Text Handling
**File:** tamil_sample.txt  
**Operations:** Open → Read → Display → Close  
**Expected:** Handle Unicode correctly

### Scenario 5: Multiple CSV Records
**File:** products.csv or students.csv  
**Operations:** Open → Loop ReadCSV → Process Each → Close  
**Expected:** Process all records sequentially

---

## Notes

1. **Current Implementation:** File operations use printf/scanf simulation
2. **Future Enhancement:** Will use actual file I/O (fopen, fread, fwrite)
3. **Encoding:** All files use UTF-8 to support Tamil characters
4. **CSV Format:** Standard RFC 4180 (simplified - no quoted fields)

---

## Adding New Test Files

To add custom test files:

1. Create file in `examples/` directory
2. Use UTF-8 encoding
3. For CSV: Use comma delimiter, headers in first row
4. For text: Use Unix line endings (LF)
5. Update this README with file description

---

## File Generation Script

To regenerate test files or create additional ones:

```bash
# Create a new text file
echo "Your test content" > examples/newtest.txt

# Create a new CSV file
cat > examples/newtest.csv << 'EOF'
Header1,Header2,Header3
Value1,Value2,Value3
EOF
```

---

**Generated:** January 25, 2026  
**eTamil Compiler Version:** 1.0.0  
**Status:** ✅ Ready for Testing
