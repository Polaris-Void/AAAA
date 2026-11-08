@echo off
setlocal EnableExtensions EnableDelayedExpansion
set "APP=Polaris-Kit-Pro"
set "VERSION=3.0.0"
set "TARGET=x86_64-pc-windows-msvc"
title Polaris-Kit-Pro v3.0.0 Builder

rem Always CWD to script folder so double-click works from anywhere
cd /d "%~dp0" || (
    echo [ERROR] Cannot change directory to script folder.
    pause
    exit /b 1
)

echo ==============================================================
echo   Polaris-Kit-Pro v%VERSION%  ^|  Windows release build
echo ==============================================================
echo.
echo   Target : %TARGET%
echo   Folder : %CD%
echo.

rem --- 1) Rust toolchain presence checks -------------------------------------
where cargo >nul 2>&1
if errorlevel 1 (
    echo.
    echo [ERROR] Rust/Cargo was not found on PATH.
    echo.
    echo         Polaris-Kit-Pro is a native Rust application and must be
    echo         compiled with the stable Rust toolchain.
    echo.
    echo         Fix:  Install Rust from https://rustup.rs/  ^(Windows installer^)
    echo               Then OPEN A NEW TERMINAL and run Build-EXE.bat again.
    echo.
    echo         Tip:  After installing, verify with:  cargo --version
    echo.
    goto :fail
)

where rustup >nul 2>&1
if errorlevel 1 (
    echo.
    echo [ERROR] rustup was not found on PATH.
    echo.
    echo         Fix:  Install Rust from https://rustup.rs/
    echo.
    goto :fail
)

rem --- 2) Show versions (so the user can see what we detected) ---------------
echo [*] Detected toolchain:
call cargo --version
call rustup --version
echo.

rem --- 3) Ensure the Windows MSVC target is installed ------------------------
echo [*] Installing Rust target %TARGET% if needed...
call rustup target add %TARGET%
if errorlevel 1 (
    echo.
    echo [ERROR] Could not install target %TARGET%.
    echo         This usually means Rust/updater is offline or proxy-blocked.
    echo.
    goto :fail
)

rem --- 4) Run the actual release build ---------------------------------------
echo.
echo [*] Building %APP% ^(release, %TARGET%^) ...
echo     This can take a few minutes the first time. Please be patient.
echo.
call cargo build --locked --release --target %TARGET%
if errorlevel 1 (
    echo.
    echo [ERROR] Rust release build failed. See messages above.
    echo.
    echo         Common causes:
    echo           - MSVC build tools ^(Visual Studio C++^) not installed
    echo             Get "Build Tools for Visual Studio" from Microsoft.
    echo           - Pending Cargo.toml / Cargo.lock changes that do not
    echo             match the locked dependency set.
    echo.
    goto :fail
)

rem --- 5) Stage the executable into .\dist -----------------------------------
if not exist "dist" mkdir "dist"
copy /Y "target\%TARGET%\release\polaris-kit-pro.exe" "dist\%APP%.exe" >nul
if errorlevel 1 (
    echo.
    echo [ERROR] Could not stage the executable in dist\.
    echo         Expected: target\%TARGET%\release\polaris-kit-pro.exe
    echo.
    goto :fail
)

rem --- 5b) Ship the payloads folder next to the executable -------------------
rem Since v3.1.1 the launcher executes the scripts IN PLACE from payloads\;
rem a build without that folder would refuse to run any tool.
if not exist "payloads" (
    echo.
    echo [ERROR] payloads folder missing from the project root.
    echo         It must ship next to the executable.
    echo.
    goto :fail
)
xcopy /E /I /Y "payloads" "dist\payloads" >nul
if errorlevel 1 (
    echo.
    echo [ERROR] Could not copy payloads to dist\payloads.
    echo.
    goto :fail
)

echo.
echo ==============================================================
echo   BUILD OK
echo   Output : dist\%APP%.exe (+ payloads\ folder)
echo   Native Rust launcher ^| requests UAC before tool execution
echo   Batch and PowerShell utility payloads run hidden in-app
echo ==============================================================
echo.
echo You can close this window now, or press any key to exit.
pause >nul
exit /b 0

:fail
echo.
echo ==============================================================
echo   BUILD FAILED  ^|  see the error message above
echo ==============================================================
echo.
echo Press any key to close this window.
pause >nul
exit /b 1
