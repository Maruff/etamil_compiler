// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Mohammed Maruff (Esan Maruff) <esan@etamil.in>
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
// Authenticated encryption behind மறை and வெளிப்படு. Portable: XChaCha20-
// Poly1305 and Argon2id are pure Rust, and getrandom already has its wasm_js
// feature enabled below, so this works in the browser build too.
pub mod crypt;
// Portable: HMAC-SHA256 over hmac/sha2/subtle, all pure Rust.
pub mod signing;
#[cfg(not(target_family = "wasm"))]
pub mod redis;
// What the LLVM backend's emitted IR calls into: every eTamil value as a handle
// into an arena, and every operation on one as a C-ABI call. Not behind the
// `llvm` feature and not behind `cfg(not(wasm))` — it links no LLVM and touches
// no OS beyond stdout, and the `cdylib` has to export it on any machine that
// might link an `output.ll`.
pub mod runtime;
// Documents, behind a feature like the other non-bundled drivers.
#[cfg(all(feature = "mongodb", not(target_family = "wasm")))]
pub mod mongo;
// Client certificates are only meaningful when there is a client.
#[cfg(all(feature = "http-client", not(target_family = "wasm")))]
pub mod mtls;
// Portable with `http-client` off, which a wasm build implies: `request` then
// takes the existing no-HTTP-client fallback, and sign/verify are pure Rust.
pub mod net;
#[cfg(not(target_family = "wasm"))]
pub mod db;
// Internally `#[cfg(feature = "llvm")]` throughout, and llvm-sys has no wasm
// build; gated here too so a wasm build does not depend on that staying true.
#[cfg(not(target_family = "wasm"))]
pub mod codegen;
// Portable: the bytecode compiler and value layer touch no OS at all, and the
// interpreter's input and output go through vm::host, which has a browser
// implementation. The archive and subprocess helpers inside it are gated
// individually.
pub mod vm;
#[cfg(not(target_family = "wasm"))]
pub mod http;

// --- Browser bindings ---

// Stand-ins for db, redis and http, re-exported under those names so the
// interpreter's `crate::db::...` paths resolve unchanged. See wasm_stubs.rs for
// why this is done as substitution rather than as gating.
#[cfg(target_family = "wasm")]
mod wasm_stubs;
#[cfg(target_family = "wasm")]
pub use wasm_stubs::{db, http, redis};

#[cfg(target_family = "wasm")]
pub mod wasm;
