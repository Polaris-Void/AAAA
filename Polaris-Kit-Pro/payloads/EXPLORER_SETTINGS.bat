@echo off
setlocal EnableExtensions DisableDelayedExpansion
title File Explorer Settings - Windows 10 / 11

rem =====================================================================
rem  FILE EXPLORER SETTINGS  (Windows 10 + Windows 11)
rem  Version 1.0
rem ---------------------------------------------------------------------
rem  Applies exactly the options shown in the reference screenshots:
rem
rem    General tab
rem      - Open File Explorer to ................. This PC
rem      - Show recently used files .............. OFF (unchecked)
rem      - Show frequently used folders .......... OFF (unchecked)
rem      - Show files from Office.com ............ OFF (unchecked)
rem
rem    View tab
rem      - Navigation pane ....................... already ON by default,
rem        intentionally NOT modified: its state lives inside a binary
rem        UI blob (PageSpaceControlSizer) that also stores the pane
rem        width, and touching it would break the "nothing unrelated"
rem        safety rule.
rem      - Compact view .......................... OFF (unchecked)
rem      - Item check boxes ...................... ON  (checked)
rem      - File name extensions .................. ON  (checked)
rem      - Hidden items .......................... ON  (checked)
rem
rem  SAFETY RULES
rem    - Only the eight per-user (HKCU) values listed above are written.
rem    - ShellState / Bags / BagMRU binaries are never touched.
rem    - No policy keys, no value is deleted, nothing else is modified.
rem    - Run while signed in to the account you want to configure,
rem      because HKCU belongs to the account that started the script.
rem =====================================================================

set "REG_FAIL=0"
set "EXP=HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer"
set "ADV=%EXP%\Advanced"

rem =====================================================================
rem  STEP 1 - Privilege guard (no self-elevation; the launcher is elevated)
rem =====================================================================
net session >nul 2>&1
if errorlevel 1 (
    echo  [ERROR] Administrator rights are required. Launch through Polaris-Kit-Pro.
    exit /b 1
)

rem =====================================================================
rem  STEP 2 - Apply the File Explorer settings (exact values only)
rem =====================================================================
echo.
echo =====================================================================
echo   APPLYING FILE EXPLORER SETTINGS
echo =====================================================================
echo.

call :SetDword "%ADV%" "LaunchTo" 1 "Open File Explorer to: This PC"
echo.
call :SetDword "%EXP%" "ShowRecent" 0 "Show recently used files                [UNCHECKED]"
call :SetDword "%EXP%" "ShowFrequent" 0 "Show frequently used folders            [UNCHECKED]"
call :SetDword "%EXP%" "ShowCloudFilesInQuickAccess" 0 "Show files from Office.com              [UNCHECKED]"
echo.
call :SetDword "%ADV%" "UseCompactMode" 0 "Compact view                            [UNCHECKED]"
call :SetDword "%ADV%" "AutoCheckSelect" 1 "Item check boxes                        [CHECKED]"
call :SetDword "%ADV%" "HideFileExt" 0 "File name extensions                    [CHECKED]"
call :SetDword "%ADV%" "Hidden" 1 "Hidden items                            [CHECKED]"
echo.

if "%REG_FAIL%"=="0" (
    echo   Registry: all requested values were written successfully.
) else (
    echo   Registry WARNING: %REG_FAIL% value^(s^) could not be written.
)

rem =====================================================================
rem  STEP 3 - Restart File Explorer so the changes take effect
rem =====================================================================
echo.
echo   Restarting File Explorer...

taskkill /f /im explorer.exe >nul 2>&1
call :Sleep 2

tasklist /fi "imagename eq explorer.exe" /nh 2>nul | find /i "explorer.exe" >nul
if errorlevel 1 (
    start "" "%SystemRoot%\explorer.exe"
    call :Sleep 3
)

tasklist /fi "imagename eq explorer.exe" /nh 2>nul | find /i "explorer.exe" >nul
if errorlevel 1 (
    echo   [WARNING] File Explorer did not restart automatically.
    echo             Open Task Manager ^(Ctrl+Shift+Esc^) and start explorer.exe.
    set "REG_FAIL=1"
) else (
    echo   File Explorer restarted successfully.
)

rem =====================================================================
rem  STEP 4 - Result (exit code: 0 clean, 1 finished with warnings)
rem =====================================================================
echo.
echo =====================================================================
if "%REG_FAIL%"=="0" (
    echo   SUCCESS - the File Explorer settings have been applied.
) else (
    echo   FINISHED WITH WARNINGS - please review the messages above.
)
echo =====================================================================
echo.
echo   This window will close automatically in 5 seconds...
exit


rem =====================================================================
rem  SUBROUTINES
rem =====================================================================

:SetDword
rem %1 = registry key   %2 = value name   %3 = DWORD data   %4 = label
reg add %~1 /v %~2 /t REG_DWORD /d %~3 /f >nul 2>&1
if errorlevel 1 (
    echo   [FAIL] %~4
    set /a REG_FAIL+=1
) else (
    echo   [ OK ] %~4
)
exit /b 0

:Sleep
rem %1 = seconds to wait; independent of stdin redirection
if "%~1"=="" exit /b 0
set /a "_PING_COUNT=%~1+1"
ping 127.0.0.1 -n %_PING_COUNT% >nul 2>&1
exit /b 0
