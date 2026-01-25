// Structured Logging Module for eTamil Backend - Phase 3
// Provides JSON-formatted logging with context, log levels, and request tracking

use chrono::Utc;
use serde::{Serialize, Deserialize};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

/// Log Level enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum LogLevel {
    Debug = 0,
    Info = 1,
    Warn = 2,
    Error = 3,
}

impl LogLevel {
    pub fn as_str(&self) -> &str {
        match self {
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
        }
    }
}

/// Request context for tracing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestContext {
    /// Unique request ID for tracing across logs
    pub request_id: String,
    /// HTTP method (GET, POST, etc)
    pub method: String,
    /// Request path
    pub path: String,
    /// Request timestamp
    pub timestamp: String,
    /// Response status code
    pub status_code: Option<u16>,
    /// Request processing duration in milliseconds
    pub duration_ms: Option<u64>,
}

/// Structured log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// Timestamp in ISO 8601 format
    pub timestamp: String,
    /// Log level (DEBUG, INFO, WARN, ERROR)
    pub level: String,
    /// Log message
    pub message: String,
    /// Request context (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request: Option<RequestContext>,
    /// Additional structured data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<HashMap<String, String>>,
    /// Error details (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorDetails>,
}

/// Error details for logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDetails {
    /// Error code or type
    pub code: String,
    /// Error message
    pub message: String,
    /// Stack trace or context
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
}

impl LogEntry {
    /// Create a new log entry
    pub fn new(level: LogLevel, message: impl Into<String>) -> Self {
        LogEntry {
            timestamp: Utc::now().to_rfc3339(),
            level: level.as_str().to_string(),
            message: message.into(),
            request: None,
            context: None,
            error: None,
        }
    }

    /// Add request context
    pub fn with_request(mut self, request: RequestContext) -> Self {
        self.request = Some(request);
        self
    }

    /// Add structured context data
    pub fn with_context(mut self, context: HashMap<String, String>) -> Self {
        self.context = Some(context);
        self
    }

    /// Add error details
    pub fn with_error(mut self, code: impl Into<String>, message: impl Into<String>) -> Self {
        self.error = Some(ErrorDetails {
            code: code.into(),
            message: message.into(),
            context: None,
        });
        self
    }

    /// Serialize to JSON string
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| {
            format!("{{\"timestamp\":\"{}\",\"level\":\"{}\",\"message\":\"{}\"}}",
                self.timestamp, self.level, self.message)
        })
    }

    /// Serialize to pretty JSON string (for debugging)
    pub fn to_pretty_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| {
            format!("{{\"timestamp\":\"{}\",\"level\":\"{}\",\"message\":\"{}\"}}",
                self.timestamp, self.level, self.message)
        })
    }
}

/// Global logger instance
pub struct Logger {
    min_level: LogLevel,
    outputs: Vec<LogOutput>,
}

/// Log output destination
#[derive(Clone)]
pub enum LogOutput {
    /// Output to stdout
    Stdout,
    /// Output to stderr
    Stderr,
    /// In-memory buffer (for testing)
    Memory(Arc<Mutex<Vec<LogEntry>>>),
}

impl Logger {
    /// Create a new logger with console output
    pub fn new(min_level: LogLevel) -> Self {
        Logger {
            min_level,
            outputs: vec![LogOutput::Stdout],
        }
    }

    /// Add a log output destination
    pub fn with_output(mut self, output: LogOutput) -> Self {
        self.outputs.push(output);
        self
    }

    /// Log a message
    pub fn log(&self, entry: LogEntry) {
        if LogLevel::from_str(&entry.level) >= self.min_level {
            self.write_output(&entry);
        }
    }

    /// Log debug message
    pub fn debug(&self, message: impl Into<String>) {
        let entry = LogEntry::new(LogLevel::Debug, message);
        self.log(entry);
    }

    /// Log info message
    pub fn info(&self, message: impl Into<String>) {
        let entry = LogEntry::new(LogLevel::Info, message);
        self.log(entry);
    }

    /// Log warning message
    pub fn warn(&self, message: impl Into<String>) {
        let entry = LogEntry::new(LogLevel::Warn, message);
        self.log(entry);
    }

    /// Log error message
    pub fn error(&self, message: impl Into<String>) {
        let entry = LogEntry::new(LogLevel::Error, message);
        self.log(entry);
    }

    /// Write log entry to all outputs
    fn write_output(&self, entry: &LogEntry) {
        let json = entry.to_json();
        for output in &self.outputs {
            match output {
                LogOutput::Stdout => println!("{}", json),
                LogOutput::Stderr => eprintln!("{}", json),
                LogOutput::Memory(buffer) => {
                    if let Ok(mut buf) = buffer.lock() {
                        buf.push(entry.clone());
                    }
                }
            }
        }
    }
}

impl LogLevel {
    fn from_str(s: &str) -> Self {
        match s {
            "DEBUG" => LogLevel::Debug,
            "INFO" => LogLevel::Info,
            "WARN" => LogLevel::Warn,
            "ERROR" => LogLevel::Error,
            _ => LogLevel::Info,
        }
    }
}

/// Generate a unique request ID
pub fn generate_request_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    
    format!("req-{:x}", nanos)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_entry_creation() {
        let entry = LogEntry::new(LogLevel::Info, "Test message");
        assert_eq!(entry.level, "INFO");
        assert_eq!(entry.message, "Test message");
    }

    #[test]
    fn test_log_json_serialization() {
        let entry = LogEntry::new(LogLevel::Error, "Database connection failed");
        let json = entry.to_json();
        assert!(json.contains("\"level\":\"ERROR\""));
        assert!(json.contains("\"message\":\"Database connection failed\""));
    }

    #[test]
    fn test_log_with_context() {
        let mut ctx = HashMap::new();
        ctx.insert("user_id".to_string(), "12345".to_string());
        
        let entry = LogEntry::new(LogLevel::Info, "User action")
            .with_context(ctx);
        
        let json = entry.to_json();
        assert!(json.contains("user_id"));
    }

    #[test]
    fn test_log_with_error() {
        let entry = LogEntry::new(LogLevel::Error, "Operation failed")
            .with_error("DB_CONNECTION", "Failed to connect to database");
        
        let json = entry.to_json();
        assert!(json.contains("DB_CONNECTION"));
    }

    #[test]
    fn test_request_id_generation() {
        let id1 = generate_request_id();
        let id2 = generate_request_id();
        assert_ne!(id1, id2);
        assert!(id1.starts_with("req-"));
    }
}
