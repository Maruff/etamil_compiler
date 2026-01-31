#![allow(unused_imports)]
// I/O Module for eTamil Compiler
// Handles file operations, CSV processing, and encryption

#[cfg(feature = "llvm")]
pub mod csv_handler;
pub mod crypto;

#[cfg(feature = "llvm")]
pub use csv_handler::{FileIOHandler, CSVProcessor};
pub use crypto::CryptoHandler;
