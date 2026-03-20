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
- Implementation language(s):
- Stage approval authority reference (from brief section 8.2):
- Owner:
- Date:

## 2. Work Order Rules

- Tasks are strictly numbered.
- Each task references source spec requirement IDs.
- Each task must meet the DoR before work starts.
- Each task must meet the universal DoD before it is marked Done.
- Dependencies must be explicit.
- Progress is recorded against each task.
- Blocked status requires explicit blocker reason.
- Done status requires traceability links (spec -> task -> tests -> chronicle).
- Brownfield plans should prefer tiny, behavior-safe increments.
- Brownfield tasks should include parity checks at the smallest practical unit.

## 3. Task Backlog

### T-000: Establish Project Folder Structure (mandatory first task, every project)

- Description: Create the canonical folder layout for the project before any source files, configs, tests, logs, or data files are committed. Use the industry-standard convention for the declared implementation language. If no widely-adopted convention exists for the language, use Rust project conventions as the default fallback.
- Rust convention (default fallback):
  - `src/` — all source files
  - `tests/` — integration tests
  - `benches/` — benchmarks (if applicable)
  - `docs/` — project documentation
  - `config/` — configuration files
  - `data/` — static or seed data
  - `logs/` — runtime log output (gitignored)
  - `scripts/` — helper, diagnostic, and build scripts
  - `assets/` — static assets (if applicable)
- Common language overrides:
  - Python: `src/<package>/`, `tests/`, `docs/`, `config/`, `data/`, `logs/`, `scripts/`
  - Node/TypeScript: `src/`, `test/`, `docs/`, `config/`, `dist/` (gitignored), `logs/` (gitignored)
  - Bash/shell: `bin/` or root-level script entry point, `lib/` for shared functions, `tests/`, `logs/`, `data/`, `docs/`
- Acceptance criteria:
  - All top-level directories created and committed with a `.gitkeep` or equivalent placeholder where empty
  - `logs/` and build output dirs are added to `.gitignore`
  - README or directory map updated to describe each folder's purpose
  - No source files, configs, or data files exist outside this agreed structure
- Source requirements: N/A (structural prerequisite for all other tasks)
- Dependencies: none (must precede all other tasks)
- Owner role: team lead or architect
- Test-first notes (TDD): N/A — structural task; verify by visual inspection and CI path alignment
- Planned implementation chronicle entry: record chosen convention and any project-specific deviations with rationale
- DoR met? Yes (no prerequisites)
- Status: Not started
- Traceability links: —
- Progress notes:

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
- Blocker reason: (required if Status = Blocked)
- Traceability links: (required if Status = Done)
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
- Blocker reason: (required if Status = Blocked)
- Traceability links: (required if Status = Done)
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
- Blocker reason: (required if Status = Blocked)
- Traceability links: (required if Status = Done)
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
