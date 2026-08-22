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
    let ast = parser.parse().map_err(|error| error.to_string())?;

    // The same order the compiler uses, so a test exercises the real pipeline
    // rather than a shortcut through it.
    etamil_compiler::check::check(&ast).map_err(|errors| {
        errors
            .iter()
            .map(|e| e.to_string())
            .collect::<Vec<_>>()
            .join("; ")
    })?;

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
fn keyword_backed_names_keep_the_spelling_the_author_used() {
    // வருவாய் / varuvAy is the Revenue keyword rather than a plain identifier,
    // and the variable used to land under "Revenue" because the lexer threw
    // the spelling away. For a language whose purpose is letting people write
    // in their own language, having their names quietly anglicised was the
    // sharpest remaining contradiction.
    let roman = run("eN varuvAy = 5;").unwrap();
    assert_eq!(num(&roman, "varuvAy"), dec(5));
    assert!(roman.variables.get("Revenue").is_none());

    let tamil = run("எண் வருவாய் = 5;").unwrap();
    assert_eq!(num(&tamil, "வருவாய்"), dec(5));
    assert!(tamil.variables.get("Revenue").is_none());
}

// The other half of the same change: the two spellings now name two different
// variables. That is a real change in meaning, and the reason it is right is
// that a name is data — what the author typed — not a language construct.
#[test]
fn the_two_spellings_of_a_keyword_name_different_variables() {
    let vm = run("வருவாய் = 5; varuvAy = 7;").unwrap();

    assert_eq!(num(&vm, "வருவாய்"), dec(5));
    assert_eq!(num(&vm, "varuvAy"), dec(7));
}

// A quoted field name is its contents, so a record can still be reached by a
// string built at runtime.
#[test]
fn a_quoted_field_name_is_its_contents() {
    let vm = run(r#"r = {"வரி": 100}; சாவி = "வரி"; எடுத்தது = r[சாவி];"#).unwrap();

    assert_eq!(num(&vm, "எடுத்தது"), dec(100));
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

// A UTF-8 BOM is invisible and is what Windows editors produce by default.
// Before this was handled, every such program failed on line 1, column 1
// with an error naming a character the author could not see.
#[test]
fn a_utf8_bom_is_not_a_lexical_error() {
    let with_bom = "\u{FEFF}அச்சு \"வணக்கம்\";";
    let tokens = lexer::tokenize(with_bom).expect("a BOM must not fail the lexer");
    let without = lexer::tokenize("அச்சு \"வணக்கம்\";").unwrap();
    assert_eq!(tokens.len(), without.len());
}

#[test]
fn an_empty_program_lexes_to_nothing() {
    assert_eq!(lexer::tokenize("").unwrap().len(), 0);
    assert_eq!(lexer::tokenize("\u{FEFF}").unwrap().len(), 0);
    assert_eq!(lexer::tokenize("   \n\t ").unwrap().len(), 0);
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
    // A refusal is now a returned error rather than a panic, so this no longer
    // needs catch_unwind to observe it.
    let failure = run("varicY = 1;");
    assert!(
        failure.is_err(),
        "varicY (OrderBy) should still be reserved"
    );
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
    assert!(run("eN = 1;").is_err(), "eN (IntegerType) should be reserved");

    // தொகை / toqai is the Amount keyword, and is a perfectly good name for an
    // amount — which is the whole reason financial keywords are not reserved.
    let vm = run("toqai = 5;").unwrap();
    assert_eq!(num(&vm, "toqai"), dec(5));
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

// Regression: ஒழுங்கு indexed the string directly under a `மற்றும்` guard.
// The guard was right, but மற்றும் evaluates both sides, so trimming "" or a
// string of nothing but spaces ran the index off the end and aborted the
// program.
#[test]
fn stdlib_trim_handles_empty_and_all_space_strings() {
    let vm = run_with_stdlib(
        r#"இறக்கு "col.qmz";
           காலி = நீளம்(ஒழுங்கு(""));
           வெளிகள் = நீளம்(ஒழுங்கு("   "));
           ஒன்று = ஒழுங்கு("  வரி  ");
           விளிம்பில்லை = ஒழுங்கு("வரவு");"#,
    )
    .unwrap();

    assert_eq!(num(&vm, "காலி"), dec(0));
    assert_eq!(num(&vm, "வெளிகள்"), dec(0));
    assert_eq!(text(&vm, "ஒன்று"), "வரி");
    assert_eq!(text(&vm, "விளிம்பில்லை"), "வரவு");
}

#[test]
fn stdlib_character_access_is_bounds_safe() {
    let vm = run_with_stdlib(
        r#"இறக்கு "col.qmz";
           முதல் = எழுத்து("வரவு", 0);
           தாண்டியது = எழுத்து("வரவு", 99);
           எதிர்மறை = எழுத்து("வரவு", 0 - 1);"#,
    )
    .unwrap();

    assert_eq!(text(&vm, "முதல்"), "வ");
    assert_eq!(text(&vm, "தாண்டியது"), "");
    assert_eq!(text(&vm, "எதிர்மறை"), "");
}

// --- JSON (nUlakam/jEcAZ.qmz) ---------------------------------------------
// Written in eTamil, not the host: a record key can be computed at runtime,
// which is the one capability a parser needs to build a record from data.

#[test]
fn json_serializes_each_kind_of_value() {
    let vm = run_with_stdlib(
        r#"இறக்கு "jEcAZ.qmz";
           எண்ணானது = ஜேசான்_ஆக்கு(1500.5);
           சரமானது = ஜேசான்_ஆக்கு("வரவு");
           ஈர்மம் = ஜேசான்_ஆக்கு(மெய்);
           இல்லாதது = ஜேசான்_ஆக்கு(இன்மை);
           அணியானது = ஜேசான்_ஆக்கு([1, 2, 3]);"#,
    )
    .unwrap();

    assert_eq!(text(&vm, "எண்ணானது"), "1500.5");
    assert_eq!(text(&vm, "சரமானது"), r#""வரவு""#);
    assert_eq!(text(&vm, "ஈர்மம்"), "true");
    // இன்மை prints as "nil" for people; JSON needs the word null.
    assert_eq!(text(&vm, "இல்லாதது"), "null");
    assert_eq!(text(&vm, "அணியானது"), "[1,2,3]");
}

#[test]
fn json_escapes_quotes_and_newlines() {
    let vm = run_with_stdlib(
        r#"இறக்கு "jEcAZ.qmz";
           விடை = ஜேசான்_ஆக்கு("he said \"hi\"\nnext");"#,
    )
    .unwrap();

    assert_eq!(text(&vm, "விடை"), r#""he said \"hi\"\nnext""#);
}

// Record fields come out sorted, because that is the order ஒவ்வொரு walks a
// record — so a response body is stable enough to assert on.
#[test]
fn json_writes_record_fields_in_a_stable_order() {
    let vm = run_with_stdlib(
        r#"இறக்கு "jEcAZ.qmz";
           விடை = ஜேசான்_ஆக்கு({vakY: "செலவு", qokY: 250, active: பொய்});"#,
    )
    .unwrap();

    assert_eq!(
        text(&vm, "விடை"),
        r#"{"active":false,"qokY":250,"vakY":"செலவு"}"#
    );
}

#[test]
fn json_parses_an_object_into_a_usable_record() {
    let vm = run_with_stdlib(
        r#"இறக்கு "jEcAZ.qmz";
           விளைவு = ஜேசான்_படி("{\"vakY\":\"செலவு\",\"qokY\":1500,\"lines\":[1,2,3]}");
           சரியா_இருந்ததா = சரியா(விளைவு);
           ப = மதிப்பு(விளைவு);
           வகைப்_பெயர் = ப["vakY"];
           // Numbers parse as numbers, not text, so they still add up.
           கூட்டல் = ப["qokY"] + ப["lines"][2];"#,
    )
    .unwrap();

    assert_eq!(vm.variables.get("சரியா_இருந்ததா"), Some(&Value::Boolean(true)));
    assert_eq!(text(&vm, "வகைப்_பெயர்"), "செலவு");
    assert_eq!(num(&vm, "கூட்டல்"), dec(1503));
}

#[test]
fn json_survives_a_round_trip() {
    let vm = run_with_stdlib(
        r#"இறக்கு "jEcAZ.qmz";
           மூலம் = "{\"a\":[1,2],\"b\":{\"c\":\"வரி\"},\"d\":null,\"e\":true}";
           மறுபடி = ஜேசான்_ஆக்கு(மதிப்பு(ஜேசான்_படி(மூலம்)));"#,
    )
    .unwrap();

    assert_eq!(
        text(&vm, "மறுபடி"),
        r#"{"a":[1,2],"b":{"c":"வரி"},"d":null,"e":true}"#
    );
}

// Malformed input is a தவறு, never a half-read value: accepting trailing text
// would quietly treat half a request body as the whole of it.
#[test]
fn json_refuses_malformed_input() {
    let vm = run_with_stdlib(
        r#"இறக்கு "jEcAZ.qmz";
           முடிவற்றது = தவறா(ஜேசான்_படி("\"abc"));
           மதிப்பற்றது = தவறா(ஜேசான்_படி("{\"a\": }"));
           மீதியுள்ளது = தவறா(ஜேசான்_படி("{} extra"));
           சாவியற்றது = தவறா(ஜேசான்_படி("{1: 2}"));
           காலியானது = தவறா(ஜேசான்_படி(""));"#,
    )
    .unwrap();

    for name in [
        "முடிவற்றது",
        "மதிப்பற்றது",
        "மீதியுள்ளது",
        "சாவியற்றது",
        "காலியானது",
    ] {
        assert_eq!(
            vm.variables.get(name),
            Some(&Value::Boolean(true)),
            "{} should have been refused",
            name
        );
    }
}

#[test]
fn json_reads_an_empty_object_and_array() {
    let vm = run_with_stdlib(
        r#"இறக்கு "jEcAZ.qmz";
           பொருள்_நீளம் = நீளம்(மதிப்பு(ஜேசான்_படி("{}")));
           அணி_நீளம் = நீளம்(மதிப்பு(ஜேசான்_படி("[]")));
           வெண்மையுடன் = ஜேசான்_ஆக்கு(மதிப்பு(ஜேசான்_படி("  {  \"a\" : 1 }  ")));"#,
    )
    .unwrap();

    assert_eq!(num(&vm, "பொருள்_நீளம்"), dec(0));
    assert_eq!(num(&vm, "அணி_நீளம்"), dec(0));
    assert_eq!(text(&vm, "வெண்மையுடன்"), r#"{"a":1}"#);
}

// --- Scheduled blocks (இடைவெளி) -------------------------------------------
// Lifted out of the program at startup like வழி, and needing a server for the
// same reason: the VM has no clock to hang them on.

#[test]
fn a_schedule_needs_a_server() {
    let failure = run(r#"இடைவெளி 60 { அச்சு "நள்ளிரவு"; }"#);

    let message = failure.unwrap_err();
    assert!(
        message.contains("இடைவெளி") && message.contains("not implemented"),
        "should refuse loudly under the VM: {}",
        message
    );
}

// It parses, and the body is ordinary code — which is what lets --check find a
// mistake inside one without a server.
#[test]
fn a_schedule_body_is_checked() {
    let tokens = etamil_compiler::lexer::tokenize(
        r#"இடைவெளி 3600 { ஈர்ம கொடியா = [1, 2]; }"#,
    )
    .expect("should lex");
    let ast = Parser::new(tokens.iter()).parse().expect("should parse");

    let errors = etamil_compiler::check::check(&ast)
        .expect_err("an array is not a boolean, even inside a schedule");
    assert!(errors[0].to_string().contains("கொடியா"));
}

#[test]
fn a_schedule_takes_an_interval_and_a_block() {
    let tokens = etamil_compiler::lexer::tokenize(
        r#"இடைவெளி 900 { அச்சு "x"; அச்சு "y"; }"#,
    )
    .expect("should lex");
    let ast = Parser::new(tokens.iter()).parse().expect("should parse");

    assert_eq!(ast.len(), 1);
    match &ast[0] {
        etamil_compiler::parser::Stmt::Schedule { body, .. } => {
            assert_eq!(body.len(), 2, "both statements belong to the block");
        }
        other => panic!("expected a schedule, got {:?}", other),
    }
}

#[test]
fn a_schedule_without_a_block_is_a_parse_error() {
    let error = parse_error(r#"இடைவெளி 60;"#);
    assert!(error.contains("'{'"), "should want a block: {}", error);
}

// --- Bytes and encoding ---------------------------------------------------
// A byte array is an ordinary array of numbers, so no new value type was
// needed. The host supplies only the two conversions a language cannot reach —
// the UTF-8 bytes of a string, and the string back — and base64 and hex are
// written in eTamil on top of them.

#[test]
fn bytes_are_the_utf8_of_the_text() {
    let vm = run(
        r#"ascii = பைட்டுகள்("A");
           தமிழ் = பைட்டுகள்("வ");
           காலி = நீளம்(பைட்டுகள்(""));
           முதல் = ascii[0];
           நீ = நீளம்(தமிழ்);"#,
    )
    .unwrap();

    assert_eq!(num(&vm, "முதல்"), dec(65));
    // வ is U+0BB5, three bytes in UTF-8 — not one "character".
    assert_eq!(num(&vm, "நீ"), dec(3));
    assert_eq!(num(&vm, "காலி"), dec(0));
}

#[test]
fn bytes_round_trip_through_a_string() {
    let vm = run(
        r#"மூலம் = "வணக்கம் உலகம்";
           மறுபடி = மதிப்பு(பைட்டுச்_சரம்(பைட்டுகள்(மூலம்)));
           ஒன்றா = மறுபடி == மூலம்;"#,
    )
    .unwrap();

    assert_eq!(vm.variables.get("ஒன்றா"), Some(&Value::Boolean(true)));
}

// Bytes that are not valid UTF-8 arrive from outside all the time, so this is
// a தவறு a caller can handle rather than an error that stops the program.
#[test]
fn bytes_that_are_not_utf8_are_refused() {
    let vm = run(
        r#"தவறியது = தவறா(பைட்டுச்_சரம்([255, 254]));
           சரியது = சரியா(பைட்டுச்_சரம்([104, 105]));"#,
    )
    .unwrap();

    assert_eq!(vm.variables.get("தவறியது"), Some(&Value::Boolean(true)));
    assert_eq!(vm.variables.get("சரியது"), Some(&Value::Boolean(true)));
}

#[test]
fn a_byte_must_be_a_whole_number_in_range() {
    assert!(run(r#"விடை = பைட்டுச்_சரம்([300]);"#).is_err(), "300 is not a byte");
    assert!(run(r#"விடை = பைட்டுச்_சரம்([1.5]);"#).is_err(), "1.5 is not a byte");
    assert!(run(r#"விடை = பைட்டுச்_சரம்(65);"#).is_err(), "needs an array");
}

// The RFC 4648 vectors, so the answers are checked against something this
// project did not produce. The padding cases are the ones that go wrong.
#[test]
fn base64_matches_the_published_vectors() {
    let vm = run_with_stdlib(
        r#"இறக்கு "kuRiyAkkam.qmz";
           காலி = அறுபத்துநான்கு_ஆக்கு("");
           ஒன்று = அறுபத்துநான்கு_ஆக்கு("f");
           இரண்டு = அறுபத்துநான்கு_ஆக்கு("fo");
           மூன்று = அறுபத்துநான்கு_ஆக்கு("foo");
           ஆறு = அறுபத்துநான்கு_ஆக்கு("foobar");"#,
    )
    .unwrap();

    assert_eq!(text(&vm, "காலி"), "");
    assert_eq!(text(&vm, "ஒன்று"), "Zg==");
    assert_eq!(text(&vm, "இரண்டு"), "Zm8=");
    assert_eq!(text(&vm, "மூன்று"), "Zm9v");
    assert_eq!(text(&vm, "ஆறு"), "Zm9vYmFy");
}

#[test]
fn base64_round_trips_tamil() {
    let vm = run_with_stdlib(
        r#"இறக்கு "kuRiyAkkam.qmz";
           மூலம் = "வணக்கம் உலகம் — ₹1,23,456.78";
           மறுபடி = மதிப்பு(அறுபத்துநான்கு_படி(அறுபத்துநான்கு_ஆக்கு(மூலம்)));
           ஒன்றா = மறுபடி == மூலம்;"#,
    )
    .unwrap();

    assert_eq!(vm.variables.get("ஒன்றா"), Some(&Value::Boolean(true)));
}

#[test]
fn base64_refuses_what_is_not_base64() {
    let vm = run_with_stdlib(
        r#"இறக்கு "kuRiyAkkam.qmz";
           குப்பை = தவறா(அறுபத்துநான்கு_படி("!!!"));
           நல்லது = சரியா(அறுபத்துநான்கு_படி("Zm9v"));"#,
    )
    .unwrap();

    assert_eq!(vm.variables.get("குப்பை"), Some(&Value::Boolean(true)));
    assert_eq!(vm.variables.get("நல்லது"), Some(&Value::Boolean(true)));
}

#[test]
fn hex_encodes_and_decodes() {
    let vm = run_with_stdlib(
        r#"இறக்கு "kuRiyAkkam.qmz";
           அ = பதினாறு_ஆக்கு("abc");
           தமிழ் = பதினாறு_ஆக்கு("வ");
           மறுபடி = மதிப்பு(பதினாறு_படி("616263"));
           பெரிய = மதிப்பு(பதினாறு_படி("616263"));"#,
    )
    .unwrap();

    assert_eq!(text(&vm, "அ"), "616263");
    assert_eq!(text(&vm, "தமிழ்"), "e0aeb5");
    assert_eq!(text(&vm, "மறுபடி"), "abc");
    assert_eq!(text(&vm, "பெரிய"), "abc");
}

#[test]
fn hex_refuses_bad_input() {
    let vm = run_with_stdlib(
        r#"இறக்கு "kuRiyAkkam.qmz";
           ஒற்றை = தவறா(பதினாறு_படி("abc"));
           எழுத்து = தவறா(பதினாறு_படி("zz"));"#,
    )
    .unwrap();

    assert_eq!(vm.variables.get("ஒற்றை"), Some(&Value::Boolean(true)));
    assert_eq!(vm.variables.get("எழுத்து"), Some(&Value::Boolean(true)));
}

// மீதி follows from தரை and the arithmetic already present, so it belongs in
// nUlakam rather than the host — and base64 cannot be written without it,
// there being no % operator.
#[test]
fn stdlib_remainder() {
    let vm = run_with_stdlib(
        r#"இறக்கு "kaNiqam.qmz";
           அ = மீதி(17, 5);
           ஆ = மீதி(64, 64);
           இ = மீதி(3, 5);
           ஈ = மீதி(5, 0);"#,
    )
    .unwrap();

    assert_eq!(num(&vm, "அ"), dec(2));
    assert_eq!(num(&vm, "ஆ"), dec(0));
    assert_eq!(num(&vm, "இ"), dec(3));
    // Dividing by zero would otherwise be a runtime error; 0 is the safe answer.
    assert_eq!(num(&vm, "ஈ"), dec(0));
}

// --- Signing (HMAC) -------------------------------------------------------
// What makes a payment webhook trustworthy. HMAC needs bytes and a
// constant-time comparison, so it is a host primitive; what gets signed, and
// what a signature means, stays in eTamil.

#[test]
fn a_signature_is_sha256_hex() {
    let vm = run(r#"கை = கையொப்பம்("secret", "hello"); நீ = நீளம்(கை);"#).unwrap();

    // Checked against the published HMAC-SHA256("secret", "hello").
    assert_eq!(
        text(&vm, "கை"),
        "88aab3ede8d3adf94d26ab90d3bafd4a2083070c3bcce9c014ee04a443847c0b"
    );
    assert_eq!(num(&vm, "நீ"), dec(64));
}

// The attack this exists to stop: an amount changed in transit must invalidate
// the signature that came with it.
#[test]
fn tampering_with_a_signed_payload_invalidates_it() {
    let vm = run(
        r#"இரகசியம் = "wh_secret";
           உண்மை = "{\"amount\":45000}";
           போலி = "{\"amount\":1}";
           கை = கையொப்பம்(இரகசியம், உண்மை);

           உண்மையா = கையொப்பம்_சரியா(இரகசியம், உண்மை, கை);
           திருத்தியது = கையொப்பம்_சரியா(இரகசியம், போலி, கை);
           வேறு_விசை = கையொப்பம்_சரியா("guess", உண்மை, கை);
           குப்பை = கையொப்பம்_சரியா(இரகசியம், உண்மை, "deadbeef");
           காலி = கையொப்பம்_சரியா(இரகசியம், உண்மை, "");"#,
    )
    .unwrap();

    assert_eq!(vm.variables.get("உண்மையா"), Some(&Value::Boolean(true)));
    for refused in ["திருத்தியது", "வேறு_விசை", "குப்பை", "காலி"] {
        assert_eq!(
            vm.variables.get(refused),
            Some(&Value::Boolean(false)),
            "{} should not verify",
            refused
        );
    }
}

// Gateways disagree about the case of the hex they send.
#[test]
fn a_signature_verifies_in_either_case() {
    let vm = run(
        r#"கை = கையொப்பம்("k", "payload");
           பெரிய = கையொப்பம்_சரியா("k", "payload", மேல்_எழுத்து(கை));"#,
    )
    .unwrap();

    assert_eq!(vm.variables.get("பெரிய"), Some(&Value::Boolean(true)));
}

// Signing the same thing twice must agree, or a webhook check would be a
// coin toss.
#[test]
fn signing_is_deterministic() {
    let vm = run(
        r#"அ = கையொப்பம்("k", "m"); ஆ = கையொப்பம்("k", "m"); ஒன்றா = அ == ஆ;"#,
    )
    .unwrap();

    assert_eq!(vm.variables.get("ஒன்றா"), Some(&Value::Boolean(true)));
}

// --- Outbound HTTP --------------------------------------------------------
// Only the offline behaviour is asserted here. A test that reached the network
// would fail on a machine without it, and would be testing the network rather
// than this code.

#[test]
fn an_unreachable_host_is_a_failure_not_an_error() {
    let vm = run(
        r#"விளைவு = வலை_பெறு("https://no-such-host-etamil-test.invalid", {});
           தோல்வியா = தவறா(விளைவு);"#,
    )
    .unwrap();

    // A தவறு, so a caller handles it — not a runtime error that stops the VM.
    assert_eq!(vm.variables.get("தோல்வியா"), Some(&Value::Boolean(true)));
}

#[test]
fn a_request_needs_the_right_number_of_arguments() {
    assert!(run(r#"விளைவு = வலை_பெறு("https://example.invalid");"#).is_err());
    assert!(run(r#"கை = கையொப்பம்("only-a-key");"#).is_err());
}

// --- Authentication -------------------------------------------------------
// bcrypt and HMAC live in the host because eTamil cannot express them; the
// policy built on top stays in the language. A token's payload crosses the
// boundary as JSON text, so the host never learns what a claim means.

#[test]
fn a_password_hash_verifies_only_the_right_password() {
    let vm = run(
        r#"மறையீடு = கடவுச்சொல்_மறை("correct-horse");
           சரியானது = கடவுச்சொல்_சரியா("correct-horse", மறையீடு);
           தவறானது = கடவுச்சொல்_சரியா("guess", மறையீடு);
           காலியானது = கடவுச்சொல்_சரியா("", மறையீடு);"#,
    )
    .unwrap();

    assert_eq!(vm.variables.get("சரியானது"), Some(&Value::Boolean(true)));
    assert_eq!(vm.variables.get("தவறானது"), Some(&Value::Boolean(false)));
    assert_eq!(vm.variables.get("காலியானது"), Some(&Value::Boolean(false)));
}

// The same password hashed twice must not produce the same text, or a stolen
// table would reveal which accounts share a password.
#[test]
fn hashing_the_same_password_twice_gives_different_hashes() {
    let vm = run(
        r#"அ = கடவுச்சொல்_மறை("same");
           ஆ = கடவுச்சொல்_மறை("same");
           வேறா = அ != ஆ;
           இரண்டும்_சரி = கடவுச்சொல்_சரியா("same", அ) மற்றும் கடவுச்சொல்_சரியா("same", ஆ);"#,
    )
    .unwrap();

    assert_eq!(vm.variables.get("வேறா"), Some(&Value::Boolean(true)));
    assert_eq!(vm.variables.get("இரண்டும்_சரி"), Some(&Value::Boolean(true)));
}

#[test]
fn a_token_round_trips_its_claims() {
    let vm = run_with_stdlib(
        r#"இறக்கு "jEcAZ.qmz";
           சுமை = ஜேசான்_ஆக்கு({sub: "user-1", roles: ["kaNakkar"]});
           சீட்டு = சீட்டு_ஆக்கு(சுமை, 3600);
           பகுதிகள் = நீளம்(பிரி(சீட்டு, "."));
           விளைவு = சீட்டு_சரிபார்(சீட்டு);
           ஏற்கப்பட்டது = சரியா(விளைவு);
           கூற்றுகள் = மதிப்பு(ஜேசான்_படி(மதிப்பு(விளைவு)));
           யார் = கூற்றுகள்["sub"];
           பங்கு_பெயர் = கூற்றுகள்["roles"][0];
           காலாவதி_உள்ளதா = கூற்றுகள்["exp"] > 0;"#,
    )
    .unwrap();

    assert_eq!(num(&vm, "பகுதிகள்"), dec(3));
    assert_eq!(vm.variables.get("ஏற்கப்பட்டது"), Some(&Value::Boolean(true)));
    assert_eq!(text(&vm, "யார்"), "user-1");
    assert_eq!(text(&vm, "பங்கு_பெயர்"), "kaNakkar");
    assert_eq!(
        vm.variables.get("காலாவதி_உள்ளதா"),
        Some(&Value::Boolean(true))
    );
}

// A rejected token is a தவறு rather than a runtime error, so turning a bad
// request into a 401 is ordinary control flow.
#[test]
fn a_tampered_or_expired_token_is_refused() {
    let vm = run_with_stdlib(
        r#"இறக்கு "jEcAZ.qmz";
           சுமை = ஜேசான்_ஆக்கு({sub: "user-1"});
           நல்லது = சீட்டு_ஆக்கு(சுமை, 3600);
           திருத்தப்பட்டது = தவறா(சீட்டு_சரிபார்(நல்லது & "x"));
           குப்பை = தவறா(சீட்டு_சரிபார்("not.a.token"));
           காலியானது = தவறா(சீட்டு_சரிபார்(""));
           // Past its expiry, and beyond the five seconds of clock skew the
           // verifier tolerates.
           காலாவதியானது = தவறா(சீட்டு_சரிபார்(சீட்டு_ஆக்கு(சுமை, 0 - 120)));"#,
    )
    .unwrap();

    for name in ["திருத்தப்பட்டது", "குப்பை", "காலியானது", "காலாவதியானது"] {
        assert_eq!(
            vm.variables.get(name),
            Some(&Value::Boolean(true)),
            "{} should have been refused",
            name
        );
    }
}

// The expiry is set by the host, not the caller: an expiry a handler could
// choose is an expiry an attacker could choose.
#[test]
fn a_token_payload_must_be_a_record() {
    let failure = run(r#"சீட்டு = சீட்டு_ஆக்கு("[1,2,3]", 60);"#);

    assert!(
        failure.is_err(),
        "an array payload should be refused, got {:?}",
        failure.map(|_| ())
    );
}

#[test]
fn stdlib_files_all_parse() {
    for file in ["col.qmz", "kaNiqam.qmz", "aNi.qmz", "paNam.qmz", "jEcAZ.qmz", "kuRiyAkkam.qmz"] {
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
    // Detached: not from the pool, so it is neither rolled back nor cached on
    // release — the log below would otherwise show a ROLLBACK no program wrote.
    vm.connections.insert(
        "SQLite".to_string(),
        etamil_compiler::db::pool::Lease::detached(Box::new(FakeDb {
            log: Arc::clone(&log),
            rows,
        })),
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

// --- Entities, currency and the year-end close -----------------------------

#[test]
fn entities_partition_one_ledger() {
    let vm = run_niluvy(
        r#"இறக்கு "kaNakkiyal/niRuvaZam.qmz";
           ப2 = [];
           ப2 = மதிப்பு(நிறுவனத்துடன்_பதிவிடு(ப2, பரிவர்த்தனை_ஆக்கு(
                 "A1", "2026-04-01", "அ நிறுவனம்",
                 [பற்று_வரிசை("1000", 1000), வரவு_வரிசை("4000", 1000)]), "AAA"));
           ப2 = மதிப்பு(நிறுவனத்துடன்_பதிவிடு(ப2, பரிவர்த்தனை_ஆக்கு(
                 "B1", "2026-04-01", "ஆ நிறுவனம்",
                 [பற்று_வரிசை("1000", 2500), வரவு_வரிசை("4000", 2500)]), "BBB"));

           அ_மட்டும் = நிறுவன_வடிகட்டு(ப2, "AAA");
           அ_வங்கி = கணக்கு_இருப்பு(அ_மட்டும், "1000", "சொத்து");
           ஆ_வங்கி = கணக்கு_இருப்பு(நிறுவன_வடிகட்டு(ப2, "BBB"), "1000", "சொத்து");
           ஒருங்கிணைந்த = கணக்கு_இருப்பு(ப2, "1000", "சொத்து");"#,
    )
    .unwrap();
    assert_eq!(num(&vm, "அ_வங்கி"), dec(1000));
    assert_eq!(num(&vm, "ஆ_வங்கி"), dec(2500));
    // not filtering consolidates the group
    assert_eq!(num(&vm, "ஒருங்கிணைந்த"), dec(3500));
}

#[test]
fn foreign_amounts_convert_at_a_rate() {
    let vm = run_niluvy(
        r#"இறக்கு "kaNakkiyal/niRuvaZam.qmz";
           அடிப்படை = அடிப்படைக்கு_மாற்று(1200, 83.45);
           வேறு = வேறுபாட்டுத்_தொகை(1200, 83.45, 84.10);"#,
    )
    .unwrap();
    assert_eq!(text(&vm, "அடிப்படை"), "100140");
    // 1200 * 0.65 = 780 gain
    assert_eq!(num(&vm, "வேறு"), dec(780));
}

#[test]
fn an_exchange_gain_posts_to_the_customer_and_gain_account() {
    let vm = run_niluvy(
        r#"இறக்கு "kaNakkiyal/niRuvaZam.qmz";
           த = மதிப்பு(அன்னிய_வேறுபாடு("FX1", "2026-04-30", "மாற்று வேறுபாடு",
                 "1100", "4000", "4000", 780));
           பற்றுத்_தொகை = மொத்த_பற்று(த.வரிசைகள்);
           வரவுத்_தொகை = மொத்த_வரவு(த.வரிசைகள்);"#,
    )
    .unwrap();
    assert_eq!(num(&vm, "பற்றுத்_தொகை"), dec(780));
    assert_eq!(num(&vm, "வரவுத்_தொகை"), dec(780));
}

#[test]
fn closing_a_year_empties_income_into_retained_earnings() {
    let vm = run_niluvy(
        r#"இறக்கு "kaNakkiyal/mutippu.qmz";
           கணக்குகள் = இணை(கணக்குகள்,
               மதிப்பு(கணக்கு_ஆக்கு("3100", "தேக்கிய வருவாய்", வகை_பங்கு(), "பங்கு")));

           ஆண்டு = இந்திய_ஆண்டு(2025);          // covers Jan and Mar invoices
           முன்பு = கால_வருமான_அறிக்கை(பேரேடு, கணக்குகள், ஆண்டு).நிகர_லாபம்;

           பேரேடு = மதிப்பு(ஆண்டை_முடி(பேரேடு, கணக்குகள், ஆண்டு, "3100", "CLOSE2025"));

           பின்பு = கால_வருமான_அறிக்கை(பேரேடு, கணக்குகள், ஆண்டு).நிகர_லாபம்;
           தேக்கியது = கணக்கு_இருப்பு(பேரேடு, "3100", "பங்கு");
           முடிந்ததா = முடிக்கப்பட்டதா(பேரேடு, கணக்குகள், ஆண்டு);"#,
    )
    .unwrap();
    // January 100,000 + March 50,000 of revenue, no expenses
    assert_eq!(num(&vm, "முன்பு"), dec(150000));
    assert_eq!(num(&vm, "பின்பு"), dec(0));
    assert_eq!(num(&vm, "தேக்கியது"), dec(150000));
    assert_eq!(vm.variables.get("முடிந்ததா"), Some(&Value::Boolean(true)));
}

#[test]
fn closing_an_empty_period_is_an_error() {
    let vm = run_niluvy(
        r#"இறக்கு "kaNakkiyal/mutippu.qmz";
           காலி = இந்திய_ஆண்டு(2030);
           விளைவு = முடிப்பு_பரிவர்த்தனை("X", பேரேடு, கணக்குகள், காலி, "3100");
           மறுக்கப்பட்டதா = தவறா(விளைவு);"#,
    )
    .unwrap();
    assert_eq!(vm.variables.get("மறுக்கப்பட்டதா"), Some(&Value::Boolean(true)));
}

// --- Type checking --------------------------------------------------------
// Type keywords used to be parsed and thrown away, so `சொல் x = 5;` was
// accepted. What the author declared is now held to — and only that: the
// checker states no rule the rest of the language does not follow.

#[test]
fn a_declared_type_is_enforced() {
    let failure = run(r#"ஈர்ம கொடியா = "ஆம்";"#);
    assert!(failure.is_err(), "a string is not a boolean");

    let message = failure.unwrap_err();
    assert!(message.contains("கொடியா"), "should name the variable: {}", message);
    assert!(message.contains("line 1"), "should give a position: {}", message);
}

#[test]
fn a_type_error_names_both_the_declared_and_the_actual_type() {
    let message = run("அணி பட்டியல் = 5;").unwrap_err();

    assert!(message.contains("அணி"), "should name the declared type: {}", message);
    assert!(message.contains("a number"), "should name what it got: {}", message);
}

// A later assignment is held to the declaration too, or the check would only
// cover the line that made it.
#[test]
fn a_later_assignment_is_held_to_the_declaration() {
    let failure = run(r#"ஈர்ம கொடியா = மெய்; கொடியா = [1, 2];"#);

    assert!(failure.is_err(), "an array is not a boolean");
}

// Every error is reported, not just the first: a wrong declaration is usually
// one of several, and stopping at the first makes fixing them a sequence of
// recompiles.
#[test]
fn every_type_error_is_reported() {
    let message = run("ஈர்ம அ = 1; அணி ஆ = 2; பொருள் இ = 3;").unwrap_err();

    assert!(message.contains("அ"), "missing the first: {}", message);
    assert!(message.contains("ஆ"), "missing the second: {}", message);
    assert!(message.contains("இ"), "missing the third: {}", message);
}

// The declarations the language's own examples use must keep working.
#[test]
fn correct_declarations_are_accepted() {
    let vm = run(
        r#"எண் வருவாய் = 950000;
           சொல் பெயர் = "ரவி";
           ஈர்ம செல்லுமா = மெய்;
           அணி வரிசைகள் = [1, 2, 3];
           பொருள் பதிவு = {qokY: 500};
           எண் எண்ணிக்கை;"#,
    )
    .unwrap();

    assert_eq!(num(&vm, "வருவாய்"), dec(950000));
    assert_eq!(text(&vm, "பெயர்"), "ரவி");
    assert_eq!(num(&vm, "எண்ணிக்கை"), dec(0));
}

// A number satisfies சொல், because every value renders as text, `&`
// concatenates whatever it is given, and உள்ளிடு hands back text that is
// routinely compared with numbers. Refusing this would be a rule the rest of
// the language does not follow.
#[test]
fn a_number_may_stand_where_text_was_declared() {
    let vm = run(r#"சொல் குறி = 1234; தேதி நாள் = "2026-04-01";"#).unwrap();

    assert_eq!(text(&vm, "குறி"), "1234");
    assert_eq!(text(&vm, "நாள்"), "2026-04-01");
}

// The checker makes no claim about what it cannot know, and silence there is
// the absence of a claim rather than approval.
#[test]
fn what_cannot_be_known_is_not_rejected() {
    let vm = run(
        r#"ஈர்ம விளைவு = சரியா(சரி(1));
           எண் நீ = நீளம்("வணக்கம்");
           பொருள் ப = {a: 1};
           எண் உள்ளது = ப["a"];
           ஈர்ம ஏதுமில்லை = இன்மை;"#,
    )
    .unwrap();

    assert_eq!(num(&vm, "நீ"), dec(5));
    assert_eq!(num(&vm, "உள்ளது"), dec(1));
}

// A parameter is a different variable from an outer name that happens to match,
// so a declaration outside must not be imposed on a function body.
#[test]
fn a_function_parameter_does_not_inherit_an_outer_declaration() {
    let vm = run(
        r#"ஈர்ம மதிப்பு = மெய்;
           செயல் இரட்டை(மதிப்பு) {
               மதிப்பு = மதிப்பு * 2;
               திரும்பு மதிப்பு;
           }
           விளைவு = இரட்டை(21);"#,
    )
    .unwrap();

    assert_eq!(num(&vm, "விளைவு"), dec(42));
}

// A loop variable takes whatever the collection holds, which the checker
// cannot see, so an earlier declaration of that name no longer describes it.
#[test]
fn a_loop_variable_drops_an_earlier_declaration() {
    let vm = run(
        r#"ஈர்ம உறுப்பு = மெய்;
           மொத்தம் = 0;
           ஒவ்வொரு உறுப்பு இல் [1, 2, 3] {
               மொத்தம் = மொத்தம் + உறுப்பு;
           }"#,
    )
    .unwrap();

    assert_eq!(num(&vm, "மொத்தம்"), dec(6));
}

// --- Parse errors carry a position ----------------------------------------
// Regression: the parser panicked with `Expected Semicolon` and nothing else.
// tokenize() returned a bare Vec<Token>, so there was no position to report
// even though the lexer had computed one — the biggest usability gap in the
// language for anyone learning it.

/// Parse only, so the error text can be inspected without running anything.
fn parse_error(source: &str) -> String {
    let tokens = etamil_compiler::lexer::tokenize(source).expect("should lex cleanly");
    let mut parser = Parser::new(tokens.iter());
    parser
        .parse()
        .map(|_| panic!("expected a parse error, but it parsed"))
        .unwrap_err()
        .to_string()
}

#[test]
fn a_missing_semicolon_is_reported_with_its_line_and_column() {
    let error = parse_error("x = 1;\ny = 2\nz = 3;");

    // The offending token is `z`, at the start of line 3.
    assert!(error.contains("வரி 3"), "no Tamil line number in: {}", error);
    assert!(error.contains("line 3"), "no English line number in: {}", error);
    assert!(error.contains("column 1"), "no column in: {}", error);
    assert!(error.contains("';'"), "does not say what was wanted: {}", error);
}

#[test]
fn a_parse_error_names_the_token_it_found() {
    let error = parse_error("x = 1 y;");

    assert!(error.contains("'y'"), "does not quote the token found: {}", error);
}

#[test]
fn running_off_the_end_is_reported_at_the_last_token() {
    let error = parse_error("செயல் கூட்டு(அ) {");

    assert!(
        error.contains("உள்ளீடு முடிந்தது") && error.contains("the input ended"),
        "should say the input ended: {}",
        error
    );
}

#[test]
fn a_parse_error_message_is_bilingual() {
    let error = parse_error("(x > 1) { y = 2; }");

    // Both spellings of the keyword it wanted, in one message.
    assert!(error.contains("எனில்"), "no Tamil keyword in: {}", error);
    assert!(error.contains("eZil"), "no romanized keyword in: {}", error);
}

#[test]
fn a_column_counts_written_letters_not_bytes() {
    // Each Tamil letter here is several bytes; the column must still read as
    // the position a person would point at.
    let error = parse_error("வருவாய் = 5 வரி;");

    assert!(error.contains("வரி 1"), "wrong line: {}", error);
    // "வருவாய் = 5 " is 12 written letters, so the offending token starts at 13.
    assert!(error.contains("column 13"), "wrong column: {}", error);
}

// --- String escapes -------------------------------------------------------
// Regression: literals were kept exactly as written, so "a\nb" was four
// characters and a double quote could not be put in a string at all — which
// is why nUlakam/jEcAZ.qmz could not have been written before this.

#[test]
fn escape_sequences_become_the_characters_they_name() {
    let vm = run(r#"a = "வரி\nவரவு"; n = nILam(a);"#).unwrap();

    assert_eq!(text(&vm, "a"), "வரி\nவரவு");
    // Counted as written letters: வ ரி + newline + வ ர வு.
    assert_eq!(num(&vm, "n"), dec(6));
}

#[test]
fn a_string_can_contain_a_double_quote() {
    let vm = run(r#"a = "{\"vakY\": \"varavu\"}";"#).unwrap();

    assert_eq!(text(&vm, "a"), r#"{"vakY": "varavu"}"#);
}

#[test]
fn a_backslash_can_be_written_literally() {
    let vm = run(r#"a = "one\\two"; n = nILam(a);"#).unwrap();

    assert_eq!(text(&vm, "a"), r"one\two");
    // o n e \ t w o — the pair collapses to one backslash.
    assert_eq!(num(&vm, "n"), dec(7));
}

// An unknown escape keeps both characters, so a path is not silently mangled.
#[test]
fn an_unrecognized_escape_is_left_alone() {
    let vm = run(r#"a = "C:\kaNakku";"#).unwrap();

    assert_eq!(text(&vm, "a"), r"C:\kaNakku");
}

#[test]
fn tabs_and_carriage_returns_are_recognized() {
    let vm = run(r#"a = "a\tb\rc";"#).unwrap();

    assert_eq!(text(&vm, "a"), "a\tb\rc");
}

// --- Responses (பதில்) -----------------------------------------------------
// பதில் writes to globals rather than the calling frame, because the server
// reads them back off the VM after the handler has returned.

#[test]
fn a_response_records_its_status_and_body() {
    let vm = run(r#"பதில் 201, "created";"#).unwrap();

    assert_eq!(num(&vm, "response_status"), dec(201));
    assert_eq!(text(&vm, "response_body"), "created");
}

#[test]
fn a_response_without_headers_records_none() {
    let vm = run(r#"பதில் 200, "ok";"#).unwrap();

    assert_eq!(vm.variables.get("response_headers"), Some(&Value::Null));
}

// Headers are an ordinary record, so a route can serve something other than
// the JSON the server assumes by default.
#[test]
fn a_response_carries_the_headers_it_was_given() {
    let vm = run(
        r#"பதில் 200, "<h1>வணக்கம்</h1>", {"Content-Type": "text/html", "X-Rows": "2"};"#,
    )
    .unwrap();

    match vm.variables.get("response_headers") {
        Some(Value::Map(fields)) => {
            assert_eq!(
                fields.get("Content-Type").map(|v| v.to_string()),
                Some("text/html".to_string())
            );
            assert_eq!(
                fields.get("X-Rows").map(|v| v.to_string()),
                Some("2".to_string())
            );
        }
        other => panic!("expected a record of headers, got {:?}", other),
    }
}

// Regression: headers used to be parsed and thrown away — the parser matched
// a third argument and substituted an empty list, so Content-Type could never
// be set and every response claimed to be JSON.
#[test]
fn response_headers_are_not_silently_discarded() {
    let vm = run(r#"பதில் 200, "x", {"Content-Type": "text/csv"};"#).unwrap();

    assert_ne!(vm.variables.get("response_headers"), Some(&Value::Null));
}

// பதில் is normally reached from inside a route's helper function, so it must
// not land in that function's locals.
#[test]
fn a_response_sent_from_a_function_is_still_visible() {
    let vm = run(
        r#"செயல் அனுப்பு(உரை_மதிப்பு) {
               பதில் 202, உரை_மதிப்பு;
               திரும்பு 0;
           }
           விளைவு = அனுப்பு("from a function");"#,
    )
    .unwrap();

    assert_eq!(num(&vm, "response_status"), dec(202));
    assert_eq!(text(&vm, "response_body"), "from a function");
}

// --- Text over a whole string ---------------------------------------------
// மாற்று, பிரி and ஒன்றிணை moved from nUlakam/col.qmz into the host because
// the eTamil versions re-segmented the string on every letter read. These
// assert the semantics the old versions had, so the move is not a change in
// what a program computes.

#[test]
fn replace_changes_every_occurrence() {
    let vm = run(r#"விளைவு = மாற்று("அ,ஆ,இ", ",", "-");"#).unwrap();

    assert_eq!(text(&vm, "விளைவு"), "அ-ஆ-இ");
}

#[test]
fn replace_with_nothing_to_find_returns_the_text_unchanged() {
    // An empty needle used to be a special case worth stating: the answer is
    // the original string, not a copy of the replacement between every letter.
    let vm = run(r#"விளைவு = மாற்று("வணக்கம்", "", "x");"#).unwrap();

    assert_eq!(text(&vm, "விளைவு"), "வணக்கம்");
}

#[test]
fn replace_will_not_cut_a_letter_in_half() {
    // கா is one written letter: க plus the vowel sign ா. Searching for the
    // sign alone finds nothing, because it never begins a letter.
    let vm = run(r#"விளைவு = மாற்று("கா", "ா", "x");"#).unwrap();

    assert_eq!(text(&vm, "விளைவு"), "கா");
}

#[test]
fn split_gives_one_piece_per_gap() {
    let vm = run(r#"பட்டி = பிரி("அ,ஆ,இ", ",");
                    முதல் = பட்டி[0];
                    கடை = பட்டி[2];
                    எண்ணிக்கை = நீளம்(பட்டி);"#)
    .unwrap();

    assert_eq!(num(&vm, "எண்ணிக்கை"), dec(3));
    assert_eq!(text(&vm, "முதல்"), "அ");
    assert_eq!(text(&vm, "கடை"), "இ");
}

#[test]
fn a_trailing_separator_leaves_an_empty_last_piece() {
    let vm = run(r#"பட்டி = பிரி("அ,", ",");
                    எண்ணிக்கை = நீளம்(பட்டி);
                    கடை = பட்டி[1];"#)
    .unwrap();

    assert_eq!(num(&vm, "எண்ணிக்கை"), dec(2));
    assert_eq!(text(&vm, "கடை"), "");
}

#[test]
fn splitting_on_nothing_returns_the_whole_string() {
    let vm = run(r#"பட்டி = பிரி("வணக்கம்", "");
                    எண்ணிக்கை = நீளம்(பட்டி);
                    முதல் = பட்டி[0];"#)
    .unwrap();

    assert_eq!(num(&vm, "எண்ணிக்கை"), dec(1));
    assert_eq!(text(&vm, "முதல்"), "வணக்கம்");
}

#[test]
fn split_and_join_are_inverses() {
    let vm = run(r#"விளைவு = ஒன்றிணை(பிரி("அ|ஆ|இ", "|"), "|");"#).unwrap();

    assert_eq!(text(&vm, "விளைவு"), "அ|ஆ|இ");
}

#[test]
fn join_needs_an_array_and_says_so() {
    // The documented signature is ஒன்றிணை(அணி, இணைப்பான்). A record has no
    // order to join in, so it is refused rather than answered arbitrarily.
    let failure = run(r#"விளைவு = ஒன்றிணை("சரம்", ",");"#).unwrap_err();

    assert!(failure.contains("அணி"), "unexpected message: {}", failure);
}

#[test]
fn a_document_sized_string_is_not_quadratic() {
    // The whole point of the move: a marker at the end of a document-sized
    // string, found and replaced. Reading letter by letter cost 14 seconds
    // over 8 KB, so at 256 KB this test would never have returned. No timing
    // assertion — the complexity class is what is being pinned, and a test
    // that finishes at all is the evidence.
    let vm = run(
        r#"பெரிது = "a";
           எண்ணி = 0;
           (எண்ணி < 18) சுற்று {
               பெரிது = பெரிது & பெரிது;
               எண்ணி = எண்ணி + 1;
           }
           குறியிட்ட = பெரிது & "MARK";
           விளைவு = மாற்று(குறியிட்ட, "MARK", "xy");
           அளவு = நீளம்(விளைவு);"#,
    )
    .unwrap();

    // 2^18 letters of padding, and the four-letter marker became two.
    assert_eq!(num(&vm, "அளவு"), dec(262_146));
}

// --- Whole-file write -----------------------------------------------------

#[test]
fn saving_a_file_writes_exactly_the_string() {
    // கோப்பு_எழுது appends a line and adds a newline, which a CSV row wants
    // and a document does not. The file this writes holds the string and
    // nothing after it.
    let path = std::env::temp_dir().join("etamil_kOppu_cEmi_test.txt");
    let program = format!(
        r#"விளைவு = கோப்பு_சேமி("{}", "அஆ");
           அளவு = மதிப்பு(விளைவு);"#,
        path.to_string_lossy().replace(std::path::MAIN_SEPARATOR, "/"),
    );
    let vm = run(&program).unwrap();

    let written = std::fs::read_to_string(&path).unwrap();
    let _ = std::fs::remove_file(&path);

    assert_eq!(written, "அஆ", "exact bytes, and no newline appended");
    // The answer is a result carrying the byte count — six, two Tamil letters
    // of three bytes each — so a failed write is a value a program can test
    // rather than a crash.
    assert_eq!(num(&vm, "அளவு"), dec(6));
}

// --- ODF and OOXML packages -----------------------------------------------
// .odt, .ods, .docx and .xlsx are all zip archives holding XML. A template is
// copied entry by entry with the text entry swapped, so that the pictures
// beside it — which are not text and could not survive being a சரம் — arrive
// unchanged, and the mimetype entry keeps the position and the compression
// the ODF format requires of it.

/// A package with the shape an .odt has: mimetype first and stored, an XML
/// entry, and a byte entry that is not valid UTF-8.
fn write_test_package(path: &std::path::Path, content: &str) {
    let file = std::fs::File::create(path).unwrap();
    let mut zip = zip::ZipWriter::new(file);

    let stored: zip::write::SimpleFileOptions = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Stored);
    zip.start_file("mimetype", stored).unwrap();
    std::io::Write::write_all(&mut zip, b"application/vnd.oasis.opendocument.text").unwrap();

    let deflated: zip::write::SimpleFileOptions = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    zip.start_file("content.xml", deflated).unwrap();
    std::io::Write::write_all(&mut zip, content.as_bytes()).unwrap();

    zip.start_file("Pictures/one.png", deflated).unwrap();
    // Deliberately not UTF-8: this is what must survive untouched.
    std::io::Write::write_all(&mut zip, &[0x89, 0x50, 0x4E, 0x47, 0xFF, 0xFE, 0x00, 0x01]).unwrap();

    zip.finish().unwrap();
}

#[test]
fn a_package_entry_reads_as_text() {
    let path = std::env::temp_dir().join("etamil_poqi_read.odt");
    write_test_package(&path, "<office:text>வணக்கம்</office:text>");

    let program = format!(
        r#"படித்தது = பொதி_படி("{}", "content.xml");
           உரைப்_பாடம் = மதிப்பு(படித்தது);"#,
        path.to_string_lossy().replace(std::path::MAIN_SEPARATOR, "/"),
    );
    let vm = run(&program).unwrap();
    let _ = std::fs::remove_file(&path);

    assert_eq!(text(&vm, "உரைப்_பாடம்"), "<office:text>வணக்கம்</office:text>");
}

#[test]
fn a_missing_entry_is_a_result_not_a_crash() {
    let path = std::env::temp_dir().join("etamil_poqi_missing.odt");
    write_test_package(&path, "<x/>");

    let program = format!(
        r#"படித்தது = பொதி_படி("{}", "styles.xml");
           தவறுதானா = தவறா(படித்தது);"#,
        path.to_string_lossy().replace(std::path::MAIN_SEPARATOR, "/"),
    );
    let vm = run(&program).unwrap();
    let _ = std::fs::remove_file(&path);

    assert_eq!(vm.variables.get("தவறுதானா"), Some(&Value::Boolean(true)));
}

#[test]
fn rewriting_a_package_leaves_the_other_entries_alone() {
    let source = std::env::temp_dir().join("etamil_poqi_source.odt");
    let target = std::env::temp_dir().join("etamil_poqi_target.odt");
    write_test_package(&source, "<office:text>{{ பெயர் }}</office:text>");

    let program = format!(
        r#"எழுதியது = பொதி_மாற்று("{}", "{}", {{"content.xml": "<office:text>ராஜா</office:text>"}});
           எத்தனை = மதிப்பு(எழுதியது);"#,
        source.to_string_lossy().replace(std::path::MAIN_SEPARATOR, "/"),
        target.to_string_lossy().replace(std::path::MAIN_SEPARATOR, "/"),
    );
    let vm = run(&program).unwrap();

    assert_eq!(num(&vm, "எத்தனை"), dec(3));

    let written = std::fs::File::open(&target).unwrap();
    let mut archive = zip::ZipArchive::new(written).unwrap();

    // Order, and the two rules ODF imposes on the first entry.
    let names: Vec<String> = (0..archive.len())
        .map(|i| archive.by_index(i).unwrap().name().to_string())
        .collect();
    assert_eq!(names, vec!["mimetype", "content.xml", "Pictures/one.png"]);
    assert_eq!(
        archive.by_name("mimetype").unwrap().compression(),
        zip::CompressionMethod::Stored,
        "mimetype must stay uncompressed"
    );

    // The replacement landed.
    let mut got = String::new();
    std::io::Read::read_to_string(&mut archive.by_name("content.xml").unwrap(), &mut got).unwrap();
    assert_eq!(got, "<office:text>ராஜா</office:text>");

    // And the bytes that are not text came through untouched.
    let mut picture = Vec::new();
    std::io::Read::read_to_end(&mut archive.by_name("Pictures/one.png").unwrap(), &mut picture)
        .unwrap();
    assert_eq!(picture, vec![0x89, 0x50, 0x4E, 0x47, 0xFF, 0xFE, 0x00, 0x01]);

    let _ = std::fs::remove_file(&source);
    let _ = std::fs::remove_file(&target);
}

#[test]
fn replacing_an_entry_that_is_not_there_is_refused() {
    // Writing an unchanged document would be the worst way to report a typo.
    let source = std::env::temp_dir().join("etamil_poqi_typo_source.odt");
    let target = std::env::temp_dir().join("etamil_poqi_typo_target.odt");
    write_test_package(&source, "<x/>");

    let program = format!(
        r#"எழுதியது = பொதி_மாற்று("{}", "{}", {{"contnet.xml": "<x/>"}});
           தவறுதானா = தவறா(எழுதியது);"#,
        source.to_string_lossy().replace(std::path::MAIN_SEPARATOR, "/"),
        target.to_string_lossy().replace(std::path::MAIN_SEPARATOR, "/"),
    );
    let vm = run(&program).unwrap();
    let _ = std::fs::remove_file(&source);
    let _ = std::fs::remove_file(&target);

    assert_eq!(vm.variables.get("தவறுதானா"), Some(&Value::Boolean(true)));
}

// --- Running another program ----------------------------------------------

#[test]
fn nothing_runs_unless_it_is_allowed() {
    // Deny by default. This asserts the refusal without setting the variable,
    // because setting an environment variable is process-wide and the tests
    // share a process.
    let vm = run(r#"விடை = கட்டளை_ஓட்டு("definitely-not-a-real-program", [], 5);
                    தவறுதானா = தவறா(விடை);"#)
    .unwrap();

    assert_eq!(vm.variables.get("தவறுதானா"), Some(&Value::Boolean(true)));
}

#[test]
fn command_arguments_must_be_a_list() {
    // A string of arguments would have to be split by something, and the only
    // thing that splits arguments correctly is not doing it at all.
    let failure = run(r#"விடை = கட்டளை_ஓட்டு("ls", "-l", 5);"#).unwrap_err();

    assert!(failure.contains("அணி"), "unexpected message: {}", failure);
}

// --- Bilingual equivalence ------------------------------------------------

#[test]
fn tamil_and_romanized_forms_compute_the_same_answers() {
    let tamil = run("எண் வருவாய் = 950000; வரி = 20%; விளைவு = (வருவாய் - 800000) * வரி;")
        .unwrap();
    let roman = run("eN varuvAy = 950000; vari = 20%; viLYvu = (varuvAy - 800000) * vari;")
        .unwrap();

    // The two spellings are the same *program* — same tokens, same bytecode,
    // same arithmetic. What differs now is only the name each result is filed
    // under, because a name is what the author wrote.
    assert_eq!(num(&tamil, "விளைவு"), num(&roman, "viLYvu"));
    assert_eq!(num(&tamil, "விளைவு"), dec(30000));
}
