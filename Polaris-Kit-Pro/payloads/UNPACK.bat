@echo off
setlocal enabledelayedexpansion

:: Target directory: argument 1 (set by Polaris-Kit-Pro); when
:: omitted the script operates on its own folder (standalone use).
set "TARGET_DIR=%~dp0"
if not "%~1"=="" set "TARGET_DIR=%~1"
if "%TARGET_DIR:~-1%" neq "\" set "TARGET_DIR=%TARGET_DIR%\"
cd /d "%TARGET_DIR%"

echo Moving files from subdirectories to parent folder...
echo ---------------------------------------------------

:: Move files from all subfolders to current folder
for /r "%TARGET_DIR%" %%F in (*) do (
    if /i not "%%~fF"=="%~f0" (
        if /i not "%%~dpF"=="%TARGET_DIR%" (
            echo Moving: "%%~nxF"
            move /y "%%~fF" "%TARGET_DIR%" >nul
        )
    )
)

echo ---------------------------------------------------
echo Operation completed successfully.
echo.

exit /b 0
