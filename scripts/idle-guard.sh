#!/usr/bin/env bash
set -euo pipefail

POLL_SECONDS="${POLL_SECONDS:-30}"
SAVE_AFTER_MINUTES="${SAVE_AFTER_MINUTES:-5}"
COMMIT_AFTER_MINUTES="${COMMIT_AFTER_MINUTES:-15}"

if [[ ! -d .git ]]; then
  echo "Run this from a git repository root." >&2
  exit 1
fi

repo_state() {
  git status --porcelain
}

write_memory_snapshot() {
  local ts
  ts="$(date '+%Y-%m-%d %H:%M:%S')"
  cat >> memory.md <<EOF

## ${ts} Auto Snapshot

- Timestamp: ${ts}
- Current stage: In progress
- Completed since last update: Auto-snapshot due to inactivity.
- In progress: Active work not changed during idle window.
- Decisions made: None recorded by automation.
- Open questions: None recorded by automation.
- Blockers: None recorded by automation.
- Next step: Resume from latest staged state.
EOF
}

last_state="$(repo_state)"
last_activity_epoch="$(date +%s)"
saved_for_idle_window=0
committed_for_idle_window=0

echo "idle-guard started (Linux): save=${SAVE_AFTER_MINUTES} min, commit=${COMMIT_AFTER_MINUTES} min, poll=${POLL_SECONDS} s"

while true; do
  sleep "$POLL_SECONDS"

  current_state="$(repo_state)"
  if [[ "$current_state" != "$last_state" ]]; then
    last_activity_epoch="$(date +%s)"
    last_state="$current_state"
    saved_for_idle_window=0
    committed_for_idle_window=0
    continue
  fi

  now_epoch="$(date +%s)"
  idle_minutes=$(( (now_epoch - last_activity_epoch) / 60 ))

  if (( idle_minutes >= SAVE_AFTER_MINUTES )) && (( saved_for_idle_window == 0 )); then
    write_memory_snapshot
    git add -A
    saved_for_idle_window=1
    echo "auto-save complete after ${idle_minutes} minutes idle"
  fi

  if (( idle_minutes >= COMMIT_AFTER_MINUTES )) && (( committed_for_idle_window == 0 )); then
    if ! git diff --cached --quiet; then
      msg_ts="$(date '+%Y-%m-%d %H:%M:%S')"
      git commit -m "chore: idle autosave ${msg_ts}" >/dev/null
      echo "auto-commit complete after ${idle_minutes} minutes idle"
    fi
    committed_for_idle_window=1
  fi
done
