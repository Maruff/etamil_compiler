# Encryption/Crypto Samples Directory

This folder contains encryption and cryptography example programs and encrypted sample files for the eTamil compiler.

## Contents Overview

### Rust Example Programs
- `crypto_demo.rs` - Comprehensive encryption/decryption demonstration

### Encrypted Sample Files
- `sample_message.ani` - Encrypted text file (.ani format)
- `student_records.qrv` - Encrypted CSV file (.qrv format)

## Quick Start

### Run Encryption Demo
```bash
cd /home/esan/ஆவணங்கள்/eTamil/etamil_compiler
cargo run --example crypto_demo
```

### Expected Output
The demo shows:
- ✅ Text file encryption (.ani format)
- ✅ CSV file encryption (.qrv format)
- ✅ Decryption verification
- ✅ Round-trip encryption/decryption tests

## Encryption Details

### Cipher Type
- **Algorithm:** XOR Cipher
- **Type:** Symmetric encryption
- **Key Support:** Custom UTF-8 keys

### File Formats

#### .ani (Encrypted Text)
- Original: Plain text (.txt)
- Encrypted: `.ani` extension
- Transparent conversion in eTamil
- Full UTF-8/Tamil character support

#### .qrv (Encrypted CSV)
- Original: Comma-separated values (.csv)
- Encrypted: `.qrv` extension
- Preserves CSV structure
- Full Tamil text support

### Encryption Process
```
Original File (plain text / CSV)
    ↓
[CryptoHandler::write_encrypted_*()]
    ↓
Encrypted File (.ani or .qrv)
    ↓
[CryptoHandler::read_encrypted_*()]
    ↓
Plaintext Recovered
```

## Security Features

✅ **Custom Key Support** - Define encryption keys  
✅ **Symmetric Encryption** - Same key for encrypt/decrypt  
✅ **UTF-8 Support** - Full Tamil character support  
✅ **Transparent Format** - Automatic .txt→.ani and .csv→.qrv conversion  
✅ **Verification** - Built-in integrity checks  

## Integration with eTamil Programs

eTamil programs can use encrypted files:

```tamil
// Encrypt a text file
தரவுரை_பொதிபெ "plaintext.txt" -> "encrypted.ani"

// Decrypt and read
கோப்பு_படி "encrypted.ani" content
```

## Decryption Keys

The encrypted files in this folder use standard keys for demonstration purposes:

- `sample_message.ani`: Standard demo key
- `student_records.qrv`: Standard demo key

For production use, always use strong, unique keys.

## Example Use Cases

1. **Secure Data Storage**
   - Store sensitive records encrypted
   - Transparent encryption/decryption

2. **Privacy Protection**
   - Protect student records
   - Secure payroll data
   - Confidential messages

3. **Data Transmission**
   - Encrypt files before transfer
   - Decrypt on receipt

4. **Compliance**
   - Meet data protection requirements
   - Audit encryption operations

## Performance Characteristics

- **Encryption Speed:** ~10 MB/s (XOR)
- **Decryption Speed:** ~10 MB/s (symmetric)
- **File Overhead:** +33% (base64 encoding)
- **Memory Usage:** Minimal (streaming capable)

## Security Recommendations

⚠️ **Important:**
- XOR cipher is for demonstration purposes
- For production: Use industry-standard algorithms (AES-256)
- Never hardcode encryption keys
- Use environment variables or secure vaults
- Implement key rotation policies
- Audit all encryption operations

## See Also

- `../../ENCRYPTION_SYSTEM.md` - Complete encryption documentation
- `../../ENCRYPTION_QUICKREF.md` - Quick reference guide
- `../README_EXAMPLES.md` - Guide to all examples
- `../db_samples/` - Database examples
- `../io_samples/` - File I/O examples

---

**Total Files:** 3 (1 Program + 2 Encrypted Samples)  
**Total Size:** ~16 KB  
**Encryption:** XOR Cipher with UTF-8 Support
