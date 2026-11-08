@echo off
setlocal enabledelayedexpansion
title Driver Backup Utility

:: --- Privilege guard (no self-elevation; the launcher is elevated) ---
net session >nul 2>&1
if %errorLevel% NEQ 0 (
    echo [ERROR] Administrator privileges are required. Launch through Polaris-Kit-Pro.
    exit /b 1
)

:: --- Configuration ---
set "BACKUP_DIR=%SystemDrive%\Backup_Driver"
cls

echo ===================================================
echo               DRIVER BACKUP UTILITY                
echo ===================================================
echo.
echo [STATUS] Target Directory: %BACKUP_DIR%

:: --- Create Directory ---
if not exist "%BACKUP_DIR%" (
    echo [INFO] Creating backup directory...
    mkdir "%BACKUP_DIR%" >nul 2>&1
    if !errorlevel! NEQ 0 (
        echo [ERROR] Failed to create directory. Check permissions.
        goto :END
    )
)

:: --- Execute Backup ---
echo [INFO] Exporting system drivers.
echo Please wait...
echo ---------------------------------------------------
dism /online /export-driver /destination:"%BACKUP_DIR%"
set "DRV_EXIT=!errorlevel!"
if !DRV_EXIT! EQU 0 (
    echo ---------------------------------------------------
    echo [SUCCESS] Drivers backed up successfully.
) else (
    echo ---------------------------------------------------
    echo [ERROR] Driver export failed with exit code !DRV_EXIT!.
)

:END
echo.
exit /b %DRV_EXIT%
