//! The eTamil compiler.
//!
//! Pipeline: `lexer` -> `parser` -> `vm::bytecode::compiler` -> `vm::interpreter`.
//! `codegen` is an optional LLVM backend replacing the last two stages,
//! enabled with `--features llvm`.

pub mod lexer;
pub mod parser;
pub mod module;
pub mod codegen;
pub mod fileio;
pub mod vm;
pub mod http;
