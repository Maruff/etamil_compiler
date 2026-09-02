// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Mohammed Maruff (Esan Maruff) <esan@etamil.in>
//! MongoDB — documents, which the language already has a shape for.
//!
//! The roadmap said this needed a design first, because it does not fit a trait
//! shaped as `execute(sql)` / `query(sql)`. It does not, but the mismatch is
//! smaller than Redis's: a MongoDB document *is* a `பொருள்`, a collection of
//! them is an array of records, and a filter is a record too. The language's
//! value model was already document-shaped.
//!
//! So the mapping is direct, and the interface is again one generic command
//! plus convenience: MongoDB's own API is `runCommand` with a document, and
//! everything else is sugar over it.
//!
//! ## Money is Decimal128, not double
//!
//! This is the part worth being careful about. Every number in eTamil is a
//! fixed-point decimal, precisely so that money does not drift — and the
//! ordinary thing for a MongoDB driver to do with a number is store a double,
//! which puts the drift straight back. A balance written as a double and read
//! back is not reliably the balance that was written.
//!
//! So a whole number is stored as an integer and anything with a fraction as
//! `Decimal128`, which is what that type is for. Reading back reverses it. A
//! double arriving from a document some other program wrote is accepted and
//! converted, because refusing it would make this unable to read anything but
//! its own writes — but nothing written from here is a double.
//!
//! Behind `--features mongodb`, like the other two non-bundled drivers.

use std::str::FromStr;

use mongodb::bson::{Bson, Document};
use mongodb::sync::Client;
use rust_decimal::Decimal;

use crate::vm::Value;

/// An eTamil value as BSON.
///
/// A record becomes a document, an array an array, and a number an integer or a
/// Decimal128 — never a double. See the note at the top of this file.
pub fn to_bson(value: &Value) -> Bson {
    match value {
        Value::Number(number) => {
            if number.fract().is_zero() {
                // An integer where one fits, so a count reads as a count and
                // a query for 5 matches a stored 5.
                match rust_decimal::prelude::ToPrimitive::to_i64(number) {
                    Some(whole) => Bson::Int64(whole),
                    None => decimal_to_bson(number),
                }
            } else {
                decimal_to_bson(number)
            }
        }
        Value::String(text) => Bson::String(text.clone()),
        Value::Boolean(flag) => Bson::Boolean(*flag),
        Value::Array(items) => Bson::Array(items.iter().map(to_bson).collect()),
        Value::Map(fields) => {
            let mut document = Document::new();
            for (key, held) in fields {
                document.insert(key.clone(), to_bson(held));
            }
            Bson::Document(document)
        }
        // A result crossing into a database would be storing "this succeeded"
        // as data. Its contents are what was meant.
        Value::Ok(inner) => to_bson(inner),
        Value::Err(inner) => to_bson(inner),
        Value::Null => Bson::Null,
    }
}

fn decimal_to_bson(number: &Decimal) -> Bson {
    // Through the decimal's own text, which is exact, rather than through a
    // float — which is the whole point of not using a double.
    match mongodb::bson::Decimal128::from_str(&number.to_string()) {
        Ok(exact) => Bson::Decimal128(exact),
        // Unreachable for any value a Decimal can hold, but a silent zero here
        // would be a wrong balance rather than a missing one.
        Err(_) => Bson::String(number.to_string()),
    }
}

/// BSON as an eTamil value.
pub fn from_bson(value: &Bson) -> Value {
    match value {
        Bson::Double(number) => {
            // Not written by us, but readable: another program's document is
            // still a document. Through text, so 0.1 arrives as 0.1.
            Decimal::from_str(&number.to_string())
                .map(Value::Number)
                .unwrap_or(Value::Null)
        }
        Bson::Decimal128(number) => Decimal::from_str(&number.to_string())
            .map(Value::Number)
            .unwrap_or(Value::Null),
        Bson::Int32(number) => Value::Number(Decimal::from(*number)),
        Bson::Int64(number) => Value::Number(Decimal::from(*number)),
        Bson::String(text) => Value::String(text.clone()),
        Bson::Boolean(flag) => Value::Boolean(*flag),
        Bson::Array(items) => Value::Array(items.iter().map(from_bson).collect()),
        Bson::Document(document) => {
            let mut fields = std::collections::HashMap::new();
            for (key, held) in document {
                fields.insert(key.clone(), from_bson(held));
            }
            Value::Map(fields)
        }
        Bson::Null => Value::Null,
        // An ObjectId is the identifier every document has, and its text form
        // is what a program passes back to find the document again.
        Bson::ObjectId(id) => Value::String(id.to_hex()),
        Bson::DateTime(when) => Value::String(when.try_to_rfc3339_string().unwrap_or_default()),
        // Anything else — binary, regex, timestamps — as its debug text rather
        // than as nil, so a program can at least see that something is there.
        other => Value::String(format!("{}", other)),
    }
}

/// The document a record describes, or an explanation.
pub fn to_document(value: &Value) -> Result<Document, String> {
    match to_bson(value) {
        Bson::Document(document) => Ok(document),
        _ => Err(format!(
            "ஒரு பொருள் தேவை  (a record is needed here, got {})",
            match value {
                Value::Array(_) => "an array",
                Value::String(_) => "a string",
                Value::Number(_) => "a number",
                _ => "something else",
            }
        )),
    }
}

/// An open connection to a MongoDB deployment.
pub struct Connection {
    client: Client,
    database: String,
}

impl std::fmt::Debug for Connection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MongoDB({})", self.database)
    }
}

impl Connection {
    /// Connect with a URI, and name the database to work in.
    pub fn open(uri: &str, database: &str) -> Result<Self, String> {
        let client = Client::with_uri_str(uri).map_err(|e| {
            format!(
                "மொங்கோ இணைக்க முடியவில்லை  (cannot connect to MongoDB): {}",
                e
            )
        })?;
        Ok(Connection {
            client,
            database: database.to_string(),
        })
    }

    pub fn database_name(&self) -> &str {
        &self.database
    }

    /// Run a command document against the database.
    ///
    /// The generic door. `{"ping": 1}`, `{"count": "orders"}`, an aggregation —
    /// whatever the server takes, including commands newer than this file.
    pub fn command(&self, command: Document) -> Result<Value, String> {
        let database = self.client.database(&self.database);
        database
            .run_command(command)
            .run()
            .map(|reply| from_bson(&Bson::Document(reply)))
            .map_err(|e| format!("மொங்கோ கட்டளை தோல்வி  (the command failed): {}", e))
    }

    /// Insert one document. Answers the identifier it was given.
    pub fn insert(&self, collection: &str, document: Document) -> Result<Value, String> {
        let handle = self
            .client
            .database(&self.database)
            .collection::<Document>(collection);
        handle
            .insert_one(document)
            .run()
            .map(|outcome| from_bson(&outcome.inserted_id))
            .map_err(|e| format!("மொங்கோ செருக முடியவில்லை  (cannot insert): {}", e))
    }

    /// Every document matching a filter, as an array of records.
    pub fn find(&self, collection: &str, filter: Document) -> Result<Value, String> {
        let handle = self
            .client
            .database(&self.database)
            .collection::<Document>(collection);
        let cursor = handle
            .find(filter)
            .run()
            .map_err(|e| format!("மொங்கோ தேட முடியவில்லை  (cannot query): {}", e))?;

        let mut found = Vec::new();
        for document in cursor {
            let document =
                document.map_err(|e| format!("மொங்கோ வரிசைப் பிழை  (cursor error): {}", e))?;
            found.push(from_bson(&Bson::Document(document)));
        }
        Ok(Value::Array(found))
    }

    /// Update documents matching a filter. Answers how many changed.
    pub fn update(
        &self,
        collection: &str,
        filter: Document,
        change: Document,
        many: bool,
    ) -> Result<i64, String> {
        let handle = self
            .client
            .database(&self.database)
            .collection::<Document>(collection);
        let outcome = if many {
            handle.update_many(filter, change).run()
        } else {
            handle.update_one(filter, change).run()
        };
        outcome
            .map(|done| done.modified_count as i64)
            .map_err(|e| format!("மொங்கோ புதுப்பிக்க முடியவில்லை  (cannot update): {}", e))
    }

    /// Delete documents matching a filter. Answers how many went.
    pub fn delete(&self, collection: &str, filter: Document, many: bool) -> Result<i64, String> {
        let handle = self
            .client
            .database(&self.database)
            .collection::<Document>(collection);
        let outcome = if many {
            handle.delete_many(filter).run()
        } else {
            handle.delete_one(filter).run()
        };
        outcome
            .map(|done| done.deleted_count as i64)
            .map_err(|e| format!("மொங்கோ நீக்க முடியவில்லை  (cannot delete): {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn record(pairs: &[(&str, Value)]) -> Value {
        let mut fields = HashMap::new();
        for (key, value) in pairs {
            fields.insert((*key).to_string(), value.clone());
        }
        Value::Map(fields)
    }

    fn decimal(text: &str) -> Value {
        Value::Number(Decimal::from_str(text).unwrap())
    }

    #[test]
    fn a_record_becomes_a_document() {
        let value = record(&[("peyar", Value::String("ராஜா".into()))]);

        match to_bson(&value) {
            Bson::Document(document) => {
                assert_eq!(document.get_str("peyar").unwrap(), "ராஜா");
            }
            other => panic!("expected a document, got {:?}", other),
        }
    }

    #[test]
    fn a_whole_number_is_an_integer() {
        // So that a stored 5 is found by a filter of 5, rather than by 5.0.
        assert_eq!(to_bson(&decimal("5")), Bson::Int64(5));
        assert_eq!(to_bson(&decimal("-42")), Bson::Int64(-42));
        assert_eq!(to_bson(&decimal("0")), Bson::Int64(0));
    }

    #[test]
    fn money_is_never_stored_as_a_double() {
        // The point of the whole module. A double would put back exactly the
        // drift the language exists to avoid.
        match to_bson(&decimal("1234.56")) {
            Bson::Decimal128(_) => {}
            other => panic!("a fractional amount must not be {:?}", other),
        }
    }

    #[test]
    fn a_decimal_round_trips_exactly() {
        // 0.1 + 0.2 is 0.3 in this language, and it has to still be 0.3 after
        // going to the database and back.
        for text in ["0.3", "1234.56", "0.01", "99999999.99", "-0.05"] {
            let original = decimal(text);
            let returned = from_bson(&to_bson(&original));
            assert_eq!(returned, original, "{} did not survive the round trip", text);
        }
    }

    #[test]
    fn a_double_from_elsewhere_is_still_readable() {
        // Another program's document is still a document. Refusing doubles
        // would make this able to read only its own writes.
        assert_eq!(from_bson(&Bson::Double(0.5)), decimal("0.5"));
    }

    #[test]
    fn nested_records_and_arrays_survive() {
        let value = record(&[
            ("items", Value::Array(vec![decimal("1"), decimal("2.5")])),
            ("inner", record(&[("k", Value::Boolean(true))])),
        ]);

        assert_eq!(from_bson(&to_bson(&value)), value);
    }

    #[test]
    fn nil_stays_nil() {
        // A field that is absent must not come back as an empty string, for the
        // same reason a missing Redis key must not.
        assert_eq!(from_bson(&to_bson(&Value::Null)), Value::Null);
    }

    #[test]
    fn an_object_id_reads_as_text_that_can_be_sent_back() {
        let id = mongodb::bson::oid::ObjectId::new();
        let as_value = from_bson(&Bson::ObjectId(id));

        assert_eq!(as_value, Value::String(id.to_hex()));
    }

    #[test]
    fn something_that_is_not_a_record_is_refused_as_a_document() {
        let outcome = to_document(&Value::Array(vec![decimal("1")]));

        let why = outcome.err().expect("an array is not a document");
        assert!(why.contains("பொருள்"), "unexpected: {}", why);
    }

    #[test]
    fn a_result_stores_what_it_carries() {
        // Storing சரி(5) as a database value would be recording "this
        // succeeded" rather than the five.
        assert_eq!(to_bson(&Value::Ok(Box::new(decimal("5")))), Bson::Int64(5));
    }
}

