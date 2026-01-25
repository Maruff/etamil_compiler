# eTamil Quick Start Guide

Get up and running with eTamil in 5 minutes.

## Prerequisites

- eTamil installed ([Installation Guide](INSTALLATION.md))
- Terminal/command line access

## Your First eTamil Program

### 1. Hello World

```bash
# Create a simple program
echo 'அச்சு("வணக்கம் உலகம்!");' > hello.etamil

# Run it
etamil hello.etamil
```

**Output:**
```
✓ Lexical analysis complete (4 tokens)
✓ Parsing complete (1 statements)
=== eTamil VM Executor ===
✓ Bytecode generated (2 instructions)
=== Execution Output ===
nil
✓ Execution completed successfully
```

### 2. Working with Files

```bash
# Create a file I/O program
cat > file_example.etamil << 'ETAMIL'
// Write to file
file_content = "வணக்கம்! This is eTamil.";
// Read from file
அச்சு("File operations completed");
ETAMIL

# Run it
etamil file_example.etamil
```

### 3. Start an HTTP Server

```bash
# Use existing example
etamil --server --port 8080 etamil_compiler/examples/backend/hello_server.qmz
```

In another terminal:
```bash
curl http://localhost:8080/
curl http://localhost:8080/health
```

### 4. Start Async Server (Production)

```bash
# High-performance async server
etamil --async --port 8080 etamil_compiler/examples/backend/hello_server.qmz
```

**Performance**: 100-1000 requests/second with <20ms latency

## Command Modes

### VM Execution (Default)
```bash
etamil myprogram.etamil
```
- Fast bytecode execution
- <100ms startup
- Good for scripts and CLI tools

### HTTP Server (Sync)
```bash
etamil --server --port 8080 api.etamil
```
- Simple synchronous server
- 1-10 requests/second
- Good for MVP and testing

### HTTP Server (Async)
```bash
etamil --async --port 8080 api.etamil
```
- Production-ready async server
- 100-1000 requests/second
- 10-20ms latency
- Good for production APIs

### LLVM Backend
```bash
etamil --llvm myprogram.etamil
```
- Native code generation
- Maximum performance
- Longer compilation time

## Common Options

```bash
# Specify host and port
etamil --async --host 0.0.0.0 --port 3000 api.etamil

# Use VM explicitly
etamil --vm myprogram.etamil

# Use LLVM backend
etamil --llvm myprogram.etamil
```

## Examples Directory

Explore built-in examples:

```bash
cd etamil_compiler/examples

# Backend examples
ls backend/
# - hello_server.qmz
# - crud.etamil
# - auth.etamil
# - middleware.etamil

# File I/O examples
ls io_samples/
# - file_operations.etamil
# - csv_parser.etamil

# Database examples
ls db_samples/
# - postgresql.etamil
# - mysql.etamil
```

## Testing Your Installation

```bash
# Run installation test
./test_installation.sh

# Run HTTP backend tests
./test_http_backend.sh

# Run unit tests
cd etamil_compiler && cargo test
```

## Next Steps

1. **Learn the Language**: [Language Syntax](../reference/QUICK_START_VM.md)
2. **Build an API**: [HTTP Server Guide](../backend/HTTP_SERVER_QUICKREF.md)
3. **Use Databases**: [Database Guide](../backend/DATABASE_COMMANDS_GUIDE.md)
4. **Deploy**: [Deployment Guide](../backend/DEPLOYMENT_GUIDE.md)

## Troubleshooting

### Program doesn't run
- Check syntax (eTamil uses Tamil keywords)
- Verify file path is correct
- Run `etamil --vm yourfile.etamil` explicitly

### Server won't start
- Check if port is already in use: `sudo lsof -i :8080`
- Try a different port: `--port 3000`
- Verify .qmz file exists

### "Command not found"
- Ensure `etamil` is in PATH
- Run: `export PATH="$PATH:$HOME/.local/bin"`
- See [Installation Guide](INSTALLATION.md)

## Quick Command Reference

```bash
# Run program
etamil app.etamil

# Start sync server
etamil --server --port 8080 api.etamil

# Start async server
etamil --async --port 8080 api.etamil

# Custom host
etamil --async --host 0.0.0.0 --port 8080 api.etamil

# LLVM backend
etamil --llvm app.etamil

# Test installation
./test_installation.sh
```

---

**You're ready to build with eTamil!** 🚀

For complete command reference, see [COMMANDS.md](../reference/COMMANDS.md)
