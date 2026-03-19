# Numbered Task List

## Reference Standards

- Definition of Ready (DoR): see `02_WORKFLOW_STAGES.md`
- Definition of Done (DoD): see `02_WORKFLOW_STAGES.md`
- Implementation chronicle template: `templates/IMPLEMENTATION_CHRONICLE_TEMPLATE.md`

## 1. Planning Metadata

- Plan ID: PLAN-AGT-001
- Source formal spec: FORMAL_SPEC.md (FSP-AGT-001 v1.0)
- Version: 0.1-draft
- Project mode: Greenfield
- Owner: Lefty
- Date: 2026-03-19

## 2. Work Order Rules

- Tasks are strictly numbered.
- Each task references source spec requirement IDs.
- Each task must meet the DoR before work starts.
- Each task must meet the universal DoD before it is marked Done.
- Dependencies must be explicit.
- Progress is recorded against each task.

## 3. Task Backlog

1. T-001: Implement mode-first startup flow
- Description: Ensure startup always asks whether project is Greenfield or Brownfield before domain-specific discovery questions.
- Source requirements: FR-001
- Acceptance criteria:
  - First startup question is mode selection.
  - No domain assumptions made before mode is selected.
  - Mode recorded in project brief metadata.
- Dependencies: None
- Owner role: greenfield-domain-discovery-expert
- Test-first notes (TDD): Write failing test for startup path that incorrectly assumes project type; make it pass with mode-first prompt logic.
- Planned implementation chronicle entry: CHR-GOV-STARTUP
- DoR met? Yes
- Definition of done: inherits universal DoD from `02_WORKFLOW_STAGES.md`; add task-specific additions here:
  - Include one example startup transcript for Greenfield and one for Brownfield.
- Status: Not started
- Progress notes:

2. T-002: Enforce stage transition gate checks
- Description: Enforce explicit approval and stage-completion commit requirement before any next-stage work.
- Source requirements: FR-002, NFR-001
- Acceptance criteria:
  - Transition denied without explicit yes.
  - Transition denied if stage-completion commit missing.
  - Transition log captures denial reason.
- Dependencies: T-001
- Owner role: traceability-mapper
- Test-first notes (TDD): Add failing checks for approval-missing and commit-missing transitions.
- Planned implementation chronicle entry: CHR-GOV-STAGE-GATE
- DoR met? Yes
- Definition of done: inherits universal DoD from `02_WORKFLOW_STAGES.md`; add task-specific additions here:
  - Include audit example showing allowed and denied transitions.
- Status: Not started
- Progress notes:

3. T-003: Add practical task tracker flow
- Description: Implement simple per-task status flow (To do, In progress, Blocked, Done) with clear blocker reason capture.
- Source requirements: FR-003
- Acceptance criteria:
  - Status changes follow DoR/DoD rules.
  - Blocked state requires reason text.
  - Done state requires traceability links.
- Dependencies: T-001
- Owner role: tdd-driver
- Test-first notes (TDD): Start with failing tests for invalid status transitions.
- Planned implementation chronicle entry: CHR-GOV-TASK-DISCIPLINE
- DoR met? Yes
- Definition of done: inherits universal DoD from `02_WORKFLOW_STAGES.md`; add task-specific additions here:
  - Include one example task lifecycle record from start to done.
- Status: Not started
- Progress notes:

4. T-004: Implement cross-agent clarification routing
- Description: Enforce remit-holder consultation before escalation to Lefty.
- Source requirements: FR-004
- Acceptance criteria:
  - Ambiguity triggers peer clarification step first.
  - Escalation requires unresolved-ambiguity note.
  - Clarification trail stored in artifacts.
- Dependencies: T-001
- Owner role: tdd-navigator
- Test-first notes (TDD): Add failing tests for direct escalation path without peer consultation.
- Planned implementation chronicle entry: CHR-GOV-COLLAB
- DoR met? Yes
- Definition of done: inherits universal DoD from `02_WORKFLOW_STAGES.md`; add task-specific additions here:
  - Include one sample clarification exchange and one escalation case.
- Status: Not started
- Progress notes:

5. T-005: Preserve brownfield confidence gate template
- Description: Keep Brownfield uncertainty protocol ready for reuse and verify it blocks unsafe feature promises in Brownfield mode.
- Source requirements: FR-005
- Acceptance criteria:
  - Brownfield mode requires evidence checklist and confidence rating.
  - If confidence below threshold, plan blocks and suggests allowed actions.
  - Greenfield mode does not activate Brownfield gate.
- Dependencies: T-001
- Owner role: brownfield-parity-test-engineer
- Test-first notes (TDD): Add failing mode-specific tests to ensure gate only runs in Brownfield.
- Planned implementation chronicle entry: CHR-GOV-BROWNFIELD-GATE
- DoR met? Yes
- Definition of done: inherits universal DoD from `02_WORKFLOW_STAGES.md`; add task-specific additions here:
  - Include one Brownfield decision-table example.
- Status: Not started
- Progress notes:

6. T-006: Validate traceability and maintainability metrics flow
- Description: Ensure requirements -> tests -> chronicle mapping and trend metric capture are operational.
- Source requirements: NFR-002, NFR-003, NFR-004
- Acceptance criteria:
  - Every FR/NFR has mapped tests and chronicle IDs.
  - Missing links are reported as blockers.
  - Maintenability trend metric collection path is documented.
- Dependencies: T-002, T-003, T-004, T-005
- Owner role: maintainability-reviewer
- Test-first notes (TDD): Add failing checks for incomplete traceability matrix rows.
- Planned implementation chronicle entry: CHR-GOV-AUDIT
- DoR met? Yes
- Definition of done: inherits universal DoD from `02_WORKFLOW_STAGES.md`; add task-specific additions here:
  - Include one complete traceability row walkthrough example.
- Status: Not started
- Progress notes:

## 4. XP and DDD Checks

- Pairing or review approach: TDD driver/navigator pairing for behavior tasks; specialist review for routing and traceability tasks.
- Refactoring checkpoints: after T-003 and after T-006.
- Domain model alignment check: verify stage/task/approval vocabulary matches FORMAL_SPEC.md section 3.
- CI quality gates: run formatting, linting, and stage-gate rule checks on each commit.

## 5. Stage Approval

- Approved by: Pending
- Approval date: Pending
- Notes: Draft task plan generated from approved Stage 2 formal specification.
