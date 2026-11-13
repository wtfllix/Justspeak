param(
  [Parameter(Mandatory=$true)][int]$ClientPid,
  [int]$Seconds = 300,
  [string]$Output = '.\m0-measurements'
)
$ErrorActionPreference = 'Stop'
New-Item -ItemType Directory -Force -Path $Output | Out-Null
$logicalCores = (Get-CimInstance Win32_ComputerSystem).NumberOfLogicalProcessors
$os = Get-CimInstance Win32_OperatingSystem
@{ utc = [DateTime]::UtcNow.ToString('o'); rootPid = $ClientPid; seconds = $Seconds; logicalCores = $logicalCores; os = $os.Caption; build = $os.BuildNumber; metric = 'WorkingSetPrivate of current process descendants; CPU normalized to whole-machine logical cores' } | ConvertTo-Json | Set-Content "$Output\environment.json"
$clock = [Diagnostics.Stopwatch]::StartNew()
$previous = @{}
$lastTime = $clock.Elapsed.TotalSeconds
$rows = [Collections.Generic.List[object]]::new()
while ($clock.Elapsed.TotalSeconds -lt $Seconds) {
  if (-not (Get-Process -Id $ClientPid -ErrorAction SilentlyContinue)) { throw 'Client exited. Test did not complete.' }
  $processes = @(Get-CimInstance Win32_Process)
  $ids = [Collections.Generic.HashSet[int]]::new()
  [void]$ids.Add($ClientPid)
  do {
    $added = $false
    foreach ($item in $processes) {
      if ($ids.Contains([int]$item.ParentProcessId) -and $ids.Add([int]$item.ProcessId)) { $added = $true }
    }
  } while ($added)
  $perf = @(Get-CimInstance Win32_PerfFormattedData_PerfProc_Process | Where-Object { $ids.Contains([int]$_.IDProcess) })
  if ($perf.Count -ne $ids.Count) { throw 'Incomplete process-tree memory sample. Retry and retain this failure.' }
  $privateWorkingSet = ($perf | Measure-Object WorkingSetPrivate -Sum).Sum
  $now = $clock.Elapsed.TotalSeconds
  $deltaCpu = 0.0
  $current = @{}
  foreach ($processId in $ids) {
    $p = Get-Process -Id $processId -ErrorAction Stop
    $key = "$processId/$($p.StartTime.ToUniversalTime().Ticks)"
    $current[$key] = $p.TotalProcessorTime.TotalSeconds
    if ($previous.ContainsKey($key)) { $deltaCpu += [Math]::Max(0, $current[$key] - $previous[$key]) }
  }
  if ($previous.Count -gt 0) {
    $rows.Add([pscustomobject]@{ utc=[DateTime]::UtcNow.ToString('o'); elapsedSeconds=$now; processCount=$ids.Count; privateWorkingSetBytes=$privateWorkingSet; cpuPercentWholeMachine=100*$deltaCpu/($now-$lastTime)/$logicalCores })
    $rows | Export-Csv -NoTypeInformation "$Output\samples.csv"
  }
  $previous = $current
  $lastTime = $now
  Start-Sleep -Seconds 1
}
if ($rows.Count -lt 2) { throw 'Insufficient samples' }
function Percentile($values, $fraction) { $sorted = @($values | Sort-Object); $sorted[[Math]::Max(0, [Math]::Ceiling($fraction*$sorted.Count)-1)] }
@{ samples=$rows.Count; memoryP50Bytes=(Percentile $rows.privateWorkingSetBytes 0.5); memoryPeakBytes=($rows.privateWorkingSetBytes | Measure-Object -Maximum).Maximum; cpuMeanPercent=($rows.cpuPercentWholeMachine | Measure-Object -Average).Average; cpuP95Percent=(Percentile $rows.cpuPercentWholeMachine 0.95); limitations='1s requested cadence; actual timestamps recorded. Exited transient processes may be missed. Does not measure audio latency or quality.' } | ConvertTo-Json | Set-Content "$Output\summary.json"
Write-Output "Recorded real samples in $Output. This does not establish voice acceptance."
