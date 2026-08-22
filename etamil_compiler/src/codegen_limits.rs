//! What the LLVM backend must refuse, decided by reading the program.
//!
//! `codegen.rs` already carries the right principle in a comment on its own
//! `unsupported` field: emitting IR that evaluates an expression as `0.0` would
//! make the compiled program quietly disagree with the same source run on the
//! VM, so the caller must refuse to emit. The README says the same — anything
//! unsupported is "rejected rather than emitted as incorrect IR".
//!
//! One thing escaped that rule, and it is the largest one.
//!
//! ## The backend computes in `f64`
//!
//! `codegen.rs` holds twenty-three references to `LLVMDoubleType` and not one
//! to `Decimal`. Every number in eTamil is a fixed-point decimal, and the whole
//! reason for that is the language's opening claim: `0.1 + 0.2` is exactly
//! `0.3`, and a ledger balances to the paisa. Compiled through `--llvm` it is
//! not, and nothing says so — the IR is emitted, it runs, and it answers
//! `0.30000000000000004`.
//!
//! That is not a missing feature. It is the one failure mode this project
//! refuses everywhere else: a wrong answer with no warning.
//!
//! Integers are a different matter. A double holds every integer up to 2^53
//! exactly, so a program that only ever deals in whole numbers compiles to IR
//! that agrees with the VM. That is why this refuses *decimal* arithmetic
//! rather than all arithmetic: refusing everything would take away the numeric
//! functions the backend does handle correctly.
//!
//! ## Why this is a separate module
//!
//! It walks the AST and nothing else — no `llvm-sys`, no `unsafe`, no LLVM
//! installation. So it compiles and is tested on any machine, including the
//! ones that cannot build the backend it guards.

use crate::parser::{Expr, Stmt};

/// Everything in this program that the LLVM backend would get wrong or drop.
///
/// Empty means the program is safe to emit. Non-empty means refuse, and the
/// strings are what to tell the author.
pub fn refusals(program: &[Stmt]) -> Vec<String> {
    let mut found = Vec::new();
    for statement in program {
        walk_stmt(statement, &mut found);
    }
    found.sort();
    found.dedup();
    found
}

fn note(found: &mut Vec<String>, reason: &str) {
    found.push(reason.to_string());
}

const DECIMAL_LITERAL: &str = "பதின்ம எண் — இந்த பின்தளம் f64 இல் கணக்கிடும்  \
                               (a decimal number: this backend computes in f64, \
                               so 0.1 + 0.2 would not be exactly 0.3)";

const DIVISION: &str = "வகுத்தல் — f64 இல் துல்லியம் இழக்கும்  \
                        (division: an exact result needs decimal arithmetic, \
                        which this backend does not have)";

fn walk_stmt(statement: &Stmt, found: &mut Vec<String>) {
    match statement {
        Stmt::Assign { value, .. } => walk_expr(value, found),
        Stmt::Return(Some(value)) => walk_expr(value, found),
        Stmt::Return(None) => {}
        Stmt::SetIndex { index, value, .. } => {
            walk_expr(index, found);
            walk_expr(value, found);
        }
        Stmt::SetField { value, .. } => walk_expr(value, found),
        Stmt::Expression(value) | Stmt::Print(value) | Stmt::Input(value) => {
            walk_expr(value, found)
        }
        Stmt::FunctionDef { body, .. } => {
            for inner in body {
                walk_stmt(inner, found);
            }
        }
        Stmt::If {
            condition,
            then_branch,
            else_branch,
        } => {
            walk_expr(condition, found);
            for inner in then_branch {
                walk_stmt(inner, found);
            }
            if let Some(otherwise) = else_branch {
                for inner in otherwise {
                    walk_stmt(inner, found);
                }
            }
        }
        Stmt::Loop { condition, body } => {
            walk_expr(condition, found);
            for inner in body {
                walk_stmt(inner, found);
            }
        }
        Stmt::ForEach {
            collection, body, ..
        } => {
            walk_expr(collection, found);
            for inner in body {
                walk_stmt(inner, found);
            }
        }
        // Everything else — file, database, HTTP, scheduling — carries
        // expressions this walk does not reach, and codegen refuses those
        // statements on its own account. Reaching into them here would report
        // the same program twice for two different reasons.
        _ => {}
    }
}

fn walk_expr(expression: &Expr, found: &mut Vec<String>) {
    match expression {
        Expr::Number(number) => {
            // A literal with a fractional part cannot survive f64 intact, and
            // it is the clearest signal that a program is about money.
            if !number.fract().is_zero() {
                note(found, DECIMAL_LITERAL);
            }
        }
        Expr::BinaryOp { op, left, right } => {
            // Addition, subtraction and multiplication of whole numbers stay
            // exact in a double up to 2^53. Division does not: a third of a
            // rupee has no exact binary form, and the error compounds through
            // everything downstream of it.
            //
            // Division only. There is no infix modulo to catch — `%` is the
            // postfix percentage operator — and nUlakam's மீதி computes a
            // remainder *through* division, so it is caught by this same rule
            // rather than needing one of its own.
            if op == "/" {
                note(found, DIVISION);
            }
            walk_expr(left, found);
            walk_expr(right, found);
        }
        Expr::Comparison { left, right, .. } | Expr::Concat { left, right } => {
            walk_expr(left, found);
            walk_expr(right, found);
        }
        Expr::Logical { left, right, .. } => {
            walk_expr(left, found);
            walk_expr(right, found);
        }
        Expr::Not(inner) | Expr::Try(inner) => walk_expr(inner, found),
        Expr::Call { args, .. } => {
            for argument in args {
                walk_expr(argument, found);
            }
        }
        Expr::ArrayLiteral(items) => {
            for item in items {
                walk_expr(item, found);
            }
        }
        Expr::RecordLiteral(fields) => {
            for (_, value) in fields {
                walk_expr(value, found);
            }
        }
        Expr::Index { base, index } => {
            walk_expr(base, found);
            walk_expr(index, found);
        }
        Expr::Field { base, .. } => walk_expr(base, found),
        Expr::String(_) | Expr::Boolean(_) | Expr::Null | Expr::Variable(_) => {}
    }
}

#[cfg(test)]
mod tests {
    use crate::lexer;
    use crate::parser::Parser;

    fn refusals_for(source: &str) -> Vec<String> {
        let tokens = lexer::tokenize(source).expect("the fixture should lex");
        let ast = Parser::new(tokens.iter()).parse().expect("and parse");
        super::refusals(&ast)
    }

    fn refuses(source: &str) -> bool {
        !refusals_for(source).is_empty()
    }

    #[test]
    fn whole_number_arithmetic_is_allowed() {
        // A double holds every integer up to 2^53 exactly, so this compiles to
        // IR that agrees with the VM. Refusing it would take away the numeric
        // functions the backend does handle.
        assert!(!refuses("அ = 1 + 2;"));
        assert!(!refuses("அ = 1000000 * 3;"));
        assert!(!refuses("அ = 10 - 4;"));
    }

    #[test]
    fn a_decimal_literal_is_refused() {
        // The language's opening claim is that 0.1 + 0.2 is exactly 0.3. Under
        // f64 it is not, and nothing warned.
        let why = refusals_for("அ = 0.1 + 0.2;");

        assert_eq!(why.len(), 1, "one reason, not one per literal: {:?}", why);
        assert!(why[0].contains("f64"), "it should say why: {}", why[0]);
    }

    #[test]
    fn a_percentage_is_a_decimal() {
        // 18% is 0.18, so every tax program is caught by this — which is the
        // point, since a tax program is exactly what must not run on f64.
        assert!(refuses("வ = 18%;"));
    }

    #[test]
    fn division_is_refused_even_between_whole_numbers() {
        // A third of a rupee has no exact binary form, and the error compounds
        // through everything downstream of it.
        assert!(refuses("அ = 100 / 3;"));
    }

    #[test]
    fn a_remainder_is_caught_through_the_division_it_is_built_from() {
        // There is no infix modulo in this language — `%` is postfix
        // percentage. nUlakam's மீதி computes a remainder by dividing, so it
        // needs no rule of its own.
        assert!(refuses("செயல் மீதி(அ, ஆ) { திரும்பு அ - தரை(அ / ஆ) * ஆ; }"));
    }

    #[test]
    fn a_decimal_hidden_inside_a_function_is_still_found() {
        assert!(refuses(
            "செயல் வரி(த) { திரும்பு த * 0.18; } அ = வரி(100);"
        ));
    }

    #[test]
    fn a_decimal_hidden_inside_a_branch_or_a_loop_is_still_found() {
        assert!(refuses("(1 == 1) எனில் { அ = 2.5; }"));
        assert!(refuses("(0 == 1) எனில் { அ = 1; } இன்றேல் { அ = 2.5; }"));
        assert!(refuses("எ = 0; (எ < 3) சுற்று { அ = 1.5; எ = எ + 1; }"));
    }

    #[test]
    fn a_decimal_inside_a_collection_is_still_found() {
        assert!(refuses("அ = [1, 2.5];"));
        assert!(refuses("அ = {\"க\": 0.5};"));
    }

    #[test]
    fn a_decimal_passed_to_a_call_is_still_found() {
        assert!(refuses("அ = நீளம்(1.5);"));
    }

    #[test]
    fn a_program_with_no_arithmetic_at_all_is_allowed() {
        assert!(!refuses("அ = \"வணக்கம்\";"));
        assert!(!refuses("அ = மெய்;"));
    }

    #[test]
    fn each_reason_is_reported_once_however_often_it_occurs() {
        // A hundred decimal literals is one problem, not a hundred, and a list
        // of a hundred identical lines is a list nobody reads.
        let why = refusals_for("அ = 0.1; ஆ = 0.2; இ = 0.3; ஈ = 1 / 2; உ = 2 / 3;");

        assert_eq!(why.len(), 2, "one for the decimals, one for division: {:?}", why);
    }
}
