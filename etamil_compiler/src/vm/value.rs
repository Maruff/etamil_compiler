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
    Map(HashMap<String, Value>),
    /// சரி — a successful result, as in Rust's Ok.
    Ok(Box<Value>),
    /// தவறு — a failed result, as in Rust's Err. Failure is a value that
    /// must be handled, not an exception that unwinds silently.
    Err(Box<Value>),
    Null,
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

    pub fn to_string(&self) -> String {
        match self {
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
                    .map(|k| format!("{}: {}", k, fields[*k].to_string()))
                    .collect();
                format!("{{{}}}", inner.join(", "))
            }
            Value::Ok(inner) => format!("சரி({})", inner.to_string()),
            Value::Err(inner) => format!("தவறு({})", inner.to_string()),
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
