// Encryption/Decryption Module for eTamil Compiler
// Handles secure file storage with .ani (encrypted txt) and .qrv (encrypted csv) formats

use std::fs::{self, File};
use std::io::{Read, Write, Result as IoResult};
use std::path::Path;

/// Encryption handler for secure file storage
#[allow(dead_code)]
pub struct CryptoHandler {
    /// Encryption key (simple XOR-based for demonstration)
    key: Vec<u8>,
}

#[allow(dead_code)]
impl CryptoHandler {
    /// Create a new CryptoHandler with default key
    pub fn new() -> Self {
        // Default encryption key (can be customized)
        let key = b"eTamil_Secure_Key_2026".to_vec();
        CryptoHandler { key }
    }

    /// Create a new CryptoHandler with custom key
    pub fn with_key(key: &str) -> Self {
        CryptoHandler {
            key: key.as_bytes().to_vec(),
        }
    }

    /// Encrypt data using XOR cipher
    fn encrypt_data(&self, data: &[u8]) -> Vec<u8> {
        data.iter()
            .enumerate()
            .map(|(i, &byte)| byte ^ self.key[i % self.key.len()])
            .collect()
    }

    /// Decrypt data using XOR cipher
    fn decrypt_data(&self, data: &[u8]) -> Vec<u8> {
        // XOR is symmetric, so encryption and decryption are the same
        self.encrypt_data(data)
    }

    /// Write encrypted text file (.ani format)
    /// Takes plain text content and saves as encrypted .ani file
    pub fn write_encrypted_txt(&self, filename: &str, content: &str) -> IoResult<()> {
        let encrypted_filename = self.get_encrypted_filename(filename, "ani");
        let encrypted_data = self.encrypt_data(content.as_bytes());
        
        let mut file = File::create(&encrypted_filename)?;
        file.write_all(&encrypted_data)?;
        
        println!("[Encrypted and saved to {}]", encrypted_filename);
        Ok(())
    }

    /// Read encrypted text file (.ani format)
    /// Loads .ani file and returns decrypted plain text
    pub fn read_encrypted_txt(&self, filename: &str) -> IoResult<String> {
        let encrypted_filename = self.get_encrypted_filename(filename, "ani");
        
        let mut file = File::open(&encrypted_filename)?;
        let mut encrypted_data = Vec::new();
        file.read_to_end(&mut encrypted_data)?;
        
        let decrypted_data = self.decrypt_data(&encrypted_data);
        let content = String::from_utf8_lossy(&decrypted_data).to_string();
        
        println!("[Decrypted from {}]", encrypted_filename);
        Ok(content)
    }

    /// Write encrypted CSV file (.qrv format)
    /// Takes plain CSV content and saves as encrypted .qrv file
    pub fn write_encrypted_csv(&self, filename: &str, content: &str) -> IoResult<()> {
        let encrypted_filename = self.get_encrypted_filename(filename, "qrv");
        let encrypted_data = self.encrypt_data(content.as_bytes());
        
        let mut file = File::create(&encrypted_filename)?;
        file.write_all(&encrypted_data)?;
        
        println!("[Encrypted CSV and saved to {}]", encrypted_filename);
        Ok(())
    }

    /// Read encrypted CSV file (.qrv format)
    /// Loads .qrv file and returns decrypted plain CSV
    pub fn read_encrypted_csv(&self, filename: &str) -> IoResult<String> {
        let encrypted_filename = self.get_encrypted_filename(filename, "qrv");
        
        let mut file = File::open(&encrypted_filename)?;
        let mut encrypted_data = Vec::new();
        file.read_to_end(&mut encrypted_data)?;
        
        let decrypted_data = self.decrypt_data(&encrypted_data);
        let content = String::from_utf8_lossy(&decrypted_data).to_string();
        
        println!("[Decrypted CSV from {}]", encrypted_filename);
        Ok(content)
    }

    /// Convert plain filename to encrypted filename
    /// Examples: "data.txt" -> "data.ani", "records.csv" -> "records.qrv"
    fn get_encrypted_filename(&self, filename: &str, extension: &str) -> String {
        let path = Path::new(filename);
        let stem = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("encrypted");
        
        // If path has directory, preserve it
        if let Some(parent) = path.parent() {
            if parent.as_os_str().is_empty() {
                format!("{}.{}", stem, extension)
            } else {
                format!("{}/{}.{}", parent.display(), stem, extension)
            }
        } else {
            format!("{}.{}", stem, extension)
        }
    }

    /// Check if encrypted file exists
    pub fn encrypted_txt_exists(&self, filename: &str) -> bool {
        let encrypted_filename = self.get_encrypted_filename(filename, "ani");
        Path::new(&encrypted_filename).exists()
    }

    /// Check if encrypted CSV file exists
    pub fn encrypted_csv_exists(&self, filename: &str) -> bool {
        let encrypted_filename = self.get_encrypted_filename(filename, "qrv");
        Path::new(&encrypted_filename).exists()
    }

    /// Delete encrypted text file
    pub fn delete_encrypted_txt(&self, filename: &str) -> IoResult<()> {
        let encrypted_filename = self.get_encrypted_filename(filename, "ani");
        fs::remove_file(&encrypted_filename)?;
        println!("[Deleted encrypted file: {}]", encrypted_filename);
        Ok(())
    }

    /// Delete encrypted CSV file
    pub fn delete_encrypted_csv(&self, filename: &str) -> IoResult<()> {
        let encrypted_filename = self.get_encrypted_filename(filename, "qrv");
        fs::remove_file(&encrypted_filename)?;
        println!("[Deleted encrypted CSV file: {}]", encrypted_filename);
        Ok(())
    }
}

impl Default for CryptoHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let crypto = CryptoHandler::new();
        let original = "Hello, eTamil!";
        let encrypted = crypto.encrypt_data(original.as_bytes());
        let decrypted = crypto.decrypt_data(&encrypted);
        let result = String::from_utf8_lossy(&decrypted);
        assert_eq!(original, result);
    }

    #[test]
    fn test_filename_conversion() {
        let crypto = CryptoHandler::new();
        assert_eq!(crypto.get_encrypted_filename("data.txt", "ani"), "data.ani");
        assert_eq!(crypto.get_encrypted_filename("records.csv", "qrv"), "records.qrv");
        assert_eq!(
            crypto.get_encrypted_filename("path/to/file.txt", "ani"),
            "path/to/file.ani"
        );
    }

    #[test]
    fn test_write_read_txt() {
        let crypto = CryptoHandler::new();
        let test_file = "test_crypto.txt";
        let content = "This is a test message!";

        // Write encrypted
        crypto.write_encrypted_txt(test_file, content).unwrap();
        
        // Verify encrypted file exists
        assert!(crypto.encrypted_txt_exists(test_file));

        // Read and verify
        let decrypted = crypto.read_encrypted_txt(test_file).unwrap();
        assert_eq!(content, decrypted);

        // Cleanup
        crypto.delete_encrypted_txt(test_file).unwrap();
    }

    #[test]
    fn test_write_read_csv() {
        let crypto = CryptoHandler::new();
        let test_file = "test_crypto.csv";
        let content = "name,age,city\nRaja,25,Chennai\nDevi,30,Madurai";

        // Write encrypted
        crypto.write_encrypted_csv(test_file, content).unwrap();
        
        // Verify encrypted file exists
        assert!(crypto.encrypted_csv_exists(test_file));

        // Read and verify
        let decrypted = crypto.read_encrypted_csv(test_file).unwrap();
        assert_eq!(content, decrypted);

        // Cleanup
        crypto.delete_encrypted_csv(test_file).unwrap();
    }

    #[test]
    fn test_custom_key() {
        let crypto1 = CryptoHandler::with_key("key1");
        let crypto2 = CryptoHandler::with_key("key2");
        
        let data = "Secret message";
        let encrypted1 = crypto1.encrypt_data(data.as_bytes());
        let encrypted2 = crypto2.encrypt_data(data.as_bytes());
        
        // Different keys should produce different ciphertext
        assert_ne!(encrypted1, encrypted2);
        
        // Same key should decrypt correctly
        let decrypted1 = crypto1.decrypt_data(&encrypted1);
        assert_eq!(data, String::from_utf8_lossy(&decrypted1));
    }
}
