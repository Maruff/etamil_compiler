@echo off
call "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat" >nul 2>&1
cl /nologo /O2 /Fe:bin\tax_int_c.exe tax_int.c >nul
cl /nologo /O2 /Fe:bin\tax_double_c.exe tax_double.c >nul
cl /nologo /O2 /Fe:bin\empty_c.exe empty.c >nul
del *.obj 2>nul
echo C build done
