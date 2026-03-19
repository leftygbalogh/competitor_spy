# 02 Workflow Stages

## Project Mode Selection

Select one mode at project start and record it in project artifacts:

- Greenfield: New system design and implementation.
- Brownfield: Existing system preservation, refactor, migration, or rewrite.

Mode selection changes priorities, role emphasis, and validation strategy.

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

For Brownfield tasks, add:
8. Behavior parity confirmed against baseline for any changed behavior.

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
- Q3 module triggers declared (data quality, compliance) or explicitly ruled out

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

Definition of done:

- Functional and non-functional requirements defined
- Interfaces, behaviors, and constraints specified
- Risks and assumptions documented
- Behavioral specification rigor applied to core behaviors using statecharts, design by contract, and decision tables
- Performance targets, reliability failure modes, and maintainability seams defined

Mode-specific done criteria:

- Greenfield: architecture options and forward-evolution assumptions captured.
- Brownfield: behavior parity requirements defined at function, class, and module levels where needed.

Quality pack activation:

- Q2 unlocked at this stage: Behavioral Specification Rigor (behavioral contract produced), Performance & Efficiency (targets defined), Reliability & Resilience (failure modes designed), Maintainability Over Time (change seams defined).
- If Q2 targets are absent at stage-gate review, treat as a spec gap and block approval.
- See `07_QUALITY_DIMENSIONS.md` for behavioral specification guidance and escalation rules.

## Stage 3: Plan

Purpose: Convert specification into ordered implementation tasks.

Inputs:

- Approved formal specification

Definition of done:

- Numbered task list with dependencies
- Work order and owners proposed
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
- Observability alerting confirmed operational

Quality pack activation:

- Q2 finalized at this stage: Observability (confirm alerting is live and meaningful, runbooks written), Documentation (runbooks, getting-started guide, changelog finalized).
- See `07_QUALITY_DIMENSIONS.md` for release-readiness sub-items.

## Stage Gate Enforcement

- Do not move to next stage without explicit user approval.
- Explicit approval means an affirmative yes; silence is not approval.
- Stage-level done requires all tasks in the stage to meet the universal task-level DoD.
- Any role may raise a DoR or DoD violation; it must be resolved before work continues.
- After stage completion is approved, save and create a stage-completion commit before any next-stage work begins.
