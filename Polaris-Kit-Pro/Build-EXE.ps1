[CmdletBinding()]
param(
    [ValidateSet('x86_64-pc-windows-msvc', 'x86_64-pc-windows-gnu')]
    [string] $Target = 'x86_64-pc-windows-msvc'
)

$ErrorActionPreference = 'Stop'
$AppName = 'Polaris-Kit-Pro'
$ProjectRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location -LiteralPath $ProjectRoot

function Pause-Exit($code) {
    Write-Host ''
    Write-Host 'Press Enter to close this window...' -ForegroundColor DarkGray
    [void](Read-Host)
    exit $code
}

function Fail($message) {
    Write-Host ''
    Write-Host "[ERROR] $message" -ForegroundColor Red
    Pause-Exit 1
}

Write-Host '==============================================================' -ForegroundColor Cyan
Write-Host "  $AppName v3.0.0  |  Windows release build" -ForegroundColor Cyan
Write-Host '==============================================================' -ForegroundColor Cyan
Write-Host ''
Write-Host "  Target : $Target"
Write-Host "  Folder : $ProjectRoot"
Write-Host ''

# --- 1) Toolchain presence checks -------------------------------------------
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Fail 'Rust/Cargo was not found on PATH. Install the stable Rust toolchain from https://rustup.rs/ and open a NEW terminal.'
}
if (-not (Get-Command rustup -ErrorAction SilentlyContinue)) {
    Fail 'rustup was not found on PATH. Install the stable Rust toolchain from https://rustup.rs/.'
}

# --- 2) Show versions -------------------------------------------------------
Write-Host '[*] Detected toolchain:' -ForegroundColor Green
& cargo --version
& rustup --version
Write-Host ''

# --- 3) Ensure target is installed ------------------------------------------
Write-Host "[*] Installing Rust target $Target if needed..." -ForegroundColor Green
& rustup target add $Target
if ($LASTEXITCODE -ne 0) { Fail "Could not install target $Target." }

# --- 4) Run the actual release build ----------------------------------------
Write-Host ''
Write-Host "[*] Building $AppName (release, $Target) ..." -ForegroundColor Green
Write-Host '     This can take a few minutes the first time. Please be patient.'
Write-Host ''
& cargo build --locked --release --target $Target
if ($LASTEXITCODE -ne 0) {
    Write-Host ''
    Write-Host 'Common causes:' -ForegroundColor Yellow
    Write-Host '  - MSVC build tools (Visual Studio C++) not installed.'
    Write-Host '    Get "Build Tools for Visual Studio" from Microsoft.'
    Write-Host '  - Cargo.lock mismatch.'
    Fail 'Rust release build failed. See messages above.'
}

# --- 5) Stage the executable into .\dist -------------------------------------
$Source = Join-Path $ProjectRoot "target\$Target\release\polaris-kit-pro.exe"
$Dist   = Join-Path $ProjectRoot 'dist'
$Dest   = Join-Path $Dist "$AppName.exe"

if (-not (Test-Path -LiteralPath $Source -PathType Leaf)) {
    Fail "Build output not found: $Source"
}
New-Item -ItemType Directory -Path $Dist -Force | Out-Null
Copy-Item -LiteralPath $Source -Destination $Dest -Force

Write-Host ''
Write-Host '==============================================================' -ForegroundColor Green
Write-Host "  BUILD OK" -ForegroundColor Green
Write-Host "  Output : $Dest" -ForegroundColor Green
Write-Host '  Native Rust launcher | requests UAC before tool execution' -ForegroundColor Green
Write-Host '  Batch and PowerShell utility payloads run hidden in-app'   -ForegroundColor Green
Write-Host '==============================================================' -ForegroundColor Green

Pause-Exit 0
