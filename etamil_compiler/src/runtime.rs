//! The runtime the LLVM backend's emitted IR calls into.
//!
//! ## Why this exists
//!
//! The backend used to compute in `f64`, then in `i64`. Both are the same
//! bargain in different clothes: the IR holds the value directly, so the
//! language shrinks to whatever fits in a register. Numbers fit. Strings,
//! arrays, records, results and `இன்மை` do not, and neither does a decimal —
//! `1 / 3` is `0.3333333333333333333333333333` on the VM and there is no
//! register that holds that.
//!
//! So the IR stops holding values. Every eTamil value is an `i64` **handle**
//! into an arena that lives here, and every operation on one is a call into
//! this module. That is the ordinary way to compile a dynamically typed
//! language, and it buys three things that matter more than the call overhead:
//!
//! 1. **Decimals are exact, because this is the same `rust_decimal` the VM
//!    uses.** Not a fixed two-place approximation — the actual type. `0.1 + 0.2`
//!    is `0.3` and `1 / 3` prints all twenty-eight digits, in both backends,
//!    for the same reason rather than by coincidence.
//! 2. **Formatting cannot drift**, because printing calls `Value::to_string`,
//!    the function the VM prints through. Trailing zeros are trimmed in one
//!    place, not two.
//! 3. **All fifty-nine builtins work at once**, because `etamil_call` goes
//!    through `VM::invoke_builtin` — the dispatch the bytecode interpreter
//!    uses. `நீளம்` is not reimplemented here, so it cannot disagree.
//!
//! ## The cost
//!
//! `output.ll` is no longer self-contained. It has to be linked against this:
//!
//! ```text
//! clang output.ll -o prog -L target/release -letamil_compiler \
//!       -Wl,-rpath,target/release
//! ```
//!
//! `Cargo.toml` already builds the `cdylib`, so nothing new is produced; these
//! functions are simply exported from it.
//!
//! ## Handles
//!
//! A handle is an index into a thread-local arena. Handle 0 is always `இன்மை`.
//! Nothing is ever freed: a compiled program runs and exits, and an arena that
//! only grows cannot dangle, cannot double-free, and needs no reference counts
//! in the emitted IR. A long-running program compiled this way would grow
//! without bound, which is the honest limitation of the approach and the reason
//! the VM remains the way to run a server.
//!
//! Every function here is `extern "C"` and takes and returns handles or plain
//! integers, so `codegen.rs` needs no knowledge of `Value`'s layout.

use std::cell::RefCell;
use std::ffi::CStr;
use std::os::raw::c_char;

use crate::vm::value::Value;
use crate::vm::VM;

thread_local! {
    /// Index 0 is `இன்மை`, so a zeroed handle is a valid value rather than a
    /// wild index.
    static ARENA: RefCell<Vec<Value>> = RefCell::new(vec![Value::Null]);

    /// Kept only for its builtin dispatch and its stack. Nothing here executes
    /// bytecode; the compiled program is the program.
    static HOST: RefCell<VM> = RefCell::new(VM::new());
}

/// A runtime error, reported the way the VM reports one and with the same
/// exit status, because the two backends failing differently on the same
/// program is its own kind of disagreement.
fn fail(message: &str) -> ! {
    eprintln!("✗ Runtime error: {}", message);
    std::process::exit(1);
}

fn put(value: Value) -> i64 {
    ARENA.with(|arena| {
        let mut arena = arena.borrow_mut();
        arena.push(value);
        (arena.len() - 1) as i64
    })
}

/// Cloned out rather than borrowed, so that a caller can hold two values at
/// once without holding two borrows of the arena.
fn get(handle: i64) -> Value {
    ARENA.with(|arena| {
        let arena = arena.borrow();
        match arena.get(handle.max(0) as usize) {
            Some(value) => value.clone(),
            None => fail("இயக்க நேரப் பிழை: செல்லாத மதிப்பு  (invalid runtime handle)"),
        }
    })
}

/// # Safety
///
/// `text` must be a NUL-terminated string the emitted IR keeps alive, which a
/// global string constant is.
unsafe fn borrow_str(text: *const c_char) -> String {
    if text.is_null() {
        return String::new();
    }
    unsafe { CStr::from_ptr(text) }
        .to_string_lossy()
        .into_owned()
}

// --- Constructors ---------------------------------------------------------

/// A number, from the decimal's own text. Going through the string is what
/// makes it exact: `codegen.rs` has a `Decimal` at compile time and writes it
/// out, and `Decimal::from_str` reads back the identical value. Passing a
/// double here would lose the thing this whole module exists to keep.
#[unsafe(no_mangle)]
pub extern "C" fn etamil_number(text: *const c_char) -> i64 {
    let text = unsafe { borrow_str(text) };
    match text.parse::<rust_decimal::Decimal>() {
        Ok(number) => put(Value::Number(number)),
        Err(_) => fail(&format!("'{}' ஒரு எண் அல்ல  ('{}' is not a number)", text, text)),
    }
}

/// A number from a machine integer, for counters the IR generates itself
/// rather than reads from the program — a `ஒவ்வொரு` loop's position, mainly.
/// No text to parse, because there is no literal to be faithful to.
#[unsafe(no_mangle)]
pub extern "C" fn etamil_from_int(value: i64) -> i64 {
    put(Value::Number(rust_decimal::Decimal::from(value)))
}

#[unsafe(no_mangle)]
pub extern "C" fn etamil_text(text: *const c_char) -> i64 {
    let text = unsafe { borrow_str(text) };
    put(Value::String(text))
}

#[unsafe(no_mangle)]
pub extern "C" fn etamil_boolean(flag: i32) -> i64 {
    put(Value::Boolean(flag != 0))
}

#[unsafe(no_mangle)]
pub extern "C" fn etamil_nil() -> i64 {
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn etamil_array() -> i64 {
    put(Value::Array(Vec::new()))
}

/// Grows an array in place. Used to build a literal, where the array is fresh
/// and nothing else refers to it yet.
#[unsafe(no_mangle)]
pub extern "C" fn etamil_array_push(array: i64, value: i64) {
    let value = get(value);
    ARENA.with(|arena| {
        let mut arena = arena.borrow_mut();
        match arena.get_mut(array.max(0) as usize) {
            Some(Value::Array(items)) => items.push(value),
            _ => fail("அணி எதிர்பார்க்கப்பட்டது  (expected an array)"),
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn etamil_record() -> i64 {
    put(Value::Map(std::collections::HashMap::new()))
}

#[unsafe(no_mangle)]
pub extern "C" fn etamil_record_put(record: i64, key: *const c_char, value: i64) {
    let key = unsafe { borrow_str(key) };
    let value = get(value);
    ARENA.with(|arena| {
        let mut arena = arena.borrow_mut();
        match arena.get_mut(record.max(0) as usize) {
            Some(Value::Map(fields)) => {
                fields.insert(key, value);
            }
            _ => fail("பொருள் எதிர்பார்க்கப்பட்டது  (expected a record)"),
        }
    })
}

// --- Arithmetic -----------------------------------------------------------
//
// `to_number` is `Value`'s own coercion, so a string that looks like a number
// behaves here exactly as it does on the VM.

#[unsafe(no_mangle)]
pub extern "C" fn etamil_add(left: i64, right: i64) -> i64 {
    put(Value::Number(get(left).to_number() + get(right).to_number()))
}

#[unsafe(no_mangle)]
pub extern "C" fn etamil_subtract(left: i64, right: i64) -> i64 {
    put(Value::Number(get(left).to_number() - get(right).to_number()))
}

#[unsafe(no_mangle)]
pub extern "C" fn etamil_multiply(left: i64, right: i64) -> i64 {
    put(Value::Number(get(left).to_number() * get(right).to_number()))
}

/// Exact to the decimal type's full precision, and rounding is deliberately
/// not applied — the VM's `Divide` says the same, because Indian tax
/// computation rounds once at the end and rounding intermediates compounds.
#[unsafe(no_mangle)]
pub extern "C" fn etamil_divide(left: i64, right: i64) -> i64 {
    let divisor = get(right).to_number();
    if divisor.is_zero() {
        fail("பூஜ்ஜியத்தால் வகுத்தல்  (division by zero)");
    }
    put(Value::Number(get(left).to_number() / divisor))
}

/// `&`. The VM builds `format!("{}{}", ...)` over `to_string`, so this is that.
#[unsafe(no_mangle)]
pub extern "C" fn etamil_concat(left: i64, right: i64) -> i64 {
    let joined = format!("{}{}", get(left).to_string(), get(right).to_string());
    put(Value::String(joined))
}

// --- Comparison and logic -------------------------------------------------

/// Operator codes, fixed here and mirrored in `codegen.rs`. Kept as small
/// integers rather than strings so the IR passes a constant.
pub const COMPARE_LT: i32 = 0;
pub const COMPARE_LE: i32 = 1;
pub const COMPARE_GT: i32 = 2;
pub const COMPARE_GE: i32 = 3;
pub const COMPARE_EQ: i32 = 4;
pub const COMPARE_NE: i32 = 5;

#[unsafe(no_mangle)]
pub extern "C" fn etamil_compare(left: i64, right: i64, operator: i32) -> i64 {
    let left = get(left);
    let right = get(right);

    // Equality is `Value`'s own, which compares numbers exactly and knows how
    // to compare arrays and records. Ordering goes through `partial_cmp`, and
    // two values with no ordering between them answer false rather than
    // guessing — the same thing the VM's LessThan does.
    let answer = match operator {
        COMPARE_EQ => left == right,
        COMPARE_NE => left != right,
        _ => match left.partial_cmp(&right) {
            Some(ordering) => match operator {
                COMPARE_LT => ordering == std::cmp::Ordering::Less,
                COMPARE_LE => ordering != std::cmp::Ordering::Greater,
                COMPARE_GT => ordering == std::cmp::Ordering::Greater,
                COMPARE_GE => ordering != std::cmp::Ordering::Less,
                _ => false,
            },
            None => false,
        },
    };
    put(Value::Boolean(answer))
}

#[unsafe(no_mangle)]
pub extern "C" fn etamil_and(left: i64, right: i64) -> i64 {
    put(Value::Boolean(get(left).is_truthy() && get(right).is_truthy()))
}

#[unsafe(no_mangle)]
pub extern "C" fn etamil_or(left: i64, right: i64) -> i64 {
    put(Value::Boolean(get(left).is_truthy() || get(right).is_truthy()))
}

#[unsafe(no_mangle)]
pub extern "C" fn etamil_not(value: i64) -> i64 {
    put(Value::Boolean(!get(value).is_truthy()))
}

/// The one place a handle becomes something the IR can branch on.
#[unsafe(no_mangle)]
pub extern "C" fn etamil_truthy(value: i64) -> i32 {
    i32::from(get(value).is_truthy())
}

/// For `?`. The IR tests this and returns the value unchanged when it is a
/// `தவறு`, which is what propagating an error out of a function means.
#[unsafe(no_mangle)]
pub extern "C" fn etamil_is_error(value: i64) -> i32 {
    i32::from(matches!(get(value), Value::Err(_)))
}

/// `?` on a success unwraps it; the IR only calls this once `etamil_is_error`
/// has said no.
#[unsafe(no_mangle)]
pub extern "C" fn etamil_unwrap(value: i64) -> i64 {
    match get(value) {
        Value::Ok(inner) => put(*inner),
        other => put(other),
    }
}

// --- Access ---------------------------------------------------------------

#[unsafe(no_mangle)]
pub extern "C" fn etamil_index(base: i64, index: i64) -> i64 {
    match VM::index_of(&get(base), &get(index)) {
        Ok(value) => put(value),
        Err(why) => fail(&why),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn etamil_index_set(base: i64, index: i64, value: i64) {
    let position = get(index).to_number();
    let value = get(value);
    let position = match rust_decimal::prelude::ToPrimitive::to_usize(&position) {
        Some(position) => position,
        None => fail("அணி முகவரி ஒரு முழு எண்  (an array index must be a whole number)"),
    };
    ARENA.with(|arena| {
        let mut arena = arena.borrow_mut();
        match arena.get_mut(base.max(0) as usize) {
            Some(Value::Array(items)) if position < items.len() => items[position] = value,
            Some(Value::Array(_)) => fail(&format!(
                "அணி முகவரி வரம்பை மீறியது: {}  (array index {} is out of range)",
                position, position
            )),
            _ => fail("அணி எதிர்பார்க்கப்பட்டது  (expected an array)"),
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn etamil_field(base: i64, key: *const c_char) -> i64 {
    let key = unsafe { borrow_str(key) };
    match get(base) {
        Value::Map(fields) => match fields.get(&key) {
            Some(value) => put(value.clone()),
            None => fail(&format!(
                "புலம் '{}' இல்லை  (no field '{}' on this record)",
                key, key
            )),
        },
        _ => fail(&format!(
            "'{}' ஒரு பொருள் அல்ல  ('.{}' needs a record)",
            key, key
        )),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn etamil_field_set(base: i64, key: *const c_char, value: i64) {
    let key = unsafe { borrow_str(key) };
    let value = get(value);
    ARENA.with(|arena| {
        let mut arena = arena.borrow_mut();
        match arena.get_mut(base.max(0) as usize) {
            Some(Value::Map(fields)) => {
                fields.insert(key, value);
            }
            _ => fail("பொருள் எதிர்பார்க்கப்பட்டது  (expected a record)"),
        }
    })
}

/// How many items a `ஒவ்வொரு` has to walk. Separate from the `நீளம்` builtin
/// only because the loop needs a plain integer to compare against, not a
/// handle.
#[unsafe(no_mangle)]
pub extern "C" fn etamil_count(value: i64) -> i64 {
    match get(value) {
        Value::Array(items) => items.len() as i64,
        Value::Map(fields) => fields.len() as i64,
        Value::String(text) => text.chars().count() as i64,
        _ => fail("ஒவ்வொரு க்கு ஒரு அணி தேவை  (for-each needs an array)"),
    }
}

// --- Builtins and output --------------------------------------------------

/// Any of the fifty-nine, by name, through the interpreter's own dispatch.
///
/// # Safety
///
/// `argv` must point to `argc` handles.
#[unsafe(no_mangle)]
pub extern "C" fn etamil_call(name: *const c_char, argv: *const i64, argc: i64) -> i64 {
    let name = unsafe { borrow_str(name) };
    let count = argc.max(0) as usize;

    let mut arguments = Vec::with_capacity(count);
    for position in 0..count {
        // Read the handle, then resolve it: `get` borrows the arena, and doing
        // both in one expression would hold that borrow across the loop.
        let handle = if argv.is_null() {
            0
        } else {
            unsafe { *argv.add(position) }
        };
        arguments.push(get(handle));
    }

    let answer = HOST.with(|host| host.borrow_mut().invoke_builtin(&name, arguments));
    match answer {
        Ok(value) => put(value),
        Err(why) => fail(&why),
    }
}

/// `அச்சு`. One `to_string` and one newline, the same call the VM's `Print`
/// makes — which is why `&` between a label and a number comes out identically
/// without the backend having to know anything about text.
#[unsafe(no_mangle)]
pub extern "C" fn etamil_print(value: i64) {
    println!("{}", get(value).to_string());
}

/// `உள்ளிடு`. A line of input, as text, exactly as the VM's `Input` leaves it.
#[unsafe(no_mangle)]
pub extern "C" fn etamil_read_line() -> i64 {
    let mut line = String::new();
    match std::io::stdin().read_line(&mut line) {
        Ok(_) => put(Value::String(line.trim().to_string())),
        Err(why) => fail(&format!("உள்ளிட முடியவில்லை  (could not read input: {})", why)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// What a compiled program would print for this handle.
    fn shown(handle: i64) -> String {
        get(handle).to_string()
    }

    fn number(text: &str) -> i64 {
        let c = std::ffi::CString::new(text).unwrap();
        etamil_number(c.as_ptr())
    }

    fn text(value: &str) -> i64 {
        let c = std::ffi::CString::new(value).unwrap();
        etamil_text(c.as_ptr())
    }

    // --- the claim the language opens with -------------------------------

    #[test]
    fn a_tenth_and_a_fifth_are_exactly_three_tenths() {
        // The f64 backend answered 0.30000000000000004 and said nothing. This
        // is the whole reason the runtime exists.
        assert_eq!(shown(etamil_add(number("0.1"), number("0.2"))), "0.3");
    }

    #[test]
    fn a_third_keeps_every_digit_the_vm_keeps() {
        // A fixed two-place representation would answer 0.33 here, which is
        // why that workaround was not taken.
        assert_eq!(
            shown(etamil_divide(number("1"), number("3"))),
            "0.3333333333333333333333333333"
        );
    }

    #[test]
    fn trailing_zeros_are_trimmed_the_way_the_vm_trims_them() {
        // 1000 * 18% is 180.00 by scale and prints as 180.
        assert_eq!(shown(etamil_multiply(number("1000"), number("0.18"))), "180");
        assert_eq!(shown(etamil_add(number("2.50"), number("0"))), "2.5");
        assert_eq!(shown(etamil_divide(number("10"), number("2"))), "5");
    }

    #[test]
    fn money_to_the_paisa_still_balances() {
        // The paise convention keeps working, and now so does writing it in
        // rupees, which the i64 backend refused outright.
        let total = etamil_add(number("2.05"), number("1.95"));
        assert_eq!(shown(total), "4");
        assert_eq!(shown(etamil_multiply(number("2.05"), number("7"))), "14.35");
    }

    // --- text, and why concat needed no knowledge of text ----------------

    #[test]
    fn concat_renders_each_side_the_way_print_would() {
        let label = text("விலை: ");
        assert_eq!(shown(etamil_concat(label, number("2.05"))), "விலை: 2.05");
        // A boolean renders as the VM renders it, not as 1.
        assert_eq!(
            shown(etamil_concat(text("x"), etamil_boolean(1))),
            "xtrue"
        );
    }

    #[test]
    fn nil_is_handle_zero_and_prints_as_nil() {
        assert_eq!(etamil_nil(), 0);
        assert_eq!(shown(etamil_nil()), "nil");
    }

    // --- comparison ------------------------------------------------------

    #[test]
    fn ordering_and_equality_go_through_value() {
        assert_eq!(shown(etamil_compare(number("2"), number("3"), COMPARE_LT)), "true");
        assert_eq!(shown(etamil_compare(number("3"), number("3"), COMPARE_LE)), "true");
        assert_eq!(shown(etamil_compare(number("3"), number("3"), COMPARE_NE)), "false");
        // Exactly equal, not equal within an epsilon: the f64 backend compared
        // two amounts a hundredth of a paisa apart as the same.
        assert_eq!(
            shown(etamil_compare(number("2.05"), number("2.0500000001"), COMPARE_EQ)),
            "false"
        );
    }

    #[test]
    fn truthiness_matches_what_the_vm_branches_on() {
        // Checked against the VM: 5 takes the branch and 0 does not.
        assert_eq!(etamil_truthy(number("5")), 1);
        assert_eq!(etamil_truthy(number("0")), 0);
        assert_eq!(etamil_truthy(etamil_boolean(0)), 0);
    }

    // --- collections -----------------------------------------------------

    #[test]
    fn an_array_literal_prints_the_way_the_vm_prints_one() {
        let array = etamil_array();
        etamil_array_push(array, number("1"));
        etamil_array_push(array, number("2.5"));
        assert_eq!(shown(array), "[1, 2.5]");
        assert_eq!(etamil_count(array), 2);
    }

    #[test]
    fn a_loop_counter_becomes_a_number_without_going_through_text() {
        assert_eq!(shown(etamil_from_int(0)), "0");
        assert_eq!(shown(etamil_from_int(-7)), "-7");
        // And it indexes, which is the only reason it exists.
        let array = etamil_array();
        etamil_array_push(array, text("அ"));
        etamil_array_push(array, text("ஆ"));
        assert_eq!(shown(etamil_index(array, etamil_from_int(1))), "ஆ");
    }

    #[test]
    fn indexing_goes_through_the_vms_own_index_of() {
        let array = etamil_array();
        etamil_array_push(array, number("10"));
        etamil_array_push(array, number("20"));
        assert_eq!(shown(etamil_index(array, number("1"))), "20");

        etamil_index_set(array, number("0"), number("99"));
        assert_eq!(shown(etamil_index(array, number("0"))), "99");
    }

    #[test]
    fn a_record_holds_and_returns_its_fields() {
        let record = etamil_record();
        let key = std::ffi::CString::new("பெயர்").unwrap();
        etamil_record_put(record, key.as_ptr(), text("ராஜா"));
        assert_eq!(shown(etamil_field(record, key.as_ptr())), "ராஜா");
    }

    // --- builtins, through the interpreter's dispatch ---------------------

    #[test]
    fn a_builtin_is_the_interpreters_builtin() {
        // நீளம் is not reimplemented here, so it cannot disagree with the VM's.
        let array = etamil_array();
        etamil_array_push(array, number("1"));
        etamil_array_push(array, number("2"));
        etamil_array_push(array, number("3"));

        let name = std::ffi::CString::new("நீளம்").unwrap();
        let argv = [array];
        assert_eq!(shown(etamil_call(name.as_ptr(), argv.as_ptr(), 1)), "3");
    }

    #[test]
    fn floor_and_ceiling_come_from_the_same_place_too() {
        let floor = std::ffi::CString::new("தரை").unwrap();
        let argv = [etamil_divide(number("205"), number("100"))];
        assert_eq!(shown(etamil_call(floor.as_ptr(), argv.as_ptr(), 1)), "2");

        let ceil = std::ffi::CString::new("மேல்").unwrap();
        let argv = [etamil_divide(number("1000"), number("3"))];
        assert_eq!(shown(etamil_call(ceil.as_ptr(), argv.as_ptr(), 1)), "334");
    }

    #[test]
    fn a_result_can_be_tested_and_unwrapped() {
        let ok = std::ffi::CString::new("சரி").unwrap();
        let argv = [number("7")];
        let wrapped = etamil_call(ok.as_ptr(), argv.as_ptr(), 1);
        assert_eq!(etamil_is_error(wrapped), 0);
        assert_eq!(shown(etamil_unwrap(wrapped)), "7");

        let err = std::ffi::CString::new("தவறு").unwrap();
        let argv = [text("போச்சு")];
        let failed = etamil_call(err.as_ptr(), argv.as_ptr(), 1);
        assert_eq!(etamil_is_error(failed), 1);
    }
}
