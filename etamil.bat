@echo off
REM eTamil Compiler - Windows Wrapper Script
REM This script provides easy access to the eTamil compiler from anywhere

setlocal

REM Set the path to the eTamil compiler executable
set ETAMIL_EXE=%~dp0etamil_compiler\target\release\etamil_compiler.exe

REM Check if the executable exists
if not exist "%ETAMIL_EXE%" (
    echo Error: eTamil compiler not found at:
    echo %ETAMIL_EXE%
    echo.
    echo Please run 'install_windows.bat' first to build the compiler.
    exit /b 1
)

REM Pass all arguments to the eTamil compiler
"%ETAMIL_EXE%" %*

endlocal
