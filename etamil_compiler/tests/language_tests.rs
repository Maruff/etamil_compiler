//! End-to-end tests for the eTamil front end: source -> tokens -> AST ->
//! bytecode -> VM. Each test asserts on the values left in the VM rather than
//! on the exit status, because every bug these cover exited 0 while producing
//! the wrong answer.

use etamil_compiler::lexer;
use etamil_compiler::parser::Parser;
use etamil_compiler::vm::{BytecodeCompiler, Value, VM};

/// Run a program to completion and hand back the finished VM.
fn run(source: &str) -> Result<VM, String> {
    let tokens = lexer::tokenize(source).map_err(|errors| {
        errors
            .iter()
            .map(|e| e.to_string())
            .collect::<Vec<_>>()
            .join("; ")
    })?;
    let mut parser = Parser::new(tokens.iter());
    let ast = parser.parse();
    let bytecode = BytecodeCompiler::compile_statements(ast);
    let mut vm = VM::new();
    vm.execute(bytecode)?;
    Ok(vm)
}

fn num(vm: &VM, name: &str) -> f64 {
    vm.variables
        .get(name)
        .unwrap_or_else(|| panic!("variable '{}' was never assigned", name))
        .to_number()
}

fn text(vm: &VM, name: &str) -> String {
    vm.variables
        .get(name)
        .unwrap_or_else(|| panic!("variable '{}' was never assigned", name))
        .to_string()
}

// --- Equality -------------------------------------------------------------
// Regression: the parser emitted "==" while the bytecode compiler matched "=",
// so no instruction was emitted and JumpIfFalse tested the truthiness of the
// right-hand operand instead. Both cases below silently took the wrong branch.

#[test]
fn equality_false_branch_is_taken_when_operands_differ() {
    let vm = run("x = 3; (x == 5) enil { y = 1; } inREl { y = 2; }").unwrap();
    assert_eq!(num(&vm, "y"), 2.0);
}

#[test]
fn equality_true_branch_is_taken_when_comparing_against_zero() {
    let vm = run("x = 0; (x == 0) enil { y = 1; } inREl { y = 2; }").unwrap();
    assert_eq!(num(&vm, "y"), 1.0);
}

#[test]
fn inequality_and_ordering_operators() {
    let vm = run("a = 5; (a != 4) enil { p = 1; } (a >= 5) enil { q = 1; } (a <= 4) enil { r = 1; } inREl { r = 0; }").unwrap();
    assert_eq!(num(&vm, "p"), 1.0);
    assert_eq!(num(&vm, "q"), 1.0);
    assert_eq!(num(&vm, "r"), 0.0);
}

// --- Unary minus ----------------------------------------------------------

#[test]
fn negative_literals_parse() {
    let vm = run("x = -5;").unwrap();
    assert_eq!(num(&vm, "x"), -5.0);
}

#[test]
fn subtracting_a_negative() {
    let vm = run("x = 10 - -3;").unwrap();
    assert_eq!(num(&vm, "x"), 13.0);
}

// --- Logical operators ----------------------------------------------------

#[test]
fn logical_and_or_not() {
    let vm = run("a = 1 _and 0; b = 1 _or 0; c = _not 0;").unwrap();
    assert_eq!(num(&vm, "a"), 0.0);
    assert_eq!(num(&vm, "b"), 1.0);
    assert_eq!(num(&vm, "c"), 1.0);
}

#[test]
fn logical_operators_bind_looser_than_comparison() {
    // Parsed as (x > 1) && (x < 10), not x > (1 && x) < 10.
    let vm = run("x = 5; (x > 1 _and x < 10) enil { hit = 1; } inREl { hit = 0; }").unwrap();
    assert_eq!(num(&vm, "hit"), 1.0);
}

#[test]
fn boolean_literals_are_values_not_variables() {
    let vm = run("t = mey; f = poy;").unwrap();
    assert_eq!(vm.variables.get("t"), Some(&Value::Boolean(true)));
    assert_eq!(vm.variables.get("f"), Some(&Value::Boolean(false)));
}

// --- Arithmetic and control flow -----------------------------------------

#[test]
fn percentage_literals_convert_to_fractions() {
    let vm = run("rate = 20%; amount = 150000 * rate;").unwrap();
    assert!((num(&vm, "amount") - 30000.0).abs() < 1e-6);
}

#[test]
fn while_loop_accumulates() {
    let vm = run("i = 0; s = 0; (i < 3) cuRRu { s = s + i; i = i + 1; }").unwrap();
    assert_eq!(num(&vm, "s"), 3.0);
    assert_eq!(num(&vm, "i"), 3.0);
}

#[test]
fn declaration_without_initializer_defaults_to_zero() {
    let vm = run("eN varuvAy;").unwrap();
    assert_eq!(num(&vm, "varuvAy"), 0.0);
}

#[test]
fn string_concatenation() {
    let vm = run(r#"name = "Ravi"; greeting = "vaNakkam " & name;"#).unwrap();
    assert_eq!(text(&vm, "greeting"), "vaNakkam Ravi");
}

// --- Diagnostics ----------------------------------------------------------
// Regression: tokenize() used filter_map(|t| t.ok()), so unrecognized input
// vanished and the program ran with a different meaning.

#[test]
fn unrecognized_input_is_reported_with_a_position() {
    let errors = lexer::tokenize("x = 5 @ 3;").expect_err("'@' should not lex");
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].line, 1);
    assert_eq!(errors[0].column, 7);
    assert_eq!(errors[0].text, "@");
}

#[test]
fn valid_source_lexes_cleanly() {
    assert!(lexer::tokenize("accu \"vaNakkam\";").is_ok());
}

// Regression: an unknown name loaded Null, which to_number() turned into 0.0,
// so a typo became a wrong number with no diagnostic.
#[test]
fn undefined_variables_are_an_error() {
    let err = run("accu missing_var;").expect_err("undefined variable should fail");
    assert!(err.contains("undefined variable"), "unexpected error: {}", err);
}

// Regression: database and server statements compiled to nothing and the
// program exited 0 having done none of what it said.
#[test]
fn database_statements_fail_loudly() {
    let err = run(r#"qaLam_iNY cIkulYt, "students.db";"#)
        .expect_err("database statements are not implemented in the VM");
    assert!(
        err.contains("not implemented"),
        "unexpected error: {}",
        err
    );
}

// --- File I/O -------------------------------------------------------------
// Regression: FileRead was compiled but never executed, so StoreVar popped the
// *filename* and the variable silently held "data.txt" instead of the contents.

#[test]
fn file_write_then_read_round_trips() {
    let path = "etamil_test_roundtrip.txt";
    let _ = std::fs::remove_file(path);

    let source = format!(
        r#"kOppu_qiRa "{path}", "write";
           kOppu_ezuqu "{path}", "vaNakkam";
           kOppu_mUtu "{path}";
           kOppu_pati "{path}", contents;"#
    );
    let vm = run(&source).unwrap();

    assert_eq!(text(&vm, "contents"), "vaNakkam");
    let _ = std::fs::remove_file(path);
}

#[test]
fn opening_for_write_truncates_then_writes_append() {
    let path = "etamil_test_truncate.txt";
    std::fs::write(path, "stale\n").unwrap();

    let source = format!(
        r#"kOppu_qiRa "{path}", "write";
           kOppu_ezuqu "{path}", "one";
           kOppu_ezuqu "{path}", "two";
           kOppu_pati "{path}", contents;"#
    );
    let vm = run(&source).unwrap();

    assert_eq!(text(&vm, "contents"), "one\ntwo");
    let _ = std::fs::remove_file(path);
}

#[test]
fn reading_a_missing_file_is_an_error() {
    let err = run(r#"kOppu_pati "etamil_test_does_not_exist.txt", x;"#)
        .expect_err("reading a missing file should fail");
    assert!(err.contains("cannot read"), "unexpected error: {}", err);
}

#[test]
fn csv_row_count_excludes_the_header() {
    let path = "etamil_test_rows.csv";
    std::fs::write(path, "id,name\n1,Ravi\n2,Priya\n").unwrap();

    let vm = run(&format!(r#"qaravurY_pati "{path}", total;"#)).unwrap();

    assert_eq!(num(&vm, "total"), 2.0);
    let _ = std::fs::remove_file(path);
}

// --- Bilingual equivalence ------------------------------------------------

#[test]
fn tamil_and_romanized_forms_are_equivalent() {
    let tamil = run("எண் வருவாய் = 950000; வரி = 20%;").unwrap();
    let roman = run("eN varuvAy = 950000; vari = 20%;").unwrap();

    // வருவாய் and varuvAy are the same keyword, so both land under the same name.
    assert_eq!(num(&tamil, "Revenue"), num(&roman, "Revenue"));
    assert_eq!(num(&tamil, "Tax"), num(&roman, "Tax"));
}
