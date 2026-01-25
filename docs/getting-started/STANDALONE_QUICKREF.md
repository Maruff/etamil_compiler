# eTamil Standalone Binary - Quick Reference

## Installation

```bash
# Clone repository and install
git clone <repo-url>
cd eTamil
./install.sh

# Add to PATH (if not already)
export PATH="$PATH:$HOME/.local/bin"
```

---

## Basic Commands

### Run Programs
```bash
# Run eTamil source file
etamil myapp.etamil

# Explicitly use VM mode (default)
etamil --vm myapp.etamil

# Use LLVM backend
etamil --llvm myapp.etamil
```

### HTTP Servers
```bash
# Synchronous server (simple, MVP)
etamil --server --port 8080 api.etamil

# Asynchronous server (production-ready, 100x faster)
etamil --async --port 8080 api.etamil

# Custom host and port
etamil --async --host 0.0.0.0 --port 3000 api.etamil
```

---

## Quick Examples

### Hello World
```bash
# Create file
echo 'அச்சு("வணக்கம்!");' > hello.etamil

# Run it
etamil hello.etamil
```

### Simple Backend
```bash
# Use existing example
cd etamil_compiler/examples/backend

# Run CRUD API
etamil --server --port 8080 crud.etamil

# Test it
curl http://localhost:8080/health
```

### File I/O
```bash
# Use file I/O example
cd etamil_compiler/examples/io_samples

# Run file operations
etamil file_operations.etamil
```

---

## Command Options

| Option | Description | Example |
|--------|-------------|---------|
| `--vm` | VM execution (default) | `etamil --vm app.etamil` |
| `--llvm` | LLVM backend | `etamil --llvm app.etamil` |
| `--server` | HTTP server (sync) | `etamil --server --port 8080 api.etamil` |
| `--async` | HTTP server (async) | `etamil --async --port 8080 api.etamil` |
| `--host <host>` | Server host (default: 127.0.0.1) | `--host 0.0.0.0` |
| `--port <port>` | Server port (default: 8080) | `--port 3000` |

---

## Common Tasks

### Install System-Wide
```bash
# Root installation
sudo ./install.sh

# Binary will be at /usr/local/bin/etamil
```

### Install for Current User
```bash
# User installation (default)
./install.sh

# Binary will be at ~/.local/bin/etamil
```

### Uninstall
```bash
# Remove binary
rm ~/.local/bin/etamil
# OR
sudo rm /usr/local/bin/etamil
```

### Check Installation
```bash
# Verify binary exists
which etamil

# Check binary size
ls -lh $(which etamil)

# Test execution
cd /path/to/eTamil
./test_installation.sh
```

---

## Production Deployment

### Simple Deployment
```bash
# Copy binary to server
scp $(which etamil) user@server:/usr/local/bin/

# Copy your application
scp myapp.etamil user@server:/opt/myapp/

# Run on server
ssh user@server
etamil /opt/myapp/myapp.etamil
```

### Backend Deployment
```bash
# Copy binary and app
scp $(which etamil) user@server:/usr/local/bin/
scp api.etamil user@server:/opt/myapp/

# Run as systemd service
ssh user@server

# Create service file
sudo cat > /etc/systemd/system/etamil-app.service << 'EOF'
[Unit]
Description=eTamil Application
After=network.target

[Service]
Type=simple
User=etamil
WorkingDirectory=/opt/myapp
ExecStart=/usr/local/bin/etamil --async --host 0.0.0.0 --port 8080 api.etamil
Restart=always

[Install]
WantedBy=multi-user.target
EOF

# Start service
sudo systemctl daemon-reload
sudo systemctl enable etamil-app
sudo systemctl start etamil-app
```

---

## Performance

### Server Modes

| Mode | Throughput | Latency | Use Case |
|------|-----------|---------|----------|
| `--server` | 1-10 req/sec | 100-200ms | MVP, testing |
| `--async` | 100-1000 req/sec | 10-20ms | Production |

### Optimization Tips
```bash
# Use async mode for production
etamil --async --port 8080 api.etamil

# Bind to all interfaces for external access
etamil --async --host 0.0.0.0 --port 8080 api.etamil

# Use environment variables for configuration
export ETAMIL_PORT=8080
export ETAMIL_HOST=0.0.0.0
```

---

## Troubleshooting

### Binary not found
```bash
# Check if installed
ls -la ~/.local/bin/etamil

# Add to PATH
export PATH="$PATH:$HOME/.local/bin"
echo 'export PATH="$PATH:$HOME/.local/bin"' >> ~/.bashrc
```

### Permission denied
```bash
chmod +x ~/.local/bin/etamil
```

### Port already in use
```bash
# Check what's using the port
sudo lsof -i :8080

# Use different port
etamil --server --port 3000 api.etamil
```

---

## Key Features (No Rust Required!)

✅ **File I/O**: Read/write files, CSV parsing  
✅ **Database**: PostgreSQL, MySQL, SQLite  
✅ **HTTP Server**: REST APIs with routing  
✅ **Async**: High-concurrency support  
✅ **JSON**: Parse and generate JSON  
✅ **Logging**: Structured logging  
✅ **Auth**: JWT, bcrypt (modules ready)  
✅ **Caching**: In-memory cache (modules ready)  

---

## Documentation

- [INSTALLATION_GUIDE.md](INSTALLATION_GUIDE.md) - Complete installation guide
- [QUICK_START_VM.md](QUICK_START_VM.md) - VM executor guide
- [HTTP_SERVER_QUICKREF.md](HTTP_SERVER_QUICKREF.md) - Backend development
- [QUICK_START_DATABASE_EXAMPLES.md](QUICK_START_DATABASE_EXAMPLES.md) - Database usage

---

**Binary Size**: 2.1 MB  
**Rust Required**: ❌ No (standalone binary)  
**Platforms**: Linux, macOS (Windows: WSL)
