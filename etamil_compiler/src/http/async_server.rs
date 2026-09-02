// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Mohammed Maruff (Esan Maruff) <esan@etamil.in>
//! The concurrent HTTP server: `--async`.
//!
//! Accepting connections is async; running a handler is not. That split is the
//! whole design, and it is why this needed no new dependencies — tokio was
//! already here, and axum would only have replaced routing this crate does
//! itself.
//!
//! The VM is synchronous and stays that way. Making it async would mean a
//! yield point at every I/O instruction, and it would rule out the blocking
//! database drivers that are far simpler to bind into a synchronous
//! interpreter. So each request is handed to `spawn_blocking`: the accept loop
//! never waits on a handler, and the handler never learns it is inside a
//! runtime. See docs/ARCHITECTURE.md.
//!
//! What this buys over `--server`: that one runs a fixed pool of `2 × cores`
//! threads, so a slow client occupies a worker for as long as it takes to send
//! its request. Here, a connection costs a task rather than a thread, and only
//! the handler itself needs a thread.

use std::collections::HashMap;
use std::sync::Arc;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use crate::http::handler::dispatch;
use crate::http::{HttpRequest, HttpResponse};
use crate::parser::Stmt;
use crate::vm::{Bytecode, BytecodeCompiler};

/// One HTTP request may be at most this large, so a client cannot exhaust
/// memory by promising a body it never sends.
const MAX_REQUEST: usize = 1024 * 1024;
const CHUNK: usize = 4096;

pub struct AsyncHttpServer {
    host: String,
    port: u16,
    /// Compiled once at registration and shared with every task; a request
    /// should not pay to recompile its handler.
    handlers: HashMap<String, Bytecode>,
    /// Timed jobs: how often, and what to run.
    schedules: Vec<(u64, Bytecode)>,
}

impl AsyncHttpServer {
    pub fn new(host: &str, port: u16) -> Self {
        AsyncHttpServer {
            host: host.to_string(),
            port,
            handlers: HashMap::new(),
            schedules: Vec::new(),
        }
    }

    pub fn register_route(&mut self, method: &str, path: &str, handler: Vec<Stmt>) {
        let bytecode = BytecodeCompiler::compile_statements(handler);
        self.handlers
            .insert(format!("{} {}", method.to_uppercase(), path), bytecode);
    }

    /// Register a block to run on a timer.
    ///
    /// The interval is the gap *between* runs. A tick waits for the previous
    /// one, so a job slower than its interval runs late rather than twice at
    /// once — two copies of a reconciliation overlapping is worse than one
    /// running behind.
    pub fn register_schedule(&mut self, seconds: u64, body: Vec<Stmt>) {
        let bytecode = BytecodeCompiler::compile_statements(body);
        self.schedules.push((seconds.max(1), bytecode));
    }

    pub fn routes(&self) -> impl Iterator<Item = &String> {
        self.handlers.keys()
    }

    /// Serve until Ctrl-C.
    pub async fn start(self) -> Result<(), Box<dyn std::error::Error>> {
        let address = format!("{}:{}", self.host, self.port);
        let listener = TcpListener::bind(&address).await?;

        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("🚀 eTamil HTTP Server (--async)");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("📍 Listening on: http://{}", address);
        println!("📋 Registered Routes:");
        let mut routes: Vec<&String> = self.handlers.keys().collect();
        routes.sort();
        for route in routes {
            println!("   {}", route);
        }
        println!("🧵 Handlers run on tokio's blocking pool; the VM stays synchronous");
        println!("   Press Ctrl-C to stop.");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

        // Timed jobs run on the blocking pool like handlers, for the same
        // reason: the VM blocks, and so does whatever driver a job reaches for.
        for (index, (seconds, bytecode)) in self.schedules.into_iter().enumerate() {
            let label = format!("#{} every {}s", index + 1, seconds);
            println!("⏱️  Scheduled job {}", label);
            tokio::spawn(async move {
                loop {
                    tokio::time::sleep(std::time::Duration::from_secs(seconds)).await;
                    let bytecode = bytecode.clone();
                    let label = label.clone();
                    // Awaited, so the next tick cannot start before this one
                    // finishes.
                    let _ = tokio::task::spawn_blocking(move || {
                        crate::http::handler::run_scheduled(&label, &bytecode)
                    })
                    .await;
                }
            });
        }

        // Shared rather than cloned per request: the handlers are read-only
        // once the server has started.
        let handlers = Arc::new(self.handlers);

        loop {
            tokio::select! {
                // Shutdown wins over accepting more work.
                _ = tokio::signal::ctrl_c() => {
                    println!("\n⏹️  Shutting down.");
                    return Ok(());
                }
                accepted = listener.accept() => {
                    match accepted {
                        Ok((stream, _)) => {
                            let handlers = Arc::clone(&handlers);
                            tokio::spawn(async move {
                                serve_connection(stream, handlers).await;
                            });
                        }
                        Err(e) => eprintln!("⚠️  Connection error: {}", e),
                    }
                }
            }
        }
    }
}

async fn serve_connection(mut stream: TcpStream, handlers: Arc<HashMap<String, Bytecode>>) {
    let raw = match read_request(&mut stream).await {
        Some(raw) => raw,
        None => return,
    };

    let response = match HttpRequest::parse_bytes(&raw) {
        Ok(request) => {
            // The VM blocks — on the interpreter itself, and on whatever
            // database driver the handler reaches for. Running it here would
            // stall this runtime thread and every other task sharing it.
            match tokio::task::spawn_blocking(move || dispatch(&handlers, &request)).await {
                Ok(response) => response,
                // The handler panicked. One request must not take the server
                // with it, so it becomes a 500 like any other failure.
                Err(e) => {
                    eprintln!("❌ Handler task failed: {}", e);
                    HttpResponse::internal_error("handler task failed")
                }
            }
        }
        Err(e) => HttpResponse::bad_request(&e.to_string()),
    };

    let _ = stream.write_all(&response.to_http_bytes()).await;
    let _ = stream.flush().await;
}

/// Read one whole request: headers, then exactly as many body bytes as
/// `Content-Length` promises.
///
/// The same rule as the synchronous server. Reading a fixed amount once would
/// truncate a larger body while leaving the request line and headers intact,
/// so the parse would still succeed and the handler would act on half a
/// payload.
async fn read_request(stream: &mut TcpStream) -> Option<Vec<u8>> {
    const SEPARATOR: &[u8] = b"\r\n\r\n";

    let mut raw: Vec<u8> = Vec::with_capacity(CHUNK);
    let mut chunk = [0u8; CHUNK];
    let mut searched = 0usize;

    let header_end = loop {
        if let Some(offset) = raw[searched..]
            .windows(SEPARATOR.len())
            .position(|window| window == SEPARATOR)
        {
            break searched + offset + SEPARATOR.len();
        }
        searched = raw.len().saturating_sub(SEPARATOR.len() - 1);

        if raw.len() > MAX_REQUEST {
            return None;
        }
        match stream.read(&mut chunk).await {
            Ok(0) | Err(_) => return None,
            Ok(n) => raw.extend_from_slice(&chunk[..n]),
        }
    };

    let content_length = String::from_utf8_lossy(&raw[..header_end])
        .lines()
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            if name.trim().eq_ignore_ascii_case("content-length") {
                value.trim().parse::<usize>().ok()
            } else {
                None
            }
        })
        .unwrap_or(0);

    let want = header_end.saturating_add(content_length).min(MAX_REQUEST);
    while raw.len() < want {
        match stream.read(&mut chunk).await {
            // Closed early: serve what arrived rather than dropping the
            // connection without an answer.
            Ok(0) | Err(_) => break,
            Ok(n) => raw.extend_from_slice(&chunk[..n]),
        }
    }

    Some(raw)
}
