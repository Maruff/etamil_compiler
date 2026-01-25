// Error Handling Module for eTamil Backend - Phase 3
// Provides custom error types, error recovery, and detailed error context

use serde::{Serialize, Deserialize};
use std::fmt;

/// eTamil Backend Error Type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EtamilError {
    /// HTTP parsing error
    HttpParseError {
        message: String,
        details: Option<String>,
    },
    /// Handler execution error
    HandlerError {
        message: String,
        request_path: Option<String>,
        details: Option<String>,
    },
    /// Database connection error
    DatabaseError {
        message: String,
        error_code: Option<String>,
        details: Option<String>,
    },
    /// File I/O error
    IoError {
        message: String,
        file_path: Option<String>,
        details: Option<String>,
    },
    /// Validation error
    ValidationError {
        message: String,
        field: Option<String>,
        details: Option<String>,
    },
    /// Configuration error
    ConfigError {
        message: String,
        config_key: Option<String>,
        details: Option<String>,
    },
    /// Internal server error
    InternalError {
        message: String,
        error_code: Option<String>,
        details: Option<String>,
    },
    /// Timeout error
    TimeoutError {
        message: String,
        timeout_ms: Option<u64>,
        details: Option<String>,
    },
    /// Resource not found
    NotFound {
        message: String,
        resource: Option<String>,
        details: Option<String>,
    },
    /// Unauthorized access
    Unauthorized {
        message: String,
        reason: Option<String>,
        details: Option<String>,
    },
}

impl EtamilError {
    /// Get HTTP status code for this error
    pub fn status_code(&self) -> u16 {
        match self {
            EtamilError::HttpParseError { .. } => 400,
            EtamilError::ValidationError { .. } => 400,
            EtamilError::Unauthorized { .. } => 401,
            EtamilError::NotFound { .. } => 404,
            EtamilError::TimeoutError { .. } => 408,
            EtamilError::DatabaseError { .. } => 500,
            EtamilError::HandlerError { .. } => 500,
            EtamilError::IoError { .. } => 500,
            EtamilError::ConfigError { .. } => 500,
            EtamilError::InternalError { .. } => 500,
        }
    }

    /// Get error code string
    pub fn error_code(&self) -> &str {
        match self {
            EtamilError::HttpParseError { .. } => "HTTP_PARSE_ERROR",
            EtamilError::HandlerError { .. } => "HANDLER_ERROR",
            EtamilError::DatabaseError { .. } => "DATABASE_ERROR",
            EtamilError::IoError { .. } => "IO_ERROR",
            EtamilError::ValidationError { .. } => "VALIDATION_ERROR",
            EtamilError::ConfigError { .. } => "CONFIG_ERROR",
            EtamilError::InternalError { .. } => "INTERNAL_ERROR",
            EtamilError::TimeoutError { .. } => "TIMEOUT_ERROR",
            EtamilError::NotFound { .. } => "NOT_FOUND",
            EtamilError::Unauthorized { .. } => "UNAUTHORIZED",
        }
    }

    /// Get user-friendly error message
    pub fn user_message(&self) -> String {
        match self {
            EtamilError::HttpParseError { message, .. } => message.clone(),
            EtamilError::HandlerError { message, .. } => format!("Handler error: {}", message),
            EtamilError::DatabaseError {  .. } => "Database operation failed".to_string(),
            EtamilError::IoError {  .. } => "File operation failed".to_string(),
            EtamilError::ValidationError { message, .. } => format!("Validation error: {}", message),
            EtamilError::ConfigError { message, .. } => format!("Configuration error: {}", message),
            EtamilError::InternalError {  .. } => "Internal server error".to_string(),
            EtamilError::TimeoutError {  .. } => "Request timeout".to_string(),
            EtamilError::NotFound { message, .. } => message.clone(),
            EtamilError::Unauthorized { message, .. } => message.clone(),
        }
    }

    /// Get detailed error message (for logs)
    pub fn detailed_message(&self) -> String {
        match self {
            EtamilError::HttpParseError { message, details } => {
                format!("{}{}", message, details.as_ref().map(|d| format!(": {}", d)).unwrap_or_default())
            }
            EtamilError::HandlerError { message, request_path, details } => {
                let path = request_path.as_ref().map(|p| format!(" [{}]", p)).unwrap_or_default();
                let detail = details.as_ref().map(|d| format!(": {}", d)).unwrap_or_default();
                format!("{}{}{}", message, path, detail)
            }
            EtamilError::DatabaseError { message, error_code, details } => {
                let code = error_code.as_ref().map(|c| format!(" [{}]", c)).unwrap_or_default();
                let detail = details.as_ref().map(|d| format!(": {}", d)).unwrap_or_default();
                format!("{}{}{}", message, code, detail)
            }
            EtamilError::IoError { message, file_path, details } => {
                let path = file_path.as_ref().map(|p| format!(" [{}]", p)).unwrap_or_default();
                let detail = details.as_ref().map(|d| format!(": {}", d)).unwrap_or_default();
                format!("{}{}{}", message, path, detail)
            }
            EtamilError::ValidationError { message, field, details } => {
                let f = field.as_ref().map(|fld| format!(" ({})", fld)).unwrap_or_default();
                let detail = details.as_ref().map(|d| format!(": {}", d)).unwrap_or_default();
                format!("{}{}{}", message, f, detail)
            }
            EtamilError::ConfigError { message, config_key, details } => {
                let key = config_key.as_ref().map(|k| format!(" [{}]", k)).unwrap_or_default();
                let detail = details.as_ref().map(|d| format!(": {}", d)).unwrap_or_default();
                format!("{}{}{}", message, key, detail)
            }
            EtamilError::InternalError { message, error_code, details } => {
                let code = error_code.as_ref().map(|c| format!(" [{}]", c)).unwrap_or_default();
                let detail = details.as_ref().map(|d| format!(": {}", d)).unwrap_or_default();
                format!("{}{}{}", message, code, detail)
            }
            EtamilError::TimeoutError { message, timeout_ms, details } => {
                let timeout = timeout_ms.as_ref().map(|t| format!(" ({}ms)", t)).unwrap_or_default();
                let detail = details.as_ref().map(|d| format!(": {}", d)).unwrap_or_default();
                format!("{}{}{}", message, timeout, detail)
            }
            EtamilError::NotFound { message, resource, details } => {
                let res = resource.as_ref().map(|r| format!(" [{}]", r)).unwrap_or_default();
                let detail = details.as_ref().map(|d| format!(": {}", d)).unwrap_or_default();
                format!("{}{}{}", message, res, detail)
            }
            EtamilError::Unauthorized { message, reason, details } => {
                let r = reason.as_ref().map(|re| format!(" ({})", re)).unwrap_or_default();
                let detail = details.as_ref().map(|d| format!(": {}", d)).unwrap_or_default();
                format!("{}{}{}", message, r, detail)
            }
        }
    }

    /// Create a recovery suggestion
    pub fn recovery_suggestion(&self) -> String {
        match self {
            EtamilError::HttpParseError { .. } => 
                "Check request format and ensure it complies with HTTP/1.1 specification".to_string(),
            EtamilError::HandlerError { .. } => 
                "Review handler code for syntax errors or runtime issues".to_string(),
            EtamilError::DatabaseError { .. } => 
                "Verify database connection and ensure database is running".to_string(),
            EtamilError::IoError { .. } => 
                "Check file paths and ensure proper file permissions".to_string(),
            EtamilError::ValidationError { .. } => 
                "Review input data and ensure it matches expected format".to_string(),
            EtamilError::ConfigError { .. } => 
                "Check configuration settings and environment variables".to_string(),
            EtamilError::InternalError { .. } => 
                "Contact support with request ID and error details".to_string(),
            EtamilError::TimeoutError { .. } => 
                "Retry request or increase timeout threshold".to_string(),
            EtamilError::NotFound { .. } => 
                "Verify resource exists and path is correct".to_string(),
            EtamilError::Unauthorized { .. } => 
                "Provide valid credentials or token".to_string(),
        }
    }
}

impl fmt::Display for EtamilError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.error_code(), self.user_message())
    }
}

impl std::error::Error for EtamilError {}

/// Result type for eTamil operations
pub type EtamilResult<T> = Result<T, EtamilError>;

/// Error response JSON structure
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: ErrorInfo,
    pub timestamp: String,
    pub request_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorInfo {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,
}

impl ErrorResponse {
    /// Create error response from EtamilError
    pub fn from_error(error: &EtamilError, request_id: Option<String>) -> Self {
        ErrorResponse {
            error: ErrorInfo {
                code: error.error_code().to_string(),
                message: error.user_message(),
                details: Some(error.detailed_message()),
                suggestion: Some(error.recovery_suggestion()),
            },
            timestamp: chrono::Utc::now().to_rfc3339(),
            request_id,
        }
    }

    /// Serialize to JSON
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| {
            format!(r#"{{"error":{{"code":"{}","message":"{}"}}}}"#,
                self.error.code, self.error.message)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_status_codes() {
        let not_found = EtamilError::NotFound {
            message: "User not found".to_string(),
            resource: Some("users/123".to_string()),
            details: None,
        };
        assert_eq!(not_found.status_code(), 404);

        let unauth = EtamilError::Unauthorized {
            message: "Invalid token".to_string(),
            reason: Some("expired".to_string()),
            details: None,
        };
        assert_eq!(unauth.status_code(), 401);
    }

    #[test]
    fn test_error_response_json() {
        let error = EtamilError::ValidationError {
            message: "Invalid email format".to_string(),
            field: Some("email".to_string()),
            details: None,
        };
        let response = ErrorResponse::from_error(&error, Some("req-123".to_string()));
        let json = response.to_json();
        assert!(json.contains("VALIDATION_ERROR"));
        assert!(json.contains("Invalid email format"));
    }
}
