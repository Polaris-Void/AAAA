@echo off
rem ============================================================================
rem  Polaris-Kit-Pro :: NCSI No-Internet Fix - NLA service restart
rem  Restarts the Network Location Awareness service (NlaSvc) so Windows
rem  re-evaluates NCSI active probing immediately. Runs inside an elevated
rem  session started by Polaris-Kit-Pro (single UAC prompt at launch).
rem  Source : https://github.com/Polaris-Void/Windows-No-Internet-Fix
rem  Output : [INFO]/[WARN]/[ERROR]/[SUCCESS] markers stream to the app log.
rem  Exit   : 0 = success, 1 = service control failure.
rem ============================================================================
setlocal EnableExtensions
echo [INFO] Restarting the Network Location Awareness service (NlaSvc)...
net stop NlaSvc /y
if errorlevel 1 (
    echo [ERROR] Failed to stop the NlaSvc service.
    exit /b 1
)
net start NlaSvc
if errorlevel 1 (
    echo [ERROR] Failed to start the NlaSvc service.
    exit /b 1
)
echo [SUCCESS] NlaSvc restarted - connectivity probing will re-evaluate.
exit /b 0
