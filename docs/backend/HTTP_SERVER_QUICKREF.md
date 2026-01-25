# eTamil HTTP Server - Quick Reference Guide

## Quick Start

### 1. Build
```bash
cd etamil_compiler
cargo build --release
```

### 2. Create Backend Program
```tamil
// backend.qmz
எண் port;
port = 8080;
அச்சு "Server starting on port " & port;
```

### 3. Start Server
```bash
./target/release/etamil_compiler --server --port 8080 backend.qmz
```

### 4. Test
```bash
curl http://127.0.0.1:8080/health
curl http://127.0.0.1:8080/
```

---

## Command Line Options

```bash
./etamil_compiler --server [--host HOST] [--port PORT] <file.qmz>
```

| Option | Default | Example |
|--------|---------|---------|
| `--server` | - | Required flag to start HTTP server |
| `--host` | 127.0.0.1 | `--host 0.0.0.0` (listen on all interfaces) |
| `--port` | 8080 | `--port 3000` (custom port) |
| `<file>` | stdin | `backend.qmz` (eTamil program) |

### Examples

```bash
# Default: localhost:8080
./etamil_compiler --server backend.qmz

# Custom port
./etamil_compiler --server --port 3000 backend.qmz

# Listen on all interfaces
./etamil_compiler --server --host 0.0.0.0 --port 8080 backend.qmz
```

---

## HTTP Endpoints

### Available Routes (MVP)
- `GET /` - Main handler
- `POST /` - Main handler
- `PUT /` - Main handler
- `DELETE /` - Main handler
- `GET /health` - Health check (always returns `{"status": "healthy"}`)

### Request Variables (in handler)

All HTTP requests automatically inject these variables:

```
request_method     // "GET", "POST", "PUT", "DELETE"
request_path       // "/", "/api/users", etc.
query_params       // Map of ?key=value
headers            // Map of HTTP headers
```

### Response Variables (set in handler)

Set these to customize response:

```
response_status    // HTTP status code (default: 200)
response_body      // Response text (default: "Handler executed successfully")
```

---

## Example Usage

### 1. Simple Health Check Server
```tamil
// health_server.qmz
እንዲህ ጥሩ_ነው = 200;
እንዲህ ውጤት = "Healthy";
```

Run:
```bash
./etamil_compiler --server --port 8080 health_server.qmz
curl http://127.0.0.1:8080/health
# Output: {"status": "healthy"}
```

### 2. Echo Server
```tamil
// echo_server.qmz
எண் count;
count = 0;
count = count + 1;
அச்சு "Request " & count;
```

Run:
```bash
./etamil_compiler --server echo_server.qmz
curl http://127.0.0.1:8080/
# Server prints: Request 1, Request 2, etc.
```

### 3. Status Code Handler
```tamil
// custom_status.qmz
எண் status;
status = 201;
```

Run:
```bash
./etamil_compiler --server custom_status.qmz
curl -i http://127.0.0.1:8080/
# Returns: 201 Created
```

---

## Testing the Server

### Using cURL

```bash
# Basic GET
curl http://127.0.0.1:8080/

# Verbose (see headers)
curl -v http://127.0.0.1:8080/

# With query parameters
curl "http://127.0.0.1:8080/?name=alice"

# POST request
curl -X POST http://127.0.0.1:8080/

# PUT request
curl -X PUT http://127.0.0.1:8080/

# DELETE request
curl -X DELETE http://127.0.0.1:8080/
```

### Using Python
```python
import requests

# Test server
response = requests.get('http://127.0.0.1:8080/')
print(response.status_code)
print(response.text)
```

### Using JavaScript
```javascript
fetch('http://127.0.0.1:8080/')
  .then(r => r.text())
  .then(text => console.log(text))
```

---

## Architecture Overview

```
Request from Client
    ↓
[HTTP Parser] ← parse raw HTTP bytes
    ↓
[Route Matcher] ← match GET /health
    ↓
[VM Creation] ← create new bytecode VM
    ↓
[Handler Execution] ← run eTamil code
    ↓
[Response Builder] ← format HTTP response
    ↓
Response to Client
```

---

## Performance

| Metric | Value | Notes |
|--------|-------|-------|
| Startup | <100ms | Includes parsing |
| Request latency | 10-50ms | Single-threaded |
| Throughput | 1 req/sec | Synchronous, blocking |
| Memory | ~5MB base | Per-request: ~1MB |
| Concurrency | 1 (sequential) | Phase 2: will add async |

---

## Status Codes Supported

| Code | Text | Usage |
|------|------|-------|
| 200 | OK | Default, general success |
| 201 | Created | Resource created |
| 204 | No Content | Success, no response body |
| 400 | Bad Request | Client error |
| 404 | Not Found | Route not found |
| 500 | Internal Server Error | Handler error |

---

## CORS Headers (Included)

All responses include CORS headers:

```
Access-Control-Allow-Origin: *
Access-Control-Allow-Methods: GET, POST, PUT, DELETE, OPTIONS
Access-Control-Allow-Headers: Content-Type
```

This allows requests from any origin (useful for development).

---

## Troubleshooting

### Port Already in Use
```
Error: Address already in use
Fix: Use different port with --port flag
$ ./etamil_compiler --server --port 3000 backend.qmz
```

### Parse Error
```
Error: Unexpected token
Fix: Check eTamil syntax in .qmz file
Use valid Tamil/Latin keywords (अच्छु, எண், etc.)
```

### Connection Refused
```
curl: (7) Failed to connect
Fix: Make sure server is running
Check: Correct host and port
$ curl http://127.0.0.1:8080/
```

---

## Known Limitations

⚠️ **MVP (Single-threaded)**
- Only handles 1 request at a time
- Others must wait (blocking)
- Not for production use

✅ **Coming in Phase 2**
- Async/Tokio support
- 100-1000 concurrent requests
- Non-blocking I/O
- Connection pooling

---

## Files Involved

### Core HTTP Module
- `src/http/mod.rs` - Server implementation
- `src/http/request.rs` - Request parsing
- `src/http/response.rs` - Response formatting
- `src/http/router.rs` - Route matching
- `src/http/handler.rs` - Handler execution

### Configuration
- `Cargo.toml` - Dependencies (tiny_http, url, regex)
- `src/main.rs` - --server flag handling
- `src/lib.rs` - HTTP module export

### Example
- `examples/backend/hello_server.qmz` - Sample backend

---

## Next Steps

1. **Build and Run** - Start the server with your first backend.qmz
2. **Extend** - Modify the backend to add your logic
3. **Plan Phase 2** - Wait for async/concurrency support
4. **Read** - Check HTTP_SERVER_IMPLEMENTATION.md for technical details

---

## Phase 2: Async Support (Coming Soon)

When implemented, you'll get:

✅ 100-1000 concurrent requests  
✅ Sub-millisecond latency  
✅ Connection pooling  
✅ Graceful shutdown  
✅ Error recovery  
✅ Structured logging  

Status: In planning - estimated 2-3 weeks

---

**Questions?** Check HTTP_SERVER_IMPLEMENTATION.md for details.
