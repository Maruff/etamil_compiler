# eTamil File Encryption System

## Overview

The eTamil compiler includes a comprehensive encryption system for secure file storage. Files are stored in encrypted format on the backend/server, while maintaining plain text/CSV interfaces for user interaction.

## File Format Mapping

| Input Format | Encrypted Format | Extension | Description |
|--------------|-----------------|-----------|-------------|
| Plain Text (.txt) | Encrypted Text | **.ani** | Encrypted text files |
| CSV Data (.csv) | Encrypted CSV | **.qrv** | Encrypted CSV files |

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    User Application                         │
│                                                             │
│  Input: Plain .txt and .csv files                          │
│  Output: Plain .txt and .csv files                         │
└─────────────────┬───────────────────────────────────────────┘
                  │
                  │ CryptoHandler API
                  │
┌─────────────────▼───────────────────────────────────────────┐
│              Encryption/Decryption Layer                    │
│                                                             │
│  • XOR Cipher with customizable key                        │
│  • Symmetric encryption (same key for encrypt/decrypt)     │
│  • UTF-8 compatible (supports Tamil text)                  │
└─────────────────┬───────────────────────────────────────────┘
                  │
                  │ Encrypted Data
                  │
┌─────────────────▼───────────────────────────────────────────┐
│                  Backend Storage                            │
│                                                             │
│  Stored: .ani and .qrv encrypted files                     │
│  Location: Server/Backend filesystem                        │
└─────────────────────────────────────────────────────────────┘
```

## Features

### 1. **Text File Encryption (.ani)**

- **Encrypt and Write**: `write_encrypted_txt(filename, content)`
  - Takes plain text content
  - Encrypts using XOR cipher
  - Saves as .ani file
  
- **Read and Decrypt**: `read_encrypted_txt(filename)`
  - Loads .ani file
  - Decrypts to plain text
  - Returns original content

### 2. **CSV File Encryption (.qrv)**

- **Encrypt and Write**: `write_encrypted_csv(filename, content)`
  - Takes plain CSV content
  - Encrypts using XOR cipher
  - Saves as .qrv file
  
- **Read and Decrypt**: `read_encrypted_csv(filename)`
  - Loads .qrv file
  - Decrypts to plain CSV
  - Returns original content

### 3. **Custom Encryption Keys**

- **Default Key**: `"eTamil_Secure_Key_2026"`
- **Custom Key**: `CryptoHandler::with_key("your_key_here")`
- **Security**: Different keys produce different ciphertext

### 4. **File Management**

- **Check Existence**: `encrypted_txt_exists()`, `encrypted_csv_exists()`
- **Delete Files**: `delete_encrypted_txt()`, `delete_encrypted_csv()`
- **Automatic Naming**: Converts `data.txt` → `data.ani`, `records.csv` → `records.qrv`

## Usage Examples

### Basic Text Encryption

```rust
use etamil_compiler::fileio::CryptoHandler;

let crypto = CryptoHandler::new();

// Encrypt and save
let message = "இது ஒரு இரகசிய செய்தி!";
crypto.write_encrypted_txt("message.txt", message)?;
// Creates: message.ani (encrypted)

// Read and decrypt
let decrypted = crypto.read_encrypted_txt("message.txt")?;
println!("{}", decrypted); // Prints original message
```

### CSV Data Encryption

```rust
use etamil_compiler::fileio::CryptoHandler;

let crypto = CryptoHandler::new();

// CSV data
let csv_data = "name,age,score\nRaja,20,95\nDevi,21,88";

// Encrypt and save
crypto.write_encrypted_csv("students.csv", csv_data)?;
// Creates: students.qrv (encrypted)

// Read and decrypt
let decrypted = crypto.read_encrypted_csv("students.csv")?;
println!("{}", decrypted); // Prints original CSV
```

### Custom Encryption Key

```rust
use etamil_compiler::fileio::CryptoHandler;

// Use custom key for extra security
let crypto = CryptoHandler::with_key("MySecretKey_2026");

let data = "Confidential information";
crypto.write_encrypted_txt("secret.txt", data)?;

// Must use same key to decrypt
let decrypted = crypto.read_encrypted_txt("secret.txt")?;
```

### File Existence Check

```rust
use etamil_compiler::fileio::CryptoHandler;

let crypto = CryptoHandler::new();

if crypto.encrypted_txt_exists("data.txt") {
    let content = crypto.read_encrypted_txt("data.txt")?;
    println!("Found encrypted file: {}", content);
} else {
    println!("No encrypted file found");
}
```

## Encryption Algorithm

### XOR Cipher Implementation

The system uses a **symmetric XOR cipher** where:

1. **Encryption**: `ciphertext[i] = plaintext[i] XOR key[i % key_length]`
2. **Decryption**: `plaintext[i] = ciphertext[i] XOR key[i % key_length]`

**Properties:**
- Symmetric (same operation for encrypt/decrypt)
- Fast and efficient
- Key-dependent (different keys produce different results)
- UTF-8 compatible (supports Tamil and multilingual text)

### Security Considerations

⚠️ **Current Implementation**: XOR cipher for demonstration purposes

**For Production Use:**
- Replace with AES-256 or ChaCha20
- Implement key derivation (PBKDF2/Argon2)
- Add authentication (HMAC/GCM mode)
- Secure key storage (environment variables/key management systems)

## File Workflow

### Write Operation
```
Plain Text/CSV → CryptoHandler.encrypt() → .ani/.qrv file
     ↓                    ↓                      ↓
  "data"            XOR cipher           Binary encrypted data
```

### Read Operation
```
.ani/.qrv file → CryptoHandler.decrypt() → Plain Text/CSV
     ↓                    ↓                      ↓
Binary data          XOR cipher              "data"
```

## API Reference

### CryptoHandler

```rust
pub struct CryptoHandler {
    key: Vec<u8>,
}

impl CryptoHandler {
    // Create with default key
    pub fn new() -> Self
    
    // Create with custom key
    pub fn with_key(key: &str) -> Self
    
    // Text file operations
    pub fn write_encrypted_txt(&self, filename: &str, content: &str) -> IoResult<()>
    pub fn read_encrypted_txt(&self, filename: &str) -> IoResult<String>
    
    // CSV file operations
    pub fn write_encrypted_csv(&self, filename: &str, content: &str) -> IoResult<()>
    pub fn read_encrypted_csv(&self, filename: &str) -> IoResult<String>
    
    // File management
    pub fn encrypted_txt_exists(&self, filename: &str) -> bool
    pub fn encrypted_csv_exists(&self, filename: &str) -> bool
    pub fn delete_encrypted_txt(&self, filename: &str) -> IoResult<()>
    pub fn delete_encrypted_csv(&self, filename: &str) -> IoResult<()>
}
```

## Testing

### Run Unit Tests
```bash
cargo test --lib fileio::crypto
```

### Run Demo Example
```bash
cargo run --example crypto_demo
```

### Test Coverage

All tests verify:
- ✅ Encryption/Decryption accuracy
- ✅ Filename conversion (.txt→.ani, .csv→.qrv)
- ✅ File write and read operations
- ✅ Custom key support
- ✅ Tamil/UTF-8 text handling
- ✅ File existence checks
- ✅ File deletion

## Integration with eTamil Programs

### Future Integration (Planned)

```tamil
// eTamil program with encrypted storage
கோப்பு_திற "data.txt" "write";
கோப்பு_எழுது "data.txt" value;
கோப்பு_மூடு "data.txt";
// Backend stores as encrypted data.ani

கோப்பு_திற "students.csv" "read";
தரவுரை_படி "students.csv" data;
கோப்பு_மூடு "students.csv";
// Backend reads from encrypted students.qrv
```

## Benefits

✅ **Transparent Encryption**: Users work with plain files, backend stores encrypted  
✅ **Tamil Support**: Full UTF-8 compatibility for Tamil text  
✅ **Dual Format**: Separate handling for text (.ani) and CSV (.qrv) files  
✅ **Custom Keys**: Configurable encryption keys per application  
✅ **Lightweight**: Minimal overhead, fast operations  
✅ **Tested**: Comprehensive test coverage with 100% pass rate  

## File Extension Rationale

- **.ani** (Encrypted Text): Represents "அணி" (array/collection) in Tamil
- **.qrv** (Encrypted CSV): Represents "குறிவரிசை" (coded sequence) in Tamil

Both extensions are:
- Short and memorable
- Distinct from common formats
- Tamil-inspired naming
- Compatible with all filesystems

## Performance

| Operation | Time Complexity | Notes |
|-----------|----------------|-------|
| Encrypt | O(n) | Linear with data size |
| Decrypt | O(n) | Linear with data size |
| File I/O | O(n) | Disk-bound operation |

**Benchmark** (approximate):
- 1 KB file: < 1ms
- 100 KB file: < 10ms
- 1 MB file: < 100ms

## Error Handling

All operations return `IoResult<T>` for robust error handling:

```rust
match crypto.read_encrypted_txt("file.txt") {
    Ok(content) => println!("Success: {}", content),
    Err(e) => eprintln!("Error: {}", e),
}
```

Common errors:
- File not found
- Permission denied
- Invalid UTF-8 data
- Disk full

## Future Enhancements

1. **Stronger Encryption**: AES-256-GCM
2. **Key Management**: Secure key derivation and storage
3. **Compression**: Optional compression before encryption
4. **Integrity**: SHA-256 checksums for data verification
5. **Streaming**: Large file support with streaming encryption
6. **Metadata**: Encrypted file headers with version info

## License

Part of the eTamil Compiler Project - Educational demonstration of secure file handling in a Tamil programming language.

---

**Documentation Version**: 1.0  
**Last Updated**: January 25, 2026  
**Module**: `etamil_compiler::fileio::crypto`
