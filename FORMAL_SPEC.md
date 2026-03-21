# Formal Specification

## 1. Metadata

- Spec ID: FSP-SNK-001
- Version: 1.0
- Mode: Greenfield
- Source brief: PROJECT_BRIEF.md
- Status: Approved

## 2. Behavioral Contracts

### 2.1 Board Contract

- Valid coordinates: `0 <= x <= maxX`, `0 <= y <= maxY`
- Any coordinate outside valid bounds is wall collision.
- Board dimensions are fixed at startup terminal size.

### 2.2 Startup Contract

- If startup terminal width < 11 or height < 11:
  - print error message
  - append crash record to `snake.log`
  - exit immediately
- Otherwise:
  - place head `:` at board center
  - choose random start direction
  - place one apple on empty cell
  - show `Press any key to start.`
  - wait for any key

### 2.3 Tick Contract

- Snake moves exactly one cell per tick.
- Direction can be changed only by arrow keys.
- Reverse direction is permitted; if head enters body, self-collision loss applies.
- If head enters apple cell:
  - score += 1 exactly
  - snake length += 1 exactly behind head
  - respawn one apple on empty cell

### 2.4 Endgame Contract

- Loss on wall collision or self-collision.
- Win when snake fills board except final apple cell and then consumes it.
- Display framed `Congratulations!` after game ends.

### 2.5 Leaderboard Contract

- File: `leaderboard.csv`
- If file missing, previous high score = 0.
- Ask player name only if current score > previous high.
- Name max length = 5.
- Record only new all-time highs.
- Insert new entry at top; keep max 10 entries.
- Show leaderboard only after recording new all-time high.

## 3. Decision Tables

### Startup size decision

| width>=11 | height>=11 | action |
| --- | --- | --- |
| yes | yes | continue |
| no | any | print+log+exit |
| any | no | print+log+exit |

### Leaderboard write decision

| score > previous_high | action |
| --- | --- |
| yes | prompt, write, display |
| no | no prompt, no write |

## 4. Traceability Seed

- FR-001 -> T-001, T-002
- FR-002 -> T-003
- FR-003 -> T-004
- FR-004 -> T-005
- FR-005 -> T-006
- FR-006 -> T-002, T-003
- FR-007 -> T-007

## 4.1 Quality Dimension Targets (Q2 Pack)

- Performance and efficiency targets:
  - Tick processing remains bounded to a single movement/update/render cycle per tick.
  - Leaderboard read/write operations complete without blocking gameplay loop (startup/endgame only).
- Reliability and resilience targets:
  - Terminal-too-small at startup and runtime must deterministically emit message + structured crash log + controlled exit.
  - Missing leaderboard file is handled as empty baseline (no crash).
- Maintainability over time targets:
  - Game loop, collision logic, and persistence logic remain separated as independently testable seams.
  - Leaderboard policy remains encapsulated to prevent gameplay-rule coupling.
- Not-applicable declarations:
  - Throughput/SLA metrics for multi-user load: N/A (single-user local terminal game).
  - Network resilience targets: N/A (no network calls).

## 5. Architecture and Design Decisions

- Decision: enforce layered structure with gameplay API separated from terminal input/output handling.
- Rationale: keeps domain behavior testable and preserves CLI independence for future interface extensions.

### 5.1 Layered Architecture (Q3-ARCH-01)

- Module interface definitions:
  - `game_state`: state transitions, movement, collision, win/loss resolution
  - `leaderboard_store`: read/write and ranking constraints for `leaderboard.csv`
  - `runtime_loop`: tick orchestration and input dispatch
- API surface:
  - Internal operations for move tick, apple consumption, collision evaluation, and leaderboard update decision
  - CLI layer calls API operations but does not host business rules
- CLI-to-API mapping:
  - arrow key event -> direction update API
  - tick event -> move/apply-rules API
  - game-end event -> leaderboard decision/write API
- GUI-to-API mapping:
  - N/A (no GUI in scope)

## 6. Stage 2 Approval

- Approved by: Team Lead Agent (delegated)
- Approval date: 2026-03-20
- Notes: Specification frozen for planning and build.
