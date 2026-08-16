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

// Regression: server statements compiled to nothing and the program exited
// 0 having done none of what it said.
#[test]
fn server_statements_fail_loudly() {
    let err = run(r#"vazawki_toqotawku "127.0.0.1", 8080;"#)
        .expect_err("server statements are not implemented in the VM");
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

// --- Results (சரி / தவறு), following Rust ---------------------------------

#[test]
fn ok_and_err_construct_results() {
    let vm = run(r#"a = cari(5); b = qavaRu("thavaru");"#).unwrap();
    assert_eq!(vm.variables.get("a"), Some(&Value::Ok(Box::new(Value::Number(dec(5))))));
    assert!(matches!(vm.variables.get("b"), Some(Value::Err(_))));
}

#[test]
fn is_ok_and_is_err() {
    let vm = run(r#"a = cariyA(cari(1)); b = qavaRA(qavaRu("x")); c = cariyA(qavaRu("x"));"#).unwrap();
    assert_eq!(vm.variables.get("a"), Some(&Value::Boolean(true)));
    assert_eq!(vm.variables.get("b"), Some(&Value::Boolean(true)));
    assert_eq!(vm.variables.get("c"), Some(&Value::Boolean(false)));
}

#[test]
fn unwrap_returns_the_value() {
    let vm = run("x = maqippu(cari(42));").unwrap();
    assert_eq!(num(&vm, "x"), dec(42));
}

#[test]
fn unwrap_on_an_error_is_a_runtime_error() {
    let err = run(r#"x = maqippu(qavaRu("pizai"));"#).expect_err("unwrap on Err must fail");
    assert!(err.contains("unwrap on an error"), "unexpected error: {}", err);
}

#[test]
fn unwrap_or_supplies_a_default() {
    let vm = run(r#"a = iyalpu(cari(1), 99); b = iyalpu(qavaRu("x"), 99);"#).unwrap();
    assert_eq!(num(&vm, "a"), dec(1));
    assert_eq!(num(&vm, "b"), dec(99));
}

#[test]
fn a_result_is_truthy_when_it_succeeded() {
    let src = r#"r = cari(1);
                 (r) eZil { a = 1; } iZREl { a = 0; }
                 e = qavaRu("x");
                 (e) eZil { b = 1; } iZREl { b = 0; }"#;
    let vm = run(src).unwrap();
    assert_eq!(num(&vm, "a"), dec(1));
    assert_eq!(num(&vm, "b"), dec(0));
}

// --- The ? operator -------------------------------------------------------

#[test]
fn question_mark_unwraps_a_success() {
    let src = "ceyal paravAyillY() { qirumpu cari(7); } \
               ceyal azY() { x = paravAyillY()?; qirumpu cari(x * 2); } \
               y = maqippu(azY());";
    let vm = run(src).unwrap();
    assert_eq!(num(&vm, "y"), dec(14));
}

#[test]
fn question_mark_propagates_a_failure_to_the_caller() {
    let src = r#"ceyal cimY() { qirumpu qavaRu("pizai"); }
                 ceyal azY() { x = cimY()?; qirumpu cari(x * 2); }
                 r = azY();
                 failed = qavaRA(r);"#;
    let vm = run(src).unwrap();
    assert_eq!(vm.variables.get("failed"), Some(&Value::Boolean(true)));
}

#[test]
fn question_mark_propagates_through_several_frames() {
    let src = r#"ceyal mUZRu() { qirumpu qavaRu("Azam"); }
                 ceyal iraNtu() { x = mUZRu()?; qirumpu cari(x); }
                 ceyal oZRu() { x = iraNtu()?; qirumpu cari(x); }
                 r = oZRu();
                 msg = maqippu(cari(qavaRA(r)));"#;
    let vm = run(src).unwrap();
    assert_eq!(vm.variables.get("msg"), Some(&Value::Boolean(true)));
}

#[test]
fn question_mark_at_top_level_is_an_error() {
    let err = run(r#"x = qavaRu("pizai")?;"#).expect_err("no frame to unwind to");
    assert!(err.contains("unhandled error"), "unexpected error: {}", err);
}

#[test]
fn question_mark_needs_a_result() {
    let err = run("x = 5?;").expect_err("? on a number");
    assert!(err.contains("needs a result"), "unexpected error: {}", err);
}

#[test]
fn propagation_leaves_no_stack_residue() {
    // The abandoned expression had operands pending when ? unwound.
    let src = r#"ceyal cimY() { qirumpu qavaRu("x"); }
                 ceyal azY() { qirumpu cari(100 + cimY()? + 200); }
                 r = azY();"#;
    let vm = run(src).unwrap();
    assert!(matches!(vm.variables.get("r"), Some(Value::Err(_))));
    assert!(vm.stack.is_empty(), "stack leaked: {:?}", vm.stack);
}

#[test]
fn results_in_tamil_script() {
    let src = "செயல் வகு(அ, ஆ) { \
                 (ஆ == 0) எனில் { திரும்பு தவறு(\"பூஜ்ஜியம்\"); } \
                 திரும்பு சரி(அ / ஆ); \
               } \
               நல்லது = மதிப்பு(வகு(10, 2)); \
               கெட்டது = தவறா(வகு(10, 0));";
    let vm = run(src).unwrap();
    assert_eq!(num(&vm, "நல்லது"), dec(5));
    assert_eq!(vm.variables.get("கெட்டது"), Some(&Value::Boolean(true)));
}

#[test]
fn results_print_readably() {
    let vm = run(r#"a = cari(5); b = qavaRu("pizai");"#).unwrap();
    assert_eq!(vm.variables.get("a").unwrap().to_string(), "சரி(5)");
    assert_eq!(vm.variables.get("b").unwrap().to_string(), "தவறு(pizai)");
}

// --- Modules (இறக்கு) ------------------------------------------------------

/// Write a module file next to the test binary's working directory.
fn write_module(name: &str, source: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!("etamil_mod_{}", name));
    std::fs::write(&path, source).unwrap();
    path
}

fn run_program(statements: Vec<etamil_compiler::parser::Stmt>) -> Result<VM, String> {
    let bytecode = BytecodeCompiler::compile_statements(statements);
    let mut vm = VM::new();
    vm.execute(bytecode)?;
    Ok(vm)
}

#[test]
fn import_brings_in_a_function() {
    let lib = write_module(
        "vari_lib.qmz",
        "ceyal vari(varumAZam) { qirumpu varumAZam * 20%; }",
    );
    let main = format!(r#"iRakku "{}"; moqqam = vari(100000);"#, lib.file_name().unwrap().to_string_lossy());
    let ast = etamil_compiler::module::load_source(&main, &std::env::temp_dir()).unwrap();
    let vm = run_program(ast).unwrap();
    assert_eq!(num(&vm, "moqqam"), dec(20000));
    let _ = std::fs::remove_file(lib);
}

#[test]
fn importing_the_same_module_twice_includes_it_once() {
    let lib = write_module("once.qmz", "eNNikkY = 1;");
    let name = lib.file_name().unwrap().to_string_lossy().to_string();
    let main = format!(r#"iRakku "{}"; iRakku "{}"; x = eNNikkY;"#, name, name);
    let ast = etamil_compiler::module::load_source(&main, &std::env::temp_dir()).unwrap();
    // Only one copy of the module's single statement, plus our own.
    assert_eq!(ast.len(), 2);
    let vm = run_program(ast).unwrap();
    assert_eq!(num(&vm, "x"), dec(1));
    let _ = std::fs::remove_file(lib);
}

#[test]
fn a_circular_import_terminates() {
    let a = write_module("cycle_a.qmz", r#"iRakku "etamil_mod_cycle_b.qmz"; a_ready = 1;"#);
    let b = write_module("cycle_b.qmz", r#"iRakku "etamil_mod_cycle_a.qmz"; b_ready = 1;"#);
    let main = r#"iRakku "etamil_mod_cycle_a.qmz";"#;
    let ast = etamil_compiler::module::load_source(main, &std::env::temp_dir()).unwrap();
    let vm = run_program(ast).unwrap();
    assert_eq!(num(&vm, "a_ready"), dec(1));
    assert_eq!(num(&vm, "b_ready"), dec(1));
    let _ = std::fs::remove_file(a);
    let _ = std::fs::remove_file(b);
}

#[test]
fn importing_a_missing_file_is_an_error() {
    let main = r#"iRakku "etamil_mod_does_not_exist.qmz";"#;
    let err = etamil_compiler::module::load_source(main, &std::env::temp_dir())
        .expect_err("missing module");
    assert!(err.contains("cannot open"), "unexpected error: {}", err);
}

#[test]
fn a_lexical_error_inside_a_module_is_reported() {
    let lib = write_module("bad.qmz", "x = 5 @ 3;");
    let main = r#"iRakku "etamil_mod_bad.qmz";"#;
    let err = etamil_compiler::module::load_source(main, &std::env::temp_dir())
        .expect_err("module has a lexical error");
    assert!(err.contains("unrecognized input"), "unexpected error: {}", err);
    let _ = std::fs::remove_file(lib);
}

#[test]
fn an_unresolved_import_fails_loudly_at_runtime() {
    // Compiling without going through module resolution must not silently
    // skip the import.
    let err = run(r#"iRakku "anything.qmz";"#).expect_err("unresolved import");
    assert!(err.contains("not implemented"), "unexpected error: {}", err);
}

// --- The eTamil standard library (nUlakam/) --------------------------------
// These load the real library files from disk and run them, so a broken
// library is a failing build.

fn stdlib_dir() -> std::path::PathBuf {
    // ETAMIL_STDLIB lets an out-of-tree harness point at the real library.
    if let Ok(dir) = std::env::var("ETAMIL_STDLIB") {
        return std::path::PathBuf::from(dir);
    }
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("nUlakam")
}

/// Run a program with the standard library on the import path.
fn run_with_stdlib(source: &str) -> Result<VM, String> {
    let ast = etamil_compiler::module::load_source(source, &stdlib_dir())?;
    let bytecode = BytecodeCompiler::compile_statements(ast);
    let mut vm = VM::new();
    vm.execute(bytecode)?;
    Ok(vm)
}

#[test]
fn stdlib_files_all_parse() {
    for file in ["col.qmz", "kaNiqam.qmz", "aNi.qmz", "paNam.qmz"] {
        let path = stdlib_dir().join(file);
        etamil_compiler::module::load_file(&path)
            .unwrap_or_else(|e| panic!("nUlakam/{} failed to load: {}", file, e));
    }
}

#[test]
fn stdlib_substring_and_search() {
    let src = r#"இறக்கு "col.qmz";
                 a = துண்டு("வணக்கம் உலகம்", 0, 5);
                 b = தேடு("abcdef", "cd");
                 c = தேடு("abcdef", "zz");
                 d = தொடங்குகிறதா("GSTIN123", "GST");
                 e = முடிகிறதா("invoice.pdf", ".pdf");"#;
    let vm = run_with_stdlib(src).unwrap();
    assert_eq!(text(&vm, "a"), "வணக்கம்");
    assert_eq!(num(&vm, "b"), dec(2));
    assert_eq!(num(&vm, "c"), dec(-1));
    assert_eq!(vm.variables.get("d"), Some(&Value::Boolean(true)));
    assert_eq!(vm.variables.get("e"), Some(&Value::Boolean(true)));
}

#[test]
fn stdlib_trim_split_and_join() {
    let src = r#"இறக்கு "col.qmz";
                 a = ஒழுங்கு("   ravi   ");
                 parts = பிரி("1,2,3", ",");
                 n = நீளம்(parts);
                 first = parts[0];
                 joined = ஒன்றிணை(["a", "b", "c"], "-");"#;
    let vm = run_with_stdlib(src).unwrap();
    assert_eq!(text(&vm, "a"), "ravi");
    assert_eq!(num(&vm, "n"), dec(3));
    assert_eq!(text(&vm, "first"), "1");
    assert_eq!(text(&vm, "joined"), "a-b-c");
}

#[test]
fn stdlib_math() {
    let src = r#"இறக்கு "kaNiqam.qmz";
                 a = முழுமதிப்பு(0 - 5);
                 b = சிறியது(3, 7);
                 c = பெரியது(3, 7);
                 d = கூட்டு([1, 2, 3, 4]);
                 e = மதிப்பு(சராசரி([2, 4, 6]));
                 f = சதவீதம்(1000, 18);"#;
    let vm = run_with_stdlib(src).unwrap();
    assert_eq!(num(&vm, "a"), dec(5));
    assert_eq!(num(&vm, "b"), dec(3));
    assert_eq!(num(&vm, "c"), dec(7));
    assert_eq!(num(&vm, "d"), dec(10));
    assert_eq!(num(&vm, "e"), dec(4));
    assert_eq!(num(&vm, "f"), dec(180));
}

#[test]
fn stdlib_average_of_an_empty_array_is_an_error() {
    let src = r#"இறக்கு "kaNiqam.qmz"; r = சராசரி([]); failed = தவறா(r);"#;
    let vm = run_with_stdlib(src).unwrap();
    assert_eq!(vm.variables.get("failed"), Some(&Value::Boolean(true)));
}

#[test]
fn stdlib_array_helpers() {
    let src = r#"இறக்கு "aNi.qmz";
                 a = உள்ளதா([1, 2, 3], 2);
                 b = இடம்_காண்(["x", "y"], "y");
                 c = தலைகீழ்([1, 2, 3]);
                 d = வெட்டு([1, 2, 3, 4, 5], 1, 3);"#;
    let vm = run_with_stdlib(src).unwrap();
    assert_eq!(vm.variables.get("a"), Some(&Value::Boolean(true)));
    assert_eq!(num(&vm, "b"), dec(1));
    assert_eq!(vm.variables.get("c").unwrap().to_string(), "[3, 2, 1]");
    assert_eq!(vm.variables.get("d").unwrap().to_string(), "[2, 3, 4]");
}

#[test]
fn stdlib_plucks_a_column_from_rows() {
    let src = r#"இறக்கு "aNi.qmz";
                 இறக்கு "kaNiqam.qmz";
                 // Note: a keyword used as a field name is stored under its
                 // token name — {vari: ...} would become the field "Tax", so
                 // that Tamil and romanized spellings agree. Plain names like
                 // these are stored verbatim.
                 வரிசைகள் = [{peyar: "Ravi", amount_due: 1000},
                              {peyar: "Priya", amount_due: 2500}];
                 வரிகள் = புலம்_எடு(வரிசைகள், "amount_due");
                 மொத்தம் = கூட்டு(வரிகள்);"#;
    let vm = run_with_stdlib(src).unwrap();
    assert_eq!(num(&vm, "மொத்தம்"), dec(3500));
}

#[test]
fn stdlib_formats_money_with_indian_grouping() {
    let src = r#"இறக்கு "paNam.qmz";
                 a = காசு_வடிவம்(12345678.5);
                 b = காசு_வடிவம்(1000);
                 c = காசு_வடிவம்(999.99);
                 d = காசு_வடிவம்(0 - 4500.5);
                 e = ரூபாய்(100000);"#;
    let vm = run_with_stdlib(src).unwrap();
    assert_eq!(text(&vm, "a"), "1,23,45,678.50");
    assert_eq!(text(&vm, "b"), "1,000.00");
    assert_eq!(text(&vm, "c"), "999.99");
    assert_eq!(text(&vm, "d"), "-4,500.50");
    assert_eq!(text(&vm, "e"), "₹1,00,000.00");
}

#[test]
fn stdlib_lakhs_and_crores() {
    let src = r#"இறக்கு "paNam.qmz"; a = லட்சம்(2500000); b = கோடி(35000000);"#;
    let vm = run_with_stdlib(src).unwrap();
    assert_eq!(num(&vm, "a"), dec(25));
    assert_eq!(text(&vm, "b"), "3.5");
}

// Strings are measured and indexed by written letter, not code point. A
// Tamil letter is often a consonant plus a vowel sign or pulli, so counting
// chars would give 7 here and every string helper would be off on Tamil text.
#[test]
fn strings_count_written_letters_not_code_points() {
    let vm = run(r#"n = nILam("வணக்கம்"); first = "வணக்கம்"[0]; last = "வணக்கம்"[4];"#).unwrap();
    assert_eq!(num(&vm, "n"), dec(5));
    assert_eq!(text(&vm, "first"), "வ");
    assert_eq!(text(&vm, "last"), "ம்");
}

// --- Host numeric primitives ----------------------------------------------

#[test]
fn rounding_is_half_away_from_zero() {
    let vm = run("a = vattamitu(2.345, 2); b = vattamitu(2.355, 2); c = vattamitu(0 - 2.345, 2);")
        .unwrap();
    assert_eq!(text(&vm, "a"), "2.35");
    assert_eq!(text(&vm, "b"), "2.36");
    assert_eq!(text(&vm, "c"), "-2.35");
}

#[test]
fn floor_and_ceiling() {
    let vm = run("a = qarY(2.9); b = mEl(2.1); c = qarY(0 - 2.1);").unwrap();
    assert_eq!(num(&vm, "a"), dec(2));
    assert_eq!(num(&vm, "b"), dec(3));
    assert_eq!(num(&vm, "c"), dec(-3));
}

#[test]
fn text_to_number_returns_a_result() {
    let vm = run(r#"a = maqippu(eNNAkku("42.5")); b = qavaRA(eNNAkku("abc"));"#).unwrap();
    assert_eq!(text(&vm, "a"), "42.5");
    assert_eq!(vm.variables.get("b"), Some(&Value::Boolean(true)));
}

#[test]
fn case_folding_affects_latin_only() {
    let vm = run(r#"a = mEl_ezuqqu("gstin27"); b = kIz_ezuqqu("PAN"); c = mEl_ezuqqu("வணக்கம்");"#)
        .unwrap();
    assert_eq!(text(&vm, "a"), "GSTIN27");
    assert_eq!(text(&vm, "b"), "pan");
    assert_eq!(text(&vm, "c"), "வணக்கம்");
}

// --- Database ------------------------------------------------------------
// A stand-in backend records what it was asked and returns canned rows, so
// the VM wiring, parameter binding and row conversion can all be checked
// without a driver. The real SQLite backend is behind the `sqlite` feature.

use std::sync::{Arc, Mutex};

#[derive(Default)]
struct Recorded {
    sql: Vec<String>,
    params: Vec<Vec<Value>>,
}

struct FakeDb {
    log: Arc<Mutex<Recorded>>,
    rows: Vec<Value>,
}

impl etamil_compiler::db::Database for FakeDb {
    fn execute(&mut self, sql: &str, params: &[Value]) -> Result<i64, String> {
        let mut log = self.log.lock().unwrap();
        log.sql.push(sql.to_string());
        log.params.push(params.to_vec());
        Ok(1)
    }

    fn query(&mut self, sql: &str, params: &[Value]) -> Result<Vec<Value>, String> {
        let mut log = self.log.lock().unwrap();
        log.sql.push(sql.to_string());
        log.params.push(params.to_vec());
        Ok(self.rows.clone())
    }
}

fn record(pairs: &[(&str, Value)]) -> Value {
    let mut fields = std::collections::HashMap::new();
    for (name, value) in pairs {
        fields.insert(name.to_string(), value.clone());
    }
    Value::Map(fields)
}

/// Run a program with a pre-opened stand-in connection.
fn run_with_db(source: &str, rows: Vec<Value>) -> (Result<VM, String>, Arc<Mutex<Recorded>>) {
    let log = Arc::new(Mutex::new(Recorded::default()));
    let ast = match etamil_compiler::module::load_source(source, &std::env::temp_dir()) {
        Ok(ast) => ast,
        Err(e) => return (Err(e), log),
    };
    let bytecode = BytecodeCompiler::compile_statements(ast);
    let mut vm = VM::new();
    vm.connections.insert(
        "SQLite".to_string(),
        Box::new(FakeDb { log: Arc::clone(&log), rows }),
    );
    let outcome = vm.execute(bytecode).map(|_| vm);
    (outcome, log)
}

#[test]
fn query_returns_an_array_of_records() {
    let rows = vec![
        record(&[("peyar", Value::String("Ravi".into())), ("vari", Value::Number(dec(1000)))]),
        record(&[("peyar", Value::String("Priya".into())), ("vari", Value::Number(dec(2500)))]),
    ];
    let src = r#"qaLam_viZA "SELECT peyar, vari FROM kaNakku", [], varicYkaL;
                 eNNikkY = nILam(varicYkaL);
                 muqal = varicYkaL[0].peyar;"#;
    let (vm, _) = run_with_db(src, rows);
    let vm = vm.unwrap();
    assert_eq!(num(&vm, "eNNikkY"), dec(2));
    assert_eq!(text(&vm, "muqal"), "Ravi");
}

#[test]
fn a_result_set_can_be_iterated_and_summed() {
    let rows = vec![
        record(&[("amount", Value::Number(dec(1000)))]),
        record(&[("amount", Value::Number(dec(2500)))]),
        record(&[("amount", Value::Number(dec(499)))]),
    ];
    let src = r#"qaLam_viZA "SELECT amount FROM ledger", [], varicYkaL;
                 moqqam = 0;
                 ovvoru paqivu il varicYkaL { moqqam = moqqam + paqivu.amount; }"#;
    let (vm, _) = run_with_db(src, rows);
    assert_eq!(num(&vm.unwrap(), "moqqam"), dec(3999));
}

#[test]
fn parameters_are_bound_never_spliced() {
    let src = r#"peyar = "Ravi'; DROP TABLE kaNakku; --";
                 qaLam_viZA "SELECT * FROM kaNakku WHERE peyar = ?", [peyar], varicYkaL;"#;
    let (vm, log) = run_with_db(src, vec![]);
    vm.unwrap();

    let log = log.lock().unwrap();
    // The SQL reaching the driver is exactly what was written.
    assert_eq!(log.sql[0], "SELECT * FROM kaNakku WHERE peyar = ?");
    // The hostile value arrives as a bound parameter, inert.
    assert_eq!(log.params[0].len(), 1);
    assert_eq!(log.params[0][0], Value::String("Ravi'; DROP TABLE kaNakku; --".into()));
}

#[test]
fn execute_binds_several_parameters_in_order() {
    let src = r#"qaLam_cey "INSERT INTO kaNakku (peyar, vari) VALUES (?, ?)", ["Devi", 4200];"#;
    let (vm, log) = run_with_db(src, vec![]);
    vm.unwrap();

    let log = log.lock().unwrap();
    assert_eq!(log.params[0][0], Value::String("Devi".into()));
    assert_eq!(log.params[0][1], Value::Number(dec(4200)));
}

#[test]
fn querying_without_a_connection_is_an_error() {
    let err = run(r#"qaLam_viZA "SELECT 1", [], r;"#)
        .expect_err("no connection is open");
    assert!(err.contains("not connected"), "unexpected error: {}", err);
}

#[test]
fn parameters_must_be_an_array() {
    let (vm, _) = run_with_db(r#"qaLam_viZA "SELECT 1", 5, r;"#, vec![]);
    let err = vm.expect_err("5 is not a parameter array");
    assert!(err.contains("must be an array"), "unexpected error: {}", err);
}

#[test]
fn an_unsupported_database_type_says_so() {
    let err = run(r#"qaLam_iNY mAwkOtipi, "mongodb://localhost";"#)
        .expect_err("MongoDB has no backend");
    assert!(err.contains("not supported yet"), "unexpected error: {}", err);
}

// --- The accounting framework (nUlakam/kaNakkiyal/) ------------------------
// Written in eTamil. These load the real framework files, so a broken
// framework is a failing build.

/// A short chart of accounts and a ledger, as a prelude for the tests below.
const KANAKKIYAL_PRELUDE: &str = r#"
இறக்கு "kaNakkiyal/kaNakkukaL.qmz";
இறக்கு "kaNakkiyal/pErEtu.qmz";
இறக்கு "kaNakkiyal/vari.qmz";
இறக்கு "kaNakkiyal/aRikkYkaL.qmz";

கணக்குகள் = [
    மதிப்பு(கணக்கு_ஆக்கு("1000", "வங்கி",       வகை_சொத்து(),   "நடப்பு")),
    மதிப்பு(கணக்கு_ஆக்கு("1100", "வாங்குநர்",    வகை_சொத்து(),   "நடப்பு")),
    மதிப்பு(கணக்கு_ஆக்கு("2100", "வரி",          வகை_பொறுப்பு(), "நடப்பு")),
    மதிப்பு(கணக்கு_ஆக்கு("3000", "மூலதனம்",     வகை_பங்கு(),    "பங்கு")),
    மதிப்பு(கணக்கு_ஆக்கு("4000", "விற்பனை",     வகை_வருவாய்(),  "இயக்கம்")),
    மதிப்பு(கணக்கு_ஆக்கு("5000", "வாடகை",       வகை_செலவு(),    "இயக்கம்"))
];
பேரேடு = [];
"#;

fn run_kanakkiyal(body: &str) -> Result<VM, String> {
    let source = format!("{}{}", KANAKKIYAL_PRELUDE, body);
    let ast = etamil_compiler::module::load_source(&source, &stdlib_dir())?;
    let bytecode = BytecodeCompiler::compile_statements(ast);
    let mut vm = VM::new();
    vm.execute(bytecode)?;
    Ok(vm)
}

#[test]
fn accounting_posts_a_balanced_entry() {
    let vm = run_kanakkiyal(
        r#"த = பரிவர்த்தனை_ஆக்கு("JV1", "2026-04-01", "மூலதனம்",
               [பற்று_வரிசை("1000", 500000), வரவு_வரிசை("3000", 500000)]);
           பேரேடு = மதிப்பு(பதிவிடு(பேரேடு, த));
           எத்தனை = நீளம்(பேரேடு);
           வங்கி_இருப்பு = கணக்கு_இருப்பு(பேரேடு, "1000", "சொத்து");"#,
    )
    .unwrap();
    assert_eq!(num(&vm, "எத்தனை"), dec(2)); // one ledger line per side
    assert_eq!(num(&vm, "வங்கி_இருப்பு"), dec(500000));
}

#[test]
fn accounting_refuses_an_unbalanced_entry() {
    let vm = run_kanakkiyal(
        r#"த = பரிவர்த்தனை_ஆக்கு("JV9", "2026-04-01", "தவறு",
               [பற்று_வரிசை("5000", 1000), வரவு_வரிசை("1000", 900)]);
           விளைவு = பதிவிடு(பேரேடு, த);
           மறுக்கப்பட்டதா = தவறா(விளைவு);"#,
    )
    .unwrap();
    assert_eq!(vm.variables.get("மறுக்கப்பட்டதா"), Some(&Value::Boolean(true)));
}

#[test]
fn accounting_balances_by_normal_side() {
    // A liability with more credits than debits reads positive, like an
    // asset with more debits than credits.
    let vm = run_kanakkiyal(
        r#"த = பரிவர்த்தனை_ஆக்கு("JV1", "2026-04-01", "மூலதனம்",
               [பற்று_வரிசை("1000", 500000), வரவு_வரிசை("3000", 500000)]);
           பேரேடு = மதிப்பு(பதிவிடு(பேரேடு, த));
           சொத்து_இருப்பு = கணக்கு_இருப்பு(பேரேடு, "1000", "சொத்து");
           பங்கு_இருப்பு = கணக்கு_இருப்பு(பேரேடு, "3000", "பங்கு");"#,
    )
    .unwrap();
    assert_eq!(num(&vm, "சொத்து_இருப்பு"), dec(500000));
    assert_eq!(num(&vm, "பங்கு_இருப்பு"), dec(500000));
}

#[test]
fn gst_sale_splits_net_and_tax() {
    // A ₹2,00,000 sale at 18% posts 2,36,000 receivable, 2,00,000 revenue
    // and 36,000 to the tax control account.
    let vm = run_kanakkiyal(
        r#"வி = விற்பனை_பரிவர்த்தனை("INV1", "2026-04-05", "சேவை",
                 "1100", "4000", "2100", 200000, 18);
           பேரேடு = மதிப்பு(பதிவிடு(பேரேடு, வி));
           வாங்குநர்_இருப்பு = கணக்கு_இருப்பு(பேரேடு, "1100", "சொத்து");
           வருவாய்_இருப்பு = கணக்கு_இருப்பு(பேரேடு, "4000", "வருவாய்");
           வரி_இருப்பு = கணக்கு_இருப்பு(பேரேடு, "2100", "பொறுப்பு");"#,
    )
    .unwrap();
    assert_eq!(num(&vm, "வாங்குநர்_இருப்பு"), dec(236000));
    assert_eq!(num(&vm, "வருவாய்_இருப்பு"), dec(200000));
    assert_eq!(num(&vm, "வரி_இருப்பு"), dec(36000));
}

#[test]
fn cgst_and_sgst_always_sum_to_the_whole_tax() {
    // An odd number of paise must not go missing in the halving.
    let vm = run_kanakkiyal(
        r#"ப = மாநில_பிரிப்பு(29699.47, மெய்);
           மொத்தம் = ப.cgst + ப.sgst;
           igst_மதிப்பு = மாநில_பிரிப்பு(29699.47, பொய்).igst;"#,
    )
    .unwrap();
    assert_eq!(text(&vm, "மொத்தம்"), "29699.47");
    assert_eq!(text(&vm, "igst_மதிப்பு"), "29699.47");
}

#[test]
fn trial_balance_agrees_and_the_sheet_balances() {
    let vm = run_kanakkiyal(
        r#"த1 = பரிவர்த்தனை_ஆக்கு("JV1", "2026-04-01", "மூலதனம்",
                [பற்று_வரிசை("1000", 500000), வரவு_வரிசை("3000", 500000)]);
           பேரேடு = மதிப்பு(பதிவிடு(பேரேடு, த1));

           வி = விற்பனை_பரிவர்த்தனை("INV1", "2026-04-05", "சேவை",
                 "1100", "4000", "2100", 200000, 18);
           பேரேடு = மதிப்பு(பதிவிடு(பேரேடு, வி));

           த2 = பரிவர்த்தனை_ஆக்கு("JV2", "2026-04-10", "வாடகை",
                [பற்று_வரிசை("5000", 45000), வரவு_வரிசை("1000", 45000)]);
           பேரேடு = மதிப்பு(பதிவிடு(பேரேடு, த2));

           ஆய்வு = இருப்பாய்வு(பேரேடு, கணக்குகள்);
           ஆய்வு_சரியா = ஆய்வு.சமநிலையா;

           வரு = வருமான_அறிக்கை(பேரேடு, கணக்குகள்);
           லாபம் = வரு.நிகர_லாபம்;

           நிலை = இருப்புநிலை(பேரேடு, கணக்குகள்);
           நிலை_சரியா = நிலை.சமநிலையா;
           மொத்த_சொத்து = நிலை.மொத்த_சொத்து;"#,
    )
    .unwrap();
    assert_eq!(vm.variables.get("ஆய்வு_சரியா"), Some(&Value::Boolean(true)));
    // 200,000 revenue less 45,000 rent
    assert_eq!(num(&vm, "லாபம்"), dec(155000));
    // bank 455,000 + receivable 236,000
    assert_eq!(num(&vm, "மொத்த_சொத்து"), dec(691000));
    assert_eq!(vm.variables.get("நிலை_சரியா"), Some(&Value::Boolean(true)));
}

// --- Dates ----------------------------------------------------------------

#[test]
fn date_arithmetic() {
    let vm = run(r#"a = nAL_vERupAtu("2026-01-10", "2026-04-30");
                    b = nAL_kUttu("2026-02-28", 1);
                    c = nAL_kUttu("2024-02-28", 1);
                    d = nAL_vERupAtu("2026-04-30", "2026-01-10");"#)
        .unwrap();
    assert_eq!(num(&vm, "a"), dec(110));
    assert_eq!(text(&vm, "b"), "2026-03-01");
    assert_eq!(text(&vm, "c"), "2024-02-29"); // 2024 is a leap year
    assert_eq!(num(&vm, "d"), dec(-110));
}

#[test]
fn a_malformed_date_is_an_error() {
    let err = run(r#"x = nAL_kUttu("not-a-date", 1);"#).expect_err("bad date");
    assert!(err.contains("is not a date"), "unexpected error: {}", err);
}

// --- Periods, clearing and ageing -----------------------------------------

/// A ledger with three invoices and one receipt, as a prelude.
const NILUVY_PRELUDE: &str = r#"
இறக்கு "kaNakkiyal/kaNakkukaL.qmz";
இறக்கு "kaNakkiyal/pErEtu.qmz";
இறக்கு "kaNakkiyal/vari.qmz";
இறக்கு "kaNakkiyal/oqukkItu.qmz";
இறக்கு "kaNakkiyal/aRikkYkaL.qmz";

கணக்குகள் = [
    மதிப்பு(கணக்கு_ஆக்கு("1000", "வங்கி",       வகை_சொத்து(),   "நடப்பு")),
    மதிப்பு(கணக்கு_ஆக்கு("1100", "வாங்குநர்",   வகை_சொத்து(),   "நடப்பு")),
    மதிப்பு(கணக்கு_ஆக்கு("2100", "வணிகவரி",    வகை_பொறுப்பு(), "நடப்பு")),
    மதிப்பு(கணக்கு_ஆக்கு("4000", "விற்பனை",    வகை_வருவாய்(),  "இயக்கம்"))
];
பேரேடு = [];
ஒதுக்கீடுகள் = [];

பேரேடு = மதிப்பு(பதிவிடு(பேரேடு, விற்பனை_பரிவர்த்தனை(
    "INV001", "2026-01-10", "ஜனவரி", "1100", "4000", "2100", 100000, 18)));
பேரேடு = மதிப்பு(பதிவிடு(பேரேடு, விற்பனை_பரிவர்த்தனை(
    "INV002", "2026-03-05", "மார்ச்", "1100", "4000", "2100", 50000, 18)));
பேரேடு = மதிப்பு(பதிவிடு(பேரேடு, விற்பனை_பரிவர்த்தனை(
    "INV003", "2026-04-12", "ஏப்ரல்", "1100", "4000", "2100", 75000, 18)));
பேரேடு = மதிப்பு(பதிவிடு(பேரேடு, பணம்_பெறு(
    "RCT001", "2026-04-20", "பணம்", "1000", "1100", 118000)));
"#;

fn run_niluvy(body: &str) -> Result<VM, String> {
    let source = format!("{}{}", NILUVY_PRELUDE, body);
    let ast = etamil_compiler::module::load_source(&source, &stdlib_dir())?;
    let bytecode = BytecodeCompiler::compile_statements(ast);
    let mut vm = VM::new();
    vm.execute(bytecode)?;
    Ok(vm)
}

#[test]
fn an_unassigned_invoice_is_outstanding_in_full() {
    let vm = run_niluvy(
        r#"மீதம் = நிலுவைத்_தொகை(பேரேடு, ஒதுக்கீடுகள், "INV001", "1100", "சொத்து");"#,
    )
    .unwrap();
    assert_eq!(num(&vm, "மீதம்"), dec(118000)); // 100,000 + 18%
}

#[test]
fn assigning_a_receipt_clears_the_invoice() {
    let vm = run_niluvy(
        r#"ஒதுக்கீடுகள் = மதிப்பு(ஒதுக்கு(ஒதுக்கீடுகள், பேரேடு,
               "RCT001", "INV001", 118000, "1100", "சொத்து"));
           மீதம் = நிலுவைத்_தொகை(பேரேடு, ஒதுக்கீடுகள், "INV001", "1100", "சொத்து");"#,
    )
    .unwrap();
    assert_eq!(num(&vm, "மீதம்"), dec(0));
}

#[test]
fn clearing_more_than_the_invoice_is_refused() {
    let vm = run_niluvy(
        r#"விளைவு = ஒதுக்கு(ஒதுக்கீடுகள், பேரேடு,
               "RCT001", "INV002", 999999, "1100", "சொத்து");
           மறுக்கப்பட்டதா = தவறா(விளைவு);"#,
    )
    .unwrap();
    assert_eq!(vm.variables.get("மறுக்கப்பட்டதா"), Some(&Value::Boolean(true)));
}

#[test]
fn spreading_a_receipt_beyond_its_own_value_is_refused() {
    let vm = run_niluvy(
        r#"ஒதுக்கீடுகள் = மதிப்பு(ஒதுக்கு(ஒதுக்கீடுகள், பேரேடு,
               "RCT001", "INV001", 118000, "1100", "சொத்து"));
           விளைவு = ஒதுக்கு(ஒதுக்கீடுகள், பேரேடு,
               "RCT001", "INV002", 1, "1100", "சொத்து");
           மறுக்கப்பட்டதா = தவறா(விளைவு);"#,
    )
    .unwrap();
    assert_eq!(vm.variables.get("மறுக்கப்பட்டதா"), Some(&Value::Boolean(true)));
}

#[test]
fn ageing_buckets_by_invoice_date() {
    // As at 30 April: INV003 is 18 days old, INV002 is 56.
    let vm = run_niluvy(
        r#"ஒதுக்கீடுகள் = மதிப்பு(ஒதுக்கு(ஒதுக்கீடுகள், பேரேடு,
               "RCT001", "INV001", 118000, "1100", "சொத்து"));
           அ = வயது_அட்டவணை(பேரேடு, ஒதுக்கீடுகள், "1100", "சொத்து", "2026-04-30");
           நடப்பு_தொகை = அ.நடப்பு;
           முப்பது = அ.முப்பது;
           மொத்தம் = அ.மொத்தம்;"#,
    )
    .unwrap();
    assert_eq!(num(&vm, "நடப்பு_தொகை"), dec(88500)); // INV003
    assert_eq!(num(&vm, "முப்பது"), dec(59000)); // INV002
    assert_eq!(num(&vm, "மொத்தம்"), dec(147500));
}

#[test]
fn an_income_statement_covers_only_its_period() {
    // FY 2026-27 starts 1 April, so only INV003 falls inside it.
    let vm = run_niluvy(
        r#"ஆண்டு = இந்திய_ஆண்டு(2026);
           வரு = கால_வருமான_அறிக்கை(பேரேடு, கணக்குகள், ஆண்டு);
           கால_வருவாய் = வரு.மொத்த_வருவாய்;
           முழு = வருமான_அறிக்கை(பேரேடு, கணக்குகள்);
           மொத்த_வருவாய்_எல்லாம் = முழு.மொத்த_வருவாய்;"#,
    )
    .unwrap();
    assert_eq!(num(&vm, "கால_வருவாய்"), dec(75000));
    assert_eq!(num(&vm, "மொத்த_வருவாய்_எல்லாம்"), dec(225000));
}

#[test]
fn a_balance_sheet_is_cumulative_to_its_date() {
    let vm = run_niluvy(
        r#"நிலை = நாள்_இருப்புநிலை(பேரேடு, கணக்குகள், "2026-04-30");
           சொத்து_மொத்தம் = நிலை.மொத்த_சொத்து;
           சமமா = நிலை.சமநிலையா;
           முன்பு = நாள்_இருப்புநிலை(பேரேடு, கணக்குகள், "2026-02-01");
           முன்பு_சொத்து = முன்பு.மொத்த_சொத்து;"#,
    )
    .unwrap();
    // bank 118,000 + receivables 147,500
    assert_eq!(num(&vm, "சொத்து_மொத்தம்"), dec(265500));
    assert_eq!(vm.variables.get("சமமா"), Some(&Value::Boolean(true)));
    // as at 1 February only INV001 exists
    assert_eq!(num(&vm, "முன்பு_சொத்து"), dec(118000));
}

#[test]
fn an_account_statement_carries_a_running_balance() {
    let vm = run_niluvy(
        r#"அ = கணக்கு_அறிக்கை(பேரேடு, "1100", "சொத்து");
           எத்தனை = நீளம்(அ.வரிகள்);
           இறுதி = அ.இறுதி_இருப்பு;
           முதல் = அ.வரிகள்[0].ஓட்ட_இருப்பு;"#,
    )
    .unwrap();
    assert_eq!(num(&vm, "எத்தனை"), dec(4));
    assert_eq!(num(&vm, "முதல்"), dec(118000));
    assert_eq!(num(&vm, "இறுதி"), dec(147500));
}

#[test]
fn cash_flow_over_a_period() {
    let vm = run_niluvy(
        r#"ஆண்டு = இந்திய_ஆண்டு(2026);
           ப = பணப்புழக்க_அறிக்கை(பேரேடு, ["1000"], ஆண்டு);
           உள்வரவு = ப.உள்வரவு;
           இறுதி = ப.இறுதி_இருப்பு;"#,
    )
    .unwrap();
    // the only cash movement is the 118,000 receipt on 20 April
    assert_eq!(num(&vm, "உள்வரவு"), dec(118000));
    assert_eq!(num(&vm, "இறுதி"), dec(118000));
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
