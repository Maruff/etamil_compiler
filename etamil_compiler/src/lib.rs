//! The eTamil compiler.
//!
//! Pipeline: `lexer` -> `parser` -> `vm::bytecode::compiler` -> `vm::interpreter`.
//! `codegen` is an optional LLVM backend replacing the last two stages,
//! enabled with `--features llvm`.
//!
//! # wasm builds
//!
//! `lexer`, `parser` and `check` depend on nothing but `logos`, `rust_decimal`
//! and `std` collections, so they compile for `wasm32-unknown-unknown` as they
//! are. Everything below them needs an operating system -- sockets, a
//! filesystem, threads -- and is gated out of a wasm build rather than being
//! ported. That is what lets the editor on etamil.in report real diagnostics
//! with no server: it runs the real front end, not a reimplementation of it.
//!
//! Build it with:
//!
//! ```text
//! cargo build --release --target wasm32-unknown-unknown --no-default-features
//! ```

// --- Front end: portable, and the whole of what a browser build exposes ---
pub mod lexer;
pub mod parser;
pub mod check;

// --- Everything below needs an OS ---

// Reads imported files from disk.
#[cfg(not(target_family = "wasm"))]
pub mod module;
// Reads stdin, writes stdout.
#[cfg(not(target_family = "wasm"))]
pub mod repl;
#[cfg(not(target_family = "wasm"))]
pub mod signing;
#[cfg(not(target_family = "wasm"))]
pub mod redis;
// What the LLVM backend must refuse, decided by reading the program. Walks the
// AST and nothing else, so it builds on machines that cannot build the backend
// it guards — which is most of them.
pub mod codegen_limits;
// Documents, behind a feature like the other non-bundled drivers.
#[cfg(all(feature = "mongodb", not(target_family = "wasm")))]
pub mod mongo;
// Client certificates are only meaningful when there is a client.
#[cfg(all(feature = "http-client", not(target_family = "wasm")))]
pub mod mtls;
#[cfg(not(target_family = "wasm"))]
pub mod net;
#[cfg(not(target_family = "wasm"))]
pub mod db;
// Internally `#[cfg(feature = "llvm")]` throughout, and llvm-sys has no wasm
// build; gated here too so a wasm build does not depend on that staying true.
#[cfg(not(target_family = "wasm"))]
pub mod codegen;
#[cfg(not(target_family = "wasm"))]
pub mod fileio;
// The interpreter reads and writes files for the File I/O statements.
#[cfg(not(target_family = "wasm"))]
pub mod vm;
#[cfg(not(target_family = "wasm"))]
pub mod http;

// --- Browser bindings ---
#[cfg(target_family = "wasm")]
pub mod wasm;
