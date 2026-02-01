@echo off
REM eTamil Cross-Platform Compatibility Test (Windows)

echo ==========================================
echo eTamil Cross-Platform Compatibility Test
echo ==========================================
echo Platform: Windows
echo Date: %date% %time%
echo.

REM Determine binary location
set ETAMIL_BIN=
if exist "etamil.bat" (
    set ETAMIL_BIN=etamil.bat
) else if exist "etamil_compiler\target\release\etamil_compiler.exe" (
    set ETAMIL_BIN=etamil_compiler\target\release\etamil_compiler.exe
) else (
    echo Error: eTamil compiler not found
    echo Please build first: install_windows.bat
    pause
    exit /b 1
)

echo Using: %ETAMIL_BIN%
echo.

REM Test counters
set PASSED=0
set FAILED=0

REM Test 1: VM Executor
echo [1/5] Testing VM Executor...
echo அச்சு("Hello from eTamil!"); > test_temp.etamil
%ETAMIL_BIN% --vm test_temp.etamil >nul 2>&1
if %errorlevel% equ 0 (
    echo ✓ VM Executor: PASS
    set /a PASSED+=1
) else (
    echo ✗ VM Executor: FAIL
    set /a FAILED+=1
)
echo.

REM Test 2: Example file execution
echo [2/5] Testing Example File...
if exist "test_standalone.etamil" (
    %ETAMIL_BIN% --vm test_standalone.etamil >nul 2>&1
    if %errorlevel% equ 0 (
        echo ✓ Example File Execution: PASS
        set /a PASSED+=1
    ) else (
        echo ✗ Example File Execution: FAIL
        set /a FAILED+=1
    )
) else (
    echo ℹ Example File: SKIPPED (file not found)
)
echo.

REM Test 3: HTTP Sync Server
echo [3/5] Testing HTTP Sync Server...
start /B "" %ETAMIL_BIN% --server --port 8765 test_temp.etamil >nul 2>&1
timeout /t 2 /nobreak >nul 2>&1
where curl >nul 2>&1
if %errorlevel% equ 0 (
    curl -s http://localhost:8765/ >nul 2>&1
    if %errorlevel% equ 0 (
        echo ✓ HTTP Sync Server: PASS
        set /a PASSED+=1
    ) else (
        echo ✗ HTTP Sync Server: FAIL
        set /a FAILED+=1
    )
) else (
    echo ℹ curl not available, checking process only
    tasklist | findstr /I "etamil_compiler.exe" >nul 2>&1
    if %errorlevel% equ 0 (
        echo ✓ HTTP Sync Server: PASS (process running)
        set /a PASSED+=1
    ) else (
        echo ✗ HTTP Sync Server: FAIL
        set /a FAILED+=1
    )
)
taskkill /IM etamil_compiler.exe /F >nul 2>&1
echo.

REM Test 4: HTTP Async Server
echo [4/5] Testing HTTP Async Server...
start /B "" %ETAMIL_BIN% --async --port 8766 test_temp.etamil >nul 2>&1
timeout /t 2 /nobreak >nul 2>&1
where curl >nul 2>&1
if %errorlevel% equ 0 (
    curl -s http://localhost:8766/ >nul 2>&1
    if %errorlevel% equ 0 (
        echo ✓ HTTP Async Server: PASS
        set /a PASSED+=1
    ) else (
        echo ✗ HTTP Async Server: FAIL
        set /a FAILED+=1
    )
) else (
    tasklist | findstr /I "etamil_compiler.exe" >nul 2>&1
    if %errorlevel% equ 0 (
        echo ✓ HTTP Async Server: PASS (process running)
        set /a PASSED+=1
    ) else (
        echo ✗ HTTP Async Server: FAIL
        set /a FAILED+=1
    )
)
taskkill /IM etamil_compiler.exe /F >nul 2>&1
echo.

REM Test 5: LLVM Backend (should not be available on Windows)
echo [5/5] Testing LLVM Backend...
%ETAMIL_BIN% --llvm test_temp.etamil 2>&1 | findstr /C:"not available" >nul
if %errorlevel% equ 0 (
    echo ℹ LLVM Backend: NOT AVAILABLE (expected on Windows)
) else (
    echo ? LLVM Backend: Unexpected result
)
echo.

REM Cleanup
del test_temp.etamil 2>nul
del output.ll 2>nul
del output.bin 2>nul

REM Summary
echo ==========================================
echo Test Summary
echo ==========================================
echo Passed: %PASSED%
echo Failed: %FAILED%
echo.

if %FAILED% equ 0 (
    echo ✓ All tests passed! eTamil is working correctly on Windows.
    echo.
    echo Platform compatibility: ✅ VERIFIED
    pause
    exit /b 0
) else (
    echo ✗ Some tests failed. Please check the errors above.
    pause
    exit /b 1
)
