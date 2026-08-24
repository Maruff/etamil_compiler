//! Tests for `vm::host`'s capture mode — the arrangement that lets something
//! with no console embed the VM.
//!
//! It exists for the Android app (see android/rust/src/lib.rs), which cannot be
//! built on a machine without an NDK. These tests can: capture is ordinary
//! native code, so every behaviour the app depends on is checked here, on
//! whatever platform happens to be running the suite.
//!
//! The `வெளியேறு` tests earn their place more than the others. If capture ever
//! stops intercepting `exit`, the native implementation calls
//! `std::process::exit` — and in an app that ends the app. Here it would end the
//! test runner mid-suite, which is at least a failure nobody can overlook.

use etamil_compiler::lexer;
use etamil_compiler::parser::Parser;
use etamil_compiler::vm::host;
use etamil_compiler::vm::{BytecodeCompiler, VM};

/// Run a program under capture and hand back what it printed, or how it failed.
///
/// Capture is begun and ended inside one call on one thread, which is the
/// discipline the JNI bridge follows too — `end_capture` is reached on the
/// failure path as well, so a test that fails cannot leave capture on and
/// silence the tests that run after it on the same thread.
fn run_captured(source: &str, input: &[&str]) -> (Result<(), String>, String) {
    host::begin_capture();
    for line in input {
        host::push_input(line);
    }

    let outcome = compile_and_run(source);
    let output = host::end_capture();
    (outcome, output)
}

fn compile_and_run(source: &str) -> Result<(), String> {
    let tokens = lexer::tokenize(source).map_err(|errors| {
        errors
            .iter()
            .map(|e| e.to_string())
            .collect::<Vec<_>>()
            .join("; ")
    })?;
    let ast = Parser::new(tokens.iter())
        .parse()
        .map_err(|error| error.to_string())?;
    let bytecode = BytecodeCompiler::compile_statements(ast);
    VM::new().execute(bytecode)
}

#[test]
fn output_is_collected_rather_than_printed() {
    let (outcome, output) = run_captured("அச்சு \"வணக்கம்\";", &[]);

    assert!(outcome.is_ok(), "the program should run: {:?}", outcome);
    assert_eq!(output, "வணக்கம்\n");
}

#[test]
fn every_printed_line_is_kept_in_order() {
    let (outcome, output) = run_captured("அச்சு 1; அச்சு 2; அச்சு 3;", &[]);

    assert!(outcome.is_ok(), "{:?}", outcome);
    assert_eq!(output, "1\n2\n3\n");
}

#[test]
fn queued_input_answers_ullidu() {
    // The bundled example's shape: read a number, branch on it.
    let source = "எண் வருவாய்; உள்ளிடு வருவாய்; அச்சு வருவாய்;";
    let (outcome, output) = run_captured(source, &["950000"]);

    assert!(outcome.is_ok(), "{:?}", outcome);
    assert_eq!(output.trim(), "950000");
}

#[test]
fn input_is_read_in_the_order_it_was_queued() {
    let source = "எண் அ; உள்ளிடு அ; எண் ஆ; உள்ளிடு ஆ; அச்சு அ; அச்சு ஆ;";
    let (outcome, output) = run_captured(source, &["1", "2"]);

    assert!(outcome.is_ok(), "{:?}", outcome);
    assert_eq!(output, "1\n2\n");
}

/// The property that keeps an app from hanging.
///
/// Natively, `read_line` with nothing queued falls through to stdin, which in an
/// app is a handle that never produces a byte — indistinguishable, from the
/// outside, from a crash. Under capture it has to fail instead.
#[test]
fn exhausted_input_fails_instead_of_waiting() {
    let source = "எண் அ; உள்ளிடு அ;";
    let (outcome, _) = run_captured(source, &[]);

    let error = outcome.expect_err("reading with no input queued should fail");
    assert!(
        error.contains("தீர்ந்தது") || error.contains("no input left"),
        "the error should say the input ran out, got: {}",
        error
    );
}

/// A zero exit carries on, because there is no process to end.
#[test]
fn exit_zero_does_not_end_the_process() {
    let (outcome, output) = run_captured("அச்சு \"முன்\"; வெளியேறு(0);", &[]);

    assert!(outcome.is_ok(), "{:?}", outcome);
    assert!(
        output.contains("முன்"),
        "output before the exit should survive it, got: {:?}",
        output
    );
}

/// A non-zero exit is the program's failure, reported as one.
///
/// If this test ever kills the runner instead of failing, capture has stopped
/// intercepting `exit` — which is the bug that would take the app down with it.
#[test]
fn nonzero_exit_is_reported_not_performed() {
    let (outcome, output) = run_captured("அச்சு \"முன்\"; வெளியேறு(3);", &[]);

    let error = outcome.expect_err("a non-zero exit should be an error");
    assert!(
        error.contains('3'),
        "the error should name the status, got: {}",
        error
    );
    // Whatever it printed first is still worth having: it is usually the
    // explanation of why it exited.
    assert!(output.contains("முன்"), "got: {:?}", output);
}

/// Output produced before a failure is not thrown away.
#[test]
fn output_survives_a_runtime_error() {
    // Division by zero after something has already been printed.
    let (outcome, output) = run_captured("அச்சு \"தொடக்கம்\"; அச்சு 1/0;", &[]);

    assert!(outcome.is_err(), "dividing by zero should fail");
    assert!(
        output.contains("தொடக்கம்"),
        "the line printed before the error should survive, got: {:?}",
        output
    );
}

/// `end_capture` has to leave the thread as it found it.
///
/// A second run must not see the first one's output, and — the part that would
/// be invisible until something broke — must not inherit input the first run
/// never read.
#[test]
fn capture_does_not_leak_between_runs() {
    let (_, first) = run_captured("அச்சு \"ஒன்று\";", &["unread", "also unread"]);
    assert_eq!(first, "ஒன்று\n");

    assert!(
        !host::capturing(),
        "end_capture should have turned capture back off"
    );

    // The leftover input from the first run must be gone, so this fails for
    // want of input rather than quietly reading "unread".
    let source = "எண் அ; உள்ளிடு அ;";
    let (outcome, second) = run_captured(source, &[]);
    assert!(
        outcome.is_err(),
        "the previous run's unread input should not answer this one, got: {:?}",
        second
    );
}

/// Off by default. Nothing in the compiler calls `begin_capture`, so the command
/// line, the REPL and the servers keep printing to stdout as they always did.
#[test]
fn capture_is_off_until_it_is_asked_for() {
    assert!(!host::capturing());
}
