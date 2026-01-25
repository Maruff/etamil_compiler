# eTamil Compiler - Installation & Usage Guide

## Overview

The eTamil compiler is a standalone binary that allows you to write and execute eTamil programs **without needing Rust installed**. Once installed, you can develop eTamil applications using just the `etamil` command.

---

## Installation

### Method 1: Quick Install (Recommended)

```bash
# Clone or download the eTamil repository
cd /path/to/eTamil

# Run the installation script
chmod +x install.sh
./install.sh
```

The installer will:
- Build the compiler if needed
- Install to `/usr/local/bin` (root) or `~/.local/bin` (user)
- Make it available as the `etamil` command

### Method 2: Manual Installation

```bash
# Build the release binary
cd etamil_compiler
cargo build --release

# Copy to your preferred location
sudo cp target/release/etamil_compiler /usr/local/bin/etamil
# OR for user installation:
mkdir -p ~/.local/bin
cp target/release/etamil_compiler ~/.local/bin/etamil

# Make it executable
chmod +x /usr/local/bin/etamil
# OR
chmod +x ~/.local/bin/etamil
```

### Verify Installation

```bash
# Check if etamil is in your PATH
which etamil

# Show help
etamil --help
```

If `etamil` is not found, add the installation directory to your PATH:

```bash
# Add to ~/.bashrc or ~/.zshrc
export PATH="$PATH:$HOME/.local/bin"

# Reload shell configuration
source ~/.bashrc
```

---

## Usage

### 1. Run eTamil Programs Directly

```bash
# Execute an eTamil source file directly
etamil myprogram.etamil

# The VM compiles and runs it automatically
```

**Example:**
```bash
# Create a simple program
cat > hello.etamil << 'EOF'
அச்சு("வணக்கம் உலகம்!");
EOF

# Run it directly
etamil hello.etamil
```

### 2. Start HTTP Server (Synchronous)

```bash
# Start a basic HTTP server
etamil --server --port 8080 myapi.etamil

# With custom host
etamil --server --host 0.0.0.0 --port 8080 myapi.etamil
```

**Example:**
```bash
# Run a backend example
etamil --server --port 8080 examples/backend/crud.etamil
# Server starts on http://localhost:8080
```

### 3. Start HTTP Server (Async - Production Ready)

```bash
# Start high-performance async server
etamil --async --port 8080 myapi.etamil

# Async server supports 100-1000 req/sec
etamil --async --host 0.0.0.0 --port 3000 myapi.etamil
```

**Example:**
```bash
etamil --async --port 8080 examples/backend/rest_api.etamil
# Production-ready server with ~100x better throughput
```

### 4. Use VM Execution (Default)

```bash
# VM mode is default (fast bytecode execution)
etamil myprogram.etamil

# Explicitly specify VM mode
etamil --vm myprogram.etamil
```

### 5. LLVM Backend (Optional)

```bash
# Use LLVM for native code generation
etamil --llvm myprogram.etamil
```

---

## Command Reference

### Execution Modes

| Command | Description | Example |
|---------|-------------|---------|
| `etamil <file>` | Run eTamil program (VM mode) | `etamil app.etamil` |
| `etamil --vm <file>` | Explicitly use VM | `etamil --vm app.etamil` |
| `etamil --llvm <file>` | Use LLVM backend | `etamil --llvm app.etamil` |

### Server Modes

| Command | Description | Example |
|---------|-------------|---------|
| `etamil --server --port <port> <file>` | Start HTTP server (sync) | `etamil --server --port 8080 api.etamil` |
| `etamil --async --port <port> <file>` | Start async HTTP server | `etamil --async --port 8080 api.etamil` |
| `etamil --server --host <host> --port <port> <file>` | Custom host | `etamil --server --host 0.0.0.0 --port 8080 api.etamil` |

### Command Line Options

| Option | Description | Default |
|--------|-------------|---------|
| `--vm` | Use VM executor | Yes (default) |
| `--llvm` | Use LLVM backend | No |
| `--server` | Start HTTP server | No |
| `--async` | Use async server (Phase 2) | No |
| `--host <host>` | Server host | `127.0.0.1` |
| `--port <port>` | Server port | `8080` |

---

## Development Workflow

### 1. Create a New eTamil Application

```bash
# Create project directory
mkdir my-etamil-app
cd my-etamil-app

# Create your main file
cat > main.etamil << 'EOF'
அச்சு("வணக்கம் - eTamil Application Started");
அச்சு("Application Version: 1.0");
EOF
```

### 2. Run Your Application

```bash
# Run directly
etamil main.etamil
```

### 3. Create a Backend API

```bash
# Create API file
cat > api.etamil << 'EOF'
// Note: Backend route syntax will be implemented
// For now, use existing examples from etamil_compiler/examples/backend/
EOF

# Run the server (sync mode)
etamil --server --port 8080 api.etamil

# Or run with async mode (better performance)
etamil --async --port 8080 api.etamil
```

### 4. Test Your Application

```bash
# In another terminal
curl http://localhost:8080/api/test
```

### 5. Deploy Your Application

```bash
# Copy binary to production server
scp $(which etamil) user@server:/usr/local/bin/
scp main.etamil user@server:/opt/myapp/

# On production server
etamil /opt/myapp/main.etamil

# Or for backend
etamil --async --host 0.0.0.0 --port 8080 /opt/myapp/api.etamil
```

---

## Working with Examples

The repository includes many example applications:

```bash
# File I/O examples
cd etamil_compiler/examples/io_samples
etamil compile read_file.etamil -o read_file.qmz
etamil run read_file.qmz

# Database examples
cd ../db_samples
etamil compile basic_postgresql.etamil -o basic_postgresql.qmz
etamil run basic_postgresql.qmz

# Backend API examples
cd ../backend
etamil server 8080 crud.etamil
etamil server 8080 middleware.etamil
etamil server 8080 auth.etamil
```

---

## System Requirements

### For Using eTamil (End Users)
- **No Rust required!** ✅
- Linux, macOS, or Windows
- 2 MB disk space for binary
- Optional: PostgreSQL/MySQL for database features

### For Building eTamil (Developers)
- Rust 1.75+ (only needed once to build the compiler)
- Cargo package manager
- 500 MB disk space for dependencies

---

## Features Available Without Rust

Once installed, the `etamil` binary provides:

✅ **File I/O**: Read/write files, CSV parsing  
✅ **Database**: PostgreSQL, MySQL, SQLite support  
✅ **HTTP Server**: RESTful APIs with routing  
✅ **Async Operations**: Concurrent request handling  
✅ **JSON Processing**: Parse and generate JSON  
✅ **Authentication**: JWT, bcrypt, RBAC  
✅ **Caching**: In-memory cache with TTL  
✅ **Logging**: Structured JSON logs  
✅ **Error Handling**: Graceful error responses  
✅ **Metrics**: Performance monitoring  

---

## Troubleshooting

### "etamil: command not found"

**Solution**: Add installation directory to PATH

```bash
# Add to ~/.bashrc or ~/.zshrc
echo 'export PATH="$PATH:$HOME/.local/bin"' >> ~/.bashrc
source ~/.bashrc

# Verify
which etamil
```

### "Permission denied"

**Solution**: Make binary executable

```bash
chmod +x ~/.local/bin/etamil
# OR
chmod +x /usr/local/bin/etamil
```

### "Binary not found during installation"

**Solution**: Build manually first

```bash
cd etamil_compiler
cargo build --release
cd ..
./install.sh
```

### Program execution issues

**Solution**: Check your eTamil syntax

```bash
# Test with a simple example first
cat > test.etamil << 'EOF'
அச்சு("Test");
EOF

etamil test.etamil
```

### Server won't start

**Solution**: Check if port is already in use

```bash
# Check what's using the port
sudo lsof -i :8080

# Try a different port
etamil --server --port 3000 myapi.etamil
```

---

## Uninstallation

```bash
# Remove the binary
sudo rm /usr/local/bin/etamil
# OR
rm ~/.local/bin/etamil
```

---

## Next Steps

- Read [QUICK_START_VM.md](QUICK_START_VM.md) for VM features
- Read [QUICK_START_DATABASE_EXAMPLES.md](QUICK_START_DATABASE_EXAMPLES.md) for database usage
- Read [HTTP_SERVER_QUICKREF.md](HTTP_SERVER_QUICKREF.md) for backend development
- Explore examples in `etamil_compiler/examples/`

---

## Support

- **Documentation**: See `*.md` files in repository
- **Examples**: `etamil_compiler/examples/`
- **Test Suite**: Run `bash test_all_examples.sh`

---

**You're now ready to build eTamil applications without Rust! 🚀**
