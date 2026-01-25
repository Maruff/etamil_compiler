# eTamil Command Reference

Complete reference for all eTamil commands and options.

## Basic Usage

```bash
etamil [OPTIONS] <SOURCE_FILE>
```

## Execution Modes

### VM Execution (Default)
```bash
etamil myprogram.etamil
etamil --vm myprogram.etamil
```
- Fast bytecode execution
- <100ms startup time
- Good for: Scripts, CLI tools, development

### LLVM Backend
```bash
etamil --llvm myprogram.etamil
```
- Native code generation
- Maximum performance
- Longer compilation time
- Good for: Production applications

## Server Modes

### Synchronous HTTP Server
```bash
etamil --server --port <PORT> <FILE>
etamil --server --host <HOST> --port <PORT> <FILE>
```
- Simple synchronous request handling
- Throughput: 1-10 requests/second
- Latency: 100-200ms
- Good for: MVP, testing, simple APIs

**Examples:**
```bash
etamil --server --port 8080 api.etamil
etamil --server --host 0.0.0.0 --port 8080 api.etamil
```

### Asynchronous HTTP Server
```bash
etamil --async --port <PORT> <FILE>
etamil --async --host <HOST> --port <PORT> <FILE>
```
- High-performance async request handling
- Throughput: 100-1000 requests/second
- Latency: 10-20ms
- Good for: Production APIs, high traffic

**Examples:**
```bash
etamil --async --port 8080 api.etamil
etamil --async --host 0.0.0.0 --port 3000 api.etamil
```

## Command Line Options

| Option | Description | Default | Example |
|--------|-------------|---------|---------|
| `--vm` | Use VM executor | Yes | `etamil --vm app.etamil` |
| `--llvm` | Use LLVM backend | No | `etamil --llvm app.etamil` |
| `--server` | Start HTTP server (sync) | No | `etamil --server --port 8080 api.etamil` |
| `--async` | Start HTTP server (async) | No | `etamil --async --port 8080 api.etamil` |
| `--host <host>` | Server bind address | 127.0.0.1 | `--host 0.0.0.0` |
| `--port <port>` | Server port number | 8080 | `--port 3000` |

## Input Sources

### File Input (Default)
```bash
etamil myprogram.etamil
```

### Standard Input
```bash
echo 'அச்சு("Hello");' | etamil
cat program.etamil | etamil
```

## Examples by Use Case

### 1. CLI Application
```bash
# Simple script
etamil script.etamil

# With LLVM optimization
etamil --llvm script.etamil
```

### 2. Development Server
```bash
# Quick testing
etamil --server --port 8080 api.etamil

# With hot reload (manual restart)
etamil --server --port 8080 api.etamil
```

### 3. Production API
```bash
# High performance
etamil --async --host 0.0.0.0 --port 8080 api.etamil

# Behind reverse proxy
etamil --async --host 127.0.0.1 --port 8080 api.etamil
```

### 4. File Processing
```bash
# Process files
etamil process_data.etamil

# With input redirection
etamil process.etamil < input.txt > output.txt
```

### 5. Database Operations
```bash
# Run database script
etamil db_operations.etamil

# With async for better performance
etamil --async --port 8080 db_api.etamil
```

## Environment Variables

### PATH
Ensure `etamil` is in your PATH:
```bash
export PATH="$PATH:$HOME/.local/bin"
```

### Custom Configuration (Future)
```bash
export ETAMIL_PORT=8080
export ETAMIL_HOST=0.0.0.0
export ETAMIL_LOG_LEVEL=debug
```

## Performance Comparison

| Mode | Startup | Throughput | Latency | Use Case |
|------|---------|-----------|---------|----------|
| VM | <100ms | N/A | N/A | Scripts, CLI |
| LLVM | 2-5s | N/A | N/A | Performance-critical |
| Sync Server | <1s | 1-10 req/s | 100-200ms | MVP, testing |
| Async Server | <1s | 100-1000 req/s | 10-20ms | Production |

## Common Workflows

### Development
```bash
# Edit code
vim api.etamil

# Run locally
etamil --server --port 8080 api.etamil

# Test
curl http://localhost:8080/api/test
```

### Testing
```bash
# Run tests
./test_installation.sh
./test_http_backend.sh

# Run specific test
etamil test_file.etamil
```

### Deployment
```bash
# Build optimized binary (if needed)
cd etamil_compiler && cargo build --release

# Copy to server
scp ~/.local/bin/etamil server:/usr/local/bin/
scp api.etamil server:/opt/myapp/

# Run on server
ssh server
etamil --async --host 0.0.0.0 --port 8080 /opt/myapp/api.etamil
```

## Troubleshooting Commands

### Check Installation
```bash
# Verify binary exists
which etamil

# Check version/help
etamil --help  # (if implemented)

# Test execution
etamil test_standalone.etamil
```

### Debug Server
```bash
# Check port availability
sudo lsof -i :8080
netstat -tuln | grep 8080

# Test connectivity
curl http://localhost:8080/health
telnet localhost 8080
```

### Performance Testing
```bash
# Simple test
curl http://localhost:8080/

# Load test (requires wrk or ab)
wrk -t4 -c100 -d30s http://localhost:8080/
ab -n 1000 -c 10 http://localhost:8080/
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 130 | Interrupted (Ctrl+C) |

## Signal Handling

### Graceful Shutdown (Async Mode)
```bash
# Send SIGTERM
kill -TERM <pid>

# Send SIGINT (Ctrl+C)
^C
```

Server will:
1. Stop accepting new connections
2. Complete in-flight requests
3. Clean up resources
4. Exit cleanly

## Best Practices

### Development
- Use `--server` for quick iterations
- Use `--port` different from production
- Keep server logs visible

### Production
- Use `--async` for better performance
- Bind to `0.0.0.0` for external access
- Use process manager (systemd, supervisor)
- Set up reverse proxy (nginx, Apache)
- Monitor with health checks

### Security
- Don't expose server directly to internet
- Use firewall rules
- Run as non-root user
- Keep binary updated

---

**Last Updated**: January 26, 2026  
**Version**: 1.0.0
