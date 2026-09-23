// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Mohammed Maruff (Esan Maruff) <esan@etamil.in>
// Bytecode value types for the eTamil VM
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::str::FromStr;

/// Runtime values in the eTamil VM.
///
/// Numbers are fixed-point decimals rather than `f64`. eTamil is a language
/// for tax and accounting, where `0.1 + 0.2` must be exactly `0.3` and a
/// ledger has to balance to the paisa; binary floating point cannot promise
/// either.
#[derive(Debug, Clone)]
pub enum Value {
    Number(Decimal),
    String(String),
    Boolean(bool),
    Array(Vec<Value>),
    Map(Record),
    /// சரி — a successful result, as in Rust's Ok.
    Ok(Box<Value>),
    /// தவறு — a failed result, as in Rust's Err. Failure is a value that
    /// must be handled, not an exception that unwinds silently.
    Err(Box<Value>),
    /// A செயல் held as a value: `ச = இரட்டி;`, or `செயல்(x) { … }` written
    /// where a value goes.
    Function(Box<FunctionValue>),
    Null,
}

/// A பொருள்: fields by name, and the வடிவம் it was made as, if any.
///
/// It dereferences to the map, so everything that reads or writes a record as
/// a map still does — indexing, iterating its field names, the drivers, the
/// JSON encoder. What the shape adds is a promise about which fields there are
/// and what they hold, kept by `vm::shape` at the few places a record can be
/// changed. A record read from a database or built from a literal has none.
#[derive(Debug, Clone, Default)]
pub struct Record {
    pub fields: HashMap<String, Value>,
    pub shape: Option<String>,
}

impl Record {
    pub fn shaped(shape: &str, fields: HashMap<String, Value>) -> Self {
        Record {
            fields,
            shape: Some(shape.to_string()),
        }
    }
}

impl std::ops::Deref for Record {
    type Target = HashMap<String, Value>;
    fn deref(&self) -> &Self::Target {
        &self.fields
    }
}

impl std::ops::DerefMut for Record {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.fields
    }
}

impl From<HashMap<String, Value>> for Record {
    fn from(fields: HashMap<String, Value>) -> Self {
        Record {
            fields,
            shape: None,
        }
    }
}

impl FromIterator<(String, Value)> for Record {
    fn from_iter<I: IntoIterator<Item = (String, Value)>>(pairs: I) -> Self {
        HashMap::from_iter(pairs).into()
    }
}

impl<'a> IntoIterator for &'a Record {
    type Item = (&'a String, &'a Value);
    type IntoIter = std::collections::hash_map::Iter<'a, String, Value>;
    fn into_iter(self) -> Self::IntoIter {
        self.fields.iter()
    }
}

/// Two records are the same record when they hold the same fields and were
/// made as the same shape. A `கடன்` and a plain record with the same fields
/// are two different things, as two Rust structs with the same fields are.
impl PartialEq for Record {
    fn eq(&self, other: &Self) -> bool {
        self.shape == other.shape && self.fields == other.fields
    }
}

/// What a function value refers to, and what it carried away with it.
///
/// `name` is the function to run — one the program defined, one of the
/// builtins, or the hidden name an anonymous செயல் was compiled under.
/// `captured` holds the values of the enclosing function's locals it uses, taken
/// when the value was made. They are copies, like every other value here, so a
/// function value cannot see or change the variables it was made beside after
/// the fact: the same thing Rust's `move` closures promise.
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionValue {
    pub name: String,
    pub captured: Vec<Value>,
}

impl FunctionValue {
    /// The name an anonymous செயல் is compiled under starts with `#`, which no
    /// source identifier can, so it can never collide with the author's own.
    pub fn is_anonymous(name: &str) -> bool {
        name.starts_with('#')
    }

    /// How an error message names this function.
    pub fn shown(name: &str) -> String {
        if Self::is_anonymous(name) {
            "பெயரில்லா செயல் (an anonymous function)".to_string()
        } else {
            format!("'{}'", name)
        }
    }
}

impl Value {
    pub fn to_number(&self) -> Decimal {
        match self {
            Value::Number(n) => *n,
            Value::Boolean(true) => Decimal::ONE,
            Value::Boolean(false) => Decimal::ZERO,
            // Input arrives as text, so strings coerce when used as numbers.
            Value::String(s) => Decimal::from_str(s.trim()).unwrap_or(Decimal::ZERO),
            _ => Decimal::ZERO,
        }
    }

    pub fn to_boolean(&self) -> bool {
        match self {
            Value::Null => false,
            Value::Number(n) => *n != Decimal::ZERO,
            Value::String(s) => !s.is_empty(),
            Value::Boolean(b) => *b,
            Value::Array(a) => !a.is_empty(),
            Value::Map(m) => !m.is_empty(),
            // A result is truthy when it succeeded, so `(r) எனில்` reads the
            // way you would expect without unwrapping first.
            Value::Ok(_) => true,
            Value::Err(_) => false,
            Value::Function(_) => true,
        }
    }

    pub fn is_truthy(&self) -> bool {
        self.to_boolean()
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            // Exact comparison. The old f64 representation needed an epsilon
            // here, which meant two amounts a hundredth of a paisa apart
            // compared equal.
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Ok(a), Value::Ok(b)) => a == b,
            (Value::Err(a), Value::Err(b)) => a == b,
            (Value::Null, Value::Null) => true,

            // Arrays and records had no arm here at all, so they fell to the
            // catch-all and `[1, 2] == [1, 2]` was false. So was `[] == []`.
            // Nothing warned: comparing two of them simply always answered no,
            // which is the worst way for an equality to be wrong — a program
            // checking whether a result matched what it expected got "no" and
            // read it as a difference in the data.
            //
            // An array is ordered, so position matters.
            (Value::Array(a), Value::Array(b)) => a == b,

            // A record is not ordered — `{அ: 1, ஆ: 2}` and `{ஆ: 2, அ: 1}` are
            // the same record — so this compares by field rather than by
            // sequence, which is what HashMap's own equality does.
            (Value::Map(a), Value::Map(b)) => a == b,

            // The same function with the same captured values. Two anonymous
            // செயல்s written identically are still two functions.
            (Value::Function(a), Value::Function(b)) => a == b,

            _ => false,
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => a.partial_cmp(b),
            (Value::String(a), Value::String(b)) => a.partial_cmp(b),
            (Value::String(_), Value::Number(_)) | (Value::Number(_), Value::String(_)) => {
                self.to_number().partial_cmp(&other.to_number())
            }
            _ => None,
        }
    }
}

/// How a value becomes text — printing, string coercion and record keys all
/// come through here.
///
/// This is `Display` rather than an inherent `to_string`, so `to_string()` on a
/// Value is the standard one every Rust reader expects and `{}` in a format
/// string works. The output is unchanged.
impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&match self {
            Value::Number(n) => {
                if n.fract() == Decimal::ZERO {
                    n.trunc().to_string()
                } else {
                    // normalize() drops trailing zeros, so 1.50 prints as 1.5
                    // while 1.05 keeps both digits.
                    n.normalize().to_string()
                }
            }
            Value::String(s) => s.clone(),
            Value::Boolean(b) => b.to_string(),
            Value::Null => "nil".to_string(),
            Value::Array(items) => {
                let inner: Vec<String> = items.iter().map(|v| v.to_string()).collect();
                format!("[{}]", inner.join(", "))
            }
            Value::Map(fields) => {
                // Sorted so printing a record is deterministic.
                let mut keys: Vec<&String> = fields.keys().collect();
                keys.sort();
                let inner: Vec<String> = keys
                    .iter()
                    .map(|k| format!("{}: {}", k, fields[*k]))
                    .collect();
                // A shaped record prints as it is written: `கடன்{அசல்: 1}`.
                match &fields.shape {
                    Some(shape) => format!("{}{{{}}}", shape, inner.join(", ")),
                    None => format!("{{{}}}", inner.join(", ")),
                }
            }
            Value::Ok(inner) => format!("சரி({inner})"),
            Value::Err(inner) => format!("தவறு({inner})"),
            Value::Function(function) => {
                if FunctionValue::is_anonymous(&function.name) {
                    "<செயல்>".to_string()
                } else {
                    format!("<செயல் {}>", function.name)
                }
            }
        })
    }
}
