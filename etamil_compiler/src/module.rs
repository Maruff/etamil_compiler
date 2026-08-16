//! Module loading: `இறக்கு "kOppu.qmz";`
//!
//! A program is assembled by splicing each imported file's statements in
//! ahead of the importer's own. Paths resolve relative to the importing
//! file, a file imported twice is included once, and an import cycle stops
//! rather than looping — the same guarantees `#pragma once` gives, without
//! needing the author to think about it.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::lexer;
use crate::parser::{Parser, Stmt};

/// Parse one source string into statements, with lexical errors reported.
fn parse_source(source: &str) -> Result<Vec<Stmt>, String> {
    let tokens = lexer::tokenize(source).map_err(|errors| {
        errors
            .iter()
            .map(|e| e.to_string())
            .collect::<Vec<_>>()
            .join("\n  ")
    })?;
    let mut parser = Parser::new(tokens.iter());
    Ok(parser.parse())
}

/// Load a program from disk, resolving its imports.
pub fn load_file(path: &Path) -> Result<Vec<Stmt>, String> {
    let mut visited = HashSet::new();
    load_inner(path, &mut visited)
}

/// Load a program held in memory. Imports resolve relative to `base_dir`.
pub fn load_source(source: &str, base_dir: &Path) -> Result<Vec<Stmt>, String> {
    let mut visited = HashSet::new();
    let statements = parse_source(source)?;
    resolve(statements, base_dir, &mut visited)
}

fn load_inner(path: &Path, visited: &mut HashSet<PathBuf>) -> Result<Vec<Stmt>, String> {
    // Canonicalize so the same file reached by two different paths is still
    // recognised as already imported.
    let canonical = path.canonicalize().map_err(|e| {
        format!(
            "கோப்பு '{}' திறக்க முடியவில்லை  (cannot open '{}'): {}",
            path.display(),
            path.display(),
            e
        )
    })?;

    if !visited.insert(canonical.clone()) {
        return Ok(Vec::new()); // already imported
    }

    let source = std::fs::read_to_string(&canonical).map_err(|e| {
        format!(
            "கோப்பு '{}' படிக்க முடியவில்லை  (cannot read '{}'): {}",
            canonical.display(),
            canonical.display(),
            e
        )
    })?;

    let statements = parse_source(&source)?;
    let base_dir = canonical
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    resolve(statements, &base_dir, visited)
}

fn resolve(
    statements: Vec<Stmt>,
    base_dir: &Path,
    visited: &mut HashSet<PathBuf>,
) -> Result<Vec<Stmt>, String> {
    let mut out = Vec::new();
    for statement in statements {
        match statement {
            Stmt::Import(relative) => {
                let imported = load_inner(&base_dir.join(&relative), visited)?;
                out.extend(imported);
            }
            other => out.push(other),
        }
    }
    Ok(out)
}
