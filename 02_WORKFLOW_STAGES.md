# 02 Workflow Stages

## Project Mode Selection

Select one mode at project start and record it in project artifacts:

- Greenfield: New system design and implementation.
- Brownfield: Existing system preservation, refactor, migration, or rewrite.

Mode selection changes priorities, role emphasis, and validation strategy.
Mode selection must be the first discovery question before any domain-specific requirements questions.

## Implementation Language Selection

Declare intended implementation language(s) during Stage 1 Discover.

- Record at least one primary implementation language before Stage 3 Plan approval.
- Optional secondary language(s) may be declared for parity or migration scenarios.
- Behavioral specification remains language-agnostic; language choice guides tooling, owners, and task planning.

---

## Universal Definition of Ready (DoR) — Task Level

A task may not be started until all of the following are true. Any role may block a task from starting if the DoR is not met.

1. Source spec section referenced and accessible.
2. Behavioral contract exists for this behavior: statechart, pre/post conditions, or decision table in the spec.
3. Acceptance criteria defined and unambiguous.
4. Test-first approach planned (TDD notes written in the task).
5. Implementation chronicle entry planned: module or component named, chronicle ID assigned.
6. Dependencies resolved, or explicitly deferred with a written note explaining why deferral is safe.
7. No unresolved ambiguity that would require a stop mid-implementation. If ambiguity exists, resolve it first.
8. For tasks with branching logic, interactive input, or persistence side effects, a branch matrix exists covering happy path, negative path, edge/boundary states, and illegal or unexpected user actions.
	- For async runtime/orchestration layers, this matrix must be defined at that layer (not inherited from subcomponents) and enumerate terminal states, per-branch side effects, async-context-sensitive calls, and execution context at each call site.
9. For interactive CLI tasks, screen-state capture and application-state capture method are defined before implementation starts (script or equivalent mechanism), including how manual sessions produce reusable evidence.
10. If Q3-ARCH-01 is active: the module interface and API surface are defined in the formal spec before implementation begins, and each CLI entry point is mapped to a specific API call in the spec.

For Brownfield tasks, add:
8. Legacy behavior evidence identified for touched areas (code/runtime traces/tests) and minimum local setup prerequisites verified.

---

## Universal Definition of Done (DoD) — Task Level

A task is done only when all of the following are true. Any role may block done-state if any item is missing.

1. Red-green-refactor cycle complete: tests written first, passing, and readable.
2. All tests passing in CI.
3. Implementation chronicle entry written and linked to source spec sections and task IDs.
4. At least one peer review completed by a named role other than the implementer.
5. Traceability links updated: spec requirement to task to test to chronicle entry.
6. No new technical debt accepted silently: any debt is logged in the task with a rationale and a revisit trigger.
7. Code quality gates pass: linting, formatting, and static analysis clean.
8. For interactive CLI tasks, at least one exploratory/manual session is run through capture helpers (or equivalent mechanism), and resulting screen/state artifacts are stored and linked for debugging traceability.
	- For interactive CLI tasks that drive a finite-state runtime, automated tests must cover each terminal state and validate post-state behavior (side effects, output, and persistence), not only transition logic.

For Brownfield tasks, add:
9. Behavior parity confirmed against baseline for any changed behavior.

---

## Task Status Flow (Operational)

Use this status flow for every task in task records:

- Not started -> In progress: allowed only when DoR is met.
- In progress -> Blocked: requires a blocker reason in the task record.
- Blocked -> In progress: requires unblock note in progress notes.
- In progress -> Done: allowed only when DoD is met and traceability links are present.
- Done -> In progress: allowed only with explicit reopen reason logged.

Task records must include enough context for another participant to understand current state without additional chat context.

---

## Stage-Level Done Criteria

A stage is done only when:
- Every task in the stage meets the task-level DoD above.
- Stage-specific done criteria below are satisfied.
- Explicit user approval is given. Silence is not approval.

---
## Stage 1: Discover

Purpose: Understand domain, constraints, stakeholders, and outcomes.

Inputs:

- Project brief draft or discovery answers
- Domain context and constraints

Definition of done:

- Domain understanding captured
- Open questions listed and resolved or deferred
- Scope boundaries documented
- Intended implementation language(s) declared (primary required, secondary optional)
- Discovery questions explicitly cover all required fields in `PROJECT_BRIEF.md` sections 1 through 8.2; unanswered fields are marked `TBD` with named follow-up owners
- Q3 module triggers declared (data quality, compliance) or explicitly ruled out
- Approval authority selection completed for Stage 2 through Stage 6 (owner-only or delegated approver per stage)
- If delegation is enabled, exception list and prototype handback trigger are documented in the project brief

Mode-specific done criteria:

- Greenfield: target domain model and likely evolution paths documented.
- Brownfield: implemented current behavior baseline captured from code and runtime evidence, not documentation alone.
- Brownfield: legacy uncertainty handling protocol completed in the project brief (discovery timebox, legacy surface map, hidden prerequisites checklist, characterization-test baseline plan, confidence rating, and go/no-go gate for feature commitments).

Quality pack activation:

- Q1 core pack active from this stage: Problem Understanding, Architecture & Design, Code Quality, Testing, Security, Process & Workflow.
- Q3 modules declared here become active for the duration of the project.
- See `07_QUALITY_DIMENSIONS.md` for full dimension definitions and pack rules.

## Stage 2: Specify

Purpose: Convert requirements into a formal specification.

Inputs:

- Approved project brief
- Approved approval-authority selection from project brief (who approves Stage 2 through Stage 6)

Definition of done:

- Functional and non-functional requirements defined
- Interfaces, behaviors, and constraints specified
- Risks and assumptions documented
- Requirements/specification artifact index is complete and current (`REQUIREMENTS_SPEC_MANIFEST.md`)
- Language-specific implementation constraints documented without changing language-agnostic behavior contract
- Behavioral specification rigor applied to core behaviors using statecharts, design by contract, and decision tables
- Performance targets, reliability failure modes, and maintainability seams defined

Mode-specific done criteria:

- Greenfield: architecture options and forward-evolution assumptions captured.
- Brownfield: behavior parity requirements defined at function, class, and module levels where needed.

Quality pack activation:

- Q2 unlocked at this stage: Behavioral Specification Rigor (behavioral contract produced), Performance & Efficiency (targets defined), Reliability & Resilience (failure modes designed), Maintainability Over Time (change seams defined).
- If Q2 targets are absent at stage-gate review, treat as a spec gap and block approval.
- Formal spec Quality Dimension Targets section must exist in every project. If any Q2 target is not applicable, record explicit N/A with rationale and approval reference; omission of the section blocks Stage 2 approval.
- If Q3-ARCH-01 is active, the formal spec layered architecture section (template section 7.1) must be present and complete before Stage 2 can close, including module interfaces, API surface, CLI-to-API mapping, and GUI-to-API mapping when GUI exists.
- See `07_QUALITY_DIMENSIONS.md` for behavioral specification guidance and escalation rules.

## Stage 3: Plan

Purpose: Convert specification into ordered implementation tasks.

Inputs:

- Approved formal specification

Definition of done:

- Numbered task list with dependencies
- T-000 folder structure task is the first task in every plan: establish the canonical project layout before any source files are created (see task list template for T-000 definition)
- Work order and owners proposed
- Task owners, tooling, and CI choices aligned to declared implementation language(s)
- Test strategy per task defined
- CI pipeline and contribution standards planned

Mode-specific done criteria:

- Greenfield: foundation-first sequencing for architecture and domain boundaries.
- Brownfield: tiny-increment slicing plan for parity-safe migration, refactor, or rewrite.

Quality pack activation:

- Q2 unlocked at this stage: Developer Experience (CI pipeline planned, contribution standards defined, local dev setup defined).
- See `07_QUALITY_DIMENSIONS.md` for Developer Experience sub-items.

## Stage 4: Build

Purpose: Implement tasks with disciplined engineering practices.

Engineering principles:

- TDD first
- Domain-driven design
- XP principles

Definition of done:

- Code and tests complete for scoped tasks
- Evidence of passing tests
- Updates logged in memory and task records
- Observability hooks instrumented (structured logs, metrics, health endpoints)
- ADRs written for all significant design decisions
- API documentation current
- Implementation chronicle entry written for each significant task or module and linked to source spec/task IDs
- Pair-programming evidence captured in `docs/evidence/pair-programming-log.md` whenever more than one build-capable contributor or subagent participates in implementation/review cycles.

Mode-specific done criteria:

- Greenfield: implementation conforms to approved specification and architecture constraints.
- Brownfield: behavior parity maintained against baseline, with controlled and explicit deltas only.

Quality pack activation:

- Q2 unlocked at this stage: Observability (instrument logs, metrics, tracing, health endpoints), Documentation (ADRs written per decision, API docs current, in-code comments where non-obvious).
- Three-layer documentation stack enforced here: commander's intent + behavioral specification + implementation chronicle must stay linked.
- Q1 enforced throughout: all code quality, testing, and security standards apply to every task.
- See `07_QUALITY_DIMENSIONS.md` for observability and documentation sub-items.

## Stage 5: Verify

Purpose: Confirm behavior and quality goals.

Definition of done:

- Test suites passing
- Exploratory testing completed
- Non-functional expectations validated against targets defined in Stage 2
- Maintainability seams confirmed (change seams defined at Stage 2 exist and function)
- Reliability failure paths exercised
- Traceability matrix audited; missing links listed as blockers until resolved
- Maintainability trend metric captured for this cycle (template modification count and direction)
- Branch evidence collected: expected-vs-actual behavior captured for all user-visible branches and persistence-writing paths; orchestration flows (where input, output, and state combine) validated end-to-end.
- For interactive CLI projects, manual exploratory session evidence includes screen-state and application-state capture artifacts linked to observed defects or pass confirmations.
- For interactive CLI projects, a terminal environment validation matrix is captured in `docs/evidence/` (target environments, pass/fail/not-tested status, and artifact paths); untested environments are logged as explicit release risks.
- Pair-programming session log (if applicable) is audited for each task: proposal, critique, decision, and linked evidence are complete; unresolved disagreements are listed as blockers.
- Escaped-defect check: any defect discovered during Stage 5 must be converted into a permanent regression test and linked process/spec improvement before Stage 5 closes.

Mode-specific done criteria:

- Greenfield: acceptance tests validate intended new behavior.
- Brownfield: differential and regression validation confirms same implemented behavior where parity is required.

Quality pack activation:

- Q2 validated at this stage: Performance & Efficiency (measured against Stage 2 targets), Reliability & Resilience (failure paths exercised), Maintainability Over Time (seams verified).
- See `07_QUALITY_DIMENSIONS.md` for validation criteria per dimension.

## Stage 6: Release

Purpose: Deliver safely and document operational readiness.

Definition of done:

- Release checklist complete
- Operational notes and rollback plan available
- Post-release monitoring plan documented
- Runbooks for known failure scenarios written
- Getting-started guide and changelog current
- Deliverables manifest is complete and matches produced artifacts (`DELIVERABLES_MANIFEST.md`)
- Observability alerting confirmed operational
- Requirement-to-evidence map complete for user-visible branches and persistence-writing paths; no release occurs if orchestration flow evidence is incomplete.
- For interactive CLI projects, runbook includes how to execute capture helpers and where captured session artifacts are stored for post-failure diagnosis.
- For interactive CLI projects, runbook includes a Known Environment Gaps section for unvalidated terminal/shell combinations, with risk notes and post-release validation steps.
- Security and production-readiness loop is complete: identified security/ops risks are converted into mitigation tasks, reflected in specs/runbooks, verified with evidence, and closed before release approval.
- Repository identity gate is complete before any publish command: remotes are listed, intended push target is explicitly confirmed, and proof is stored in `docs/evidence/release-remote-proof.md`.
- If multiple remotes exist (or template-clone ancestry makes target uncertain), release remains blocked until owner or delegated approver confirms the exact remote and branch.
- **Joint post-mortem complete before Stage 6 close.** The agent writes all process improvement feedback entries to `templates/feedback.json` covering the full project (spec gaps, build friction, test strategy issues, tooling lessons, governance gaps). The product owner is then explicitly asked whether they have anything to add. Stage 6 is not approved until both parties have had the opportunity to contribute and the feedback file is committed. This is a mandatory gate, not optional housekeeping.

## Official Iterative Hardening Loops

The following loops are mandatory and may repeat within Stage 4-6 until closure criteria are met.

1. Manual Testing and Requirements Loop
	- Run manual/exploratory testing with evidence capture.
	- Convert findings into defects/tasks.
	- Fix code and add/expand automated regression tests.
	- Update requirements/specification text where ambiguity enabled the defect.
	- Re-verify behavior and traceability before closure.

2. Security and Production-Readiness Loop
	- Run security and operational-readiness checks.
	- Convert findings into mitigation tasks.
	- Update code, specs, runbooks, and release evidence.
	- Re-verify mitigations and operational signals.
	- Close only when unresolved high-risk items are either fixed or explicitly accepted by the approver.

Quality pack activation:

- Q2 finalized at this stage: Observability (confirm alerting is live and meaningful, runbooks written), Documentation (runbooks, getting-started guide, changelog finalized).
- See `07_QUALITY_DIMENSIONS.md` for release-readiness sub-items.

## Stage Gate Enforcement

- Do not move to next stage without explicit user approval.
- Explicit approval means an affirmative yes; silence is not approval.
- Stage-level done requires all tasks in the stage to meet the universal task-level DoD.
- Any role may raise a DoR or DoD violation; it must be resolved before work continues.
- After stage completion is approved, save and create a stage-completion commit before any next-stage work begins.
- If transition is denied, the denial reason must be logged in `memory.md` and the active stage artifact.
- Before any push in Stage 6, publish target must be restated explicitly in release notes or checklist evidence; default-origin assumptions are not allowed.
- At every stage gate closure, any participant may append a proposal to the project `feedback.json` file (one entry per proposal) suggesting an addition, modification, or removal to any template document applicable to their current stage or any prior stage. Proposals are reviewed at the gate before the next stage begins. No template document is changed until the decision owner explicitly approves. Approved proposals are fed back into the master template before the next project begins.
- Stage gate approval authority follows project brief delegation settings for each stage; delegated approvers are authoritative within their assigned stage range.
- Once a stage is approved, agents may autonomously resolve intra-stage details through role debate and conflict-resolution rules, without requiring owner approval for each micro-decision.
- Intra-stage autonomy is valid only within approved scope and artifacts; it never permits stage skipping, silent scope expansion, or assumption-based requirement changes.
- If any work is performed without required stage approval, mark it as unauthorized, stop execution, log the breach in `memory.md`, and request explicit direction to rollback or replay the stage correctly.
