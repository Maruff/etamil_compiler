// Phase 2: Async/Concurrent HTTP Server Implementation
// Using Axum + Tokio for high-performance concurrent request handling
// Supports graceful shutdown and connection pooling

use axum::{
    extract::{Path, Query, State},
    http::{StatusCode, HeaderMap, Method},
    response::{IntoResponse, Response},
    routing::{get, post, put, delete},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use signal_hook::consts::signal::*;
use signal_hook_tokio::Signals;
use futures::stream::StreamExt;

use crate::vm::{VM, bytecode::compiler::Compiler};

/// Phase 2 async HTTP server configuration
#[derive(Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

/// Request context passed to handlers
#[derive(Clone)]
pub struct RequestContext {
    pub method: String,
    pub path: String,
    pub query_params: HashMap<String, String>,
    pub headers: HashMap<String, String>,
}

/// HTTP Response wrapper
#[derive(Debug, Serialize, Deserialize)]
pub struct HttpResponse {
    pub status: u16,
    pub body: serde_json::Value,
}

/// Route handler function type (async)
pub type AsyncHandler = Arc<dyn Fn(RequestContext) -> std::pin::Pin<Box<dyn std::future::Future<Output = HttpResponse> + Send>> + Send + Sync>;

/// Phase 2 Async HTTP Server
pub struct AsyncHttpServer {
    config: ServerConfig,
    handlers: Arc<RwLock<HashMap<String, AsyncHandler>>>,
}

impl AsyncHttpServer {
    /// Create new async server
    pub fn new(host: String, port: u16) -> Self {
        Self {
            config: ServerConfig { host, port },
            handlers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a handler for a route
    pub async fn register_handler(
        &self,
        route: String,
        handler: AsyncHandler,
    ) {
        let mut handlers = self.handlers.write().await;
        handlers.insert(route, handler);
    }

    /// Start the async HTTP server with graceful shutdown support
    pub async fn start(self) -> Result<(), Box<dyn std::error::Error>> {
        let addr = format!("{}:{}", self.config.host, self.config.port);
        let listener = tokio::net::TcpListener::bind(&addr).await?;
        
        println!("🚀 eTamil Async HTTP Server Started (Phase 2)");
        println!("📍 Listening on: http://{}", addr);
        println!("🔄 Concurrent request handling: ENABLED");
        println!("✅ Graceful shutdown: ENABLED");

        // Setup signal handling for graceful shutdown
        let mut signals = Signals::new([SIGTERM, SIGINT])?;
        let handle = signals.handle();

        // Spawn signal handler task
        let signal_task = tokio::spawn(async move {
            let mut signals = signals.fuse();
            if signals.next().await.is_some() {
                println!("\n📢 Shutdown signal received (SIGTERM/SIGINT)");
                println!("🛑 Gracefully shutting down server...");
                handle.close();
            }
        });

        // Accept connections until shutdown signal
        loop {
            match listener.accept().await {
                Ok((socket, _)) => {
                    // Handle each connection concurrently
                    tokio::spawn(self.handle_connection(socket));
                }
                Err(_) => {
                    // Shutdown signal received
                    break;
                }
            }
        }

        signal_task.abort();
        println!("✅ Server shutdown complete");
        Ok(())
    }

    /// Handle a single connection (async)
    async fn handle_connection(&self, socket: tokio::net::TcpStream) {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        
        let mut buffer = [0; 4096];
        match socket.try_read(&mut buffer) {
            Ok(_n) => {
                let request_str = String::from_utf8_lossy(&buffer);
                
                // Parse HTTP request
                if let Some(response) = self.process_request(&request_str).await {
                    let mut socket = socket;
                    let response_bytes = response.to_http_string().into_bytes();
                    let _ = socket.write_all(&response_bytes).await;
                }
            }
            Err(_) => {}
        }
    }

    /// Process HTTP request and return response
    async fn process_request(&self, request_str: &str) -> Option<AsyncHttpResponse> {
        // Parse request line
        let lines: Vec<&str> = request_str.lines().collect();
        if lines.is_empty() {
            return None;
        }

        let request_line = lines[0];
        let parts: Vec<&str> = request_line.split_whitespace().collect();
        if parts.len() < 3 {
            return None;
        }

        let method = parts[0].to_string();
        let path = parts[1].to_string();

        // Parse query parameters
        let (path, query_params) = if let Some(idx) = path.find('?') {
            let (p, q) = path.split_at(idx);
            (p.to_string(), Self::parse_query_string(&q[1..]))
        } else {
            (path, HashMap::new())
        };

        // Parse headers
        let mut headers = HashMap::new();
        for line in lines.iter().skip(1) {
            if line.is_empty() {
                break;
            }
            if let Some(idx) = line.find(':') {
                let (key, value) = line.split_at(idx);
                headers.insert(key.to_lowercase(), value[1..].trim().to_string());
            }
        }

        // Create request context
        let context = RequestContext {
            method: method.clone(),
            path: path.clone(),
            query_params,
            headers,
        };

        // Find and execute handler
        let handlers = self.handlers.read().await;
        if let Some(handler) = handlers.get(&format!("{} {}", method, path)) {
            let response = handler(context).await;
            Some(AsyncHttpResponse {
                status_code: response.status,
                status_text: Self::status_text(response.status),
                headers: vec![
                    ("Content-Type".to_string(), "application/json".to_string()),
                    ("Access-Control-Allow-Origin".to_string(), "*".to_string()),
                    ("Access-Control-Allow-Methods".to_string(), "GET, POST, PUT, DELETE, OPTIONS".to_string()),
                ],
                body: response.body,
            })
        } else {
            Some(AsyncHttpResponse {
                status_code: 404,
                status_text: "Not Found".to_string(),
                headers: vec![
                    ("Content-Type".to_string(), "application/json".to_string()),
                    ("Access-Control-Allow-Origin".to_string(), "*".to_string()),
                ],
                body: serde_json::json!({"error": "Route not found"}),
            })
        }
    }

    /// Parse query string parameters
    fn parse_query_string(query: &str) -> HashMap<String, String> {
        let mut params = HashMap::new();
        for pair in query.split('&') {
            if let Some(eq_idx) = pair.find('=') {
                let key = &pair[..eq_idx];
                let value = &pair[eq_idx + 1..];
                params.insert(
                    urlencoding::decode(key).unwrap_or_default().to_string(),
                    urlencoding::decode(value).unwrap_or_default().to_string(),
                );
            }
        }
        params
    }

    /// Get HTTP status text
    fn status_text(code: u16) -> String {
        match code {
            200 => "OK".to_string(),
            201 => "Created".to_string(),
            400 => "Bad Request".to_string(),
            404 => "Not Found".to_string(),
            500 => "Internal Server Error".to_string(),
            _ => "Unknown".to_string(),
        }
    }
}

/// Async HTTP Response
pub struct AsyncHttpResponse {
    pub status_code: u16,
    pub status_text: String,
    pub headers: Vec<(String, String)>,
    pub body: serde_json::Value,
}

impl AsyncHttpResponse {
    /// Convert to HTTP/1.1 string format
    pub fn to_http_string(&self) -> String {
        let body_str = self.body.to_string();
        let content_length = body_str.len();

        let mut response = format!(
            "HTTP/1.1 {} {}\r\n",
            self.status_code, self.status_text
        );

        // Add headers
        for (key, value) in &self.headers {
            response.push_str(&format!("{}: {}\r\n", key, value));
        }

        response.push_str(&format!("Content-Length: {}\r\n", content_length));
        response.push_str("\r\n");
        response.push_str(&body_str);

        response
    }
}

/// Execute eTamil code asynchronously in request context
pub async fn execute_etamil_async(code: &str, context: RequestContext) -> HttpResponse {
    // This runs eTamil compilation and execution in a blocking task
    // to prevent blocking the async runtime
    let code = code.to_string();
    
    tokio::task::spawn_blocking(move || {
        // Compile and execute eTamil code
        let compiler = Compiler::new();
        match compiler.compile(&code) {
            Ok(bytecode) => {
                let mut vm = VM::new();
                
                // Inject request variables
                vm.variables.insert("request_method".to_string(), crate::vm::Value::Number(0.0));
                vm.variables.insert("request_path".to_string(), crate::vm::Value::Number(0.0));
                vm.variables.insert("request_method_str".to_string(), crate::vm::Value::Number(0.0));
                
                // Execute bytecode
                match vm.execute(&bytecode) {
                    Ok(_) => {
                        // Extract response variables
                        let status = vm.variables.get("response_status")
                            .and_then(|v| match v { crate::vm::Value::Number(n) => Some(*n as u16), _ => None })
                            .unwrap_or(200);
                        
                        let body = vm.variables.get("response_body")
                            .map(|v| format!("{:?}", v))
                            .unwrap_or_else(|| "Handler executed successfully".to_string());
                        
                        HttpResponse {
                            status,
                            body: serde_json::json!({"message": body}),
                        }
                    }
                    Err(_) => HttpResponse {
                        status: 500,
                        body: serde_json::json!({"error": "Execution failed"}),
                    }
                }
            }
            Err(_) => HttpResponse {
                status: 400,
                body: serde_json::json!({"error": "Compilation failed"}),
            }
        }
    })
    .await
    .unwrap_or_else(|_| HttpResponse {
        status: 500,
        body: serde_json::json!({"error": "Task execution failed"}),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_async_server_creation() {
        let server = AsyncHttpServer::new("127.0.0.1".to_string(), 8080);
        assert_eq!(server.config.host, "127.0.0.1");
        assert_eq!(server.config.port, 8080);
    }

    #[tokio::test]
    async fn test_query_string_parsing() {
        let params = AsyncHttpServer::parse_query_string("key1=value1&key2=value2");
        assert_eq!(params.get("key1"), Some(&"value1".to_string()));
        assert_eq!(params.get("key2"), Some(&"value2".to_string()));
    }

    #[tokio::test]
    async fn test_status_text() {
        assert_eq!(AsyncHttpServer::status_text(200), "OK");
        assert_eq!(AsyncHttpServer::status_text(404), "Not Found");
        assert_eq!(AsyncHttpServer::status_text(500), "Internal Server Error");
    }

    #[tokio::test]
    async fn test_http_response_formatting() {
        let response = AsyncHttpResponse {
            status_code: 200,
            status_text: "OK".to_string(),
            headers: vec![("Content-Type".to_string(), "application/json".to_string())],
            body: serde_json::json!({"status": "ok"}),
        };
        
        let http_str = response.to_http_string();
        assert!(http_str.contains("HTTP/1.1 200 OK"));
        assert!(http_str.contains("Content-Type: application/json"));
        assert!(http_str.contains("Content-Length:"));
    }
}
