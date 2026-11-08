@echo off
title Directory Configurator
cls

:: 1. Privilege guard (no self-elevation; the launcher is elevated)
net session >nul 2>&1
if %errorLevel% == 0 goto gotAdmin
echo [ERROR] Administrator privileges are required. Launch through Polaris-Kit-Pro.
exit /b 1


:gotAdmin
:: Working directory: argument 1 (set by Polaris-Kit-Pro); when
:: omitted the script operates on its own folder (standalone use).
set "WORKDIR=%~dp0"
if not "%~1"=="" set "WORKDIR=%~1"
cd /d "%WORKDIR%"

echo Running operations in the target directory: %WORKDIR%
echo.

:: 2. Remove existing attributes of created files to allow overwriting
if exist "Android" attrib -h -s "Android" >nul 2>&1
if exist "System Volume Information !" attrib -h -s "System Volume Information !" >nul 2>&1
if exist "autorun.inf" attrib -h -s "autorun.inf" >nul 2>&1

:: 3. Create and write content to the core files
type nul > "Android"
type nul > "System Volume Information !"

(
echo [autorun]
echo icon  = .autorun.ico
echo label = Files
) > "autorun.inf"

:: 4. Apply Hidden and System attributes to all specified files
attrib +h +s "Android"
attrib +h +s "System Volume Information !"
attrib +h +s "autorun.inf"

:: Apply Hidden and System attributes only (if they exist)
if exist "System Volume Information" attrib +h +s "System Volume Information" >nul 2>&1
if exist "autorun.ico" attrib +h +s "autorun.ico" >nul 2>&1
if exist ".autorun.ico" attrib +h +s ".autorun.ico" >nul 2>&1
if exist ".cm0013" attrib +h +s ".cm0013" >nul 2>&1

echo Operations successfully completed.
echo.

exit /b 0
