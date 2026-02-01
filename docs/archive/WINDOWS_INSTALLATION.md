# eTamil Compiler - Windows Installation Guide

## Quick Install (Recommended)

### Prerequisites
- **Rust** (required for building): Download from [https://rustup.rs/](https://rustup.rs/)
- **Windows 10/11** with PowerShell or Command Prompt

### One-Click Installation

1. Open Command Prompt or PowerShell in the eTamil root directory
2. Run the installation script:

```batch
install_windows.bat
```

That's it! The script will:
- ✓ Check for Rust installation
- ✓ Build the compiler in release mode
- ✓ Verify the executable
- ✓ Provide usage instructions

**Build time**: 5-10 minutes (first time only)

---

## Manual Installation

If you prefer manual steps:

### Step 1: Install Rust

```powershell
# Download and run rustup-init.exe from https://rustup.rs/
# Or use winget (Windows 11):
winget install Rustlang.Rustup
```

Restart your terminal after installation.

### Step 2: Build the Compiler

```powershell
cd etamil_compiler
cargo build --release
```

### Step 3: Verify Installation

```powershell
.\target\release\etamil_compiler.exe --vm ..\test_standalone.etamil
```

---

## Usage

### Option 1: Using the Wrapper Script (Recommended)

The `etamil.bat` wrapper makes it easy to run from anywhere:

```batch
# VM execution (default)
etamil.bat myprogram.etamil

# HTTP Async Server (Production)
etamil.bat --async --port 8080 api.etamil

# HTTP Sync Server (Development)
etamil.bat --server --port 8080 api.etamil
```

### Option 2: Direct Execution

```powershell
# Navigate to the compiler directory
cd etamil_compiler

# Run directly
.\target\release\etamil_compiler.exe --vm ..\examples\hello.etamil
.\target\release\etamil_compiler.exe --async --port 8080 ..\examples\backend\api.etamil
```

### Option 3: Add to PATH

For system-wide access:

1. Copy the executable to a permanent location:
   ```powershell
   mkdir C:\eTamil
   copy etamil_compiler\target\release\etamil_compiler.exe C:\eTamil\etamil.exe
   ```

2. Add to PATH:
   - Open System Properties → Environment Variables
   - Add `C:\eTamil` to your PATH
   - Restart your terminal

3. Use anywhere:
   ```batch
   etamil --vm myprogram.etamil
   ```

---

## Command Reference

### VM Executor (Default)
```batch
etamil.bat myprogram.etamil
etamil.bat --vm myprogram.etamil
```
- Fast startup (<100ms)
- No external dependencies
- Best for: scripts, CLI tools

### HTTP Server - Async (Production)
```batch
etamil.bat --async --port 8080 api.etamil
etamil.bat --async --host 0.0.0.0 --port 8080 api.etamil
```
- 100-1000 req/sec throughput
- Multi-threaded with Tokio
- Best for: production APIs

### HTTP Server - Sync (Development)
```batch
etamil.bat --server --port 8080 api.etamil
```
- 1-10 req/sec throughput
- Single-threaded
- Best for: testing, MVP

---

## Examples

### Hello World
```batch
echo அச்சு("வணக்கம் உலகம்!"); > hello.etamil
etamil.bat --vm hello.etamil
```

### Start a Backend Server
```batch
etamil.bat --async --port 8080 etamil_compiler\examples\backend\hello_server.qmz
```

Test with curl:
```batch
curl http://localhost:8080/
curl http://localhost:8080/health
```

### Run Examples
```batch
cd etamil_compiler\examples

REM VM execution
..\..etamil.bat --vm simple_fileio.qmz
..\..etamil.bat --vm fileio_example.qmz

REM Backend examples
..\..etamil.bat --async --port 8080 backend\hello_server.qmz
```

---

## Troubleshooting

### "Rust is not installed"
**Solution**: Install Rust from [https://rustup.rs/](https://rustup.rs/) and restart your terminal.

### "Build failed - linking error"
**Solution**: Install Visual Studio Build Tools:
```powershell
winget install Microsoft.VisualStudio.2022.BuildTools
```
Or download from: https://visualstudio.microsoft.com/downloads/

### "LLVM not found" (expected)
**Note**: LLVM backend is not available on Windows. This is intentional.
- Use `--vm` (default) or `--server`/`--async` instead
- LLVM features are optional and not needed for core functionality

### "File not found" error
**Check**:
1. Executable exists: `etamil_compiler\target\release\etamil_compiler.exe`
2. Run from correct directory
3. Use relative paths: `etamil.bat --vm examples\hello.etamil`

### Port already in use
```batch
REM Use a different port
etamil.bat --async --port 8081 api.etamil
```

---

## Performance Characteristics

| Mode | Startup | Throughput | Memory | Use Case |
|------|---------|------------|--------|----------|
| VM | <100ms | N/A | ~5MB | Scripts, CLI |
| Sync Server | ~200ms | 1-10 req/s | ~10MB | Dev, Testing |
| Async Server | ~200ms | 100-1K req/s | ~20MB | Production |

**Binary Size**: ~15-20 MB (Windows)  
**Build Time**: 5-10 minutes (first build), ~30s (incremental)

---

## Windows-Specific Notes

### What's Different from Linux?

1. **No LLVM Backend**: The `--llvm` flag is not available
   - Linux/macOS: Full LLVM support
   - Windows: VM and HTTP backends only

2. **No Signal Handling**: Graceful shutdown via Ctrl+C works but without Unix signals
   - Impact: Minor, HTTP servers still handle shutdown correctly

3. **File Paths**: Use backslashes or forward slashes
   ```batch
   etamil.bat --vm examples\hello.etamil
   etamil.bat --vm examples/hello.etamil  REM Also works
   ```

### What Works the Same?

- ✅ VM bytecode execution
- ✅ HTTP sync/async servers
- ✅ Database connectivity (PostgreSQL, MySQL, SQLite, MongoDB, Redis)
- ✅ File I/O and CSV operations
- ✅ Encryption features
- ✅ All Phase 1-3 features

---

## Updating the Compiler

### Rebuild After Code Changes

```batch
cd etamil_compiler
cargo build --release
```

### Pull Latest Changes

```batch
git pull origin main
install_windows.bat
```

---

## Development Workflow

### Build Debug Version (faster compilation)
```powershell
cd etamil_compiler
cargo build
.\target\debug\etamil_compiler.exe --vm test.etamil
```

### Run Tests
```powershell
cd etamil_compiler
cargo test
```

### Check for Errors
```powershell
cargo check
```

---

## System Requirements

- **OS**: Windows 10 or later
- **RAM**: 4GB minimum, 8GB recommended
- **Disk**: 500MB for compiler + dependencies
- **Rust**: Latest stable (2024 edition)

---

## Support & Documentation

- **Architecture**: See [docs/architecture/OVERVIEW.md](docs/architecture/OVERVIEW.md)
- **Backend Guide**: See [docs/backend/HTTP_SERVER_IMPLEMENTATION.md](docs/backend/HTTP_SERVER_IMPLEMENTATION.md)
- **Examples**: Browse `etamil_compiler/examples/`
- **AI Instructions**: See [.github/copilot-instructions.md](.github/copilot-instructions.md)

---

## Uninstallation

To remove eTamil:

```powershell
# Remove build artifacts
cd etamil_compiler
cargo clean

# Remove executable (if copied to PATH)
del C:\eTamil\etamil.exe
```

---

**Last Updated**: January 31, 2026  
**Status**: Production Ready (Windows)  
**Platforms**: Windows 10/11 (x64)
