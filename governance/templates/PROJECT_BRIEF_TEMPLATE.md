# Project Brief Template

## 1. Project Overview

Layer metadata: Layer 1 of the three-layer documentation stack (Commander's Intent -> Behavioral Specification -> Implementation Chronicle).
Expected downstream links: `FORMAL_SPEC.md` (Layer 2) and `IMPLEMENTATION_CHRONICLE.md` (Layer 3).

- Project name:
- Project mode: Greenfield | Brownfield
- Primary implementation language:
- Secondary implementation language(s):
- Language decision status: Fixed | Deferred (must be Fixed before Stage 3 Plan approval)
- Problem statement:
- Desired business/domain outcome:
- In-scope goals:
- Out-of-scope items:

## 1.1 Mode-Specific Direction

- Greenfield: domain discovery and architecture evolution priorities
- Brownfield: implemented behavior baseline and parity priorities

## 1.2 Quality Module Declarations

Declare active Q3 modules at project start. Once declared, these become core expectations for the duration of this project.

- Data Quality module active? Yes | No
  - Trigger: project uses persistent storage, schemas, or migrations
- Compliance & Auditability module active? Yes | No
  - Trigger: project has regulatory scope (GDPR, HIPAA, SOC2, financial, healthcare, etc.)
  - If yes, specify applicable regulations:
- Interactive CLI diagnostics required? Yes | No
  - Trigger: project includes interactive terminal/CLI UX where manual exploratory sessions are part of verification
  - If yes, define capture method: screen-state capture + application-state capture (scripted helper or equivalent)
  - If yes, define storage location and naming convention for captured session artifacts:

- Security and production-readiness loop required? Yes | No
  - Trigger: project handles sensitive data, user auth, network exposure, public deployment, or regulated scope
  - If yes, Stage 4-6 must run the Security and Production-Readiness loop and convert findings into mitigations or explicit risk acceptance before Stage 6 close

- Layered architecture constraint active? Yes | No
  - Trigger: project uses a language with first-class module, type, and interface support (e.g. Rust, Python, TypeScript, Go, C# — not Bash, awk, or shell scripts)
  - If yes, Q3-ARCH-01 (Interface → API → CLI → GUI hierarchy) is enforced at Stage 2 spec, Stage 4 build, and Stage 5 verify
  - If no, layered architecture is an aspiration only; no gate check is applied

Note: Q1 core pack (Problem Understanding, Architecture, Code Quality, Testing, Security, Process & Workflow) is always active. Q2 stage-unlocked dimensions activate automatically at their named stages. See `07_QUALITY_DIMENSIONS.md`.

## 2. Stakeholders and Users

- Sponsor:
- Product owner:
- Primary user groups:
- Secondary user groups:

## 3. Functional Requirements

List each requirement with stable identifiers.

1. FR-001:
2. FR-002:
3. FR-003:

## 4. Non-Functional Expectations

- Performance:
- Reliability/availability:
- Security/privacy:
- Scalability:
- Observability:
- Maintainability:
- Compliance/regulatory:

## 4.1 Determinism and Rebuild Constraints

- Deterministic constants that must not drift (tick rates, bounds, limits, defaults):
- RNG contract (where randomness is allowed, test seed strategy, replay expectations):
- Tie/ordering policy for ranking flows (if applicable):
- I/O contract (file names/paths/formats/encoding and malformed-data behavior):
- Target runtime environment matrix and support tiers (required vs optional):

## 4.2 Acceptance Scenarios (User-Visible)

Capture concrete Given/When/Then scenarios for critical outcomes, especially failure and boundary end states.

1. Scenario ID:
  - Given:
  - When:
  - Then:
2. Scenario ID:
  - Given:
  - When:
  - Then:

## 5. Domain Constraints and Assumptions

- Constraint 1:
- Constraint 2:
- Assumption 1:
- Assumption 2:

## 6. Interfaces and Dependencies

- Upstream systems:
- Downstream systems:
- External APIs/services:
- Data stores:

## 7. Acceptance Criteria

1. AC-001:
2. AC-002:
3. AC-003:

## 8. Risks and Unknowns

- Risk 1:
- Risk 2:
- Unknown 1:
- Unknown 2:

## 8.1 Brownfield Legacy Uncertainty Handling (Required if mode is Brownfield)

- Discovery timebox (days or sprint fraction):
- Legacy surface map in scope (modules/endpoints/jobs):
- Evidence sources used (code, runtime traces, logs, existing tests, SMEs):
- Hidden prerequisites and setup checklist captured? Yes | No
- Characterization test baseline planned? Yes | No
  - High-risk paths to lock first:
- Confidence rating by area: High | Medium | Low
- Delivery gate for feature commitments:
  - Go only if minimum confidence threshold is met and parity-risk controls are defined
  - If threshold is not met, choose one: extend discovery | reduce scope | run stabilization sprint
- Ambiguity escalation path:
  - Cross-role clarification attempted first? Yes | No
  - If unresolved, final decision owner:

## 8.2 Approval Authority and Delegation (Required before Stage 2 starts)

Select who approves each stage once the brief is considered spec-ready.

- Delegation mode: Owner only | Team lead for all stages | Team lead with exceptions
- Delegated approver role (if delegated):
- Delegation start stage:
- Delegation end condition:

Intra-stage autonomy profile (applies after a stage is approved):

- Autonomy level: Strict | Balanced | High
- Allowed without owner approval: implementation-detail choices within approved scope, role-to-role debate resolution per conflict protocol, task ordering refinements that do not change requirement coverage
- Must escalate to owner even during delegated stages: scope expansion, requirement reinterpretation, architecture pivots, test-strategy replacement, security/compliance impact, new external dependencies with legal/operational risk
- Assumption policy: no silent assumptions; one-question clarification first, then single explicit working assumption requiring yes/no before continuation

Stage-by-stage approver selection:

- Stage 2 Specify approved by:
- Stage 3 Plan approved by:
- Stage 4 Build approved by:
- Stage 5 Verify approved by:
- Stage 6 Release approved by:

Owner-retained exceptions (must come back to owner even when delegated):

- Scope change approvals:
- Security/compliance-impacting decisions:
- Dependency additions with legal or operational impact:
- Release approval override rule:

Prototype handback trigger (when delegated team lead must return control to owner):

- Trigger condition:
- Required handback package:
  - Prototype demo status
  - Known gaps and risks
  - Recommendation (continue | rescope | stop)

## 9. Stage Approval

- Approved by:
- Approval date:
- Notes:
