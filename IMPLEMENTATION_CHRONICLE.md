# Implementation Chronicle

## Entry CHR-SNK-001

- Task: T-001
- Requirement: FR-001, FR-006, NFR-004
- Decision: enforce startup minimum size check before any curses init.
- Why: guarantees required print+log+exit behavior even when rendering cannot start.
- Evidence: `startup_size_check` and `log_size_crash` implementation.

## Entry CHR-SNK-002

- Task: T-002
- Requirement: FR-001
- Decision: center head at `fixed_width // 2`, `fixed_height // 2`; pre-start loop waits for any key.
- Why: exact startup placement and wait behavior from brief.
- Evidence: pre-start state in `run_game`.

## Entry CHR-SNK-003

- Task: T-003
- Requirement: FR-002
- Decision: constant-tick movement with direction updates from arrow keys only.
- Why: player controls direction only; no speed control.
- Evidence: `KEY_TO_DIR`, movement loop, fixed `TICK_SECONDS`.

## Entry CHR-SNK-004

- Task: T-004
- Requirement: FR-003
- Decision: when apple eaten, insert old head behind new head: `[new_head, old_head] + rest`.
- Why: implements growth directly behind head exactly as specified.
- Evidence: apple-eat branch in game loop.

## Entry CHR-SNK-005

- Task: T-005
- Requirement: FR-004
- Decision: out-of-bounds and body-hit end game immediately.
- Why: strict wall/self collision loss semantics, no wraparound.
- Evidence: collision checks before snake update finalization.

## Entry CHR-SNK-006

- Task: T-006
- Requirement: FR-005
- Decision: win on full board after final apple consumption.
- Why: exact explicit win contract from brief.
- Evidence: `if len(snake) == fixed_width * fixed_height` after growth.

## Entry CHR-SNK-007

- Task: T-007
- Requirement: FR-007
- Decision: only write leaderboard when `score > previous_high`.
- Why: brief prohibits non-record writes and prompts.
- Evidence: conditional name prompt and write path in `main`.

## Entry CHR-SNK-008

- Task: T-008
- Requirement: runtime terminal support in Git Bash
- Decision: Bash launcher uses `winpty` on MSYS if available.
- Why: enables curses compatibility under Git Bash on Windows.
- Evidence: `run_snake.sh` conditional execution path.

## Stage 4 Approval

- Approved by: Team Lead Agent (delegated)
- Approval date: 2026-03-20
- Notes: Build complete with chronicle links for all tasks.
