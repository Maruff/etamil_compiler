// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Mohammed Maruff (Esan Maruff) <esan@etamil.in>
//! PostgreSQL backend, using the blocking `postgres` driver.
//!
//! Blocking rather than async for the same reason as SQLite: the VM is
//! synchronous and the HTTP server runs a thread per request, so a blocking
//! driver fits without adding yield points to every I/O instruction.
//!
//! Unlike SQLite, PostgreSQL has an exact decimal type of its own. Money
//! therefore travels as `NUMERIC` rather than as text, and text columns stay
//! text on the way back — a `TEXT` column holding "1500" is a string here,
//! where the SQLite backend would hand back a number. That is the right
//! trade for a typed database: the round trip is still lossless, and a column
//! declared `TEXT` no longer changes type depending on what it contains.

use bytes::BytesMut;
use postgres::types::{IsNull, ToSql, Type, to_sql_checked};
use postgres::{Client, NoTls};
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::error::Error;
use std::str::FromStr;

use super::Database;
use crate::vm::Value;

pub struct PostgresDatabase {
    client: Client,
}

impl PostgresDatabase {
    /// Connect using a libpq connection string or a postgres:// URL.
    pub fn open(connection: &str) -> Result<Self, String> {
        let client = Client::connect(connection, NoTls).map_err(|e| {
            format!(
                "போச்குரசீகுல் இணைக்க முடியவில்லை  (cannot connect to PostgreSQL): {}",
                e
            )
        })?;

        Ok(PostgresDatabase { client })
    }
}

/// An eTamil value on its way into a bound parameter.
///
/// PostgreSQL infers each parameter's type from where it appears in the
/// statement, so the conversion has to adapt to that type rather than pick
/// one. Binding `Decimal` directly would only ever satisfy `NUMERIC`, which
/// makes the commonest query of all — `WHERE id = $1` against an integer key
/// — fail with a type error.
#[derive(Debug)]
struct Bound<'a>(&'a Value);

impl ToSql for Bound<'_> {
    fn to_sql(
        &self,
        ty: &Type,
        out: &mut BytesMut,
    ) -> Result<IsNull, Box<dyn Error + Sync + Send>> {
        match self.0 {
            Value::Null => Ok(IsNull::Yes),

            Value::Boolean(b) => match *ty {
                Type::TEXT | Type::VARCHAR | Type::BPCHAR => b.to_string().to_sql(ty, out),
                _ => b.to_sql(ty, out),
            },

            Value::Number(n) => match *ty {
                Type::INT2 => i16::try_from(n.trunc())
                    .map_err(|_| -> Box<dyn Error + Sync + Send> {
                        format!("{} does not fit in a smallint", n).into()
                    })?
                    .to_sql(ty, out),
                Type::INT4 => i32::try_from(n.trunc())
                    .map_err(|_| -> Box<dyn Error + Sync + Send> {
                        format!("{} does not fit in an integer", n).into()
                    })?
                    .to_sql(ty, out),
                Type::INT8 => i64::try_from(n.trunc())
                    .map_err(|_| -> Box<dyn Error + Sync + Send> {
                        format!("{} does not fit in a bigint", n).into()
                    })?
                    .to_sql(ty, out),
                // Only at the boundary, and only because the column asked for
                // it. Nothing float-shaped reaches the value path.
                Type::FLOAT4 | Type::FLOAT8 => f64::try_from(*n)
                    .map_err(|_| -> Box<dyn Error + Sync + Send> {
                        format!("{} cannot be represented as a float", n).into()
                    })?
                    .to_sql(ty, out),
                Type::TEXT | Type::VARCHAR | Type::BPCHAR => {
                    n.normalize().to_string().to_sql(ty, out)
                }
                Type::BOOL => (!n.is_zero()).to_sql(ty, out),
                _ => n.to_sql(ty, out),
            },

            Value::String(s) => match *ty {
                // Input arrives as text, so a string coerces when the column
                // wants a number — the same rule the language uses elsewhere.
                Type::NUMERIC => Decimal::from_str(s.trim())
                    .map_err(|_| -> Box<dyn Error + Sync + Send> {
                        format!("'{}' is not a number", s).into()
                    })?
                    .to_sql(ty, out),
                Type::INT2 | Type::INT4 | Type::INT8 => {
                    let parsed =
                        s.trim()
                            .parse::<i64>()
                            .map_err(|_| -> Box<dyn Error + Sync + Send> {
                                format!("'{}' is not a whole number", s).into()
                            })?;
                    Bound(&Value::Number(Decimal::from(parsed))).to_sql(ty, out)
                }
                _ => s.as_str().to_sql(ty, out),
            },

            // Arrays, records and results have no column type of their own.
            // Rendering one would store something that cannot be read back as
            // what it was, so refuse rather than write a lossy value.
            other => Err(format!(
                "இதை ஒரு அளவுருவாக பிணைக்க முடியாது  \
                 (cannot bind {} as a query parameter)",
                match other {
                    Value::Array(_) => "an array",
                    Value::Map(_) => "a record",
                    Value::Ok(_) | Value::Err(_) => "a result",
                    _ => "this value",
                }
            )
            .into()),
        }
    }

    // Every type is attempted; to_sql above reports the ones it cannot serve.
    fn accepts(_: &Type) -> bool {
        true
    }

    to_sql_checked!();
}

/// One column coming back out.
fn value_from(row: &postgres::Row, index: usize, ty: &Type) -> Result<Value, String> {
    let read = |what: &str, e: postgres::Error| {
        format!(
            "நெடுவரிசை படிக்க முடியவில்லை  (cannot read {} column): {}",
            what, e
        )
    };

    Ok(match *ty {
        Type::BOOL => row
            .try_get::<_, Option<bool>>(index)
            .map_err(|e| read("a boolean", e))?
            .map(Value::Boolean)
            .unwrap_or(Value::Null),

        Type::INT2 => row
            .try_get::<_, Option<i16>>(index)
            .map_err(|e| read("a smallint", e))?
            .map(|n| Value::Number(Decimal::from(n)))
            .unwrap_or(Value::Null),

        Type::INT4 => row
            .try_get::<_, Option<i32>>(index)
            .map_err(|e| read("an integer", e))?
            .map(|n| Value::Number(Decimal::from(n)))
            .unwrap_or(Value::Null),

        Type::INT8 => row
            .try_get::<_, Option<i64>>(index)
            .map_err(|e| read("a bigint", e))?
            .map(|n| Value::Number(Decimal::from(n)))
            .unwrap_or(Value::Null),

        Type::NUMERIC => row
            .try_get::<_, Option<Decimal>>(index)
            .map_err(|e| read("a numeric", e))?
            .map(Value::Number)
            .unwrap_or(Value::Null),

        // Converted through text so the decimal is built from the printed
        // digits rather than from the binary float.
        Type::FLOAT4 | Type::FLOAT8 => row
            .try_get::<_, Option<f64>>(index)
            .map_err(|e| read("a float", e))?
            .and_then(|f| Decimal::from_str(&f.to_string()).ok())
            .map(Value::Number)
            .unwrap_or(Value::Null),

        // A text column stays text even when it looks like a number: the
        // schema already said what it is.
        _ => row
            .try_get::<_, Option<String>>(index)
            .map_err(|e| read("a text", e))?
            .map(Value::String)
            .unwrap_or(Value::Null),
    })
}

impl Database for PostgresDatabase {
    fn execute(&mut self, sql: &str, params: &[Value]) -> Result<i64, String> {
        let bound: Vec<Bound<'_>> = params.iter().map(Bound).collect();
        let refs: Vec<&(dyn ToSql + Sync)> =
            bound.iter().map(|b| b as &(dyn ToSql + Sync)).collect();

        self.client
            .execute(sql, refs.as_slice())
            .map(|affected| affected as i64)
            .map_err(|e| format!("தரவுத்தளப் பிழை  (database error): {}", e))
    }

    fn query(&mut self, sql: &str, params: &[Value]) -> Result<Vec<Value>, String> {
        let bound: Vec<Bound<'_>> = params.iter().map(Bound).collect();
        let refs: Vec<&(dyn ToSql + Sync)> =
            bound.iter().map(|b| b as &(dyn ToSql + Sync)).collect();

        let rows = self
            .client
            .query(sql, refs.as_slice())
            .map_err(|e| format!("தரவுத்தளப் பிழை  (database error): {}", e))?;

        let mut out = Vec::with_capacity(rows.len());
        for row in &rows {
            let columns = row.columns();
            let mut record = HashMap::with_capacity(columns.len());
            for (index, column) in columns.iter().enumerate() {
                record.insert(
                    column.name().to_string(),
                    value_from(row, index, column.type_())?,
                );
            }
            out.push(Value::Map(record));
        }

        Ok(out)
    }

    fn close(&mut self) -> Result<(), String> {
        // The driver closes the socket when the Client drops; there is no
        // separate teardown to run, and reporting success here keeps
        // தளம்_பிரி honest rather than pretending work was done.
        Ok(())
    }
}
