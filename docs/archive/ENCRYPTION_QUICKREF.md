# eTamil Encryption Quick Reference

## File Extensions
- **.ani** → Encrypted text files (from .txt)
- **.qrv** → Encrypted CSV files (from .csv)

## Quick Start

### 1. Import
```rust
use etamil_compiler::fileio::CryptoHandler;
```

### 2. Create Handler
```rust
// Default key
let crypto = CryptoHandler::new();

// Custom key
let crypto = CryptoHandler::with_key("your_secret_key");
```

### 3. Text Files (.txt → .ani)
```rust
// Write
crypto.write_encrypted_txt("file.txt", "content")?;

// Read
let content = crypto.read_encrypted_txt("file.txt")?;

// Check existence
if crypto.encrypted_txt_exists("file.txt") { }

// Delete
crypto.delete_encrypted_txt("file.txt")?;
```

### 4. CSV Files (.csv → .qrv)
```rust
// Write
crypto.write_encrypted_csv("data.csv", "col1,col2\nval1,val2")?;

// Read
let csv = crypto.read_encrypted_csv("data.csv")?;

// Check existence
if crypto.encrypted_csv_exists("data.csv") { }

// Delete
crypto.delete_encrypted_csv("data.csv")?;
```

## Common Patterns

### Save User Data
```rust
let crypto = CryptoHandler::new();
let data = format!("User: {}\nScore: {}", name, score);
crypto.write_encrypted_txt("userdata.txt", &data)?;
```

### Load Configuration
```rust
let crypto = CryptoHandler::new();
if crypto.encrypted_txt_exists("config.txt") {
    let config = crypto.read_encrypted_txt("config.txt")?;
    // Parse config
}
```

### Export CSV Report
```rust
let crypto = CryptoHandler::new();
let csv = "Name,Age\nRaja,25\nDevi,30";
crypto.write_encrypted_csv("report.csv", csv)?;
```

## File Naming Rules

| Input Filename | Encrypted Filename |
|----------------|-------------------|
| data.txt | data.ani |
| records.csv | records.qrv |
| path/to/file.txt | path/to/file.ani |
| my_data.csv | my_data.qrv |

## Testing Commands

```bash
# Run unit tests
cargo test --lib fileio::crypto

# Run demo
cargo run --example crypto_demo

# Test with output
cargo test --lib fileio::crypto -- --nocapture
```

## Security Notes

⚠️ **Current**: XOR cipher (demonstration)  
✅ **Production**: Use AES-256-GCM  
🔑 **Keys**: Store in environment variables  
📝 **Tamil**: Full UTF-8 support ✓  

## Example: Complete Workflow

```rust
use etamil_compiler::fileio::CryptoHandler;

fn main() -> std::io::Result<()> {
    let crypto = CryptoHandler::new();
    
    // Save encrypted
    let message = "Important data: தமிழ்";
    crypto.write_encrypted_txt("message.txt", message)?;
    println!("✅ Saved as message.ani");
    
    // Load encrypted
    let loaded = crypto.read_encrypted_txt("message.txt")?;
    println!("📖 Loaded: {}", loaded);
    
    // Verify
    assert_eq!(message, loaded);
    println!("✓ Verification passed!");
    
    // Cleanup
    crypto.delete_encrypted_txt("message.txt")?;
    println!("🧹 Cleaned up");
    
    Ok(())
}
```

## Error Handling

```rust
match crypto.read_encrypted_txt("file.txt") {
    Ok(content) => {
        println!("Success: {}", content);
    }
    Err(e) => {
        eprintln!("Failed to read encrypted file: {}", e);
        // Handle error
    }
}
```

## API Summary

| Method | Input | Output | Creates |
|--------|-------|--------|---------|
| `write_encrypted_txt` | .txt filename + content | IoResult<()> | .ani file |
| `read_encrypted_txt` | .txt filename | IoResult<String> | - |
| `write_encrypted_csv` | .csv filename + content | IoResult<()> | .qrv file |
| `read_encrypted_csv` | .csv filename | IoResult<String> | - |
| `encrypted_txt_exists` | .txt filename | bool | - |
| `encrypted_csv_exists` | .csv filename | bool | - |
| `delete_encrypted_txt` | .txt filename | IoResult<()> | - |
| `delete_encrypted_csv` | .csv filename | IoResult<()> | - |

---

**Quick Reference v1.0** | eTamil Compiler Encryption System
