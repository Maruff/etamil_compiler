@echo off
REM eTamil Compiler - Windows Installation Script
REM This script builds and installs the eTamil compiler on Windows

echo ===============================================
echo eTamil Compiler - Windows Installation
echo ===============================================
echo.

REM Check if Rust is installed
where cargo >nul 2>nul
if %errorlevel% neq 0 (
    echo Error: Rust is not installed!
    echo.
    echo Please install Rust from: https://rustup.rs/
    echo After installation, restart this script.
    pause
    exit /b 1
)

echo [1/4] Rust found: OK
cargo --version
echo.

REM Navigate to compiler directory
cd /d "%~dp0etamil_compiler"
if %errorlevel% neq 0 (
    echo Error: Could not find etamil_compiler directory
    pause
    exit /b 1
)

echo [2/4] Building eTamil compiler (this may take 5-10 minutes)...
echo.
cargo build --release
if %errorlevel% neq 0 (
    echo.
    echo Error: Build failed!
    echo Please check the error messages above.
    pause
    exit /b 1
)

echo.
echo [3/4] Build successful!
echo.

REM Check if executable was created
if not exist "target\release\etamil_compiler.exe" (
    echo Error: Executable not found after build
    pause
    exit /b 1
)

REM Get file size
for %%A in ("target\release\etamil_compiler.exe") do set size=%%~zA
set /a sizeMB=%size% / 1048576

echo Binary location: %CD%\target\release\etamil_compiler.exe
echo Binary size: %sizeMB% MB
echo.

echo [4/4] Setting up PATH (optional)
echo.
echo To use 'etamil' from anywhere, you have two options:
echo.
echo Option 1: Add to PATH manually
echo   Add this folder to your PATH environment variable:
echo   %~dp0
echo.
echo Option 2: Use the wrapper script
echo   Run 'etamil.bat' from the root directory:
echo   %~dp0etamil.bat
echo.

REM Test the executable
echo Testing the compiler...
"target\release\etamil_compiler.exe" --help >nul 2>&1
if %errorlevel% equ 0 (
    echo ✓ Compiler is working correctly!
) else (
    echo Note: Run with --vm, --server, or --async flag
)

echo.
echo ===============================================
echo Installation Complete!
echo ===============================================
echo.
echo Quick Start:
echo   1. Run VM mode:     etamil.bat --vm myprogram.etamil
echo   2. HTTP Sync:       etamil.bat --server --port 8080 api.etamil
echo   3. HTTP Async:      etamil.bat --async --port 8080 api.etamil
echo.
echo For more info, see: WINDOWS_INSTALLATION.md
echo.
pause
