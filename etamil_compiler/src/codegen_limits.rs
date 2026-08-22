//! What the LLVM backend must refuse, decided by reading the program.
//!
//! `codegen.rs` already carries the right principle in a comment on its own
//! `unsupported` field: emitting IR that evaluates an expression as `0` would
//! make the compiled program quietly disagree with the same source run on the
//! VM, so the caller must refuse to emit. The README says the same — anything
//! unsupported is "rejected rather than emitted as incorrect IR".
//!
//! One thing escaped that rule, and it was the largest one.
//!
//! ## The backend used to compute in `f64`
//!
//! Every number in eTamil is a fixed-point decimal, and the whole reason for
//! that is the language's opening claim: `0.1 + 0.2` is exactly `0.3`, and a
//! ledger balances to the paisa. Compiled through `--llvm` it was not, and
//! nothing said so — the IR was emitted, it ran, and it answered
//! `0.30000000000000004`. That is not a missing feature. It is the one failure
//! mode this project refuses everywhere else: a wrong answer with no warning.
//!
//! ## What it computes in now
//!
//! `i64`. Every number that reaches the backend is a whole number — this
//! module is what makes that true — so an integer register is a better fit
//! than a double in every respect: exact to 2^63 rather than 2^53, and `sdiv`
//! divides exactly where `fdiv` was out by one in the last place, which for
//! money is a paisa.
//!
//! So the refusals here are narrower than they were:
//!
//! * A **fractional literal** is still refused. `i64` cannot hold `0.1`, and
//!   `18%` is `0.18`, so every program that writes a rate as a percentage is
//!   caught — which is the point, since that is exactly the program that must
//!   not be silently rounded.
//! * A **bare division** is still refused, because its result has a fractional
//!   part and there is nowhere to put it.
//! * A division **under `தரை` or `மேல்`** is now accepted, and this is the
//!   change that pays for the rest. `தரை(அ / ஆ)` is integer floor division and
//!   `மேல்(அ / ஆ)` is integer ceiling division; both are exact. That is the
//!   one operation `nUlakam/kAcu.qmz` needs in order to hold money as whole
//!   paise — a percentage rounded once, and a split that distributes the odd
//!   paise instead of losing them.
//! * A **whole number too large for `i64`** is refused, which is new. A
//!   `Decimal` holds more than an `i64` does, and silently wrapping would be
//!   the same crime in a different register.
//!
//! ## Why this is a separate module
//!
//! It walks the AST and nothing else — no `llvm-sys`, no `unsafe`, no LLVM
//! installation. So it compiles and is tested on any machine, including the
//! ones that cannot build the backend it guards.

use crate::parser::{Expr, Stmt};

/// The builtins the LLVM backend can reach, because on whole numbers they are
/// either the identity or an integer division.
///
/// Everything else in the fifty-nine builtins takes or returns a string, an
/// array, a record or a result, and those have no representation in the
/// emitted IR at all. See `docs/llvm-backend-gaps.md`, gaps 2 and 3.
///
/// This table is the one place the rule lives. `codegen_limits` reads it to
/// decide what to allow and `codegen.rs` reads it to decide what to emit, so
/// the guard and the backend cannot drift apart into a program that is
/// accepted here and then refused there.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WholeNumberBuiltin {
    /// `தரை(n)` — floor. Over a division, an exact integer floor division.
    Floor,
    /// `மேல்(n)` — ceiling. Over a division, the same rounded the other way.
    Ceil,
    /// `வட்டமிடு(n, இடங்கள்)` — round to n places, which over a whole number
    /// is the number itself, at any number of places.
    Round,
}

/// Which of the three a name is, under any of its three spellings.
pub fn whole_number_builtin(name: &str) -> Option<WholeNumberBuiltin> {
    match name {
        "தரை" | "qarY" | "_floor" => Some(WholeNumberBuiltin::Floor),
        "மேல்" | "mEl" | "_ceil" => Some(WholeNumberBuiltin::Ceil),
        "வட்டமிடு" | "vattamitu" | "_round" => Some(WholeNumberBuiltin::Round),
        _ => None,
    }
}

/// Everything in this program that the LLVM backend would get wrong or drop.
///
/// Empty means the program is safe to emit. Non-empty means refuse, and the
/// strings are what to tell the author.
pub fn refusals(program: &[Stmt]) -> Vec<String> {
    let mut walk = Walk {
        authored: Vec::new(),
        found: Vec::new(),
    };
    walk.collect_authored(program);
    for statement in program {
        walk.stmt(statement);
    }
    let mut found = walk.found;
    found.sort();
    found.dedup();
    found
}

const DECIMAL_LITERAL: &str = "பதின்ம எண் — இந்த பின்தளம் முழு எண்களில் கணக்கிடும்  \
                               (a decimal number: this backend computes in i64, \
                               so it can hold 205 paise but not 2.05 rupees)";

/// Shared with `codegen.rs`, which refuses a bare division again on its own
/// account. Two spellings of the one cause showed up as two entries in the
/// parity run's ranked list, 36 and 35, so the most common gap in the corpus
/// read as two middling ones. The caller de-duplicates identical strings.
pub const DIVISION: &str = "வகுத்தல் — தரை() அல்லது மேல்() இல்லாமல்  \
                        (a bare division: whole-number division is exact here, \
                        but only under தரை() or மேல் — on its own its result \
                        has a fractional part and i64 has nowhere to put it)";

const TOO_LARGE: &str = "i64 இல் அடங்காத எண்  \
                         (a whole number too large for the i64 this backend \
                         computes in — the VM's decimal type holds more)";

struct Walk {
    /// Functions the program itself defines. One of these shadows a builtin of
    /// the same name, so a program that writes its own `தரை` gets none of the
    /// exactness guarantees the real one carries.
    authored: Vec<String>,
    found: Vec<String>,
}

impl Walk {
    fn note(&mut self, reason: &str) {
        self.found.push(reason.to_string());
    }

    /// Every function name the program defines, at any depth. A `செயல்` is
    /// normally top level, but nesting is not forbidden and a shadowing
    /// definition found late is still a shadowing definition.
    fn collect_authored(&mut self, program: &[Stmt]) {
        for statement in program {
            match statement {
                Stmt::FunctionDef { name, body, .. } => {
                    self.authored.push(name.clone());
                    self.collect_authored(body);
                }
                Stmt::If {
                    then_branch,
                    else_branch,
                    ..
                } => {
                    self.collect_authored(then_branch);
                    if let Some(otherwise) = else_branch {
                        self.collect_authored(otherwise);
                    }
                }
                Stmt::Loop { body, .. } | Stmt::ForEach { body, .. } => {
                    self.collect_authored(body)
                }
                _ => {}
            }
        }
    }

    fn stmt(&mut self, statement: &Stmt) {
        match statement {
            Stmt::Assign { value, .. } => self.expr(value),
            Stmt::Return(Some(value)) => self.expr(value),
            Stmt::Return(None) => {}
            Stmt::SetIndex { index, value, .. } => {
                self.expr(index);
                self.expr(value);
            }
            Stmt::SetField { value, .. } => self.expr(value),
            Stmt::Expression(value) | Stmt::Print(value) | Stmt::Input(value) => self.expr(value),
            Stmt::FunctionDef { body, .. } => {
                for inner in body {
                    self.stmt(inner);
                }
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.expr(condition);
                for inner in then_branch {
                    self.stmt(inner);
                }
                if let Some(otherwise) = else_branch {
                    for inner in otherwise {
                        self.stmt(inner);
                    }
                }
            }
            Stmt::Loop { condition, body } => {
                self.expr(condition);
                for inner in body {
                    self.stmt(inner);
                }
            }
            Stmt::ForEach {
                collection, body, ..
            } => {
                self.expr(collection);
                for inner in body {
                    self.stmt(inner);
                }
            }
            // Everything else — file, database, HTTP, scheduling — carries
            // expressions this walk does not reach, and codegen refuses those
            // statements on its own account. Reaching into them here would
            // report the same program twice for two different reasons.
            _ => {}
        }
    }

    fn expr(&mut self, expression: &Expr) {
        match expression {
            Expr::Number(number) => {
                // A literal with a fractional part has no `i64` form, and it
                // is the clearest signal that a program is about money — which
                // is why the paise convention in nUlakam/kAcu.qmz exists.
                if !number.fract().is_zero() {
                    self.note(DECIMAL_LITERAL);
                } else if rust_decimal::prelude::ToPrimitive::to_i64(number).is_none() {
                    self.note(TOO_LARGE);
                }
            }
            Expr::BinaryOp { op, left, right } => {
                // Addition, subtraction and multiplication of whole numbers
                // are exact in an i64 up to 2^63. Division is exact too — but
                // only its floor or its ceiling, because the true quotient of
                // two whole numbers usually is not one. So a division reached
                // here, rather than through the `Call` arm below, is a division
                // whose fractional part the program expects to keep.
                //
                // There is no infix modulo to catch: `%` is the postfix
                // percentage operator. nUlakam's மீதி computes a remainder as
                // `அ - தரை(அ / ஆ) * ஆ`, which is exact under this rule and no
                // longer needs one of its own.
                if op == "/" {
                    self.note(DIVISION);
                }
                self.expr(left);
                self.expr(right);
            }
            Expr::Comparison { left, right, .. } | Expr::Concat { left, right } => {
                self.expr(left);
                self.expr(right);
            }
            Expr::Logical { left, right, .. } => {
                self.expr(left);
                self.expr(right);
            }
            Expr::Not(inner) | Expr::Try(inner) => self.expr(inner),
            Expr::Call { name, args } => {
                // The exception that makes money work. `தரை(அ / ஆ)` and
                // `மேல்(அ / ஆ)` are compiled as integer division with a
                // sign correction, which is exactly floor and exactly ceiling
                // — so the division inside them is not a reason to refuse.
                //
                // Only the division directly under the call. `தரை(1 + அ / ஆ)`
                // is refused, because rounding the sum is not the same as
                // rounding the quotient, and neither is `தரை(அ / ஆ * இ)`.
                let rounds_exactly = matches!(
                    whole_number_builtin(name),
                    Some(WholeNumberBuiltin::Floor) | Some(WholeNumberBuiltin::Ceil)
                ) && !self.authored.iter().any(|authored| authored == name);

                if rounds_exactly && args.len() == 1 {
                    if let Expr::BinaryOp { op, left, right } = &args[0] {
                        if op == "/" {
                            self.expr(left);
                            self.expr(right);
                            return;
                        }
                    }
                }

                for argument in args {
                    self.expr(argument);
                }
            }
            Expr::ArrayLiteral(items) => {
                for item in items {
                    self.expr(item);
                }
            }
            Expr::RecordLiteral(fields) => {
                for (_, value) in fields {
                    self.expr(value);
                }
            }
            Expr::Index { base, index } => {
                self.expr(base);
                self.expr(index);
            }
            Expr::Field { base, .. } => self.expr(base),
            Expr::String(_) | Expr::Boolean(_) | Expr::Null | Expr::Variable(_) => {}
        }
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
        // An i64 holds every whole number up to 2^63 exactly, so this compiles
        // to IR that agrees with the VM.
        assert!(!refuses("அ = 1 + 2;"));
        assert!(!refuses("அ = 1000000 * 3;"));
        assert!(!refuses("அ = 10 - 4;"));
    }

    #[test]
    fn a_decimal_literal_is_refused() {
        // The language's opening claim is that 0.1 + 0.2 is exactly 0.3. There
        // is no i64 that holds either operand.
        let why = refusals_for("அ = 0.1 + 0.2;");

        assert_eq!(why.len(), 1, "one reason, not one per literal: {:?}", why);
        assert!(why[0].contains("i64"), "it should say why: {}", why[0]);
    }

    #[test]
    fn a_percentage_is_a_decimal() {
        // 18% is 0.18, so every tax program is caught by this — which is the
        // point, since a tax program is exactly what must not be rounded
        // without saying so.
        assert!(refuses("வ = 18%;"));
    }

    #[test]
    fn a_bare_division_is_refused() {
        // A third of a rupee is not a whole number of anything, and there is
        // nowhere in an i64 to keep the part that is left over.
        assert!(refuses("அ = 100 / 3;"));
    }

    #[test]
    fn a_whole_number_too_large_for_i64_is_refused() {
        // The VM's Decimal holds 96 bits of mantissa; an i64 holds 63. Wrapping
        // silently would be the same wrong-answer-with-no-warning in a
        // different register.
        let why = refusals_for("அ = 99999999999999999999;");

        assert_eq!(why.len(), 1, "{:?}", why);
        assert!(why[0].contains("i64"), "it should say why: {}", why[0]);
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

    // --- the exception that makes whole-number money work ------------------
    //
    // `தரை(அ / ஆ)` compiles to integer division with a sign correction, which
    // is exactly floor. These pin how far that reaches, because the boundary
    // is narrow on purpose: it is the division directly under the call, and
    // nothing else.

    #[test]
    fn a_division_under_floor_or_ceiling_is_exact_and_allowed() {
        assert!(!refuses("ரூபாய் = தரை(205 / 100);"));
        assert!(!refuses("பங்கு = தரை(1000 / 3);"));
        assert!(!refuses("மேற்பட்ட = மேல்(1000 / 3);"));
    }

    #[test]
    fn the_two_divisions_the_money_module_needs_are_allowed() {
        // Half-up rounding of a percentage, in whole paise, from
        // nUlakam/kAcu.qmz — the nested தரை is the `கீழ்/2` that does the
        // rounding, and both divisions are exact.
        assert!(!refuses("வரி = தரை((205 * 18 + தரை(100 / 2)) / 100);"));
        // And apportioning a total by weight, which divides a product.
        assert!(!refuses("பங்கு = தரை(1000 * 3 / 7);"));
    }

    #[test]
    fn a_remainder_written_the_usual_way_is_now_exact() {
        // nUlakam's மீதி. Under f64 this was refused for the division it is
        // built from; over i64 the division is exact and so is the remainder.
        assert!(!refuses(
            "செயல் மீதி(அ, ஆ) { திரும்பு அ - தரை(அ / ஆ) * ஆ; }"
        ));
    }

    #[test]
    fn only_the_division_directly_under_the_call_is_excused() {
        // Rounding a sum is not the same as rounding the quotient inside it,
        // and rounding a product is not the same as rounding its factor. A
        // division that is not the call's whole argument keeps its fractional
        // part on the way out, so it is still refused.
        assert!(refuses("அ = தரை(1 + 100 / 3);"));
        assert!(refuses("அ = தரை(100 / 3 * 3);"));
        assert!(refuses("அ = தரை(100 / 3 / 2);"));
    }

    #[test]
    fn rounding_to_places_excuses_nothing() {
        // வட்டமிடு is the identity on a whole number, which is why the backend
        // can compile it — but it rounds half away from zero, not down, so a
        // division under it is not the floor division that is exact here.
        assert!(refuses("அ = வட்டமிடு(100 / 3, 0);"));
    }

    #[test]
    fn a_program_that_defines_its_own_floor_gets_no_exception() {
        // The builtin தரை is exact. A தரை the program wrote is whatever the
        // program wrote, so the division inside the call is just a division.
        assert!(refuses(
            "செயல் தரை(அ) { திரும்பு அ; } ஆ = தரை(205 / 100);"
        ));
    }

    #[test]
    fn each_reason_is_reported_once_however_often_it_occurs() {
        // A hundred decimal literals is one problem, not a hundred, and a list
        // of a hundred identical lines is a list nobody reads.
        let why = refusals_for("அ = 0.1; ஆ = 0.2; இ = 0.3; ஈ = 1 / 2; உ = 2 / 3;");

        assert_eq!(
            why.len(),
            2,
            "one for the decimals, one for division: {:?}",
            why
        );
    }
}
