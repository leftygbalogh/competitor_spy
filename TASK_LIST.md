# Numbered Task List

## Planning Metadata

- Plan ID: PLAN-SNK-001
- Source spec: FSP-SNK-001
- Mode: Greenfield
- Date: 2026-03-20

## Backlog

1. T-000: Create project file skeleton
- Source: FR-001..FR-007
- Status: Done
- Output: `snake.py`, `run_snake.sh`

2. T-001: Implement startup size checks and crash logging
- Source: FR-001, FR-006, NFR-004
- Status: Done
- Evidence: startup-too-small exits with printed message and `snake.log` entry
- Chronicle: CHR-SNK-001

3. T-002: Implement startup placement and pre-start wait behavior
- Source: FR-001
- Status: Done
- Evidence: centered head, random start direction, wait-for-key message
- Chronicle: CHR-SNK-002

4. T-003: Implement movement loop and direction control
- Source: FR-002
- Status: Done
- Evidence: steady speed, arrow key control, reverse-direction self-collision possible
- Chronicle: CHR-SNK-003

5. T-004: Implement apple spawning/growth/scoring rules
- Source: FR-003
- Status: Done
- Evidence: one apple only, empty-space spawn, +1 score, +1 growth behind head
- Chronicle: CHR-SNK-004

6. T-005: Implement wall/self collision and no-wrap
- Source: FR-004
- Status: Done
- Evidence: death on out-of-bounds and body hit
- Chronicle: CHR-SNK-005

7. T-006: Implement win condition
- Source: FR-005
- Status: Done
- Evidence: full-board final-apple path returns win
- Chronicle: CHR-SNK-006

8. T-007: Implement leaderboard policy
- Source: FR-007
- Status: Done
- Evidence: prompt only on new all-time high; new-high-only write policy
- Chronicle: CHR-SNK-007

9. T-008: Git Bash launcher compatibility
- Source: FR-002 runtime requirement
- Status: Done
- Evidence: `run_snake.sh` uses `winpty` in MSYS when available
- Chronicle: CHR-SNK-008

## Stage 3 Approval

- Approved by: Team Lead Agent (delegated)
- Approval date: 2026-03-20
- Notes: Planning complete; tasks authorized for build.
