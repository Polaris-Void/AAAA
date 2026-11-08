# ============================================================================
#  Polaris-Kit-Pro :: Display Driver Backup (module 05)
#  Surgically exports third-party GPU display-class drivers (oem*.inf whose
#  class GUID is {4d36e968-e325-11ce-bfc1-08002be10318} or class name Display)
#  to %SystemDrive%\Backup_Display_Driver using pnputil.
#  Invocation: powershell.exe -NoProfile -ExecutionPolicy Bypass -File <this>
#  Source : https://github.com/Polaris-Void/Win-Display-Drv-Backup
#  Output : [INFO]/[WARN]/[ERROR]/[SUCCESS] markers stream to the app log.
#  Exit   : 0 = success, 1 = export failure, 2 = nothing to back up.
# ============================================================================
$ErrorActionPreference = 'Stop'
$dest = "$env:SystemDrive\Backup_Display_Driver"
$guid = '{4d36e968-e325-11ce-bfc1-08002be10318}'

Write-Host '[INFO] Exporting Display class drivers only (third-party oem*.inf).'

if (-not (Test-Path -LiteralPath $dest)) {
    try {
        New-Item -ItemType Directory -Path $dest -Force | Out-Null
    } catch {
        Write-Host '[ERROR] Failed to create the backup directory. Check permissions.'
        exit 1
    }
}
Write-Host ('[INFO] Target directory: ' + $dest)

try {
    $drivers = @(Get-WindowsDriver -Online | Where-Object {
        (($_.ClassGuid -eq $guid) -or ($_.ClassName -eq 'Display')) -and ($_.Driver -like 'oem*.inf')
    })
} catch {
    Write-Host '[ERROR] Get-WindowsDriver failed. Ensure the Windows image is healthy.'
    exit 1
}

if ($drivers.Count -eq 0) {
    Write-Host '[WARN] No third-party Display class drivers found.'
    exit 2
}

$failed = 0
foreach ($d in $drivers) {
    Write-Host ('[DRIVER] ' + $d.Driver + ' | ' + $d.ProviderName)
    pnputil.exe /export-driver $d.Driver $dest | Out-Null
    if ($LASTEXITCODE -ne 0) {
        Write-Host ('[ERROR] Export failed: ' + $d.Driver)
        $failed++
    }
}

if ($failed -gt 0) {
    Write-Host ('[ERROR] Display driver export finished with ' + $failed + ' failure(s).')
    exit 1
}
Write-Host '[SUCCESS] Display drivers backed up successfully.'
exit 0
