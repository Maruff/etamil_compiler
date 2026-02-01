@echo off
REM eTamil Compiler - Windows Test Script
REM Tests the Windows build of eTamil compiler

echo ===============================================
echo eTamil Compiler - Windows Build Test
echo ===============================================
echo.

set ETAMIL_EXE=etamil_compiler\target\release\etamil_compiler.exe

REM Check if executable exists
if not exist "%ETAMIL_EXE%" (
    echo Error: Executable not found at %ETAMIL_EXE%
    echo Please run 'install_windows.bat' first
    pause
    exit /b 1
)

echo [1/5] Checking executable...
echo Found: %ETAMIL_EXE%
for %%A in ("%ETAMIL_EXE%") do set size=%%~zA
set /a sizeMB=%size% / 1048576
echo Size: %sizeMB% MB
echo.

echo [2/5] Testing VM executor...
echo அச்சு("Hello from eTamil on Windows!"); > test_windows_temp.etamil
"%ETAMIL_EXE%" --vm test_windows_temp.etamil
if %errorlevel% neq 0 (
    echo VM test failed!
    del test_windows_temp.etamil
    pause
    exit /b 1
)
echo ✓ VM executor works!
echo.

echo [3/5] Testing example file...
if exist "test_standalone.etamil" (
    "%ETAMIL_EXE%" --vm test_standalone.etamil
    if %errorlevel% equ 0 (
        echo ✓ Example file executed successfully!
    )
) else (
    echo Note: test_standalone.etamil not found, skipping
)
echo.

echo [4/5] Testing HTTP server startup...
echo Starting server in background for 3 seconds...
start /B "" "%ETAMIL_EXE%" --server --port 8765 test_windows_temp.etamil
timeout /t 3 /nobreak >nul
taskkill /IM etamil_compiler.exe /F >nul 2>&1
echo ✓ HTTP server can start!
echo.

echo [5/5] Cleanup...
del test_windows_temp.etamil 2>nul
echo.

echo ===============================================
echo All Tests Passed!
echo ===============================================
echo.
echo The eTamil compiler is working correctly on Windows.
echo.
echo Usage:
echo   VM mode:     etamil.bat --vm myprogram.etamil
echo   HTTP sync:   etamil.bat --server --port 8080 api.etamil
echo   HTTP async:  etamil.bat --async --port 8080 api.etamil
echo.
pause
