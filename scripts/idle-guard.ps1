param(
    [int]$PollSeconds = 30,
    [int]$SaveAfterMinutes = 5,
    [int]$CommitAfterMinutes = 15
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

if (-not (Test-Path ".git")) {
    Write-Error "Run this from a git repository root."
}

function Get-RepoState {
    git status --porcelain
}

function Write-MemorySnapshot {
    $ts = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    $entry = @"

## $ts Auto Snapshot

- Timestamp: $ts
- Current stage: In progress
- Completed since last update: Auto-snapshot due to inactivity.
- In progress: Active work not changed during idle window.
- Decisions made: None recorded by automation.
- Open questions: None recorded by automation.
- Blockers: None recorded by automation.
- Next step: Resume from latest staged state.
"@
    Add-Content -Path "memory.md" -Value $entry
}

$lastState = Get-RepoState
$lastActivity = Get-Date
$savedForIdleWindow = $false
$committedForIdleWindow = $false

Write-Host "idle-guard started (Windows): save=$SaveAfterMinutes min, commit=$CommitAfterMinutes min, poll=$PollSeconds s"

while ($true) {
    Start-Sleep -Seconds $PollSeconds

    $currentState = Get-RepoState
    if ($currentState -ne $lastState) {
        $lastActivity = Get-Date
        $lastState = $currentState
        $savedForIdleWindow = $false
        $committedForIdleWindow = $false
        continue
    }

    $idleMinutes = ((Get-Date) - $lastActivity).TotalMinutes

    if (($idleMinutes -ge $SaveAfterMinutes) -and (-not $savedForIdleWindow)) {
        Write-MemorySnapshot
        git add -A | Out-Null
        $savedForIdleWindow = $true
        Write-Host "auto-save complete after $([math]::Round($idleMinutes, 1)) minutes idle"
    }

    if (($idleMinutes -ge $CommitAfterMinutes) -and (-not $committedForIdleWindow)) {
        git diff --cached --quiet
        if ($LASTEXITCODE -ne 0) {
            $msgTs = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
            git commit -m "chore: idle autosave $msgTs" | Out-Null
            Write-Host "auto-commit complete after $([math]::Round($idleMinutes, 1)) minutes idle"
        }
        $committedForIdleWindow = $true
    }
}
