// Demonstration of eTamil Encryption/Decryption for File Operations
// Shows .ani (encrypted txt) and .qrv (encrypted csv) file handling

use etamil_compiler::fileio::{CryptoHandler, CSVProcessor};

fn main() {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║   eTamil File Encryption/Decryption Demo                  ║");
    println!("║   .ani files → Encrypted Text Files                       ║");
    println!("║   .qrv files → Encrypted CSV Files                        ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    let crypto = CryptoHandler::new();

    // ========== TEXT FILE ENCRYPTION/DECRYPTION (.ani) ==========
    println!("\n📄 TEXT FILE OPERATIONS (.txt → .ani)");
    println!("═══════════════════════════════════════════════════════════\n");

    let text_filename = "examples/sample_message.txt";
    let message = "இது ஒரு இரகசிய செய்தி!\nThis is a secret message!\nSecure data storage in eTamil.";

    println!("✍️  Original Text Content:");
    println!("{}\n", message);

    // Encrypt and save as .ani
    match crypto.write_encrypted_txt(text_filename, message) {
        Ok(_) => println!("✅ Text encrypted successfully!"),
        Err(e) => println!("❌ Encryption failed: {}", e),
    }

    // Read and decrypt from .ani
    println!("\n📖 Reading encrypted file...");
    match crypto.read_encrypted_txt(text_filename) {
        Ok(decrypted) => {
            println!("✅ Text decrypted successfully!");
            println!("\n📝 Decrypted Content:");
            println!("{}\n", decrypted);
            
            // Verify content matches
            if decrypted == message {
                println!("✓ Content verification: PASSED");
            } else {
                println!("✗ Content verification: FAILED");
            }
        }
        Err(e) => println!("❌ Decryption failed: {}", e),
    }

    // ========== CSV FILE ENCRYPTION/DECRYPTION (.qrv) ==========
    println!("\n\n📊 CSV FILE OPERATIONS (.csv → .qrv)");
    println!("═══════════════════════════════════════════════════════════\n");

    let csv_filename = "examples/student_records.csv";
    let csv_data = "பெயர்,வயது,மதிப்பெண்\nராஜா,20,95\nதேவி,21,88\nகுமார்,19,92";

    println!("✍️  Original CSV Content:");
    println!("{}\n", csv_data);

    // Parse CSV to show structure
    println!("📋 Parsed CSV Structure:");
    for (i, line) in csv_data.lines().enumerate() {
        let fields = CSVProcessor::parse_csv_line(line);
        if i == 0 {
            println!("   Headers: {:?}", fields);
        } else {
            println!("   Row {}: {:?}", i, fields);
        }
    }

    // Encrypt and save as .qrv
    println!("\n🔒 Encrypting CSV data...");
    match crypto.write_encrypted_csv(csv_filename, csv_data) {
        Ok(_) => println!("✅ CSV encrypted successfully!"),
        Err(e) => println!("❌ Encryption failed: {}", e),
    }

    // Read and decrypt from .qrv
    println!("\n📖 Reading encrypted CSV file...");
    match crypto.read_encrypted_csv(csv_filename) {
        Ok(decrypted) => {
            println!("✅ CSV decrypted successfully!");
            println!("\n📝 Decrypted CSV Content:");
            println!("{}\n", decrypted);
            
            // Verify content matches
            if decrypted == csv_data {
                println!("✓ Content verification: PASSED");
            } else {
                println!("✗ Content verification: FAILED");
            }
        }
        Err(e) => println!("❌ Decryption failed: {}", e),
    }

    // ========== CUSTOM ENCRYPTION KEY DEMO ==========
    println!("\n\n🔑 CUSTOM ENCRYPTION KEY DEMO");
    println!("═══════════════════════════════════════════════════════════\n");

    let custom_key = "MySecretKey_தமிழ்_2026";
    let crypto_custom = CryptoHandler::with_key(custom_key);
    
    let secret_file = "examples/confidential.txt";
    let secret_data = "மிக இரகசிய தகவல்\nHighly confidential information";

    println!("🔐 Using custom key: {}", custom_key);
    println!("✍️  Secret Data: {}\n", secret_data);

    crypto_custom.write_encrypted_txt(secret_file, secret_data).ok();
    
    match crypto_custom.read_encrypted_txt(secret_file) {
        Ok(decrypted) => {
            println!("✅ Decrypted with custom key!");
            println!("📝 Content: {}\n", decrypted);
        }
        Err(e) => println!("❌ Failed: {}", e),
    }

    // Try reading with wrong key (will produce garbage)
    println!("⚠️  Attempting to decrypt with default key (should fail)...");
    let crypto_default = CryptoHandler::new();
    match crypto_default.read_encrypted_txt(secret_file) {
        Ok(wrong_decrypt) => {
            println!("📝 Result (with wrong key): {}", wrong_decrypt);
            println!("   ↳ Produces garbage when key doesn't match!\n");
        }
        Err(e) => println!("❌ Read failed: {}", e),
    }

    // ========== FILE EXISTENCE CHECKS ==========
    println!("\n📁 FILE EXISTENCE VERIFICATION");
    println!("═══════════════════════════════════════════════════════════\n");

    println!("Checking encrypted files:");
    println!("  • sample_message.ani exists: {}", 
             crypto.encrypted_txt_exists(text_filename));
    println!("  • student_records.qrv exists: {}", 
             crypto.encrypted_csv_exists(csv_filename));
    println!("  • confidential.ani exists: {}", 
             crypto_custom.encrypted_txt_exists(secret_file));

    // ========== CLEANUP ==========
    println!("\n\n🧹 CLEANUP");
    println!("═══════════════════════════════════════════════════════════\n");

    println!("Removing encrypted files...");
    crypto.delete_encrypted_txt(text_filename).ok();
    crypto.delete_encrypted_csv(csv_filename).ok();
    crypto_custom.delete_encrypted_txt(secret_file).ok();

    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║   ✅ DEMO COMPLETE                                         ║");
    println!("║                                                            ║");
    println!("║   Summary:                                                 ║");
    println!("║   • Text files: .txt → .ani (encrypted)                    ║");
    println!("║   • CSV files: .csv → .qrv (encrypted)                     ║");
    println!("║   • XOR cipher with customizable key                       ║");
    println!("║   • Secure backend storage                                 ║");
    println!("╚════════════════════════════════════════════════════════════╝");
}
