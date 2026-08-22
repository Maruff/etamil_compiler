//! Browser bindings for the eTamil front end.
//!
//! The editor on etamil.in runs the real compiler front end rather than a
//! reimplementation of it, so a diagnostic in the browser is the same
//! diagnostic `etamil` prints on the command line -- including the bilingual
//! message text, which comes straight from each error type's `Display`.
//!
//! Only `lexer` -> `parser` -> `check` is reachable from here. Running a
//! program needs `vm`, which reads and writes files and is gated out of a wasm
//! build; see lib.rs.
//!
//! Both entry points return JSON strings rather than `JsValue`. That keeps the
//! dependency list at `wasm-bindgen` alone -- no `serde-wasm-bindgen`, no
//! `js-sys` -- and the payloads are small enough that one `JSON.parse` on the
//! JavaScript side costs nothing measurable.

use serde::Serialize;
use std::collections::HashSet;
use wasm_bindgen::prelude::*;

use crate::check;
use crate::lexer::{self, Spanned, Token};
use crate::parser::{Parser, Stmt};

/// One diagnostic, positioned the way the compiler positions errors: 1-based
/// line, 1-based column, both counting characters rather than bytes so Tamil
/// text reports sensible columns.
///
/// `length` is in characters too. The editor needs a range to underline, and
/// the length of the offending text is the closest thing the error types carry
/// to an end position.
#[derive(Serialize)]
struct Diagnostic {
    line: usize,
    column: usize,
    length: usize,
    /// Always "error" today. Present so warnings can be added without the
    /// JavaScript side having to change shape.
    severity: &'static str,
    /// Which pass rejected the input: "lex", "parse" or "type".
    stage: &'static str,
    /// The compiler's own bilingual message.
    message: String,
}

/// A name the editor can offer as a completion.
#[derive(Serialize)]
struct Symbol {
    name: String,
    /// "function", "parameter" or "variable".
    kind: &'static str,
    /// Shown beside the name: a parameter list for functions, the declared
    /// type for variables that have one.
    detail: String,
}

/// Character count, not byte count -- the compiler's columns are in characters.
fn char_len(s: &str) -> usize {
    s.chars().count()
}

/// Diagnostics for one source file, as a JSON array.
///
/// The passes are run in order and the first one to fail wins: a file that does
/// not lex cannot be parsed, and reporting invented parse errors on top of a
/// real lexical one buries the error the author needs to see.
#[wasm_bindgen]
pub fn diagnostics(source: &str) -> String {
    let out = collect_diagnostics(source);
    serde_json::to_string(&out).unwrap_or_else(|_| "[]".to_string())
}

fn collect_diagnostics(source: &str) -> Vec<Diagnostic> {
    let tokens = match lexer::tokenize(source) {
        Ok(tokens) => tokens,
        Err(errors) => {
            return errors
                .iter()
                .map(|e| Diagnostic {
                    line: e.line,
                    column: e.column,
                    length: char_len(&e.text).max(1),
                    severity: "error",
                    stage: "lex",
                    message: e.to_string(),
                })
                .collect();
        }
    };

    let statements = match Parser::new(tokens.iter()).parse() {
        Ok(statements) => statements,
        Err(e) => {
            return vec![Diagnostic {
                line: e.line,
                column: e.column,
                // `found` is empty at end of input, where there is nothing to
                // underline; one column keeps the marker visible.
                length: char_len(&e.found).max(1),
                severity: "error",
                stage: "parse",
                message: e.to_string(),
            }];
        }
    };

    match check::check(&statements) {
        Ok(()) => Vec::new(),
        Err(errors) => errors
            .iter()
            .map(|e| Diagnostic {
                line: e.line,
                column: e.column,
                length: char_len(&e.name).max(1),
                severity: "error",
                stage: "type",
                message: e.to_string(),
            })
            .collect(),
    }
}

/// Completion candidates for one source file, as a JSON array.
///
/// Falls back to the identifiers in the token stream when the file does not
/// parse -- which is most of the time, because the moment you want a completion
/// is the moment you have typed half a statement. A completion list that
/// vanishes on the first syntax error is a completion list nobody can use.
#[wasm_bindgen]
pub fn symbols(source: &str) -> String {
    let out = collect_symbols(source);
    serde_json::to_string(&out).unwrap_or_else(|_| "[]".to_string())
}

fn collect_symbols(source: &str) -> Vec<Symbol> {
    let Ok(tokens) = lexer::tokenize(source) else {
        return Vec::new();
    };

    match Parser::new(tokens.iter()).parse() {
        Ok(statements) => {
            let mut found = Vec::new();
            let mut seen = HashSet::new();
            walk(&statements, &mut found, &mut seen);
            found
        }
        Err(_) => identifiers_from_tokens(&tokens),
    }
}

/// Every distinct identifier in the token stream, in first-appearance order.
///
/// Keyword spellings never reach here: the lexer resolves those to their own
/// token variants, so `Token::Identifier` is exactly the set of author-chosen
/// names. The editor already offers keywords from its generated token table.
fn identifiers_from_tokens(tokens: &[Spanned]) -> Vec<Symbol> {
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    for spanned in tokens {
        if let Token::Identifier(name) = &spanned.token {
            if seen.insert(name.clone()) {
                out.push(Symbol {
                    name: name.clone(),
                    kind: "variable",
                    detail: String::new(),
                });
            }
        }
    }
    out
}

/// Record one name, first occurrence winning.
///
/// A free function rather than a closure over `seen`: a closure capturing it
/// mutably cannot coexist with the recursive `walk` calls that also need it.
fn push(
    out: &mut Vec<Symbol>,
    seen: &mut HashSet<String>,
    name: &str,
    kind: &'static str,
    detail: String,
) {
    if name.is_empty() || !seen.insert(name.to_string()) {
        return;
    }
    out.push(Symbol { name: name.to_string(), kind, detail });
}

/// Collect declared names, descending into every block that can hold them.
///
/// Scope is deliberately flattened: a name declared inside a function is
/// offered everywhere. Getting this right needs the cursor position, which
/// belongs in a later pass; over-offering is the friendlier failure.
fn walk(statements: &[Stmt], out: &mut Vec<Symbol>, seen: &mut HashSet<String>) {
    for statement in statements {
        match statement {
            Stmt::Assign { name, declared, .. } => {
                let detail = declared.as_ref().map(|d| d.name().to_string()).unwrap_or_default();
                push(out, seen, name, "variable", detail);
            }
            Stmt::FunctionDef { name, params, body } => {
                push(out, seen, name, "function", format!("({})", params.join(", ")));
                for param in params {
                    push(out, seen, param, "parameter", String::new());
                }
                walk(body, out, seen);
            }
            Stmt::ForEach { var, body, .. } => {
                push(out, seen, var, "variable", String::new());
                walk(body, out, seen);
            }
            Stmt::If { then_branch, else_branch, .. } => {
                walk(then_branch, out, seen);
                if let Some(alternative) = else_branch {
                    walk(alternative, out, seen);
                }
            }
            Stmt::Loop { body, .. } => walk(body, out, seen),
            Stmt::SetIndex { name, .. } | Stmt::SetField { name, .. } => {
                push(out, seen, name, "variable", String::new());
            }
            Stmt::FileRead { variable, .. } | Stmt::ReadCSV { variable, .. } => {
                push(out, seen, variable, "variable", String::new());
            }
            Stmt::DBQuery { result_var, .. } => {
                push(out, seen, result_var, "variable", String::new());
            }
            // Statements that declare nothing. Listed as a catch-all rather
            // than exhaustively so a new Stmt variant does not break the wasm
            // build -- it only means that variant declares no completions yet.
            _ => {}
        }
    }
}

/// The compiler version this wasm module was built from, so the editor can say
/// which compiler produced a diagnostic.
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
