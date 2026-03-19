# Numbered Task List Template

## Reference Standards

- Definition of Ready (DoR): see `02_WORKFLOW_STAGES.md` — a task may not start until all DoR items are met.
- Definition of Done (DoD): see `02_WORKFLOW_STAGES.md` — a task is done only when all DoD items are met.
- Implementation chronicle template: `templates/IMPLEMENTATION_CHRONICLE_TEMPLATE.md`

## 1. Planning Metadata

- Plan ID:
- Source formal spec:
- Version:
- Project mode: Greenfield | Brownfield
- Owner:
- Date:

## 2. Work Order Rules

- Tasks are strictly numbered.
- Each task references source spec requirement IDs.
- Each task must meet the DoR before work starts.
- Each task must meet the universal DoD before it is marked Done.
- Dependencies must be explicit.
- Progress is recorded against each task.
- Brownfield plans should prefer tiny, behavior-safe increments.
- Brownfield tasks should include parity checks at the smallest practical unit.

## 3. Task Backlog

1. T-001: [Title]
- Description:
- Source requirements:
- Acceptance criteria:
- Dependencies:
- Owner role:
- Test-first notes (TDD):
- Planned implementation chronicle entry:
- DoR met? Yes | No | Blocked on: [what]
- Definition of done: inherits universal DoD from `02_WORKFLOW_STAGES.md`; add task-specific additions here:
- Status: Not started | In progress | Blocked | Done
- Progress notes:

2. T-002: [Title]
- Description:
- Source requirements:
- Acceptance criteria:
- Dependencies:
- Owner role:
- Test-first notes (TDD):
- Planned implementation chronicle entry:
- DoR met? Yes | No | Blocked on: [what]
- Definition of done: inherits universal DoD from `02_WORKFLOW_STAGES.md`; add task-specific additions here:
- Status: Not started | In progress | Blocked | Done
- Progress notes:

3. T-003: [Title]
- Description:
- Source requirements:
- Acceptance criteria:
- Dependencies:
- Owner role:
- Test-first notes (TDD):
- Planned implementation chronicle entry:
- DoR met? Yes | No | Blocked on: [what]
- Definition of done: inherits universal DoD from `02_WORKFLOW_STAGES.md`; add task-specific additions here:
- Status: Not started | In progress | Blocked | Done
- Progress notes:

## 4. XP and DDD Checks

- Pairing or review approach:
- Refactoring checkpoints:
- Domain model alignment check:
- CI quality gates:

## 5. Stage Approval

- Approved by:
- Approval date:
- Notes:
