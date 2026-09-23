// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Mohammed Maruff (Esan Maruff) <esan@etamil.in>
//! Shapes at runtime: the promise a `வடிவம்` makes, kept where a record is
//! made or changed.
//!
//! The checker refuses what it can see — a field that does not exist, a
//! literal missing one, a string where a number was declared. What it cannot
//! see is a value that only exists when the program runs: a row from a query,
//! a number typed at a prompt, a field set by a key computed at runtime. Those
//! are checked here, at the moment a shaped record is built or one of its
//! fields is set, so a `கடன்` holds what `கடன்` says wherever it came from.
//!
//! Both backends call these functions — the VM directly, and a compiled
//! program through `crate::runtime` — so the two cannot disagree about what
//! a shape allows.

use std::collections::HashMap;

use crate::parser::DeclaredType;
use crate::vm::VM;
use crate::vm::value::{FunctionValue, Record, Value};

/// One declared shape, as the running program knows it.
#[derive(Debug, Clone, Default)]
pub struct Shape {
    pub name: String,
    /// In the order they were declared, which is the order an error lists
    /// them in.
    pub fields: Vec<(String, Option<DeclaredType>)>,
    /// Each method, and whether it takes இது.
    pub methods: HashMap<String, bool>,
}

pub type Shapes = HashMap<String, Shape>;

impl Shape {
    fn field(&self, name: &str) -> Option<&Option<DeclaredType>> {
        self.fields
            .iter()
            .find(|(field, _)| field == name)
            .map(|(_, declared)| declared)
    }

    fn field_names(&self) -> String {
        self.fields
            .iter()
            .map(|(name, _)| name.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

/// The name of the function a method compiles to. The dot cannot appear in a
/// source identifier, so no function the author writes can take it.
pub fn method_function(shape: &str, method: &str) -> String {
    format!("{}.{}", shape, method)
}

/// Can this value be held where `declared` was written?
///
/// The runtime half of the checker's `Inferred::satisfies`, and deliberately
/// the same rules: text accepts a number, a date is text, a shaped record is a
/// record. இன்மை fits anywhere, as the checker has always treated it.
pub fn fits(value: &Value, declared: &DeclaredType) -> bool {
    match (declared, value) {
        (_, Value::Null) | (DeclaredType::Any, _) => true,
        (DeclaredType::Number, Value::Number(_)) => true,
        (DeclaredType::Text | DeclaredType::Date, Value::String(_) | Value::Number(_)) => true,
        (DeclaredType::Boolean, Value::Boolean(_)) => true,
        (DeclaredType::Array, Value::Array(_)) => true,
        (DeclaredType::Record, Value::Map(_)) => true,
        (DeclaredType::Function, Value::Function(_)) => true,
        (DeclaredType::Shape(shape), Value::Map(record)) => {
            record.shape.as_deref() == Some(shape.as_str())
        }
        _ => false,
    }
}

fn wrong_value(shape: &str, field: &str, declared: &DeclaredType, value: &Value) -> String {
    format!(
        "'{}.{}' {} ஆக இருக்க வேண்டும், ஆனால் {} கொடுக்கப்பட்டது  \
         ('{}.{}' is declared {}, but was given {})",
        shape,
        field,
        declared.name(),
        VM::type_name(value),
        shape,
        field,
        declared.name(),
        VM::type_name(value)
    )
}

fn unknown(shape: &str) -> String {
    format!(
        "'{}' என்ற வடிவம் இல்லை  (there is no shape named '{}')",
        shape, shape
    )
}

/// Every declared field present, nothing else, each holding what it may.
fn validate(shape: &Shape, fields: &HashMap<String, Value>) -> Result<(), String> {
    let missing: Vec<&str> = shape
        .fields
        .iter()
        .map(|(name, _)| name.as_str())
        .filter(|name| !fields.contains_key(*name))
        .collect();
    if !missing.is_empty() {
        let list = missing.join(", ");
        return Err(format!(
            "'{}' க்கு {} புலங்களும் தேவை  (a {} needs every one of its fields; missing: {})",
            shape.name, list, shape.name, list
        ));
    }

    let mut extra: Vec<&str> = fields
        .keys()
        .map(String::as_str)
        .filter(|name| shape.field(name).is_none())
        .collect();
    if !extra.is_empty() {
        extra.sort();
        let list = extra.join(", ");
        return Err(format!(
            "'{}' இல் {} புலங்கள் இல்லை  ('{}' has no field {}; its fields are: {})",
            shape.name,
            list,
            shape.name,
            list,
            shape.field_names()
        ));
    }

    for (name, declared) in &shape.fields {
        if let Some(declared) = declared
            && !fits(&fields[name], declared)
        {
            return Err(wrong_value(&shape.name, name, declared, &fields[name]));
        }
    }
    Ok(())
}

/// `கடன்{அசல்: …, ..பழையது}` — build a record of this shape.
///
/// `base` is the `..` record, whose fields fill in whatever was not written.
/// It must be the same shape: a plain record could be missing a field, or hold
/// one the shape does not have, and taking from it would pass either along.
pub fn build(
    shapes: &Shapes,
    name: &str,
    given: Vec<(String, Value)>,
    base: Option<Value>,
) -> Result<Value, String> {
    let shape = shapes.get(name).ok_or_else(|| unknown(name))?;
    let mut fields = match base {
        None => HashMap::new(),
        Some(Value::Map(record)) if record.shape.as_deref() == Some(name) => record.fields,
        Some(other) => {
            return Err(format!(
                "'..' க்கு ஒரு '{}' தேவை  ('..' needs a {} to take the other fields from, got {})",
                name,
                name,
                VM::type_name(&other)
            ));
        }
    };
    fields.extend(given);
    validate(shape, &fields)?;
    Ok(Value::Map(Record::shaped(name, fields)))
}

/// `கடன்(பதிவு)` — a plain record, checked and made into this shape.
///
/// A result, because the record usually came from outside the program — a
/// query, a JSON body — and not fitting is an ordinary outcome to be handled,
/// not a fault. `கடன்(பதிவு)?` passes the failure up.
pub fn convert(shapes: &Shapes, name: &str, value: Value) -> Result<Value, String> {
    let shape = shapes.get(name).ok_or_else(|| unknown(name))?;
    let refused = |why: String| Ok(Value::Err(Box::new(Value::String(why))));
    match value {
        Value::Map(record) => {
            match record.shape.as_deref() {
                // Already one: nothing to check.
                Some(own) if own == name => return Ok(Value::Ok(Box::new(Value::Map(record)))),
                Some(other) => {
                    return refused(format!(
                        "ஒரு '{}' ஐ '{}' ஆக மாற்ற முடியாது  (a {} is not a {})",
                        other, name, other, name
                    ));
                }
                None => {}
            }
            match validate(shape, &record.fields) {
                Ok(()) => Ok(Value::Ok(Box::new(Value::Map(Record::shaped(
                    name,
                    record.fields,
                ))))),
                Err(why) => refused(why),
            }
        }
        other => refused(format!(
            "'{}(…)' க்கு ஒரு பொருள் தேவை  ('{}(…)' needs a record, got {})",
            name,
            name,
            VM::type_name(&other)
        )),
    }
}

/// Setting one field of a record, by `r.f = v` or `r[k] = v`.
///
/// A plain record takes any field, as it always has. A shaped one takes only
/// the fields its shape has, holding what they were declared to.
pub fn check_set(
    shapes: &Shapes,
    record: &Record,
    field: &str,
    value: &Value,
) -> Result<(), String> {
    let Some(name) = record.shape.as_deref() else {
        return Ok(());
    };
    let Some(shape) = shapes.get(name) else {
        return Ok(());
    };
    match shape.field(field) {
        None => Err(format!(
            "'{}' இல் '{}' என்ற புலம் இல்லை  ('{}' has no field '{}'; its fields are: {})",
            name,
            field,
            name,
            field,
            shape.field_names()
        )),
        Some(Some(declared)) if !fits(value, declared) => {
            Err(wrong_value(name, field, declared, value))
        }
        Some(_) => Ok(()),
    }
}

/// What `r.m(…)` calls.
pub enum Target {
    /// The method `m` of r's shape, whose function takes r first.
    Method(String),
    /// A function held in r's field `m`, called with the arguments alone.
    Held(FunctionValue),
}

/// `r.m(…)`: the method of r's shape, and failing that a function in r's
/// field of that name — which is how a plain record carrying functions is
/// called, `கருவிகள்.முழுமை(2.6)`.
pub fn method_target(shapes: &Shapes, receiver: &Value, method: &str) -> Result<Target, String> {
    let record = match receiver {
        Value::Map(record) => record,
        other => {
            return Err(format!(
                "'.{}(…)' க்கு ஒரு பொருள் தேவை  ('.{}(…)' needs a record, got {})",
                method,
                method,
                VM::type_name(other)
            ));
        }
    };

    if let Some(shape) = record.shape.as_deref().and_then(|name| shapes.get(name)) {
        match shape.methods.get(method) {
            Some(true) => return Ok(Target::Method(method_function(&shape.name, method))),
            Some(false) => {
                return Err(format!(
                    "'{}' ஒரு பதிவின் மேல் அழைக்கப்படுவதில்லை  \
                     ('{}' takes no இது, so it is not called on a record: call it as {}.{}(…))",
                    method, method, shape.name, method
                ));
            }
            None => {}
        }
    }

    match record.get(method) {
        Some(Value::Function(function)) => Ok(Target::Held((**function).clone())),
        Some(other) => Err(format!(
            "'{}' ஒரு செயல் அல்ல  ('.{}' holds {}, not a function, so it cannot be called)",
            method,
            method,
            VM::type_name(other)
        )),
        None => Err(match &record.shape {
            Some(shape) => format!(
                "'{}' இல் '{}' என்ற முறை இல்லை  ('{}' has no method or field '{}')",
                shape, method, shape, method
            ),
            None => format!(
                "'{}' என்ற புலம் இல்லை  (no method or field '{}' on this record)",
                method, method
            ),
        }),
    }
}

// --- Declared types across the C ABI ---------------------------------------
//
// A compiled program registers its shapes at startup, and a field's declared
// type has to cross as plain integers. Both directions live here so the
// numbering exists once.

pub fn type_code(declared: &Option<DeclaredType>) -> i32 {
    match declared {
        None | Some(DeclaredType::Any) => 0,
        Some(DeclaredType::Number) => 1,
        Some(DeclaredType::Text) => 2,
        Some(DeclaredType::Boolean) => 3,
        Some(DeclaredType::Array) => 4,
        Some(DeclaredType::Record) => 5,
        Some(DeclaredType::Date) => 6,
        Some(DeclaredType::Function) => 7,
        Some(DeclaredType::Shape(_)) => 8,
    }
}

/// The declared type a code names; `shape` is the shape's name for code 8.
pub fn from_code(code: i32, shape: String) -> Option<DeclaredType> {
    match code {
        1 => Some(DeclaredType::Number),
        2 => Some(DeclaredType::Text),
        3 => Some(DeclaredType::Boolean),
        4 => Some(DeclaredType::Array),
        5 => Some(DeclaredType::Record),
        6 => Some(DeclaredType::Date),
        7 => Some(DeclaredType::Function),
        8 => Some(DeclaredType::Shape(shape)),
        _ => None,
    }
}

/// The shapes a program declares, as the running program needs them.
pub fn from_program(statements: &[crate::parser::Stmt]) -> Shapes {
    let mut shapes = Shapes::new();
    for statement in statements {
        if let crate::parser::Stmt::ShapeDef {
            name,
            fields,
            methods,
            ..
        } = statement
        {
            shapes.insert(
                name.clone(),
                Shape {
                    name: name.clone(),
                    fields: fields
                        .iter()
                        .map(|field| (field.name.clone(), field.declared.clone()))
                        .collect(),
                    methods: methods
                        .iter()
                        .map(|method| (method.name.clone(), method.takes_self()))
                        .collect(),
                },
            );
        }
    }
    shapes
}
