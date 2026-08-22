//! The eTamil compiler.
//!
//! Pipeline: `lexer` -> `parser` -> `vm::bytecode::compiler` -> `vm::interpreter`.
//! `codegen` is an optional LLVM backend replacing the last two stages,
//! enabled with `--features llvm`.

pub mod lexer;
pub mod parser;
pub mod check;
pub mod module;
pub mod repl;
pub mod signing;
pub mod redis;
// Client certificates are only meaningful when there is a client.
#[cfg(feature = "http-client")]
pub mod mtls;
pub mod net;
pub mod db;
pub mod codegen;
pub mod fileio;
pub mod vm;
pub mod http;
