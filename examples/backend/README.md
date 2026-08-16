# eTamil Backend Examples

This directory contains example backend programs demonstrating the HTTP server capabilities of the eTamil compiler.

## Quick Start

### 1. Build the Compiler
```bash
cd ../etamil_compiler
cargo build --release
cd ../examples/backend
```

### 2. Run an Example
```bash
../../etamil_compiler/target/release/etamil_compiler --server --port 8080 hello_server.qmz
```

### 3. Test It
```bash
# In another terminal
curl http://127.0.0.1:8080/
curl http://127.0.0.1:8080/health
```

---

## Available Examples

### hello_server.qmz
**Description**: Simple HTTP server that demonstrates basic setup

**Features**:
- Variable declarations
- Arithmetic operations
- String concatenation
- Print output

**How to Run**:
```bash
./etamil_compiler --server hello_server.qmz
```

**Expected Output**:
```
✓ Lexical analysis complete
✓ Parsing complete
🚀 eTamil HTTP Server Started
📍 Listening on: http://127.0.0.1:8080
```

**Test**:
```bash
curl http://127.0.0.1:8080/health
# Response: {"status": "healthy"}
```

---

## Server Features Demonstrated

### HTTP Methods
- GET /
- POST /
- PUT /
- DELETE /
- GET /health (always returns healthy status)

### Request Handling
When a request is received, these variables are automatically available:
- `request_method` - HTTP method (GET, POST, etc.)
- `request_path` - Request path (/api/users, etc.)
- `query_params` - Map of query parameters
- `headers` - Map of HTTP headers

### Response Control
Set these variables to control response:
- `response_status` - HTTP status code (default: 200)
- `response_body` - Response content (default: "Handler executed successfully")

---

## Example Program Structure

```tamil
// Declare variables
எண் count;
எண் port;

// Initialize values
count = 1;
port = 8080;

// Print output
அச்சு "Server starting on port " & port;
```

When run with `--server`, this program will:
1. Parse the code
2. Start HTTP server on port 8080
3. Execute the code for each request
4. Return responses

---

## Building Custom Backends

### Step 1: Create Your Backend File
```tamil
// my_backend.qmz
எண் user_count;
user_count = 0;
அச்சு "eTamil Backend Server Ready";
```

### Step 2: Start the Server
```bash
./etamil_compiler --server --port 8080 my_backend.qmz
```

### Step 3: Test the Endpoint
```bash
curl http://127.0.0.1:8080/
```

---

## eTamil DSL Syntax (Current MVP)

### Variables
```tamil
எண் x;              // Declare number
x = 42;             // Assign value
```

### Operations
```tamil
sum = a + b;        // Addition
diff = a - b;       // Subtraction
product = a * b;    // Multiplication
quotient = a / b;   // Division
remainder = a % b;  // Modulo
```

### String Operations
```tamil
greeting = "Hello " & name;  // Concatenation
```

### Output
```tamil
அச்சு x;                    // Print variable
அச்சு "Value: " & x;        // Print with text
```

### Conditionals
```tamil
(x > 10) எனில் {            // If statement
    அச்சு "Greater";
}
இன்றேல் {                   // Else statement
    அச்சு "Smaller";
}
```

### Loops
```tamil
(x > 0) ஒன்றும் {           // While loop
    x = x - 1;
    அச்சு x;
}
```

---

## Tips for Success

### 1. Keep Programs Simple
Start with basic examples and gradually add complexity.

### 2. Use Comments
eTamil supports comments with `//`:
```tamil
// This is a comment
எண் x;  // Number declaration
```

### 3. Test with curl
curl is the easiest way to test HTTP endpoints:
```bash
# Health check
curl http://127.0.0.1:8080/health

# Root endpoint
curl http://127.0.0.1:8080/

# With parameters
curl "http://127.0.0.1:8080/?name=alice"

# Different methods
curl -X POST http://127.0.0.1:8080/
curl -X PUT http://127.0.0.1:8080/
curl -X DELETE http://127.0.0.1:8080/
```

### 4. Use Python for More Complex Tests
```python
import requests

response = requests.get('http://127.0.0.1:8080/')
print(response.status_code)
print(response.json())
```

---

## Current Limitations (MVP)

⚠️ **Single-Threaded** - Can only handle 1 request at a time  
⚠️ **Synchronous** - All operations block  
⚠️ **No Error Recovery** - Errors crash the connection  
⚠️ **No Middleware** - No authentication or logging  

**These will be fixed in Phase 2 with async/Tokio support.**

---

## Troubleshooting

### "Address already in use"
Another process is using the port. Use a different port:
```bash
./etamil_compiler --server --port 3000 backend.qmz
```

### "Unexpected token"
Check eTamil syntax. Make sure:
- Variables are declared with `எண்`
- Statements end properly
- Keywords are spelled correctly

### "Connection refused"
- Make sure server is running
- Check correct port: `curl http://127.0.0.1:8080/`
- Check if firewall is blocking

### Handler doesn't execute
Ensure your .qmz file has valid eTamil code. Test with a simple program first:
```tamil
எண் x;
x = 1;
அச்சு x;
```

---

## Example: Real-World Use Case

### Simple User Counter
```tamil
// user_counter.qmz
எண் total_users;
எண் today;

total_users = 1000;
today = 42;

அச்சு "Total Users: " & total_users;
அச்சு "Today: " & today;
அச்சு "Active: " & (total_users - 1000 + today);
```

Run:
```bash
./etamil_compiler --server --port 8080 user_counter.qmz
```

Test:
```bash
# Server will print the values for each request
curl http://127.0.0.1:8080/
```

---

## Next Steps

1. ✅ Run `hello_server.qmz`
2. ✅ Understand the server startup
3. ✅ Test the HTTP endpoints
4. ✅ Create your own backend
5. 📖 Read HTTP_SERVER_IMPLEMENTATION.md for detailed information
6. 🎯 Plan Phase 2 (async support) for production-grade system

---

## Resources

- [HTTP_SERVER_QUICKREF.md](../HTTP_SERVER_QUICKREF.md) - Quick reference
- [HTTP_SERVER_IMPLEMENTATION.md](../HTTP_SERVER_IMPLEMENTATION.md) - Full technical guide
- [PHASE_1_COMPLETE.md](../PHASE_1_COMPLETE.md) - Implementation summary
- [BACKEND_ANALYSIS.md](../BACKEND_ANALYSIS.md) - Roadmap and requirements

---

**Status**: ✅ Phase 1 Complete - HTTP Server Ready

**Next**: Phase 2 - Async/Concurrency Support (Coming Soon)
