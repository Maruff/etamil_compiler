// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Mohammed Maruff (Esan Maruff) <esan@etamil.in>
//! SQLite backend, using the blocking rusqlite driver.
//!
//! Blocking rather than async is deliberate: the VM is synchronous and the
//! HTTP server runs a thread per request, so a blocking driver fits without
//! adding yield points to every I/O instruction.

use rusqlite::types::{ToSqlOutput, Value as SqlValue, ValueRef};
use rusqlite::{Connection, ToSql};
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::str::FromStr;

use super::Database;
use crate::vm::Value;

pub struct SqliteDatabase {
    connection: Connection,
}

impl SqliteDatabase {
    /// Open a database file, or an in-memory one for ":memory:".
    pub fn open(path: &str) -> Result<Self, String> {
        let connection = if path == ":memory:" {
            Connection::open_in_memory()
        } else {
            Connection::open(path)
        }
        .map_err(|e| {
            format!(
                "தரவுத்தளம் திறக்க முடியவில்லை  (cannot open database '{}'): {}",
                path, e
            )
        })?;

        Ok(SqliteDatabase { connection })
    }
}

/// An eTamil value on its way into a bound parameter.
struct Bound<'a>(&'a Value);

impl ToSql for Bound<'_> {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(match self.0 {
            // Decimals go over as text so no precision is lost on the way;
            // SQLite has no exact decimal type, and REAL would defeat the
            // point of using decimals in the first place.
            Value::Number(n) => ToSqlOutput::Owned(SqlValue::Text(n.normalize().to_string())),
            Value::String(s) => ToSqlOutput::Owned(SqlValue::Text(s.clone())),
            Value::Boolean(b) => ToSqlOutput::Owned(SqlValue::Integer(i64::from(*b))),
            Value::Null => ToSqlOutput::Owned(SqlValue::Null),
            other => ToSqlOutput::Owned(SqlValue::Text(other.to_string())),
        })
    }
}

/// A column coming back out.
fn value_from(raw: ValueRef<'_>) -> Value {
    match raw {
        ValueRef::Null => Value::Null,
        ValueRef::Integer(i) => Value::Number(Decimal::from(i)),
        // Text that parses as a decimal comes back as a number, which is what
        // makes the text round-trip above lossless.
        ValueRef::Text(bytes) => {
            let text = String::from_utf8_lossy(bytes).to_string();
            match Decimal::from_str(&text) {
                Ok(number) => Value::Number(number),
                Err(_) => Value::String(text),
            }
        }
        ValueRef::Real(f) => Decimal::from_str(&f.to_string())
            .map(Value::Number)
            .unwrap_or(Value::Null),
        ValueRef::Blob(bytes) => Value::String(String::from_utf8_lossy(bytes).to_string()),
    }
}

impl Database for SqliteDatabase {
    fn execute(&mut self, sql: &str, params: &[Value]) -> Result<i64, String> {
        let bound: Vec<Bound<'_>> = params.iter().map(Bound).collect();
        let refs: Vec<&dyn ToSql> = bound.iter().map(|b| b as &dyn ToSql).collect();

        self.connection
            .execute(sql, refs.as_slice())
            .map(|affected| affected as i64)
            .map_err(|e| format!("தரவுத்தளப் பிழை  (database error): {}", e))
    }

    fn query(&mut self, sql: &str, params: &[Value]) -> Result<Vec<Value>, String> {
        let mut statement = self
            .connection
            .prepare(sql)
            .map_err(|e| format!("வினா தயாரிக்க முடியவில்லை  (cannot prepare query): {}", e))?;

        let column_names: Vec<String> = statement
            .column_names()
            .iter()
            .map(|c| c.to_string())
            .collect();

        let bound: Vec<Bound<'_>> = params.iter().map(Bound).collect();
        let refs: Vec<&dyn ToSql> = bound.iter().map(|b| b as &dyn ToSql).collect();

        let mut rows = statement
            .query(refs.as_slice())
            .map_err(|e| format!("தரவுத்தளப் பிழை  (database error): {}", e))?;

        let mut out = Vec::new();
        while let Some(row) = rows
            .next()
            .map_err(|e| format!("வரிசை படிக்க முடியவில்லை  (cannot read row): {}", e))?
        {
            let mut record = HashMap::with_capacity(column_names.len());
            for (index, name) in column_names.iter().enumerate() {
                let raw = row.get_ref(index).map_err(|e| {
                    format!("நெடுவரிசை படிக்க முடியவில்லை  (cannot read column): {}", e)
                })?;
                record.insert(name.clone(), value_from(raw));
            }
            out.push(Value::Map(record));
        }

        Ok(out)
    }
}
