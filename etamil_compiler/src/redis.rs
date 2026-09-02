// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Mohammed Maruff (Esan Maruff) <esan@etamil.in>
//! Redis, over the protocol it actually speaks.
//!
//! The roadmap said Redis needed a design before an implementation, because it
//! does not fit a trait shaped as `execute(sql, params)` / `query(sql, params)`.
//! It does not, and forcing it there would have been the wrong answer: Redis is
//! not a query language, it is a command and a reply. `GET key`. `INCR hits`.
//! `LPUSH queue item`.
//!
//! So the design is one command, generically. Every Redis command works —
//! including the ones added after this was written — because the shape of the
//! interface is the shape of the protocol. Convenience for the common ones
//! belongs in `nUlakam`, written in eTamil, not in a builtin per command.
//!
//! RESP is implemented here rather than taken from a crate, for the same reason
//! the HTTP router was: the protocol is small, tokio and the TCP stack were
//! already present, and a dependency that carries an async runtime to send
//! `*2\r\n$3\r\nGET\r\n$1\r\nk\r\n` down a socket is a poor trade.
//!
//! A connection is not shared between requests. Redis has per-connection state
//! — MULTI, WATCH, SUBSCRIBE — so handing one connection to two requests has
//! the same hazard as sharing a SQL transaction, and the same fix would be an
//! exclusive lease. Until that exists, `ரெடிஸ்_இணை` opens its own and holds it
//! for as long as the program runs.

use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::time::Duration;

use rust_decimal::Decimal;

use crate::vm::Value;

/// One reply, in the shapes RESP has.
///
/// Kept separate from `Value` so the protocol can be tested without the VM,
/// and so that nil — which RESP distinguishes from an empty string — does not
/// have to be decided here.
#[derive(Debug, Clone, PartialEq)]
pub enum Reply {
    /// `+OK`
    Simple(String),
    /// `-ERR unknown command`
    Error(String),
    /// `:42`
    Integer(i64),
    /// `$5\r\nhello`
    Bulk(String),
    /// `$-1` — a key that is not there, which is not the same as an empty one
    Nil,
    /// `*2\r\n...`
    Array(Vec<Reply>),
}

impl Reply {
    /// The reply as an eTamil value. An error is the caller's to turn into a
    /// தவறு, because only the caller knows whether it is one.
    pub fn to_value(&self) -> Value {
        match self {
            Reply::Simple(text) => Value::String(text.clone()),
            Reply::Error(text) => Value::String(text.clone()),
            Reply::Integer(number) => Value::Number(Decimal::from(*number)),
            Reply::Bulk(text) => Value::String(text.clone()),
            // A missing key is nil, not "". A program checking வகை(x) == "nil"
            // can tell "the key is absent" from "the key holds nothing", and
            // for a cache that difference is the whole question.
            Reply::Nil => Value::Null,
            Reply::Array(items) => {
                Value::Array(items.iter().map(|item| item.to_value()).collect())
            }
        }
    }
}

/// Encode a command the way RESP wants it: an array of bulk strings.
///
/// Every argument is length-prefixed, so a value containing a newline, a space
/// or the protocol's own delimiters travels intact. Building commands by
/// joining with spaces — which is how the inline protocol works and how command
/// injection happens — is not possible through this.
pub fn encode(command: &str, arguments: &[String]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(format!("*{}\r\n", arguments.len() + 1).as_bytes());

    let mut push = |part: &str| {
        out.extend_from_slice(format!("${}\r\n", part.as_bytes().len()).as_bytes());
        out.extend_from_slice(part.as_bytes());
        out.extend_from_slice(b"\r\n");
    };

    push(command);
    for argument in arguments {
        push(argument);
    }
    out
}

/// Read one reply.
pub fn decode(reader: &mut impl BufRead) -> Result<Reply, String> {
    let mut line = String::new();
    let read = reader
        .read_line(&mut line)
        .map_err(|e| format!("ரெடிஸ் பதிலைப் படிக்க முடியவில்லை  (cannot read the reply): {}", e))?;
    if read == 0 {
        return Err("ரெடிஸ் இணைப்பு மூடப்பட்டது  (the connection closed)".to_string());
    }

    let line = line.trim_end_matches(['\r', '\n']);
    let (marker, rest) = line.split_at(1);

    match marker {
        "+" => Ok(Reply::Simple(rest.to_string())),
        "-" => Ok(Reply::Error(rest.to_string())),
        ":" => rest
            .parse::<i64>()
            .map(Reply::Integer)
            .map_err(|_| format!("'{}' ஒரு எண் அல்ல  ('{}' is not an integer)", rest, rest)),
        "$" => {
            let length: i64 = rest
                .parse()
                .map_err(|_| format!("'{}' ஒரு நீளம் அல்ல  ('{}' is not a length)", rest, rest))?;
            if length < 0 {
                return Ok(Reply::Nil);
            }
            // Exactly that many bytes, then the CRLF that follows them. Reading
            // to the next newline instead would truncate any value containing
            // one — which is most serialized things.
            let mut bytes = vec![0u8; length as usize + 2];
            std::io::Read::read_exact(reader, &mut bytes)
                .map_err(|e| format!("ரெடிஸ் மதிப்பைப் படிக்க முடியவில்லை  (cannot read the value): {}", e))?;
            bytes.truncate(length as usize);
            Ok(Reply::Bulk(String::from_utf8_lossy(&bytes).into_owned()))
        }
        "*" => {
            let count: i64 = rest
                .parse()
                .map_err(|_| format!("'{}' ஒரு எண்ணிக்கை அல்ல  ('{}' is not a count)", rest, rest))?;
            if count < 0 {
                return Ok(Reply::Nil);
            }
            let mut items = Vec::with_capacity(count as usize);
            for _ in 0..count {
                items.push(decode(reader)?);
            }
            Ok(Reply::Array(items))
        }
        other => Err(format!(
            "தெரியாத ரெடிஸ் குறி '{}'  (unknown RESP marker '{}')",
            other, other
        )),
    }
}

/// An open connection to a Redis server.
pub struct Connection {
    address: String,
    reader: BufReader<TcpStream>,
    writer: TcpStream,
}

/// Written by hand rather than derived: a socket has no useful Debug, and the
/// one thing worth printing about a connection is where it goes.
impl std::fmt::Debug for Connection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Redis({})", self.address)
    }
}

impl Connection {
    /// Connect. The address is `host:port`.
    pub fn open(address: &str) -> Result<Self, String> {
        let stream = TcpStream::connect(address).map_err(|e| {
            format!(
                "ரெடிஸ் '{}' இணைக்க முடியவில்லை  (cannot connect to Redis at '{}'): {}",
                address, address, e
            )
        })?;

        // A server that accepts the connection and then says nothing would
        // otherwise hold a worker for as long as it liked.
        let timeout = Some(Duration::from_secs(10));
        let _ = stream.set_read_timeout(timeout);
        let _ = stream.set_write_timeout(timeout);

        let writer = stream
            .try_clone()
            .map_err(|e| format!("ரெடிஸ் இணைப்பை நகலெடுக்க முடியவில்லை  (cannot split the socket): {}", e))?;

        Ok(Connection {
            address: address.to_string(),
            reader: BufReader::new(stream),
            writer,
        })
    }

    pub fn address(&self) -> &str {
        &self.address
    }

    /// Send one command and read its reply.
    pub fn command(&mut self, command: &str, arguments: &[String]) -> Result<Reply, String> {
        let request = encode(command, arguments);
        self.writer.write_all(&request).map_err(|e| {
            format!("ரெடிஸ் கட்டளையை அனுப்ப முடியவில்லை  (cannot send the command): {}", e)
        })?;
        self.writer
            .flush()
            .map_err(|e| format!("ரெடிஸ் கட்டளையை அனுப்ப முடியவில்லை  (cannot flush): {}", e))?;
        decode(&mut self.reader)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_command_is_an_array_of_length_prefixed_arguments() {
        assert_eq!(
            encode("GET", &["key".to_string()]),
            b"*2\r\n$3\r\nGET\r\n$3\r\nkey\r\n".to_vec()
        );
    }

    #[test]
    fn a_command_with_no_arguments_is_still_an_array() {
        assert_eq!(encode("PING", &[]), b"*1\r\n$4\r\nPING\r\n".to_vec());
    }

    #[test]
    fn an_argument_holding_the_delimiter_survives() {
        // The reason arguments are length-prefixed rather than joined. A value
        // containing CRLF would otherwise end the command early — which is how
        // command injection works in the inline protocol.
        let encoded = encode("SET", &["k".to_string(), "a\r\nDEL x".to_string()]);

        assert_eq!(
            encoded,
            b"*3\r\n$3\r\nSET\r\n$1\r\nk\r\n$8\r\na\r\nDEL x\r\n".to_vec()
        );
    }

    #[test]
    fn an_argument_is_measured_in_bytes_not_letters() {
        // வணக்கம் is 7 written letters and 21 bytes. RESP counts bytes, and a
        // length in letters would desynchronise the stream.
        let encoded = encode("SET", &["k".to_string(), "வணக்கம்".to_string()]);
        let text = String::from_utf8_lossy(&encoded);

        assert!(text.contains("$21\r\nவணக்கம்"), "got: {}", text);
    }

    fn read(raw: &[u8]) -> Result<Reply, String> {
        let mut reader = BufReader::new(raw);
        decode(&mut reader)
    }

    #[test]
    fn every_reply_shape_is_understood() {
        assert_eq!(read(b"+OK\r\n").unwrap(), Reply::Simple("OK".to_string()));
        assert_eq!(read(b":42\r\n").unwrap(), Reply::Integer(42));
        assert_eq!(read(b":-1\r\n").unwrap(), Reply::Integer(-1));
        assert_eq!(read(b"$5\r\nhello\r\n").unwrap(), Reply::Bulk("hello".to_string()));
        assert_eq!(
            read(b"-ERR unknown command\r\n").unwrap(),
            Reply::Error("ERR unknown command".to_string())
        );
    }

    #[test]
    fn a_missing_key_is_nil_and_an_empty_one_is_not() {
        // For a cache this is the whole question: absent, or present and empty.
        assert_eq!(read(b"$-1\r\n").unwrap(), Reply::Nil);
        assert_eq!(read(b"$0\r\n\r\n").unwrap(), Reply::Bulk(String::new()));
    }

    #[test]
    fn a_value_containing_a_newline_reads_whole() {
        // Read to the next newline and this comes back as "line one".
        let reply = read(b"$17\r\nline one\nline two\r\n").unwrap();

        assert_eq!(reply, Reply::Bulk("line one\nline two".to_string()));
    }

    #[test]
    fn arrays_nest() {
        let reply = read(b"*2\r\n$1\r\na\r\n*2\r\n:1\r\n:2\r\n").unwrap();

        assert_eq!(
            reply,
            Reply::Array(vec![
                Reply::Bulk("a".to_string()),
                Reply::Array(vec![Reply::Integer(1), Reply::Integer(2)]),
            ])
        );
    }

    #[test]
    fn an_empty_array_is_not_nil() {
        assert_eq!(read(b"*0\r\n").unwrap(), Reply::Array(vec![]));
        assert_eq!(read(b"*-1\r\n").unwrap(), Reply::Nil);
    }

    #[test]
    fn a_truncated_reply_is_an_error_rather_than_a_guess() {
        assert!(read(b"$5\r\nhel").is_err(), "a bulk string cut short");
        assert!(read(b"").is_err(), "nothing at all");
        assert!(read(b"?what\r\n").is_err(), "a marker RESP does not have");
    }

    #[test]
    fn nil_becomes_nil_and_not_an_empty_string() {
        // The mapping a cache depends on: வகை(x) == "nil" means the key was
        // absent, which is different from it holding "".
        assert_eq!(Reply::Nil.to_value(), Value::Null);
        assert_eq!(
            Reply::Bulk(String::new()).to_value(),
            Value::String(String::new())
        );
    }

    #[test]
    fn an_integer_reply_is_a_number_not_text() {
        // So that INCR can be added to without being parsed first.
        assert_eq!(Reply::Integer(7).to_value(), Value::Number(Decimal::from(7)));
    }
}
