# capture_session.ps1 -- Windows/PowerShell screen and state capture script
# Usage: .\scripts\capture_session.ps1 <label> [competitor-spy args...]
#
# Runs competitor-spy with --log-level trace, piping all output (stdout and
# stderr) to a timestamped log file while also displaying on the terminal.
# The app's structured trace log is written by the app itself to STATE_LOG.
#
# Output artifacts:
#   docs\evidence\sessions\session_YYYYMMDD_HHMMSS_<label>.log  (screen capture)
#   docs\evidence\sessions\session_YYYYMMDD_HHMMSS_<label>_state.jsonl  (app trace)

param(
    [Parameter(Mandatory = $true, Position = 0)]
    [string]$Label,

    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$AppArgs
)

$Timestamp = (Get-Date -AsUTC).ToString("yyyyMMdd_HHmmss")
$EvidenceDir = "docs\evidence\sessions"
$ScreenLog = "$EvidenceDir\session_${Timestamp}_${Label}.log"
$StateLog  = "$EvidenceDir\session_${Timestamp}_${Label}_state.jsonl"

if (-not (Test-Path $EvidenceDir)) {
    New-Item -ItemType Directory -Path $EvidenceDir | Out-Null
}

function Write-Log {
    param([string]$Message)
    $Message | Tee-Object -FilePath $ScreenLog -Append
}

Write-Log "=== Competitor Spy capture session ==="
Write-Log "Label:     $Label"
Write-Log "Timestamp: $Timestamp UTC"
Write-Log "Args:      $($AppArgs -join ' ')"
Write-Log "======================================"

# Tell the app where to write its structured JSONL trace
$env:CSPY_STATE_LOG = $StateLog

# Run the binary; merge stderr into stdout so Tee-Object captures both
$allArgs = @("--log-level", "trace") + $AppArgs
$process = Start-Process -FilePath "competitor-spy" `
    -ArgumentList $allArgs `
    -NoNewWindow `
    -PassThru `
    -RedirectStandardOutput "$ScreenLog.tmp_stdout" `
    -RedirectStandardError  "$ScreenLog.tmp_stderr"

$process.WaitForExit()
$ExitCode = $process.ExitCode

# Merge stdout and stderr into screen log and display
Get-Content "$ScreenLog.tmp_stdout" | Tee-Object -FilePath $ScreenLog -Append
Get-Content "$ScreenLog.tmp_stderr" | Tee-Object -FilePath $ScreenLog -Append
Remove-Item "$ScreenLog.tmp_stdout", "$ScreenLog.tmp_stderr" -ErrorAction SilentlyContinue

Write-Log ""
Write-Log "=== Exit code: $ExitCode ==="
Write-Log "Screen log: $ScreenLog"
Write-Log "State log:  $StateLog"

exit $ExitCode
