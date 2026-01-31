// REST API Module for eTamil Compiler
// Handles HTTP route definitions, server management, and API response generation

pub mod parser;
#[cfg(feature = "llvm")]
pub mod codegen;

pub use parser::APIParser;
#[cfg(feature = "llvm")]
pub use codegen::APICodegen;

/// REST API operation types
#[derive(Debug, Clone)]
pub enum APIOperation {
    DefineRoute {
        method: String,
        path: String,
        handler: Vec<String>,
    },
    StartServer {
        host: String,
        port: u16,
    },
    StopServer,
    SendResponse {
        status_code: u32,
        body: String,
    },
    SendJSON {
        data: String,
        status_code: Option<u32>,
    },
}

/// API Route definition
#[derive(Debug, Clone)]
pub struct APIRoute {
    pub method: String,  // GET, POST, PUT, DELETE, etc.
    pub path: String,    // /api/endpoint
    pub handler: Vec<String>,  // Handler statements
}

/// API Server configuration
#[derive(Debug, Clone)]
pub struct APIServer {
    pub host: String,
    pub port: u16,
}

/// REST API utilities
pub mod utils {
    /// Convert HTTP method string to uppercase
    pub fn normalize_http_method(method: &str) -> String {
        method.to_uppercase()
    }
    
    /// Validate HTTP status code
    pub fn is_valid_status_code(code: u32) -> bool {
        (100..600).contains(&code)
    }
    
    /// Validate URL path
    pub fn is_valid_path(path: &str) -> bool {
        path.starts_with('/')
    }
    
    /// Validate port number
    pub fn is_valid_port(port: u16) -> bool {
        port > 0 && port != 65535
    }
}
