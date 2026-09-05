// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Mohammed Maruff (Esan Maruff) <esan@etamil.in>
//! MySQL and MariaDB backend, using the blocking `mysql` driver.
//!
//! Blocking rather than async for the same reason as the other two: the VM is
//! synchronous and the HTTP server runs a thread per request.
//!
//! Placeholders are `?`, as in SQLite, not PostgreSQL's `$1`.
//!
//! MySQL is loosely typed on the way in — it will coerce a string literal to
//! `DECIMAL` or `INT` — so parameters go over as text and the driver does not
//! have to know the column's type to bind. Reading back is the opposite: the
//! result set carries a type per column, so a `VARCHAR` holding "1500" comes
//! back as a string while a `DECIMAL` comes back as a number, matching the
//! PostgreSQL backend rather than SQLite's text convention.

use mysql::consts::ColumnType;
use mysql::prelude::Queryable;
use mysql::{Conn, Opts, Params, Value as MyValue};
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::str::FromStr;

use super::Database;
use crate::vm::Value;

pub struct MysqlDatabase {
    connection: Conn,
}

impl MysqlDatabase {
    /// Connect using a mysql:// URL.
    pub fn open(connection: &str) -> Result<Self, String> {
        let options = Opts::from_url(connection).map_err(|e| {
            format!(
                "மைசீகுல் முகவரி செல்லாதது  (not a valid MySQL URL '{}'): {}",
                connection, e
            )
        })?;

        let connection = Conn::new(options)
            .map_err(|e| format!("மைசீகுல் இணைக்க முடியவில்லை  (cannot connect to MySQL): {}", e))?;

        Ok(MysqlDatabase { connection })
    }
}

/// An eTamil value on its way into a bound parameter.
///
/// Everything numeric goes over as text and lets the server coerce it, which
/// is both lossless for `DECIMAL` and correct for an integer key. Sending a
/// float would defeat the point of decimal arithmetic.
fn bind(value: &Value) -> Result<MyValue, String> {
    Ok(match value {
        Value::Null => MyValue::NULL,
        Value::Boolean(b) => MyValue::Int(i64::from(*b)),
        Value::Number(n) => MyValue::Bytes(n.normalize().to_string().into_bytes()),
        Value::String(s) => MyValue::Bytes(s.clone().into_bytes()),

        // Arrays, records and results have no column type of their own.
        // Storing a rendering of one would write something that cannot be
        // read back as what it was.
        other => {
            return Err(format!(
                "இதை ஒரு அளவுருவாக பிணைக்க முடியாது  \
                 (cannot bind {} as a query parameter)",
                match other {
                    Value::Array(_) => "an array",
                    Value::Map(_) => "a record",
                    Value::Ok(_) | Value::Err(_) => "a result",
                    _ => "this value",
                }
            ));
        }
    })
}

fn bind_all(params: &[Value]) -> Result<Params, String> {
    if params.is_empty() {
        // An empty positional list is not the same as "no parameters" to this
        // driver: Params::Positional(vec![]) makes it expect a prepared
        // statement with none, which a plain DDL statement is not.
        return Ok(Params::Empty);
    }

    let bound: Result<Vec<MyValue>, String> = params.iter().map(bind).collect();
    Ok(Params::Positional(bound?))
}

/// Text that a numeric column handed back, as an exact decimal.
fn decimal_from_bytes(bytes: &[u8]) -> Value {
    let text = String::from_utf8_lossy(bytes).to_string();
    match Decimal::from_str(text.trim()) {
        Ok(number) => Value::Number(number),
        Err(_) => Value::String(text),
    }
}

/// One column coming back out, read against the type the result set declares.
fn value_from(raw: &MyValue, column_type: ColumnType) -> Value {
    match raw {
        MyValue::NULL => Value::Null,

        MyValue::Int(n) => Value::Number(Decimal::from(*n)),
        MyValue::UInt(n) => Value::Number(Decimal::from(*n)),

        // Only at the boundary, and built from the printed digits rather than
        // the binary float, so nothing float-shaped reaches the value path.
        MyValue::Float(f) => Decimal::from_str(&f.to_string())
            .map(Value::Number)
            .unwrap_or(Value::Null),
        MyValue::Double(f) => Decimal::from_str(&f.to_string())
            .map(Value::Number)
            .unwrap_or(Value::Null),

        // DECIMAL arrives as text. So does VARCHAR — the column type is what
        // separates a number from a string that happens to look like one.
        MyValue::Bytes(bytes) => match column_type {
            ColumnType::MYSQL_TYPE_DECIMAL
            | ColumnType::MYSQL_TYPE_NEWDECIMAL
            | ColumnType::MYSQL_TYPE_TINY
            | ColumnType::MYSQL_TYPE_SHORT
            | ColumnType::MYSQL_TYPE_LONG
            | ColumnType::MYSQL_TYPE_LONGLONG
            | ColumnType::MYSQL_TYPE_INT24
            | ColumnType::MYSQL_TYPE_YEAR
            | ColumnType::MYSQL_TYPE_FLOAT
            | ColumnType::MYSQL_TYPE_DOUBLE => decimal_from_bytes(bytes),
            _ => Value::String(String::from_utf8_lossy(bytes).to_string()),
        },

        // Dates and times as ISO-8601 text, which is how the language handles
        // them everywhere else — ISO text sorts chronologically, so
        // comparison needs no primitive.
        MyValue::Date(year, month, day, hour, minute, second, micros) => {
            let date = format!("{:04}-{:02}-{:02}", year, month, day);
            if *hour == 0 && *minute == 0 && *second == 0 && *micros == 0 {
                Value::String(date)
            } else {
                Value::String(format!("{}T{:02}:{:02}:{:02}", date, hour, minute, second))
            }
        }
        MyValue::Time(negative, days, hours, minutes, seconds, _micros) => {
            let sign = if *negative { "-" } else { "" };
            Value::String(format!(
                "{}{:02}:{:02}:{:02}",
                sign,
                u32::from(*hours) + days * 24,
                minutes,
                seconds
            ))
        }
    }
}

impl Database for MysqlDatabase {
    fn execute(&mut self, sql: &str, params: &[Value]) -> Result<i64, String> {
        self.connection
            .exec_drop(sql, bind_all(params)?)
            .map_err(|e| format!("தரவுத்தளப் பிழை  (database error): {}", e))?;

        Ok(self.connection.affected_rows() as i64)
    }

    fn query(&mut self, sql: &str, params: &[Value]) -> Result<Vec<Value>, String> {
        let rows: Vec<mysql::Row> = self
            .connection
            .exec(sql, bind_all(params)?)
            .map_err(|e| format!("தரவுத்தளப் பிழை  (database error): {}", e))?;

        let mut out = Vec::with_capacity(rows.len());
        for row in &rows {
            let columns = row.columns_ref();
            let mut record = HashMap::with_capacity(columns.len());
            for (index, column) in columns.iter().enumerate() {
                let raw = row.as_ref(index).unwrap_or(&MyValue::NULL);
                record.insert(
                    column.name_str().to_string(),
                    value_from(raw, column.column_type()),
                );
            }
            out.push(Value::Map(record));
        }

        Ok(out)
    }

    fn close(&mut self) -> Result<(), String> {
        // The driver closes the socket when Conn drops; there is no separate
        // teardown, and claiming otherwise would make தளம்_பிரி dishonest.
        Ok(())
    }
}
