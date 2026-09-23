// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Mohammed Maruff (Esan Maruff) <esan@etamil.in>
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
    // Parse errors carry a line and column now, so the message a caller sees
    // says where to look rather than only what was wrong.
    parser.parse().map_err(|error| error.to_string())
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

/// Where a module's imports resolve from.
///
/// A module read from disk resolves its own imports against its directory. One
/// answered from the built-in library has no directory, so it resolves against
/// its position in the embedded tree instead — which matters because the
/// library imports its neighbours relatively (`../kaNiqam.qmz`).
enum Origin {
    Disk(PathBuf),
    Embedded(String),
}

/// Parse an embedded module and resolve whatever it imports.
fn load_embedded(virtual_path: &str, visited: &mut HashSet<PathBuf>) -> Result<Vec<Stmt>, String> {
    let source = crate::stdlib::source(virtual_path).ok_or_else(|| {
        format!(
            "உள்ளமைந்த தொகுதி '{}' இல்லை  (no built-in module '{}')",
            virtual_path, virtual_path
        )
    })?;

    // Keyed apart from any real path so a file on disk and the built-in copy
    // of the same module are never mistaken for one another.
    let key = PathBuf::from(format!(
        "<built-in>/{}",
        crate::stdlib::parent(virtual_path)
    ))
    .join(virtual_path);
    if !visited.insert(key) {
        return Ok(Vec::new()); // already imported
    }

    let statements = parse_source(source)?;
    resolve_from(
        statements,
        &Origin::Embedded(crate::stdlib::parent(virtual_path)),
        visited,
    )
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

/// Find an imported file: next to the importer first, then along
/// `ETAMIL_PATH`, then in a `nUlakam` directory beside the executable. That
/// last one is what lets `இறக்கு "nUlakam/paNam.qmz";` work from anywhere
/// once the compiler is installed.
fn locate(relative: &str, base_dir: &Path) -> Option<PathBuf> {
    let beside = base_dir.join(relative);
    if beside.exists() {
        return Some(beside);
    }

    if let Ok(search_path) = std::env::var("ETAMIL_PATH") {
        for entry in std::env::split_paths(&search_path) {
            let candidate = entry.join(relative);
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }

    if let Ok(exe) = std::env::current_exe()
        && let Some(dir) = exe.parent()
    {
        let candidate = dir.join(relative);
        if candidate.exists() {
            return Some(candidate);
        }
    }

    // Native packages keep the standard library in the platform data
    // directory rather than beside the executable in /usr/bin.
    for data_dir in [
        Path::new("/usr/share/etamil"),
        Path::new("/usr/local/share/etamil"),
    ] {
        let candidate = data_dir.join(relative);
        if candidate.exists() {
            return Some(candidate);
        }
    }

    None
}

fn resolve(
    statements: Vec<Stmt>,
    base_dir: &Path,
    visited: &mut HashSet<PathBuf>,
) -> Result<Vec<Stmt>, String> {
    resolve_from(statements, &Origin::Disk(base_dir.to_path_buf()), visited)
}

fn resolve_from(
    statements: Vec<Stmt>,
    origin: &Origin,
    visited: &mut HashSet<PathBuf>,
) -> Result<Vec<Stmt>, String> {
    let mut out = Vec::new();
    for statement in statements {
        match statement {
            Stmt::Import(relative) => {
                let imported = match origin {
                    // On disk, the filesystem is tried first and the built-in
                    // library last, so a checkout, ETAMIL_PATH or a distribution
                    // package always overrides the copy inside the binary. That
                    // ordering is what lets the library be edited without
                    // rebuilding the compiler.
                    Origin::Disk(base_dir) => match locate(&relative, base_dir) {
                        Some(found) => load_inner(&found, visited)?,
                        None if crate::stdlib::contains(&relative) => {
                            load_embedded(&relative, visited)?
                        }
                        None => return Err(not_found(&relative)),
                    },
                    // Inside the built-in library, imports stay inside it. A
                    // module answered from the binary must not start reading
                    // the invoking user's working directory.
                    Origin::Embedded(virtual_dir) => {
                        let target = crate::stdlib::join(virtual_dir, &relative);
                        if crate::stdlib::contains(&target) {
                            load_embedded(&target, visited)?
                        } else {
                            return Err(not_found(&relative));
                        }
                    }
                };
                out.extend(imported);
            }
            other => out.push(other),
        }
    }
    Ok(out)
}

fn not_found(relative: &str) -> String {
    format!(
        "தொகுதி '{}' கண்டுபிடிக்க முடியவில்லை  (cannot open module '{}'): \
         looked beside the importing file, along ETAMIL_PATH, next to the compiler, \
         and in the built-in standard library",
        relative, relative
    )
}
