//! Tests for `--check`, the front-end-only mode.
//!
//! These run the real binary rather than the library, because the whole point
//! of the mode is what the *process* does: which streams it writes to, what it
//! exits with, and — the reason it exists — that it does not run the program.
//!
//! The editor depends on all three. `eTamil_Code` reports errors by piping a
//! buffer to `etamil --check`, and if that mode ever started executing, opening
//! a file in a text editor would write that file's output and issue its
//! queries.

use std::io::Write;
use std::process::{Command, Stdio};

/// Run `--check` over source piped on stdin, in `cwd`.
fn check_in(cwd: &std::path::Path, source: &str) -> (i32, String, String) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_etamil"))
        .arg("--check")
        .current_dir(cwd)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("could not start the compiler");

    child
        .stdin
        .as_mut()
        .expect("stdin")
        .write_all(source.as_bytes())
        .expect("could not write the program");

    let output = child.wait_with_output().expect("the compiler did not finish");
    (
        output.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

fn check(source: &str) -> (i32, String, String) {
    check_in(&std::env::temp_dir(), source)
}

#[test]
fn an_accepted_program_exits_zero_and_says_nothing() {
    let (code, stdout, stderr) = check("எண் வருவாய் = 100000;\nஅச்சு வருவாய்;\n");

    assert_eq!(code, 0);
    // Silence on success is what lets a caller treat any output at all as
    // failure, so it is part of the contract rather than a nicety.
    assert_eq!(stdout, "", "--check must write nothing to stdout");
    assert_eq!(stderr, "", "--check must write nothing on success");
}

#[test]
fn a_parse_error_is_reported_with_its_position() {
    let (code, stdout, stderr) = check("அச்சு \"a\" அச்சு;\n");

    assert_eq!(code, 1);
    assert_eq!(stdout, "");
    assert!(stderr.contains("வரி 1"), "no line number: {stderr}");
    assert!(stderr.contains("நெடுவரிசை"), "no column: {stderr}");
}

#[test]
fn a_lexical_error_is_reported() {
    let (code, _, stderr) = check("அச்சு 1 @;\n");

    assert_eq!(code, 1);
    assert!(stderr.contains("அறியப்படாத உள்ளீடு"), "{stderr}");
}

#[test]
fn every_type_error_is_reported_not_only_the_first() {
    let (code, _, stderr) = check("ஈர்ம கொடி = 5;\nஅணி வரிசைகள் = 1;\n");

    assert_eq!(code, 1);
    let reported = stderr.lines().filter(|line| line.starts_with('✗')).count();
    assert_eq!(reported, 2, "expected both type errors:\n{stderr}");
}

#[test]
fn a_missing_module_is_reported_rather_than_ignored() {
    let (code, _, stderr) = check("இறக்கு \"illY/kOppu.qmz\";\n");

    assert_eq!(code, 1);
    assert!(stderr.contains("illY/kOppu.qmz"), "{stderr}");
}

/// The reason the mode exists.
#[test]
fn checking_a_program_does_not_run_it() {
    let dir = std::env::temp_dir().join(format!(
        "etamil-check-{}-{}",
        std::process::id(),
        line!()
    ));
    std::fs::create_dir_all(&dir).expect("could not create the temp directory");
    let witness = dir.join("varavu.txt");
    let _ = std::fs::remove_file(&witness);

    // A program whose only effect is to create a file. Under --vm this writes
    // it; under --check nothing may happen.
    let source = "கோப்பு_திற \"varavu.txt\", \"write\";\n\
                  கோப்பு_எழுது \"varavu.txt\", \"இது நடந்தது\";\n";

    let (code, _, stderr) = check_in(&dir, source);

    assert_eq!(code, 0, "the program should be accepted:\n{stderr}");
    assert!(
        !witness.exists(),
        "--check executed the program: {} was written",
        witness.display()
    );

    let _ = std::fs::remove_dir_all(&dir);
}

/// Imports resolve relative to the working directory when reading stdin, which
/// is how the editor checks an unsaved buffer without writing a temp file into
/// the author's project.
#[test]
fn stdin_resolves_imports_from_the_working_directory() {
    let dir = std::env::temp_dir().join(format!(
        "etamil-check-{}-{}",
        std::process::id(),
        line!()
    ));
    std::fs::create_dir_all(&dir).expect("could not create the temp directory");
    std::fs::write(dir.join("tuNY.qmz"), "செயல் இரட்டை(எ) { திரும்பு எ * 2; }\n")
        .expect("could not write the module");

    let (code, _, stderr) = check_in(&dir, "இறக்கு \"tuNY.qmz\";\nஅச்சு இரட்டை(21);\n");

    assert_eq!(code, 0, "{stderr}");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_port_that_is_not_a_number_is_refused() {
    // It used to fall back to 8080 silently, so a mistyped flag bound a port
    // the author had not asked for and the server looked like it started fine.
    let output = Command::new(env!("CARGO_BIN_EXE_etamil"))
        .args(["--check", "--port", "abc"])
        .stdin(Stdio::null())
        .output()
        .expect("could not start the compiler");

    assert_eq!(output.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("--port"),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
