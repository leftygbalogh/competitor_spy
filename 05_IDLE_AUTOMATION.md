# 05 Idle Automation

This template includes inheritable idle automation.

## What It Does

- After 5 minutes of repository inactivity:
  - Appends status snapshot to `memory.md`
  - Stages all changes with `git add -A`
- After 15 minutes of repository inactivity:
  - Creates a commit if staged changes exist

## Run on Windows

From project root:

```powershell
powershell -ExecutionPolicy Bypass -File scripts/idle-guard.ps1
```

## Run on Linux

From project root:

```bash
chmod +x scripts/idle-guard.sh
./scripts/idle-guard.sh
```

## Optional Timing Overrides

Windows:

```powershell
powershell -ExecutionPolicy Bypass -File scripts/idle-guard.ps1 -PollSeconds 30 -SaveAfterMinutes 5 -CommitAfterMinutes 15
```

Linux:

```bash
POLL_SECONDS=30 SAVE_AFTER_MINUTES=5 COMMIT_AFTER_MINUTES=15 ./scripts/idle-guard.sh
```

## Important Note

This automation tracks repository inactivity (file-change inactivity), which is a practical proxy for user idle time.
It supplements, but does not replace, explicit stage-completion milestone commits.
