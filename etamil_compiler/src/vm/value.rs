// Bytecode value types for the eTamil VM
use std::collections::HashMap;

/// Runtime values in the eTamil VM
#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    String(String),
    Boolean(bool),
    Array(Vec<Value>),
    Map(HashMap<String, Value>),
    Null,
}

impl Value {
    pub fn to_number(&self) -> f64 {
        match self {
            Value::Number(n) => *n,
            Value::Boolean(true) => 1.0,
            Value::Boolean(false) => 0.0,
            Value::String(s) => s.parse::<f64>().unwrap_or(0.0),
            _ => 0.0,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Value::Number(n) => {
                if n.fract() == 0.0 {
                    format!("{:.0}", n)
                } else {
                    format!("{}", n)
                }
            }
            Value::String(s) => s.clone(),
            Value::Boolean(b) => b.to_string(),
            Value::Null => "nil".to_string(),
            Value::Array(_) => "[Array]".to_string(),
            Value::Map(_) => "[Map]".to_string(),
        }
    }

    pub fn to_boolean(&self) -> bool {
        match self {
            Value::Null => false,
            Value::Number(n) => *n != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::Boolean(b) => *b,
            Value::Array(a) => !a.is_empty(),
            Value::Map(m) => !m.is_empty(),
        }
    }

    pub fn is_truthy(&self) -> bool {
        self.to_boolean()
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => (a - b).abs() < 1e-10,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
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
            _ => None,
        }
    }
}
