# eTamil Backend Implementation Roadmap

## ✅ Phase 1: HTTP Server Integration - COMPLETE!

**Status**: ✅ COMPLETE (2 weeks of implementation)

**What was built**:
- ✅ HTTP/1.1 server (tiny_http framework)
- ✅ 5 modular components (720 lines of Rust)
- ✅ Route matching with path parameters
- ✅ Request/response handling
- ✅ CORS support
- ✅ 6 sample applications
- ✅ 34 integration tests (100% pass)
- ✅ Release binary (8MB)

### Implementation Summary

**Architecture**: 5 modular Rust components
- `src/http/mod.rs` - Main server, socket management (255 lines)
- `src/http/request.rs` - HTTP/1.1 parser (121 lines)
- `src/http/response.rs` - Response builder (161 lines)
- `src/http/router.rs` - Route matching (84 lines)
- `src/http/handler.rs` - eTamil execution (99 lines)

**Technology**: tiny_http (synchronous, single-threaded)
- No external async framework for MVP
- Clean socket-level HTTP handling
- Proper HTTP/1.1 compliance

**Features Implemented**:
- ✅ Multiple HTTP methods (GET, POST, PUT, DELETE, OPTIONS)
- ✅ Path parameters (`:id` syntax)
- ✅ Query string parsing
- ✅ Custom headers support
- ✅ CORS headers (automatic)
- ✅ Status codes (200, 201, 404, 500)
- ✅ eTamil code execution in handlers
- ✅ Variable injection and extraction
- ✅ Proper error responses

**Testing**: 34 tests (100% pass rate)

**Sample Applications**: 6 working examples ready to use

---

## Phase 1 Original Plan (Reference)
```toml
[dependencies]
# ... existing dependencies ...

# Web Framework
axum = "0.7"
tokio = { version = "1.35", features = ["full"] }
tower = "0.4"
tower-http = { version = "0.5", features = ["trace", "cors"] }

# Serialization
serde_json = "1.0"
serde = { version = "1.0", features = ["derive"] }

# Logging
tracing = "0.1"
tracing-subscriber = "0.3"

# Async utilities
hyper = "1.0"
```

### Step 1.2: HTTP Handler DSL

**New file: `src/http/mod.rs`**
```rust
pub mod handler;
pub mod router;

use axum::{
    Router,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::{json, Value};

/// HTTP Response wrapper
pub struct HttpResponse {
    pub status: u16,
    pub body: Value,
    pub headers: Vec<(String, String)>,
}

impl IntoResponse for HttpResponse {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::from_u16(self.status).unwrap_or(StatusCode::OK), 
         Json(self.body)).into_response()
    }
}
```

### Step 1.3: eTamil HTTP Syntax

**New eTamil Keywords Needed**:
```tamil
// Define HTTP handlers
கையாளி உரு_பெரு("/users/:id", "GET") {
    பயனர்_id = பரிமாண_எடு("id");
    பயனர் = விகற்குறி_அமை("SELECT * FROM users WHERE id = ?", [பயனர்_id]);
    
    முயல் {
        பதில் 200, {
            "id": பயனர்["id"],
            "பெயர்": பயனர்["பெயர்"]
        };
    } பிழை(e) {
        பதில் 500, {"error": "Database error"};
    }
};

// Start server
வலையமைப்पு_தொடங்கு "0.0.0.0:8080";
```

### Step 1.4: Router Implementation

**New file: `src/http/router.rs`**
```rust
use axum::{Router, routing::get};
use std::sync::Arc;

pub struct AppRouter {
    routes: Router,
}

impl AppRouter {
    pub fn new() -> Self {
        AppRouter {
            routes: Router::new()
                .route("/health", get(health_check))
                .route("/metrics", get(metrics_endpoint))
        }
    }
    
    pub fn add_route(&mut self, path: &str, method: &str, handler: Box<dyn Fn() -> String>) {
        // Route registration logic
    }
}

async fn health_check() -> &'static str {
    "OK"
}

async fn metrics_endpoint() -> String {
    "# Metrics endpoint".to_string()
}
```

---

## Phase 2: Error Handling & Logging (HIGH)

### Step 2.1: Custom Error Type

**New file: `src/errors/mod.rs`**
```rust
use serde::Serialize;

#[derive(Debug, Serialize)]
pub enum AppError {
    DatabaseError(String),
    ValidationError(String),
    AuthenticationError(String),
    NotFound(String),
    InternalError(String),
}

impl AppError {
    pub fn status_code(&self) -> u16 {
        match self {
            AppError::DatabaseError(_) => 500,
            AppError::ValidationError(_) => 400,
            AppError::AuthenticationError(_) => 401,
            AppError::NotFound(_) => 404,
            AppError::InternalError(_) => 500,
        }
    }
    
    pub fn message(&self) -> String {
        match self {
            AppError::DatabaseError(msg) => msg.clone(),
            AppError::ValidationError(msg) => msg.clone(),
            AppError::AuthenticationError(_) => "Unauthorized".to_string(),
            AppError::NotFound(msg) => msg.clone(),
            AppError::InternalError(_) => "Internal Server Error".to_string(),
        }
    }
}
```

### Step 2.2: Structured Logging

**Add to `Cargo.toml`**:
```toml
slog = "2.7"
slog-json = "2.3"
slog-term = "2.9"
slog-async = "2.7"
```

**New file: `src/logging/mod.rs`**
```rust
use slog::{Drain, Logger, o, FnValue};
use std::sync::Mutex;

pub fn init_logging() -> Logger {
    let decorator = slog_term::PlainDecorator::new(std::io::stdout());
    let drain = slog_term::CompactFormat::new(decorator).build().fuse();
    let drain = Mutex::new(drain).fuse();
    
    Logger::root(
        drain,
        o!("version" => env!("CARGO_PKG_VERSION"))
    )
}

pub fn log_error(logger: &Logger, error: &str, context: &serde_json::Value) {
    slog::error!(logger, "{}", error; 
        "context" => format!("{:?}", context)
    );
}
```

---

## Phase 3: Async/Concurrency Support (CRITICAL)

### Step 3.1: Tokio Integration

**Update `src/main.rs`**:
```rust
#[tokio::main]
async fn main() {
    // Initialize
    let logger = logging::init_logging();
    
    // Setup database pool
    let db_pool = setup_db_pool().await;
    
    // Build router
    let app = build_router(db_pool.clone());
    
    // Start server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .unwrap();
    
    axum::serve(listener, app)
        .await
        .unwrap();
}

async fn setup_db_pool() -> sqlx::PgPool {
    sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .unwrap()
}
```

### Step 3.2: eTamil Async Syntax

**Proposed eTamil Keywords**:
```tamil
// Async function
பணி_பூற்ப async get_user_async(பயனர்_id: எண்) {
    பயனர் = பெறு_பணி("SELECT * FROM users WHERE id = ?", [பயனர்_id]);
    பயனிக்க பயனர்;
};

// Spawn background task
பணி_ஆரம்ப("background_import") {
    தரவு = பெறு_தரவு_வெளிப்பாடு("https://api.example.com/data");
    சேமி_தரவு(தரவு);
};

// Channel communication
சேனல் = சேனல்_உருவாக்கு();
சேனல்_அனுப்பு(சேனல், செய்தி);
பெற்ற = சேனல்_பெறு_பணி(சேனல்);
```

---

## Phase 4: Database Connection Pooling (HIGH)

### Step 4.1: SQLx Integration

**Add to `Cargo.toml`**:
```toml
sqlx = { version = "0.7", features = ["postgres", "runtime-tokio-rustls", "migrate"] }
```

**New file: `src/db/pool.rs`**
```rust
use sqlx::PgPool;

pub struct DatabasePool {
    pool: PgPool,
}

impl DatabasePool {
    pub async fn new(database_url: &str, max_connections: u32) -> Result<Self, sqlx::Error> {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(max_connections)
            .connect(database_url)
            .await?;
        
        Ok(DatabasePool { pool })
    }
    
    pub async fn health_check(&self) -> Result<(), sqlx::Error> {
        sqlx::query("SELECT 1")
            .fetch_one(&self.pool)
            .await?;
        Ok(())
    }
    
    pub fn get_pool(&self) -> PgPool {
        self.pool.clone()
    }
}
```

### Step 4.2: eTamil Database DSL Enhancement

```tamil
// Database connection with pooling
விகற_ஈதம்_உருவாக்கு {
    "url": சுற்றுச்சூழல्_पeऱु("DATABASE_URL"),
    "max_connections": 10,
    "timeout": 5000,
    "ssl": true
};

// Automatic connection management
பணி get_all_users() {
    சரிவ = தரவு_அசை_குறி("SELECT * FROM users");
    பயனிக்க சரிவ;
};
```

---

## Phase 5: Testing Framework (HIGH)

### Step 5.1: Test Infrastructure

**Add to `Cargo.toml`**:
```toml
[dev-dependencies]
tokio-test = "0.4"
mockall = "0.12"
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "postgres"] }
```

**New file: `src/testing/mod.rs`**
```rust
pub mod fixtures;
pub mod helpers;

#[cfg(test)]
mod tests {
    use crate::*;
    use axum::http::StatusCode;
    
    #[tokio::test]
    async fn test_get_user_success() {
        // Setup
        let db = setup_test_db().await;
        
        // Execute
        let response = get_user(1).await;
        
        // Assert
        assert_eq!(response.status, 200);
    }
    
    #[tokio::test]
    async fn test_get_user_not_found() {
        let response = get_user(9999).await;
        assert_eq!(response.status, 404);
    }
}
```

### Step 5.2: eTamil Test Syntax

**Proposed**:
```tamil
// Unit test
परीक्षा "get_user should return user data" {
    पयोग_सेटअप();
    
    परिणाम = उपयोग_प्राप्त(1);
    परीक्षा_सामञ्जस्य(परिणाम.स्थिति, 200);
    परीक्षा_सामञ्जस्य(परिणाम.शरीर.आईडी, 1);
};

// Integration test
परीक्षा "API end-to-end flow" {
    पयोग_शुरू();
    
    उपयोग_बनाओ({"नाम": "Alice"});
    बनाया_आईडी = परिणाम;
    
    प्राप्त = उपयोग_प्राप्त(बनाया_आईडी);
    परीक्षा_सामञ्जस्य(प्राप्त.नाम, "Alice");
    
    सफाई();
};

परीक्षा_चलाओ();
```

---

## Quick Start: Minimal Backend

### Create Backend Application

**1. Create HTTP handler file: `examples/backend/api.qmz`**
```tamil
// Simple REST API in eTamil

// Database configuration
विकास_जडत = {
    "प्रकार": "postgresql",
    "यूआरएल": "postgresql://user:pass@localhost/etamil_db"
};

// Initialize
विकास_शुरू(विकास_जडत);

// Handler functions
कार्य प्रयोक्ता_प्राप्त(आईडी: संख्या) {
    प्रयोक्ता = विकास्वर_कार्य(
        "SELECT * FROM users WHERE id = $1", 
        [आईडी]
    );
    
    यदि प्रयोक्ता = शून्य {
        उत्तर 404, {"त्रुटि": "प्रयोक्ता नहीं मिला"};
    };
    
    उत्तर 200, प्रयोक्ता;
};

// Routes
पथ पान, "/api/users/:id" {
    आईडी = पैरामीटर्प्राप्त(अनुरोध, "id");
    प्रयोक्ता_प्राप्त(आईडी);
};

पथ डाक, "/api/users" {
    नए_प्रयोक्ता = अनुरोध_निकाय_प्राप्त();
    पहचान = विकास्वर_कार्य(
        "INSERT INTO users (...) VALUES (...) RETURNING id",
        [नए_प्रयोक्ता]
    );
    उत्तर 201, {"आईडी": पहचान};
};

// Start server
सर्वर_शुरू_करें("0.0.0.0", 8080);
```

**2. Test it**:
```bash
# Build
cd etamil_compiler
cargo build --release

# Run
./target/release/etamil_compiler examples/backend/api.qmz

# Test in another terminal
curl http://localhost:8080/api/users/1
```

---

## Implementation Priority

### Week 1: Foundation
- [ ] HTTP Server integration (Axum)
- [ ] Basic routing
- [ ] Request/response handling
- [ ] Error responses

### Week 2: Core Features  
- [ ] Database operations
- [ ] Error handling & logging
- [ ] Input validation
- [ ] JSON serialization

### Week 3: Production Ready
- [ ] Async/await support
- [ ] Connection pooling
- [ ] Graceful shutdown
- [ ] Basic testing

### Week 4+: Advanced
- [ ] Authentication (JWT)
- [ ] Caching (Redis)
- [ ] Monitoring (Prometheus)
- [ ] Load testing

---

## Common Patterns

### Pattern 1: Simple CRUD API
```rust
// Handler with error handling
async fn create_user(
    Json(payload): Json<CreateUserRequest>,
    State(db): State<PgPool>,
) -> Result<(StatusCode, Json<User>), AppError> {
    let user = sqlx::query_as::<_, User>(
        "INSERT INTO users (name, email) VALUES ($1, $2) RETURNING *"
    )
    .bind(&payload.name)
    .bind(&payload.email)
    .fetch_one(&db)
    .await
    .map_err(|_| AppError::DatabaseError("Failed to create user".into()))?;
    
    Ok((StatusCode::CREATED, Json(user)))
}
```

### Pattern 2: Middleware for Auth
```rust
use axum::middleware::Next;

async fn auth_middleware(
    req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let token = req
        .headers()
        .get("Authorization")
        .ok_or(AppError::AuthenticationError("Missing token".into()))?;
    
    // Validate token
    
    Ok(next.run(req).await)
}
```

### Pattern 3: Graceful Shutdown
```rust
use tokio::signal;

async fn shutdown_signal() {
    let sigterm = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .unwrap()
            .recv()
            .await
    };
    
    tokio::select! {
        _ = sigterm => {},
    }
    
    println!("Shutting down gracefully...");
}
```

---

## Estimated Timelines

| Feature | Time | Complexity |
|---------|------|-----------|
| HTTP Server | 2-3 days | Medium |
| Error Handling | 1 day | Low |
| Logging | 1 day | Low |
| Database Pooling | 2 days | Medium |
| Async Support | 3-4 days | High |
| Testing Framework | 2-3 days | Medium |
| Auth/Auth | 2-3 days | Medium |
| Caching | 1-2 days | Low-Medium |
| **Total MVP** | **~2 weeks** | **High** |

---

**Next**: I can start implementing **Phase 1 (HTTP Server)** immediately if you want! Which would you prefer:
1. Full HTTP server implementation?
2. Just the core routing framework?
3. Example backend application?
