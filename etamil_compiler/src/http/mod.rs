// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Mohammed Maruff (Esan Maruff) <esan@etamil.in>
// HTTP Server Module for eTamil Backend
// Provides synchronous HTTP server capabilities for Minimum Viable Backend

use crate::parser::Stmt;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Mutex, mpsc};
use std::time::Instant;

pub mod async_server; // --async: tokio accept loop, blocking handlers
pub mod auth; // Backend milestone 4: Authentication & Authorization
pub mod cache; // Backend milestone 4: Caching Layer
pub mod errors;
pub mod handler;
pub mod logging;
pub mod monitoring;
pub mod multipart; // multipart/form-data, over bytes
pub mod request;
pub mod resilience;
pub mod response;
pub mod router; // Backend milestone 4: Circuit breakers, retries, timeouts

pub use self::async_server::AsyncHttpServer;
pub use self::errors::ErrorResponse;
pub use self::handler::{bind_request, response_from};
pub use self::logging::{LogEntry, LogLevel, Logger, generate_request_id};
pub use self::monitoring::HealthChecker;
pub use self::monitoring::HealthStatus;
pub use self::monitoring::MetricsCollector;
pub use self::monitoring::PerformanceReport;
pub use self::request::HttpRequest;
pub use self::response::HttpResponse;
pub use self::router::Router;

/// Main HTTP Server for eTamil Backend
pub struct HttpServer {
    pub host: String,
    pub port: u16,
    pub router: Router,
    /// Compiled once at registration; a request should not pay to
    /// recompile the handler every time.
    pub handlers: HashMap<String, crate::vm::Bytecode>,
    pub logger: Logger,
    pub metrics: MetricsCollector,
    pub health_checker: HealthChecker,
    /// Timed jobs: how often, and what to run.
    pub schedules: Vec<(u64, crate::vm::Bytecode)>,
}

impl HttpServer {
    /// Create a new HTTP server
    pub fn new(host: &str, port: u16) -> Self {
        HttpServer {
            host: host.to_string(),
            port,
            router: Router::new(),
            handlers: HashMap::new(),
            logger: Logger::new(LogLevel::Info),
            metrics: MetricsCollector::new(),
            health_checker: HealthChecker::new(),
            schedules: Vec::new(),
        }
    }

    /// Create a new HTTP server with custom logger
    pub fn with_logger(host: &str, port: u16, logger: Logger) -> Self {
        HttpServer {
            host: host.to_string(),
            port,
            router: Router::new(),
            handlers: HashMap::new(),
            logger,
            metrics: MetricsCollector::new(),
            health_checker: HealthChecker::new(),
            schedules: Vec::new(),
        }
    }

    /// Register a block to run on a timer.
    ///
    /// The interval is the gap *between* runs, not a fixed rate: a job slower
    /// than its interval would otherwise pile up on itself, and two copies of
    /// a reconciliation running at once is worse than one running late.
    pub fn register_schedule(&mut self, seconds: u64, body: Vec<Stmt>) {
        let bytecode = crate::vm::BytecodeCompiler::compile_statements(body);
        self.schedules.push((seconds.max(1), bytecode));
    }

    /// Register a route with an eTamil handler.
    ///
    /// The statements are compiled here, once, rather than on every request.
    pub fn register_route(&mut self, method: &str, path: &str, handler: Vec<Stmt>) {
        let bytecode = crate::vm::BytecodeCompiler::compile_statements(handler);
        let route_key = format!("{} {}", method.to_uppercase(), path);
        self.handlers.insert(route_key.clone(), bytecode);
        self.router.add_route(method, path);

        // Log route registration
        self.logger.info(format!(
            "Route registered: {} {}",
            method.to_uppercase(),
            path
        ));
    }

    /// Start the HTTP server and listen for requests
    pub fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let addr = format!("{}:{}", self.host, self.port);
        let listener = TcpListener::bind(&addr)?;

        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!(
            "🚀 eTamil HTTP Server Started (Backend milestone 3 - Production Logging & Error Handling)"
        );
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("📍 Listening on: http://{}", addr);
        println!("📋 Registered Routes:");
        for route in self.router.routes.iter() {
            println!("   {} {}", route.method, route.path);
        }
        println!("📊 Metrics & Logging: Enabled");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

        self.logger.info("eTamil HTTP Server started successfully");

        // Connections are handed to a fixed pool of worker threads, each of
        // which runs one request to completion on its own VM. A pool rather
        // than a thread per connection means a burst of traffic queues
        // instead of spawning unbounded threads.
        //
        // Scoped threads let the workers borrow &self directly: Logger and
        // MetricsCollector are already Arc<Mutex<..>> internally, and the
        // handler ASTs are read-only once the server has started.
        let workers = Self::worker_count();
        println!("🧵 Worker threads: {}\n", workers);

        let (sender, receiver) = mpsc::channel::<TcpStream>();
        let receiver = Mutex::new(receiver);

        std::thread::scope(|scope| {
            // Timed jobs get their own threads, outside the request pool, so a
            // long-running job cannot starve requests of a worker.
            for (index, (seconds, bytecode)) in self.schedules.iter().enumerate() {
                let label = format!("#{} every {}s", index + 1, seconds);
                println!("⏱️  Scheduled job {}", label);
                scope.spawn(move || {
                    loop {
                        std::thread::sleep(std::time::Duration::from_secs(*seconds));
                        handler::run_scheduled(&label, bytecode);
                    }
                });
            }

            for _ in 0..workers {
                scope.spawn(|| {
                    loop {
                        // The guard is released before the request is served,
                        // otherwise the pool would serialise on the queue.
                        let job = {
                            let queue = match receiver.lock() {
                                Ok(queue) => queue,
                                Err(_) => break, // a worker panicked; stop
                            };
                            queue.recv()
                        };
                        match job {
                            Ok(stream) => self.serve_connection(stream),
                            Err(_) => break, // listener closed
                        }
                    }
                });
            }

            for stream in listener.incoming() {
                match stream {
                    Ok(tcp_stream) => {
                        if sender.send(tcp_stream).is_err() {
                            break; // no workers left
                        }
                    }
                    Err(e) => {
                        let log_entry =
                            LogEntry::new(LogLevel::Warn, format!("Connection error: {}", e));
                        self.logger.log(log_entry);
                    }
                }
            }
        });

        Ok(())
    }

    /// Size of the worker pool: `ETAMIL_WORKERS` if set, otherwise twice the
    /// available parallelism, which suits handlers that block on I/O.
    fn worker_count() -> usize {
        std::env::var("ETAMIL_WORKERS")
            .ok()
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or_else(|| {
                std::thread::available_parallelism()
                    .map(|n| n.get() * 2)
                    .unwrap_or(8)
            })
            .max(1)
    }

    /// Read one whole request: headers first, then exactly as many body bytes
    /// as `Content-Length` promises.
    ///
    /// This used to be a single 4 KB read. Anything larger arrived cut in
    /// half, and because the request line and headers were intact the parse
    /// still succeeded — so a truncated journal batch looked like a valid
    /// request carrying half a payload, which is precisely the silent wrong
    /// answer this project refuses everywhere else.
    ///
    /// Returns None if the connection closes early or the request exceeds
    /// `MAX_REQUEST`, so a client cannot exhaust memory by promising a body
    /// it never sends.
    fn read_request(stream: &mut TcpStream) -> Option<Vec<u8>> {
        const CHUNK: usize = 4096;
        const MAX_REQUEST: usize = 1024 * 1024;
        const SEPARATOR: &[u8] = b"\r\n\r\n";

        let mut raw: Vec<u8> = Vec::with_capacity(CHUNK);
        let mut chunk = [0u8; CHUNK];
        let mut searched = 0usize;

        // Headers, up to the blank line that ends them.
        let header_end = loop {
            if let Some(offset) = raw[searched..]
                .windows(SEPARATOR.len())
                .position(|window| window == SEPARATOR)
            {
                break searched + offset + SEPARATOR.len();
            }
            // Only the tail can still be part of a split separator, so the
            // next scan need not start from the beginning again.
            searched = raw.len().saturating_sub(SEPARATOR.len() - 1);

            if raw.len() > MAX_REQUEST {
                return None;
            }
            match stream.read(&mut chunk) {
                Ok(0) => return None,
                Ok(n) => raw.extend_from_slice(&chunk[..n]),
                Err(_) => return None,
            }
        };

        // Content-Length, read off the headers already in hand.
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
            match stream.read(&mut chunk) {
                // The client closed early. Serve what did arrive rather than
                // dropping the connection without an answer.
                Ok(0) => break,
                Ok(n) => raw.extend_from_slice(&chunk[..n]),
                Err(_) => break,
            }
        }

        Some(raw)
    }

    /// Read one request off a connection, run it, and write the response.
    fn serve_connection(&self, mut tcp_stream: TcpStream) {
        let request_id = generate_request_id();
        let start_time = Instant::now();

        let request_raw = match Self::read_request(&mut tcp_stream) {
            Some(raw) => raw,
            None => return,
        };

        match HttpRequest::parse_bytes(&request_raw) {
            Ok(request) => {
                let mut req_context = std::collections::HashMap::new();
                req_context.insert("request_id".to_string(), request_id);

                let log_entry =
                    LogEntry::new(LogLevel::Info, "Incoming request").with_context(req_context);
                self.logger.log(log_entry);

                let response = self.handle_request(&request);
                let duration = start_time.elapsed().as_millis() as u64;

                self.metrics.record_request(
                    &request.path,
                    &request.method,
                    duration,
                    response.status_code < 400,
                );

                let _ = tcp_stream.write_all(&response.to_http_bytes());
            }
            Err(e) => {
                let log_entry = LogEntry::new(LogLevel::Error, "Failed to parse HTTP request")
                    .with_error("HTTP_PARSE_ERROR", e.to_string());
                self.logger.log(log_entry);

                let error_response = HttpResponse::bad_request(&e.to_string());
                let _ = tcp_stream.write_all(&error_response.to_http_bytes());
                self.metrics.record_request("/", "UNKNOWN", 0, false);
            }
        }
    }

    /// Handle an incoming HTTP request.
    ///
    /// Route matching and execution live in `handler::dispatch`, shared with
    /// the async server so the two cannot disagree about what a route means.
    fn handle_request(&self, request: &HttpRequest) -> HttpResponse {
        handler::dispatch(&self.handlers, request)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_server() {
        let server = HttpServer::new("127.0.0.1", 8080);
        assert_eq!(server.host, "127.0.0.1");
        assert_eq!(server.port, 8080);
    }

    // Matching lives in handler.rs now, shared by both servers so the sync and
    // async paths cannot disagree about what a route means.
    #[test]
    fn test_path_matching() {
        assert!(handler::path_matches("/users/:id", "/users/123"));
        assert!(handler::path_matches(
            "/users/:id/posts/:post_id",
            "/users/123/posts/456"
        ));
        assert!(!handler::path_matches("/users/:id", "/users/123/invalid"));
    }

    #[test]
    fn path_parameters_are_named_after_the_pattern() {
        let params = handler::extract_path_params("/kaNakku/:id/nirY/:row", "/kaNakku/1000/nirY/7");

        assert_eq!(params.get("id").map(String::as_str), Some("1000"));
        assert_eq!(params.get("row").map(String::as_str), Some("7"));
        assert!(handler::extract_path_params("/kaNakku", "/kaNakku").is_empty());
    }
}
