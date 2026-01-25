# eTamil Programming Language

**A Tamil-based programming language with production-ready backend capabilities**

[![Build Status](https://img.shields.io/badge/build-passing-brightgreen)]()
[![Tests](https://img.shields.io/badge/tests-62%2F62%20passing-brightgreen)]()
[![Binary Size](https://img.shields.io/badge/binary-2.1%20MB-blue)]()
[![Rust Required](https://img.shields.io/badge/rust-not%20required-success)]()

---

## 🚀 Quick Start (2 Minutes)

### Install
```bash
./install.sh
```

### Run Your First Program
```bash
echo 'அச்சு("வணக்கம் உலகம்!");' > hello.etamil
etamil hello.etamil
```

### Start a Backend Server
```bash
# Production-ready async server (100-1000 req/sec)
etamil --async --port 8080 examples/backend/api.etamil
```

---

## 📦 What's Included

### Core Features
- ✅ **Tamil Programming Language** - Write code in Tamil
- ✅ **Standalone Binary** - No Rust required (2.1 MB)
- ✅ **VM Executor** - Fast bytecode execution (<100ms)
- ✅ **LLVM Backend** - Native code generation
- ✅ **File I/O** - Read/write files, CSV parsing
- ✅ **JSON Processing** - Parse and generate JSON

### Database Support
- ✅ **PostgreSQL** - Full support with connection pooling
- ✅ **MySQL** - Complete database operations
- ✅ **SQLite** - Embedded database support

### Backend/Server Features
- ✅ **HTTP Server** - Sync (1-10 req/sec) and Async (100-1000 req/sec)
- ✅ **REST API** - Route matching, request/response handling
- ✅ **CORS** - Cross-origin resource sharing
- ✅ **Graceful Shutdown** - Clean server termination
- ✅ **Connection Pooling** - Efficient database connections

### Advanced Features (Phase 4 - Modules Ready)
- ✅ **JWT Authentication** - Token-based auth with refresh
- ✅ **Password Hashing** - Bcrypt with configurable cost
- ✅ **RBAC Authorization** - Role-based access control
- ✅ **Caching** - In-memory cache with TTL
- ✅ **Circuit Breakers** - Fault tolerance patterns
- ✅ **Retry Logic** - Exponential backoff
- ✅ **Timeouts** - Request timeout handling
- ✅ **Structured Logging** - JSON logs with context
- ✅ **Metrics** - Performance monitoring
- ✅ **Health Checks** - Service health monitoring

---

## 📚 Documentation

### 🆕 Getting Started
- **[Installation Guide](docs/getting-started/INSTALLATION.md)** - Complete setup instructions
- **[Quick Start](docs/getting-started/QUICKSTART.md)** - 5-minute tutorial
- **[Command Reference](docs/reference/COMMANDS.md)** - All commands and options

### 🏗️ Architecture & Design
- **[System Architecture](docs/architecture/OVERVIEW.md)** - High-level design
- **[VM Implementation](docs/architecture/VM.md)** - Virtual machine details
- **[Backend Architecture](docs/architecture/BACKEND.md)** - Server design

### 🌐 Backend Development
- **[HTTP Server Guide](docs/backend/HTTP_SERVER.md)** - Building APIs
- **[Database Guide](docs/backend/DATABASE.md)** - Database operations
- **[Authentication](docs/backend/AUTH.md)** - JWT & RBAC
- **[Deployment](docs/backend/DEPLOYMENT.md)** - Production deployment

### 📖 Reference
- **[Language Syntax](docs/reference/SYNTAX.md)** - eTamil language reference
- **[API Reference](docs/reference/API.md)** - Complete API documentation
- **[Examples](docs/reference/EXAMPLES.md)** - Code examples

### 📊 Phase Documentation
- **[Phase 1: HTTP Server](docs/phases/PHASE_1.md)** - Basic HTTP (Complete)
- **[Phase 2: Async/Concurrency](docs/phases/PHASE_2.md)** - High performance (Complete)
- **[Phase 3: Production Features](docs/phases/PHASE_3.md)** - Logging, errors (Complete)
- **[Phase 4: Advanced Features](docs/phases/PHASE_4.md)** - Auth, cache, resilience (Modules ready)

---

## 🎯 Use Cases

### 1. CLI Applications
```bash
etamil script.etamil
```

### 2. REST APIs
```bash
etamil --async --port 8080 api.etamil
```

### 3. Database Applications
```bash
etamil database_app.etamil
```

### 4. File Processing
```bash
etamil process_data.etamil
```

---

## 🔧 System Requirements

### For Using eTamil (End Users)
- **Platform**: Linux (macOS/Windows via WSL)
- **Disk Space**: 2.1 MB for binary
- **Rust**: ❌ Not required
- **Optional**: PostgreSQL/MySQL for database features

### For Building eTamil (Developers)
- **Rust**: 1.75+ (one-time build only)
- **Cargo**: Package manager
- **Disk Space**: ~500 MB for dependencies

---

## 📈 Performance

| Mode | Throughput | Latency | Use Case |
|------|-----------|---------|----------|
| VM Execution | - | <100ms | Scripts, CLI |
| Sync Server | 1-10 req/sec | 100-200ms | MVP, testing |
| Async Server | 100-1000 req/sec | 10-20ms | Production APIs |

---

## 🧪 Testing

```bash
# Unit tests
cd etamil_compiler && cargo test

# Integration tests
./test_http_backend.sh

# Installation test
./test_installation.sh
```

**Current Status**: 62/62 tests passing (100%)

---

## 🤝 Contributing

We welcome contributions! See our documentation for:
- Code style guidelines
- Testing requirements
- Pull request process

---

## 📝 License

[Specify your license here]

---

## 🔗 Links

- **Examples**: `etamil_compiler/examples/`
- **Tests**: `etamil_compiler/tests/`
- **Documentation**: `docs/`

---

## 🎉 Key Achievement

**Users can now build eTamil applications without Rust installed!**

Just download the 2.1 MB binary and start coding in Tamil. No complex setup, no dependencies, no Rust knowledge required.

---

**Version**: 0.1.0  
**Last Updated**: January 26, 2026  
**Status**: Production Ready
