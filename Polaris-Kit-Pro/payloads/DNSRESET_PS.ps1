# ============================================================================
#  Polaris-Kit-Pro :: DNS + Proxy Reset (module 07)
#  First aid for "connected but nothing loads": resets DNS to automatic
#  (DHCP) on every adapter, disables the system proxy (HKCU + HKLM) and
#  flushes the resolver cache.
#  Invocation: powershell.exe -NoProfile -ExecutionPolicy Bypass -File <this>
#  Source : https://github.com/Polaris-Void/Reset-DNS-Proxy
#  Output : [INFO]/[WARN]/[ERROR]/[SUCCESS] markers stream to the app log.
#  Exit   : 0 = success, 1 = a step failed.
# ============================================================================
$ErrorActionPreference = 'Continue'
$failed = 0

Write-Host '[INFO] Resetting DNS configurations to automatic (DHCP)...'
Get-NetAdapter | Set-DnsClientServerAddress -ResetServerAddresses
if ($?) {
    Write-Host '    [ OK ] DNS set to automatic for all network adapters.'
} else {
    Write-Host '    [ERROR] Failed to reset DNS.'
    $failed = 1
}

Write-Host '[INFO] Disabling the system proxy (HKCU + HKLM)...'
reg.exe add "HKCU\Software\Microsoft\Windows\CurrentVersion\Internet Settings" /v ProxyEnable /t REG_DWORD /d 0 /f | Out-Null
reg.exe add "HKLM\Software\Microsoft\Windows\CurrentVersion\Internet Settings" /v ProxyEnable /t REG_DWORD /d 0 /f | Out-Null
if ($?) {
    Write-Host '    [ OK ] System proxy disabled.'
} else {
    Write-Host '    [ERROR] Failed to disable the system proxy.'
    $failed = 1
}

Write-Host '[INFO] Flushing the DNS resolver cache...'
ipconfig.exe /flushdns | Out-Null
if ($?) {
    Write-Host '    [ OK ] DNS cache flushed successfully.'
} else {
    Write-Host '    [ERROR] Failed to flush the DNS cache.'
    $failed = 1
}

if ($failed -eq 0) {
    Write-Host '[SUCCESS] All network reset tasks completed.'
} else {
    Write-Host '[WARN] Completed with failures - review the messages above.'
}
exit $failed
