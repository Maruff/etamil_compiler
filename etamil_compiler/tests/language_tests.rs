//! End-to-end tests for the eTamil front end: source -> tokens -> AST ->
//! bytecode -> VM. Each test asserts on the values left in the VM rather than
//! on the exit status, because every bug these cover exited 0 while producing
//! the wrong answer.

use etamil_compiler::lexer;
use rust_decimal::Decimal;
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

/// Shorthand for a whole-number Decimal in assertions.
fn dec(n: i64) -> Decimal {
    Decimal::from(n)
}

fn num(vm: &VM, name: &str) -> Decimal {
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
    let vm = run("x = 3; (x == 5) eZil { y = 1; } iZREl { y = 2; }").unwrap();
    assert_eq!(num(&vm, "y"), dec(2));
}

#[test]
fn equality_true_branch_is_taken_when_comparing_against_zero() {
    let vm = run("x = 0; (x == 0) eZil { y = 1; } iZREl { y = 2; }").unwrap();
    assert_eq!(num(&vm, "y"), dec(1));
}

#[test]
fn inequality_and_ordering_operators() {
    let vm = run("a = 5; (a != 4) eZil { p = 1; } (a >= 5) eZil { q = 1; } (a <= 4) eZil { r = 1; } iZREl { r = 0; }").unwrap();
    assert_eq!(num(&vm, "p"), dec(1));
    assert_eq!(num(&vm, "q"), dec(1));
    assert_eq!(num(&vm, "r"), dec(0));
}

// --- Unary minus ----------------------------------------------------------

#[test]
fn negative_literals_parse() {
    let vm = run("x = -5;").unwrap();
    assert_eq!(num(&vm, "x"), dec(-5));
}

#[test]
fn subtracting_a_negative() {
    let vm = run("x = 10 - -3;").unwrap();
    assert_eq!(num(&vm, "x"), dec(13));
}

// --- Logical operators ----------------------------------------------------

#[test]
fn logical_and_or_not() {
    let vm = run("a = 1 _and 0; b = 1 _or 0; c = _not 0;").unwrap();
    assert_eq!(num(&vm, "a"), dec(0));
    assert_eq!(num(&vm, "b"), dec(1));
    assert_eq!(num(&vm, "c"), dec(1));
}

#[test]
fn logical_operators_bind_looser_than_comparison() {
    // Parsed as (x > 1) && (x < 10), not x > (1 && x) < 10.
    let vm = run("x = 5; (x > 1 _and x < 10) eZil { hit = 1; } iZREl { hit = 0; }").unwrap();
    assert_eq!(num(&vm, "hit"), dec(1));
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
    assert_eq!(num(&vm, "amount"), dec(30000));
}

// --- Decimal exactness -----------------------------------------------------
// The whole point of using a fixed-point type: these are the calculations
// f64 gets wrong, and they are the calculations a tax program performs.

#[test]
fn addition_is_exact() {
    // f64 gives 0.30000000000000004 here.
    let vm = run("x = 0.1 + 0.2;").unwrap();
    assert_eq!(num(&vm, "x"), Decimal::from_str_exact("0.3").unwrap());
    assert_eq!(text(&vm, "x"), "0.3");
}

#[test]
fn currency_multiplication_is_exact() {
    // f64 gives 299.96999999999997.
    let vm = run("total = 99.99 * 3;").unwrap();
    assert_eq!(num(&vm, "total"), Decimal::from_str_exact("299.97").unwrap());
    assert_eq!(text(&vm, "total"), "299.97");
}

#[test]
fn repeated_addition_does_not_drift() {
    // Ten 0.1 increments land exactly on 1, which f64 does not.
    let vm = run("t = 0; i = 0; (i < 10) cuRRu { t = t + 0.1; i = i + 1; }").unwrap();
    assert_eq!(num(&vm, "t"), dec(1));
}

#[test]
fn percentages_are_exact_fractions() {
    let vm = run("r = 18%;").unwrap();
    assert_eq!(num(&vm, "r"), Decimal::from_str_exact("0.18").unwrap());
}

#[test]
fn equality_is_exact_not_approximate() {
    // The old f64 Value compared equal within 1e-10, so two amounts a
    // fraction of a paisa apart were indistinguishable.
    let vm = run("a = 0.0000000001; (a == 0) eZil { same = 1; } iZREl { same = 0; }").unwrap();
    assert_eq!(num(&vm, "same"), dec(0));
}

#[test]
fn division_by_zero_is_an_error() {
    let err = run("x = 1 / 0;").expect_err("division by zero should fail");
    assert!(err.contains("division by zero"), "unexpected error: {}", err);
}

#[test]
fn while_loop_accumulates() {
    let vm = run("i = 0; s = 0; (i < 3) cuRRu { s = s + i; i = i + 1; }").unwrap();
    assert_eq!(num(&vm, "s"), dec(3));
    assert_eq!(num(&vm, "i"), dec(3));
}

#[test]
fn declaration_without_initializer_defaults_to_zero() {
    let vm = run("eN counter;").unwrap();
    assert_eq!(num(&vm, "counter"), dec(0));
}

#[test]
fn keyword_backed_names_are_stored_under_the_token_name() {
    // வருவாய் / varuvAy is the Revenue keyword, not a plain identifier, so
    // the variable lands under "Revenue". That is exactly what makes the
    // Tamil and romanized spellings name the same variable.
    let vm = run("eN varuvAy = 5;").unwrap();
    assert_eq!(num(&vm, "Revenue"), dec(5));
    assert!(vm.variables.get("varuvAy").is_none());
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

    assert_eq!(num(&vm, "total"), dec(2));
    let _ = std::fs::remove_file(path);
}

// --- Functions ------------------------------------------------------------

#[test]
fn function_returns_a_value() {
    let vm = run("ceyal iru_matangu(x) { qirumpu x * 2; } y = iru_matangu(21);").unwrap();
    assert_eq!(num(&vm, "y"), dec(42));
}

#[test]
fn function_with_several_parameters_binds_them_in_order() {
    let vm = run("ceyal kazi(a, b) { qirumpu a - b; } y = kazi(10, 3);").unwrap();
    assert_eq!(num(&vm, "y"), dec(7));
}

#[test]
fn function_body_may_branch() {
    let src = "ceyal vari(varumAZam) { \
                 (varumAZam > 800000) eZil { qirumpu (varumAZam - 800000) * 20%; } \
                 qirumpu 0; \
               } \
               a = vari(950000); b = vari(500000);";
    let vm = run(src).unwrap();
    assert_eq!(num(&vm, "a"), dec(30000));
    assert_eq!(num(&vm, "b"), dec(0));
}

#[test]
fn functions_may_recurse() {
    let src = "ceyal paktoriyal(n) { \
                 (n <= 1) eZil { qirumpu 1; } \
                 qirumpu n * paktoriyal(n - 1); \
               } \
               y = paktoriyal(5);";
    let vm = run(src).unwrap();
    assert_eq!(num(&vm, "y"), dec(120));
}

#[test]
fn functions_may_be_called_before_they_are_defined() {
    let src = "y = irattai(4); ceyal irattai(x) { qirumpu x + x; }";
    let vm = run(src).unwrap();
    assert_eq!(num(&vm, "y"), dec(8));
}

#[test]
fn falling_off_the_end_returns_nil() {
    let vm = run("ceyal onRumillY() { x = 1; } y = onRumillY();").unwrap();
    assert_eq!(vm.variables.get("y"), Some(&Value::Null));
}

#[test]
fn a_function_reads_globals_but_does_not_clobber_them() {
    let src = "moqqam = 100; \
               ceyal maRRu() { moqqam = 5; qirumpu moqqam; } \
               uLLE = maRRu();";
    let vm = run(src).unwrap();
    assert_eq!(num(&vm, "uLLE"), dec(5));
    // The global is untouched: the assignment created a local.
    assert_eq!(num(&vm, "moqqam"), dec(100));
}

#[test]
fn parameters_do_not_leak_into_the_caller() {
    let err = run("ceyal f(x) { qirumpu x; } y = f(1); accu x;")
        .expect_err("x is local to f");
    assert!(err.contains("undefined variable"), "unexpected error: {}", err);
}

#[test]
fn wrong_argument_count_is_an_error() {
    let err = run("ceyal f(a, b) { qirumpu a; } y = f(1);")
        .expect_err("arity mismatch should fail");
    assert!(err.contains("expects 2"), "unexpected error: {}", err);
}

#[test]
fn calling_an_unknown_function_is_an_error() {
    let err = run("y = illAqa_ceyal(1);").expect_err("unknown function should fail");
    assert!(err.contains("unknown function"), "unexpected error: {}", err);
}

#[test]
fn runaway_recursion_is_caught() {
    let err = run("ceyal muti_illA(n) { qirumpu muti_illA(n + 1); } y = muti_illA(1);")
        .expect_err("infinite recursion should be caught");
    assert!(err.contains("call depth"), "unexpected error: {}", err);
}

#[test]
fn a_call_may_stand_alone_as_a_statement() {
    let vm = run("ceyal kAttu(x) { accu x; } kAttu(7); y = 1;").unwrap();
    assert_eq!(num(&vm, "y"), dec(1));
    // The discarded return value must not be left on the stack.
    assert!(vm.stack.is_empty(), "stack leaked: {:?}", vm.stack);
}

#[test]
fn tamil_function_definition_and_call() {
    let src = "செயல் இரட்டை(எண்ணிக்கை) { திரும்பு எண்ணிக்கை * 2; } \
               விடை = இரட்டை(50);";
    let vm = run(src).unwrap();
    assert_eq!(num(&vm, "விடை"), dec(100));
}

// --- Arrays (அணி — a column) ----------------------------------------------

#[test]
fn array_literal_and_indexing() {
    let vm = run("a = [10, 20, 30]; x = a[0]; y = a[2];").unwrap();
    assert_eq!(num(&vm, "x"), dec(10));
    assert_eq!(num(&vm, "y"), dec(30));
}

#[test]
fn array_index_out_of_bounds_is_an_error() {
    let err = run("a = [1, 2]; x = a[5];").expect_err("index 5 is out of bounds");
    assert!(err.contains("out of bounds"), "unexpected error: {}", err);
}

#[test]
fn array_index_must_be_a_whole_number() {
    let err = run("a = [1, 2]; x = a[0.5];").expect_err("fractional index");
    assert!(err.contains("whole number"), "unexpected error: {}", err);
}

#[test]
fn array_elements_can_be_assigned() {
    let vm = run("a = [1, 2, 3]; a[1] = 99; x = a[1];").unwrap();
    assert_eq!(num(&vm, "x"), dec(99));
}

#[test]
fn empty_array_literal() {
    let vm = run("a = []; n = nILam(a);").unwrap();
    assert_eq!(num(&vm, "n"), dec(0));
}

// --- Records (பொருள் — a row) ---------------------------------------------

#[test]
fn record_literal_and_field_access() {
    let vm = run(r#"r = {peyar: "Ravi", vayaqu: 20}; p = r.peyar; v = r.vayaqu;"#).unwrap();
    assert_eq!(text(&vm, "p"), "Ravi");
    assert_eq!(num(&vm, "v"), dec(20));
}

#[test]
fn record_field_can_be_read_by_key() {
    let vm = run(r#"r = {peyar: "Ravi"}; p = r["peyar"];"#).unwrap();
    assert_eq!(text(&vm, "p"), "Ravi");
}

#[test]
fn missing_record_field_is_an_error() {
    let err = run("r = {a: 1}; x = r.b;").expect_err("field b does not exist");
    assert!(err.contains("no field"), "unexpected error: {}", err);
}

#[test]
fn record_fields_can_be_assigned_and_added() {
    let vm = run("r = {a: 1}; r.a = 5; r.b = 7; x = r.a; y = r.b;").unwrap();
    assert_eq!(num(&vm, "x"), dec(5));
    assert_eq!(num(&vm, "y"), dec(7));
}

#[test]
fn field_access_on_a_non_record_is_an_error() {
    let err = run("x = 5; y = x.peyar;").expect_err("numbers have no fields");
    assert!(err.contains("needs a record"), "unexpected error: {}", err);
}

// --- An array of records is a table --------------------------------------

#[test]
fn array_of_records_reads_like_a_result_set() {
    let src = r#"rows = [{peyar: "Ravi", vari: 1000}, {peyar: "Priya", vari: 2000}];
                 moqqam = 0;
                 i = 0;
                 (i < nILam(rows)) cuRRu {
                     oru_paqivu = rows[i];
                     moqqam = moqqam + oru_paqivu.vari;
                     i = i + 1;
                 }"#;
    let vm = run(src).unwrap();
    assert_eq!(num(&vm, "moqqam"), dec(3000));
}

#[test]
fn nested_arrays_index_in_sequence() {
    let vm = run("m = [[1, 2], [3, 4]]; x = m[1][0];").unwrap();
    assert_eq!(num(&vm, "x"), dec(3));
}

#[test]
fn a_function_can_return_a_record() {
    let src = r#"ceyal varicY_aRikkY(varumAZam) {
                     qirumpu {varumAZam: varumAZam, vari: varumAZam * 20%};
                 }
                 r = varicY_aRikkY(100000);
                 v = r.vari;"#;
    let vm = run(src).unwrap();
    assert_eq!(num(&vm, "v"), dec(20000));
}

// --- Builtins -------------------------------------------------------------

#[test]
fn length_works_on_arrays_records_and_strings() {
    let vm = run(r#"a = nILam([1,2,3]); b = nILam({x: 1, y: 2}); c = nILam("abcd");"#).unwrap();
    assert_eq!(num(&vm, "a"), dec(3));
    assert_eq!(num(&vm, "b"), dec(2));
    assert_eq!(num(&vm, "c"), dec(4));
}

#[test]
fn builtins_answer_to_tamil_and_romanized_names() {
    let vm = run("a = நீளம்([1,2,3]); b = nILam([1,2,3]);").unwrap();
    assert_eq!(num(&vm, "a"), num(&vm, "b"));
}

#[test]
fn append_extends_an_array() {
    let vm = run("a = iNY([1, 2], 3); n = nILam(a); last = a[2];").unwrap();
    assert_eq!(num(&vm, "n"), dec(3));
    assert_eq!(num(&vm, "last"), dec(3));
}

// Reserved words are a real constraint: the SQL clause keywords take several
// natural Tamil nouns, so they cannot be used as variable names.
#[test]
fn sql_clause_keywords_are_reserved() {
    let err = std::panic::catch_unwind(|| run("varicY = 1;"))
        .err()
        .map(|_| "panicked".to_string())
        .unwrap_or_else(|| "parsed".to_string());
    assert_eq!(err, "panicked", "varicY (OrderBy) should still be reserved");
}

#[test]
fn typeof_distinguishes_arrays_from_records() {
    let vm = run(r#"a = vakY([1]); r = vakY({x: 1});"#).unwrap();
    assert_eq!(text(&vm, "a"), "an array");
    assert_eq!(text(&vm, "r"), "a record");
}

#[test]
fn a_user_function_shadows_a_builtin() {
    let vm = run("ceyal nILam(x) { qirumpu 99; } y = nILam([1,2,3]);").unwrap();
    assert_eq!(num(&vm, "y"), dec(99));
}

#[test]
fn collections_print_readably() {
    let vm = run(r#"a = [1, 2]; r = {b: 2, a: 1};"#).unwrap();
    assert_eq!(vm.variables.get("a").unwrap().to_string(), "[1, 2]");
    // Keys are sorted so output is deterministic.
    assert_eq!(vm.variables.get("r").unwrap().to_string(), "{a: 1, b: 2}");
}

// --- Iteration (ஒவ்வொரு … இல்) --------------------------------------------

#[test]
fn foreach_sums_an_array() {
    let vm = run("moqqam = 0; ovvoru x il [1, 2, 3, 4] { moqqam = moqqam + x; }").unwrap();
    assert_eq!(num(&vm, "moqqam"), dec(10));
}

#[test]
fn foreach_over_an_empty_array_runs_zero_times() {
    let vm = run("n = 0; ovvoru x il [] { n = n + 1; }").unwrap();
    assert_eq!(num(&vm, "n"), dec(0));
}

#[test]
fn foreach_over_records_yields_field_names() {
    let vm = run(r#"keys = ""; ovvoru k il {b: 2, a: 1} { keys = keys & k; }"#).unwrap();
    // Keys are sorted, so the order is stable.
    assert_eq!(text(&vm, "keys"), "ab");
}

#[test]
fn foreach_over_a_table_of_records() {
    let src = r#"rows = [{peyar: "Ravi", vari: 1000}, {peyar: "Priya", vari: 2000}];
                 moqqam = 0;
                 ovvoru paqivu il rows { moqqam = moqqam + paqivu.vari; }"#;
    let vm = run(src).unwrap();
    assert_eq!(num(&vm, "moqqam"), dec(3000));
}

#[test]
fn foreach_loops_may_nest() {
    let src = "moqqam = 0;
               ovvoru a il [1, 2] { ovvoru b il [10, 20] { moqqam = moqqam + a * b; } }";
    let vm = run(src).unwrap();
    // (1*10 + 1*20) + (2*10 + 2*20) = 30 + 60
    assert_eq!(num(&vm, "moqqam"), dec(90));
}

#[test]
fn foreach_works_inside_a_function() {
    let src = "ceyal moqqam_kaNakku(pattiyal) { \
                 m = 0; \
                 ovvoru x il pattiyal { m = m + x; } \
                 qirumpu m; \
               } \
               y = moqqam_kaNakku([5, 10, 15]);";
    let vm = run(src).unwrap();
    assert_eq!(num(&vm, "y"), dec(30));
}

#[test]
fn foreach_over_a_string_yields_characters() {
    let vm = run(r#"n = 0; ovvoru c il "abc" { n = n + 1; }"#).unwrap();
    assert_eq!(num(&vm, "n"), dec(3));
}

#[test]
fn foreach_over_a_number_is_an_error() {
    let err = run("ovvoru x il 5 { }").expect_err("numbers are not iterable");
    assert!(err.contains("cannot iterate"), "unexpected error: {}", err);
}

#[test]
fn foreach_in_tamil_script() {
    // தொகை is the Amount keyword — financial vocabulary is usable as a name.
    let src = "மொத்தம் = 0; ஒவ்வொரு தொகை இல் [100, 200] { மொத்தம் = மொத்தம் + தொகை; }";
    let vm = run(src).unwrap();
    assert_eq!(num(&vm, "மொத்தம்"), dec(300));
}

// Type keywords are reserved, the way `int` is in C. Financial keywords are
// not — that distinction is what makes `தொகை` above legal.
#[test]
fn type_keywords_are_reserved_but_financial_ones_are_not() {
    let reserved = std::panic::catch_unwind(|| run("eN = 1;")).is_err();
    assert!(reserved, "eN (IntegerType) should be reserved");

    let vm = run("toqai = 5;").unwrap();
    assert_eq!(num(&vm, "Amount"), dec(5));
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
