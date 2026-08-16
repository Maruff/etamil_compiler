// HTTP Server Module for eTamil Backend
// Provides synchronous HTTP server capabilities for Minimum Viable Backend

use std::collections::HashMap;
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::sync::{mpsc, Mutex};
use std::time::Instant;
use crate::parser::Stmt;
use crate::vm::VM;

pub mod router;
pub mod request;
pub mod response;
pub mod handler;
pub mod logging;
pub mod errors;
pub mod monitoring;
pub mod auth;       // Backend milestone 4: Authentication & Authorization
pub mod cache;      // Backend milestone 4: Caching Layer
pub mod resilience; // Backend milestone 4: Circuit breakers, retries, timeouts

pub use self::router::Router;
pub use self::request::HttpRequest;
pub use self::response::HttpResponse;
pub use self::logging::{Logger, LogLevel, LogEntry, generate_request_id};
pub use self::errors::ErrorResponse;
pub use self::monitoring::MetricsCollector;
pub use self::monitoring::HealthChecker;
pub use self::monitoring::HealthStatus;
pub use self::monitoring::PerformanceReport;

/// Main HTTP Server for eTamil Backend
pub struct HttpServer {
    pub host: String,
    pub port: u16,
    pub router: Router,
    pub handlers: HashMap<String, Vec<Stmt>>,
    pub logger: Logger,
    pub metrics: MetricsCollector,
    pub health_checker: HealthChecker,
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
        }
    }

    /// Register a route with an eTamil handler
    pub fn register_route(&mut self, method: &str, path: &str, handler: Vec<Stmt>) {
        let route_key = format!("{} {}", method.to_uppercase(), path);
        self.handlers.insert(route_key.clone(), handler);
        self.router.add_route(method, path);
        
        // Log route registration
        self.logger.info(format!("Route registered: {} {}", method.to_uppercase(), path));
    }

    /// Start the HTTP server and listen for requests
    pub fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let addr = format!("{}:{}", self.host, self.port);
        let listener = TcpListener::bind(&addr)?;
        
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("🚀 eTamil HTTP Server Started (Backend milestone 3 - Production Logging & Error Handling)");
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

    /// Read one request off a connection, run it, and write the response.
    fn serve_connection(&self, mut tcp_stream: TcpStream) {
        let request_id = generate_request_id();
        let start_time = Instant::now();

        let mut buffer = vec![0; 4096];
        let read = match tcp_stream.read(&mut buffer) {
            Ok(n) if n > 0 => n,
            _ => return,
        };

        let request_str = String::from_utf8_lossy(&buffer[..read]);
        match HttpRequest::parse(&request_str) {
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

                let _ = tcp_stream.write_all(response.to_http_string().as_bytes());
            }
            Err(e) => {
                let log_entry = LogEntry::new(LogLevel::Error, "Failed to parse HTTP request")
                    .with_error("HTTP_PARSE_ERROR", &e.to_string());
                self.logger.log(log_entry);

                let error_response = HttpResponse::bad_request(&e.to_string());
                let _ = tcp_stream.write_all(error_response.to_http_string().as_bytes());
                self.metrics.record_request("/", "UNKNOWN", 0, false);
            }
        }
    }

    /// Handle an incoming HTTP request
    fn handle_request(&self, request: &HttpRequest) -> HttpResponse {
        // Match route
        let route_key = format!("{} {}", request.method, request.path);
        
        match self.handlers.get(&route_key) {
            Some(handler_stmts) => {
                // Execute handler
                let mut vm = VM::new();
                
                // Store request data in VM variables
                vm.variables.insert("request_method".to_string(), 
                    crate::vm::Value::String(request.method.clone()));
                vm.variables.insert("request_path".to_string(),
                    crate::vm::Value::String(request.path.clone()));
                
                // Store query parameters
                let mut params = HashMap::new();
                for (key, value) in &request.query_params {
                    params.insert(
                        key.clone(),
                        crate::vm::Value::String(value.clone())
                    );
                }
                vm.variables.insert("query_params".to_string(),
                    crate::vm::Value::Map(params));

                // Store headers
                let mut headers = HashMap::new();
                for (key, value) in &request.headers {
                    headers.insert(
                        key.clone(),
                        crate::vm::Value::String(value.clone())
                    );
                }
                vm.variables.insert("headers".to_string(),
                    crate::vm::Value::Map(headers));

                // Compile and execute handler
                let bytecode = crate::vm::bytecode::compiler::BytecodeCompiler::compile_statements(handler_stmts.clone());
                match vm.execute(bytecode) {
                    Ok(_) => {
                        // Get response from VM
                        if let Some(crate::vm::Value::String(body)) = vm.variables.get("response_body") {
                            let status = vm.variables.get("response_status")
                                .and_then(|v| match v {
                                    crate::vm::Value::Number(n) => rust_decimal::prelude::ToPrimitive::to_u16(n),
                                    _ => None
                                })
                                .unwrap_or(200);
                            
                            HttpResponse::success(status, body.clone())
                        } else {
                            HttpResponse::success(200, "Handler executed successfully".to_string())
                        }
                    }
                    Err(e) => {
                        eprintln!("❌ Handler execution error: {}", e);
                        HttpResponse::internal_error(&format!("Handler error: {}", e))
                    }
                }
            }
            None => {
                // Try path matching with parameters
                for (route_key, handler_stmts) in &self.handlers {
                    let parts: Vec<&str> = route_key.split(' ').collect();
                    if parts.len() == 2 && parts[0] == request.method {
                        if self.path_matches(parts[1], &request.path) {
                            // Extract path parameters
                            let mut params = HashMap::new();
                            self.extract_path_params(parts[1], &request.path, &mut params);
                            
                            let mut vm = VM::new();
                            
                            // Store path parameters
                            for (key, value) in params {
                                vm.variables.insert(format!("param_{}", key),
                                    crate::vm::Value::String(value));
                            }
                            
                            // Store other request data
                            vm.variables.insert("request_method".to_string(),
                                crate::vm::Value::String(request.method.clone()));
                            vm.variables.insert("request_path".to_string(),
                                crate::vm::Value::String(request.path.clone()));
                            
                            let bytecode = crate::vm::bytecode::compiler::BytecodeCompiler::compile_statements(handler_stmts.clone());
                            match vm.execute(bytecode) {
                                Ok(_) => {
                                    if let Some(crate::vm::Value::String(body)) = vm.variables.get("response_body") {
                                        let status = vm.variables.get("response_status")
                                            .and_then(|v| match v {
                                                crate::vm::Value::Number(n) => rust_decimal::prelude::ToPrimitive::to_u16(n),
                                                _ => None
                                            })
                                            .unwrap_or(200);
                                        
                                        return HttpResponse::success(status, body.clone());
                                    }
                                    return HttpResponse::success(200, "Handler executed successfully".to_string());
                                }
                                Err(e) => {
                                    return HttpResponse::internal_error(&format!("Handler error: {}", e));
                                }
                            }
                        }
                    }
                }
                
                HttpResponse::not_found()
            }
        }
    }

    /// Check if a route pattern matches a request path
    fn path_matches(&self, pattern: &str, path: &str) -> bool {
        let pattern_parts: Vec<&str> = pattern.split('/').filter(|p| !p.is_empty()).collect();
        let path_parts: Vec<&str> = path.split('/').filter(|p| !p.is_empty()).collect();

        if pattern_parts.len() != path_parts.len() {
            return false;
        }

        for (pattern_part, path_part) in pattern_parts.iter().zip(path_parts.iter()) {
            if !pattern_part.starts_with(':') && pattern_part != path_part {
                return false;
            }
        }

        true
    }

    /// Extract path parameters from a request path
    fn extract_path_params(&self, pattern: &str, path: &str, params: &mut HashMap<String, String>) {
        let pattern_parts: Vec<&str> = pattern.split('/').filter(|p| !p.is_empty()).collect();
        let path_parts: Vec<&str> = path.split('/').filter(|p| !p.is_empty()).collect();

        for (pattern_part, path_part) in pattern_parts.iter().zip(path_parts.iter()) {
            if pattern_part.starts_with(':') {
                let param_name = pattern_part[1..].to_string();
                params.insert(param_name, path_part.to_string());
            }
        }
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

    #[test]
    fn test_path_matching() {
        let server = HttpServer::new("127.0.0.1", 8080);
        assert!(server.path_matches("/users/:id", "/users/123"));
        assert!(server.path_matches("/users/:id/posts/:post_id", "/users/123/posts/456"));
        assert!(!server.path_matches("/users/:id", "/users/123/invalid"));
    }
}
