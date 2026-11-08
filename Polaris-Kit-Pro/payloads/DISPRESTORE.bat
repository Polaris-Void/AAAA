@echo off
setlocal EnableExtensions EnableDelayedExpansion
title Display Driver Restore Utility

:: --- Privilege guard (no self-elevation; the launcher is elevated) ---
net session >nul 2>&1
if %errorLevel% NEQ 0 (
    echo [ERROR] Administrator privileges are required. Launch through Polaris-Kit-Pro.
    exit /b 1
)

:: --- Configuration ---
set "BACKUP_DIR=%SystemDrive%\Backup_Display_Driver"

cls
echo ===================================================
echo           DISPLAY DRIVER RESTORE UTILITY
echo ===================================================
echo.
echo [STATUS] Source Directory: %BACKUP_DIR%

:: --- Validate Backup Directory ---
if not exist "%BACKUP_DIR%" (
    echo [ERROR] Display backup directory not found.
    echo [ERROR] Expected path:
    echo %BACKUP_DIR%
    goto :END
)

dir /s /b "%BACKUP_DIR%\*.inf" >nul 2>&1
if !errorlevel! NEQ 0 (
    echo [WARN] No INF files found in backup directory.
    goto :END
)

:: --- Execute Restore ---
echo [INFO] Restoring Display drivers from backup.
echo [WARN] Driver packages from this folder will be installed.
echo Please wait...
echo ---------------------------------------------------

pnputil /add-driver "%BACKUP_DIR%\*.inf" /subdirs /install
set "PS_EXIT=!errorlevel!"
if !PS_EXIT! EQU 0 (
    echo ---------------------------------------------------
    echo [SUCCESS] Display drivers restored successfully.
    echo [INFO] A system restart is recommended to apply changes.
) else (
    echo ---------------------------------------------------
    echo [ERROR] Display driver restoration failed with exit code !PS_EXIT!.
)

:END
echo.
exit /b %PS_EXIT%
