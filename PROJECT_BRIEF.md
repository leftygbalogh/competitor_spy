# Project Brief

## 1. Project Overview

- Project name: AI Governance Template
- Project mode: Greenfield
- Problem statement:
  - Current projects start with repeated governance setup effort from near-scratch.
  - Goal is to reduce governance setup effort and increase time spent on delivery.
- Desired business/domain outcome:
  - Template can be dropped into a new software project.
  - Agent reads governance constraints and immediately starts structured project discovery.
- In-scope goals (first version):
  - Minimal Governance Core:
    - Stage gates with explicit approval
    - Universal DoR/DoD
    - Prompt and memory logging
    - Basic task tracker (To do / In progress / Done)
  - Collaboration-First Core:
    - Mandatory cross-agent clarification before escalation
    - Clear owner/remit routing rules
  - Brownfield-Safe Core:
    - Legacy uncertainty protocol
    - Characterization tests and parity checks before feature promises
- Out-of-scope items (now):
  - Full multi-platform CI/CD automation templates
  - Heavy compliance packs unless project-triggered
  - Deep language-specific packs beyond Rust/Python priority
  - One-shot production code generation workflows
  - Complex dashboard/reporting UI for governance

## 1.1 Mode-Specific Direction

- Greenfield now:
  - Start with governance workflow reliability and collaboration clarity.
  - Ensure every new project runs discovery and approval gates before implementation.
- Growth path after first version works:
  - Add Automation Layer:
    - Auto-checklists at stage transitions
    - CI policy checks for missing artifacts
  - Add Analytics Layer:
    - Metrics for cycle time, blocker rate, rework, ambiguity frequency
  - Add Advanced Governance Layer:
    - More specialized personas
    - Optional formal verification escalation patterns
- Brownfield direction retained as first-class branch:
  - Require uncertainty protocol, baseline evidence, and parity-safe increments.

## 1.2 Quality Module Declarations

- Data Quality module active? No (not required for this template project at this stage)
- Compliance & Auditability module active? No (not required for this template project at this stage)

Notes:
- Q1 core pack is always active.
- Q2 dimensions activate by stage.
- Q3 modules can be activated by project trigger later.

## 2. Stakeholders and Users

- Sponsor: Lefty
- Product owner / decision owner: Lefty
- Primary user groups: Entire development team
- Secondary user groups: Out of scope for now
- Governance change model:
  - Any participant may propose a change with rationale.
  - Final approval authority: Lefty only.

## 3. Functional Requirements

1. FR-001: Template initializes governance behavior in a new project with consistent discovery flow.
2. FR-002: Workflow enforces explicit stage approvals and blocks silent stage progression.
3. FR-003: Task execution enforces universal DoR/DoD and role-based accountability.
4. FR-004: Team handoffs require remit-aware clarification before escalation.
5. FR-005: Brownfield path enforces uncertainty protocol and behavior-parity controls.

## 4. Non-Functional Expectations

- Performance: Governance overhead should be lightweight and practical for daily use.
- Reliability/availability: Stage records and task states remain consistent and auditable.
- Security/privacy: Follow baseline secure coding and review practices from Q1 standards.
- Scalability: Template should scale from small to large projects through staged artifacts.
- Observability: Decision trail is preserved through prompts, memory snapshots, and chronicle/task links.
- Maintainability: Governance updates should converge downward over successive projects.
- Compliance/regulatory: Not active for this template project unless future projects trigger it.

## 5. Domain Constraints and Assumptions

- Constraint 1: Stage progression requires explicit approval; silence is not approval.
- Constraint 2: Governance must support both Greenfield and Brownfield with distinct controls.
- Assumption 1: Team members will use a simple tracker to reduce context loss and drift.
- Assumption 2: Cross-agent clarification improves accuracy and reduces sponsor bottlenecks.
- Assumption 3: Template update frequency should decrease over repeated project use.

## 6. Interfaces and Dependencies

- Upstream systems: User directives and project context.
- Downstream systems: Formal spec, task list, implementation chronicle, stage records.
- External APIs/services: None required for baseline governance workflow.
- Data stores: Repository markdown artifacts.

## 7. Acceptance Criteria

1. AC-001: New project startup follows discovery workflow with no missing stage artifacts.
2. AC-002: Any task can be audited from requirement to tests to chronicle entry.
3. AC-003: Brownfield feature commitments are gated by uncertainty protocol and parity controls.
4. AC-004: Governance change requests are open to all participants and approvable only by Lefty.
5. AC-005: Scope sequencing is explicit: now vs later vs out for first version.

## 8. Risks and Unknowns

- Risk 1: Participants lose context during side tasks; progress tracking becomes inconsistent.
- Risk 2: Specification ambiguity discovered late during implementation.
- Risk 3: Cross-role collaboration failure increases sponsor dependency.
- Risk 4: Phase expectation mismatch (prototype vs production-ready misunderstanding).
- Risk 5: Stage gate bypass under time pressure.
- Risk 6: Debate loops without convergence.
- Risk 7: Task tracker and chronicle drift out of sync.
- Risk 8: Brownfield legacy understanding gaps due to weak docs/tests/hidden prerequisites.
- Unknown 1: Initial adoption friction level across team roles.
- Unknown 2: Effort needed to tune ambiguity and escalation thresholds.

## 8.1 Brownfield Legacy Uncertainty Handling

- Discovery timebox: Mandatory before committing uncertain brownfield feature delivery.
- Legacy surface map in scope: Required for touched modules/endpoints/jobs.
- Evidence sources: Code, runtime traces, logs, existing tests, SME input.
- Hidden prerequisites/setup checklist: Required.
- Characterization baseline: Required for high-risk paths first.
- Confidence rating by area: High | Medium | Low.
- Delivery gate:
  - Proceed only when confidence threshold and parity controls are defined.
  - If threshold not met: extend discovery, reduce scope, or run stabilization sprint.
- Ambiguity escalation path:
  - Cross-role clarification first.
  - Final decision owner if unresolved: Lefty.

## 9. Stage Approval

- Approved by: Lefty
- Approval date: 2026-03-19
- Notes: Stage 1 Discover approved explicitly by sponsor/decision owner.
