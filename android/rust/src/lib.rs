//! Android bindings for the eTamil compiler.
//!
//! The app runs the real compiler rather than a reimplementation of it, so a
//! diagnostic on a phone is the same diagnostic `etamil` prints on the command
//! line — bilingual message text included, straight from each error type's
//! `Display`. This is the same bargain `src/wasm.rs` strikes for the browser.
//!
//! Unlike the browser, a phone gets the whole language. Android is a native
//! build, so `vm`, `db`, `module` and the HTTP client are all present: a
//! program here can import from `nUlakam`, open a SQLite table and call out to
//! an API. What it cannot do is write to a console, and `வெளியேறு` must not be
//! allowed to end the process — that process is the app. Both are handled by
//! `vm::host`'s capture mode, which this module turns on around every run.
//!
//! # Why JSON
//!
//! Every entry point returns a JSON string. Building Java objects across JNI
//! means finding a class, finding each field and setting each one — a dozen
//! fallible calls per result — where `JSONObject(String)` on the Kotlin side is
//! one line. The payloads are a few hundred bytes.
//!
//! # Threading
//!
//! Capture is thread-local, so a run must begin and end on one thread. Each
//! call below does, and none of them hands out a handle that outlives it.
//! Calling from two threads at once is safe and gives each its own buffer;
//! calling from the UI thread is not *unsafe*, merely unwise, and the Kotlin
//! side keeps it off there.

use jni::JNIEnv;
use jni::objects::{JClass, JString};
use jni::sys::jstring;
use serde::Serialize;
use std::path::Path;

use etamil_compiler::vm::host;
use etamil_compiler::{check, lexer, module, parser, vm};

/// Instructions one run may retire before it is called an endless loop.
///
/// The same ceiling the browser uses. A phone is slower than a laptop, but a
/// program that needs more than ten million steps is not one somebody is
/// waiting on at a touchscreen — it is a loop with no exit.
const STEP_LIMIT: u64 = 10_000_000;

/// One diagnostic, positioned the way the compiler positions errors: 1-based
/// line and column, both counting characters rather than bytes, so Tamil text
/// reports the column a reader would count to.
///
/// `length` is in characters too — the editor needs a range to underline, and
/// the length of the offending text is the closest thing the error types carry
/// to an end position.
#[derive(Serialize)]
struct Diagnostic {
    line: usize,
    column: usize,
    length: usize,
    severity: &'static str,
    stage: &'static str,
    message: String,
}

/// What came of one run.
///
/// The same shape as the browser's, minus `files`. In a browser the filesystem
/// is a map this compiler owns, so it can list what a program wrote; on Android
/// the writes go to real files in the app's own directory, and listing them is
/// the app's job — it can read that directory without asking the compiler.
#[derive(Serialize)]
struct RunResult {
    ok: bool,
    /// Everything the program printed, including whatever it managed to print
    /// before failing.
    output: String,
    /// The failure, bilingual, from whichever stage produced it.
    error: Option<String>,
    /// Which stage failed: "load", "type" or "run".
    stage: Option<&'static str>,
}

/// Character count, not byte count — the compiler's columns are in characters.
fn char_len(text: &str) -> usize {
    text.chars().count()
}

// --- Entry points -----------------------------------------------------------

/// The compiler version this library was built from, so the app can say which
/// compiler produced a diagnostic.
#[unsafe(no_mangle)]
pub extern "system" fn Java_org_etamil_mobile_Etamil_nativeVersion<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
) -> jstring {
    into_jstring(&mut env, env!("CARGO_PKG_VERSION").to_string())
}

/// Diagnostics for one source file, as a JSON array.
///
/// The passes run in order and the first to fail wins: a file that does not lex
/// cannot be parsed, and reporting invented parse errors on top of a real
/// lexical one buries the error the author needs to see.
///
/// This reads the source alone and resolves no imports, which is what makes it
/// safe to call while somebody is still typing — see `nativeRun` for why that
/// matters.
#[unsafe(no_mangle)]
pub extern "system" fn Java_org_etamil_mobile_Etamil_nativeDiagnostics<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    source: JString<'local>,
) -> jstring {
    let source = from_jstring(&mut env, &source);
    let out = collect_diagnostics(&source);
    let json = serde_json::to_string(&out).unwrap_or_else(|_| "[]".to_string());
    into_jstring(&mut env, json)
}

/// Compile and run one source file, returning JSON.
///
/// `input` is the program's `உள்ளிடு` answers, one per line, supplied up front:
/// there is nobody at a terminal to type them while it runs, so an app collects
/// them first. Running out of them is an error rather than a wait, because a
/// wait on a phone is indistinguishable from a hang.
///
/// `baseDir` is where `இறக்கு` looks for a module and where a relative
/// `கோப்பு_எழுது` path lands. The app passes its own private directory, so a
/// program can only reach files the app could reach anyway.
#[unsafe(no_mangle)]
pub extern "system" fn Java_org_etamil_mobile_Etamil_nativeRun<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    source: JString<'local>,
    input: JString<'local>,
    base_dir: JString<'local>,
) -> jstring {
    let source = from_jstring(&mut env, &source);
    let input = from_jstring(&mut env, &input);
    let base_dir = from_jstring(&mut env, &base_dir);

    let result = execute(&source, &input, &base_dir);
    let json = serde_json::to_string(&result).unwrap_or_else(|_| {
        r#"{"ok":false,"output":"","error":"result could not be encoded","stage":"run"}"#
            .to_string()
    });
    into_jstring(&mut env, json)
}

// --- The pipeline -----------------------------------------------------------

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

    let statements = match parser::Parser::new(tokens.iter()).parse() {
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

fn execute(source: &str, input: &str, base_dir: &str) -> RunResult {
    // Capture has to be on before the first `அச்சு`, and the queued input
    // before the first `உள்ளிடு`.
    host::begin_capture();
    for line in input.lines() {
        host::push_input(line);
    }

    let outcome = compile_and_run(source, base_dir);

    // Unconditional, on both the success and failure paths: `end_capture` both
    // drains the buffer and turns capture back off, and leaving it on would
    // swallow the output of anything else this thread later runs. It also
    // discards unread input, so one run cannot answer the next one's `உள்ளிடு`.
    let output = host::end_capture();

    match outcome {
        Ok(()) => RunResult {
            ok: true,
            output,
            error: None,
            stage: None,
        },
        Err((stage, error)) => RunResult {
            ok: false,
            output,
            error: Some(error),
            stage: Some(stage),
        },
    }
}

/// Lex, parse, resolve imports, type-check, compile, interpret.
///
/// Errors come back as (stage, message) so the caller can label them without
/// parsing the text.
fn compile_and_run(source: &str, base_dir: &str) -> Result<(), (&'static str, String)> {
    // `module::load_source` rather than the lexer and parser directly, which is
    // where this parts company with the browser: it resolves `இறக்கு`, so a
    // program on a phone can import from `nUlakam` like one anywhere else. The
    // cost is that its three stages report as one — it returns a formatted
    // string, possibly several errors joined by newlines, rather than a
    // positioned error — so the editor calls `nativeDiagnostics` for underlines
    // and this reports only what stopped the run.
    let statements = module::load_source(source, Path::new(base_dir)).map_err(|e| ("load", e))?;

    // A program that does not type-check is not run, which is what the command
    // line does too. Running it anyway would produce a second, more confusing
    // error somewhere further along.
    //
    // Only the first is reported: a run either happens or it does not, and the
    // editor is already underlining every one of them.
    if let Err(errors) = check::check(&statements) {
        let first = errors
            .first()
            .map(|e| e.to_string())
            .unwrap_or_else(|| "type error".to_string());
        return Err(("type", first));
    }

    let bytecode = vm::compile_to_bytecode(statements);
    let mut machine = vm::VM::new();
    machine
        .execute_limited(bytecode, STEP_LIMIT)
        .map_err(|e| ("run", e))
}

// --- JNI string handling ----------------------------------------------------

/// A Java string as a Rust one.
///
/// An unreadable argument becomes the empty string rather than a panic.
/// Unwinding across the JNI boundary is undefined behaviour, so nothing in this
/// file is allowed to panic on bad input; an empty program produces an empty
/// result, which is the honest answer to an argument that could not be read.
fn from_jstring(env: &mut JNIEnv, value: &JString) -> String {
    env.get_string(value)
        .map(|java_str| java_str.into())
        .unwrap_or_default()
}

/// A Rust string as a Java one.
///
/// A null return is what a failed allocation looks like from Kotlin, and the
/// app treats it as such. There is nothing better to do: if the JVM cannot
/// allocate a string, it cannot allocate the exception either.
fn into_jstring(env: &mut JNIEnv, value: String) -> jstring {
    env.new_string(value)
        .map(|s| s.into_raw())
        .unwrap_or(std::ptr::null_mut())
}
