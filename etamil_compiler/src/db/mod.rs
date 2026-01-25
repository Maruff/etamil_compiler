// Database Connectivity Module for eTamil Compiler
// Supports relational (SQL) and NoSQL databases

pub mod relational;
pub mod nosql;

pub use relational::RelationalDB;
pub use nosql::NoSQLDB;

/// Database connection type
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum DBType {
    SQLite,
    MySQL,
    PostgreSQL,
    MongoDB,
    Redis,
}

/// Result type for database operations
pub type DBResult<T> = Result<T, DBError>;

/// Database error types
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum DBError {
    ConnectionFailed(String),
    QueryFailed(String),
    NotFound(String),
    InvalidData(String),
    OperationNotSupported(String),
}

impl std::fmt::Display for DBError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DBError::ConnectionFailed(msg) => write!(f, "DB Connection Failed: {}", msg),
            DBError::QueryFailed(msg) => write!(f, "Query Failed: {}", msg),
            DBError::NotFound(msg) => write!(f, "Not Found: {}", msg),
            DBError::InvalidData(msg) => write!(f, "Invalid Data: {}", msg),
            DBError::OperationNotSupported(msg) => write!(f, "Operation Not Supported: {}", msg),
        }
    }
}

impl std::error::Error for DBError {}

/// Generic database interface
#[allow(dead_code)]
pub trait Database {
    fn connect(&mut self, connection_string: &str) -> DBResult<()>;
    fn disconnect(&mut self) -> DBResult<()>;
    fn execute(&self, query: &str) -> DBResult<String>;
    fn query(&self, query: &str) -> DBResult<Vec<String>>;
}
