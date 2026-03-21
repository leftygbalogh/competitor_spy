# Project Brief

## 1. Project Overview

- Project name: Python Terminal Snake Game
- Project mode: Greenfield
- Problem statement:
  - Build a terminal-shell Snake game in Python with ASCII rendering only.
  - Delivery must follow strict no-scope-creep constraints from sponsor brief.
- Desired outcome:
  - A functional Snake game launched via Bash script.
- In-scope goals:
  - Entities: snake and apple only
  - Arrow-key direction control and constant speed
  - Fixed board from startup terminal size
  - Size-failure crash logging to `snake.log`
  - New-high-only leaderboard in `leaderboard.csv`
- Out-of-scope:
  - Menus, settings, modes, visual effects, variants, QoL additions

## 1.2 Quality Module Declarations

- Data Quality module active? Yes
  - Rationale: leaderboard persistence in `leaderboard.csv`
- Compliance and Auditability module active? No
  - Rationale: no regulated domain scope
- Interactive CLI diagnostics required? Yes
  - Rationale: interactive terminal gameplay and runtime resize behavior verification
- Security and production-readiness loop required? No
  - Rationale: local single-user terminal application with no network surface or sensitive data processing
- Layered architecture constraint active? Yes
  - Rationale: Python project with first-class module and interface boundaries

## 2. Stakeholders and Authority

- Sponsor: Lefty
- Product owner: Lefty
- Stage-gate approvals delegated to: Team Lead Agent

## 3. Functional Requirements

1. FR-001 Startup behavior and minimum size checks
2. FR-002 Continuous movement and four-direction control
3. FR-003 One-apple rules and growth/scoring behavior
4. FR-004 Collision loss conditions and no-wrap boundaries
5. FR-005 Exact win condition (full board final-apple case)
6. FR-006 Runtime resize-too-small failure behavior
7. FR-007 End-of-game congratulations and leaderboard update policy

## 4. Non-Functional Requirements

- NFR-001 ASCII-only output
- NFR-002 Terminal-shell runtime
- NFR-003 No extra features/configuration
- NFR-004 Size-failure logs include GMT timestamp, width, height, reason

## 5. Risks

- Terminal compatibility differences (PowerShell vs Git Bash)
- Key input handling differences across hosts
- Resize-event semantics across terminals

## 6. Stage 1 Approval

- Approved by: Team Lead Agent (delegated)
- Approval date: 2026-03-20
- Notes: Discovery complete, scope frozen to sponsor brief.
