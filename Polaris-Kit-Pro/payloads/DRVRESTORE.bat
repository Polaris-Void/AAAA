@echo off
setlocal enabledelayedexpansion
title Driver Restore Utility

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
echo               DRIVER RESTORE UTILITY               
echo ===================================================
echo.
:: --- Validate Backup Directory ---
if not exist "%BACKUP_DIR%" (
    echo [ERROR] Backup directory not found at: %BACKUP_DIR%
    echo [ERROR] Please ensure the backup folder exists before restoring.
    goto :END
)

:: --- Execute Restore ---
echo [STATUS] Source Directory: %BACKUP_DIR%
echo [INFO] Restoring system drivers.
echo Please wait...
echo ---------------------------------------------------
pnputil /add-driver "%BACKUP_DIR%\*.inf" /subdirs /install
set "DRV_EXIT=!errorlevel!"
if !DRV_EXIT! EQU 0 (
    echo ---------------------------------------------------
    echo [SUCCESS] Drivers restored successfully.
    echo [INFO] A system restart is recommended to apply changes.
) else (
    echo ---------------------------------------------------
    echo [ERROR] Driver restoration failed with exit code !DRV_EXIT!.
)

:END
echo.
exit /b %DRV_EXIT%
