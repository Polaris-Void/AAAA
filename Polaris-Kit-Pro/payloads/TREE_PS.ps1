# ============================================================================
#  Polaris-Kit-Pro :: Directory Tree Exporter (module 09)
#  Maps the target folder into a visual tree report with sizes and
#  creation/modification dates.
#  The report is written to "File List Log.txt" inside the target folder.
#  Invocation: powershell.exe -NoProfile -ExecutionPolicy Bypass -File <this>
#  Output : [INFO]/[WARN]/[ERROR]/[SUCCESS] markers stream to the app log.
#  Exit   : 0 = success, 1 = failure.
# ============================================================================
$ErrorActionPreference = 'Continue'

$t = (Get-Location).Path
if ($t.Length -gt 3) { $t = $t.TrimEnd([char]92) }
$log = Join-Path $t 'File List Log.txt'
$sw = [System.Diagnostics.Stopwatch]::StartNew()

Write-Host ('[INFO] Target: ' + $t)
Write-Host ('[INFO] Report: ' + $log)
Write-Host '[INFO] Scanning directory structure and extracting metadata...'

$global:d = 0
$global:f = 0
$global:sz = [long]0

function F-Sz([long]$b) {
    if ($b -ge 1GB) { return ('{0:N2} GB' -f ($b/1GB)) }
    if ($b -ge 1MB) { return ('{0:N2} MB' -f ($b/1MB)) }
    if ($b -ge 1KB) { return ('{0:N2} KB' -f ($b/1KB)) }
    return ('{0} B' -f $b)
}

$records = [System.Collections.Generic.List[PSObject]]::new()
$root = Get-Item -LiteralPath $t -ErrorAction SilentlyContinue
$rM = if ($root) { $root.LastWriteTime.ToString('yyyy-MM-dd HH:mm:ss') } else { '                   ' }
$rC = if ($root) { $root.CreationTime.ToString('yyyy-MM-dd HH:mm:ss') } else { '                   ' }
$records.Add([PSCustomObject]@{ T='.'; S='<DIR>'; M=$rM; C=$rC })

function Scan-Dir($dir, $pfx) {
    $items = @(Get-ChildItem -LiteralPath $dir -Force -ErrorAction SilentlyContinue | Sort-Object { -not $_.PSIsContainer }, Name)
    $c = $items.Count
    for ($i = 0; $i -lt $c; $i++) {
        $it = $items[$i]
        if ($it.FullName -eq $log) { continue }
        $last = ($i -eq ($c - 1))
        $br = if ($last) { [char]0x2514 + [char]0x2500 + [char]0x2500 + ' ' } else { [char]0x251C + [char]0x2500 + [char]0x2500 + ' ' }
        $nxt = if ($last) { $pfx + '    ' } else { $pfx + [char]0x2502 + '   ' }
        $cDate = $it.CreationTime.ToString('yyyy-MM-dd HH:mm:ss')
        $mDate = $it.LastWriteTime.ToString('yyyy-MM-dd HH:mm:ss')
        if ($it.PSIsContainer) {
            $global:d++
            $records.Add([PSCustomObject]@{ T=($pfx+$br+$it.Name+'/'); S='<DIR>'; M=$mDate; C=$cDate })
            Scan-Dir $it.FullName $nxt
        } else {
            $global:f++
            $global:sz += $it.Length
            $records.Add([PSCustomObject]@{ T=($pfx+$br+$it.Name); S=(F-Sz $it.Length); M=$mDate; C=$cDate })
        }
    }
}

Scan-Dir $t ''
$sw.Stop()

$maxT = 36
foreach ($r in $records) { if ($r.T.Length -gt $maxT) { $maxT = $r.T.Length } }
$colW = $maxT + 2
$bLen = [Math]::Max(95, ($colW + 56))
$banner = [string]('=' * $bLen)

function C-Txt($txt, $w) {
    $pL = [Math]::Max(0, [int](($w - $txt.Length) / 2))
    $pR = [Math]::Max(0, $w - $txt.Length - $pL)
    return ((' ' * $pL) + $txt + (' ' * $pR))
}

$fmt = '  {0,-' + $colW + '}  {1,10}   {2,19}  {3,19}'
$sep = '  ' + [string]('-'*$colW) + '  ' + [string]('-'*10) + '   ' + [string]('-'*19) + '  ' + [string]('-'*19)

$w = New-Object System.IO.StreamWriter($log, $false, (New-Object System.Text.UTF8Encoding($true)))
$w.WriteLine($banner)
$w.WriteLine((C-Txt 'DIRECTORY TREE & FILE AUDIT REPORT' $bLen))
$w.WriteLine($banner)
$w.WriteLine(('  Target Path  :  {0}' -f $t))
$w.WriteLine(('  Generated On :  {0}' -f (Get-Date -Format 'yyyy-MM-dd HH:mm:ss')))
$w.WriteLine($banner)
$w.WriteLine('')
$w.WriteLine(($fmt -f 'PATH / TREE STRUCTURE', '   SIZE   ', '   DATE MODIFIED   ', '   DATE CREATED    '))
$w.WriteLine($sep)
foreach ($r in $records) { $w.WriteLine(($fmt -f $r.T, $r.S, $r.M, $r.C)) }
$w.WriteLine($sep)
$w.WriteLine('')

$totSz = F-Sz $global:sz
$w.WriteLine($banner)
$w.WriteLine((C-Txt 'STATISTICAL SUMMARY' $bLen))
$w.WriteLine($banner)
$w.WriteLine(('  Total Folders   :  {0:N0}' -f $global:d))
$w.WriteLine(('  Total Files     :  {0:N0}' -f $global:f))
$w.WriteLine(('  Total File Size :  {0} ({1:N0} Bytes)' -f $totSz, $global:sz))
$w.WriteLine(('  Execution Time  :  {0:N2} Seconds' -f $sw.Elapsed.TotalSeconds))
$w.WriteLine($banner)
$w.WriteLine((C-Txt 'END OF REPORT' $bLen))
$w.WriteLine($banner)
$w.Close()

Write-Host ('[ OK ] Folders scanned : ' + ('{0:N0}' -f $global:d))
Write-Host ('[ OK ] Files processed : ' + ('{0:N0}' -f $global:f))
Write-Host ('[ OK ] Total data size : ' + $totSz)
Write-Host ('[ OK ] Execution time  : ' + ('{0:N2}' -f $sw.Elapsed.TotalSeconds) + ' seconds')
Write-Host '[SUCCESS] Exported To : File List Log.txt'
exit 0
