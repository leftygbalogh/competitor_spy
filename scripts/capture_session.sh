#!/usr/bin/env bash
# capture_session.sh -- Linux/Bash screen and state capture script
# Usage: ./scripts/capture_session.sh <label> [competitor-spy args...]
#
# Runs competitor-spy with --log-level trace, piping all output (stdout and
# stderr) to a timestamped log file while also displaying on the terminal.
# The app's structured trace log is written by the app itself to STATE_LOG.
#
# Output artifacts:
#   docs/evidence/sessions/session_YYYYMMDD_HHMMSS_<label>.log  (screen capture)
#   docs/evidence/sessions/session_YYYYMMDD_HHMMSS_<label>_state.jsonl  (app trace)

set -euo pipefail

if [[ $# -lt 1 ]]; then
    echo "Usage: $0 <label> [competitor-spy args...]" >&2
    exit 1
fi

LABEL="$1"
shift

TIMESTAMP=$(date -u +"%Y%m%d_%H%M%S")
EVIDENCE_DIR="docs/evidence/sessions"
SCREEN_LOG="${EVIDENCE_DIR}/session_${TIMESTAMP}_${LABEL}.log"
STATE_LOG="${EVIDENCE_DIR}/session_${TIMESTAMP}_${LABEL}_state.jsonl"

mkdir -p "$EVIDENCE_DIR"

echo "=== Competitor Spy capture session ==="  | tee "$SCREEN_LOG"
echo "Label:     $LABEL"                       | tee -a "$SCREEN_LOG"
echo "Timestamp: $TIMESTAMP UTC"               | tee -a "$SCREEN_LOG"
echo "Args:      $*"                           | tee -a "$SCREEN_LOG"
echo "======================================" | tee -a "$SCREEN_LOG"

# Run with trace logging. stdout+stderr both captured and displayed.
# CSPY_STATE_LOG tells the app where to write its structured JSONL trace.
export CSPY_STATE_LOG="$STATE_LOG"

set +e
competitor-spy --log-level trace "$@" 2>&1 | tee -a "$SCREEN_LOG"
EXIT_CODE=${PIPESTATUS[0]}
set -e

echo "" | tee -a "$SCREEN_LOG"
echo "=== Exit code: $EXIT_CODE ==="   | tee -a "$SCREEN_LOG"
echo "Screen log: $SCREEN_LOG"        | tee -a "$SCREEN_LOG"
echo "State log:  $STATE_LOG"         | tee -a "$SCREEN_LOG"

exit $EXIT_CODE

