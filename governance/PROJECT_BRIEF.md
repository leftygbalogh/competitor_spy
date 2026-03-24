# Project Brief

Purpose: capture Layer 1 command intent for the current project and drive Stage 1 discovery questions.

Layer metadata: Layer 1 of the three-layer documentation stack (Commander's Intent -> Behavioral Specification -> Implementation Chronicle).
Expected downstream links: `FORMAL_SPEC.md` (Layer 2) and `IMPLEMENTATION_CHRONICLE.md` (Layer 3).

## How to Use This Brief During Discovery

- Do not leave any prior project examples or template defaults in this file. All content must be filled in based on the current project brief and stage instructions.
- Fill sections in order during Stage 1 questioning.
- If an item is unknown, mark it `TBD` and add a specific follow-up question.
- Stage 1 cannot close until all mandatory fields are either filled or explicitly deferred with rationale.

## 1. Project Overview

- Project name: Competitor Spy
- Project mode: Greenfield
- Primary implementation language: Rust
- Secondary implementation language(s): Bash (Linux/Git Bash launcher script), PowerShell (Windows launcher)
- Language decision status: Fixed
- Problem statement: Enable entrepreneurs to make location and positioning decisions using structured competitor intelligence for a chosen business field and location. Currently no tooling exists for structured, OSINT-only local competitor analysis in CLI form.
- Desired business/domain outcome: Produce actionable, structured competitor intelligence reports (on-screen and PDF) so that solo business operators can assess the competitive landscape before committing to a location or positioning decision.
- In-scope goals:
  - User-defined business field/industry and target location
  - Configurable radius-based search (5/10/20/25/50 km)
  - Discover and list relevant competitors within the radius
  - Produce on-screen and PDF reports from the same run
  - Collect business profile data: name, website, address, phone, email, schedules, service types, studio details, pricing
  - Include keyword-relevance and search-visibility metrics
  - Use only OSINT/public data sources; login-based sources only if public information is exposed
- Out-of-scope items:
  - Private or unauthorized data
  - Paid datasets or paywall circumvention
  - Background monitoring or scheduled runs
  - Multi-user support or batch analysis
  - Legal advice
  - Guaranteed data completeness
  - macOS, GUI, ARM (v1)

## 1.1 Mode-Specific Direction

- Greenfield: Domain is local business intelligence / OSINT. Architecture must anticipate evolution beyond v1. Source adapters are the primary extension point. Module boundaries must isolate: data collection (per-source adapters), data enrichment/normalization, ranking/scoring, output rendering (terminal vs PDF), and credential management.
- Evolution paths: The CLI Rust prototype is the authoritative backbone. It establishes the developer documentation, domain model, and API surface. A future web application (possibly in a different language) will be built on top of or derived from that backbone. Key architectural implication: the core domain logic and API layer must be cleanly separated from the CLI rendering layer from day one, so that a web interface or API server can consume the same domain API without pulling in terminal-specific code. LLM summarization is already planned as opt-in, which supports a future web layer.

## 1.2 Quality Module Declarations

- Data Quality module active? **Yes**
  - Trigger: encrypted credential store and optional run-metadata persistence; missing-field marking in reports
- Compliance and Auditability module active? **Yes**
  - Applicable regulations: GDPR-aligned (no extra competitor data persistence beyond the current run; user-controlled credentials; auditable access log; redacted secrets in logs)
- Interactive CLI diagnostics required? **Yes**
  - Trigger: CLI UX with manual exploratory testing sessions during verification
  - Capture method: screen-state capture + application-state capture (scripted helper or equivalent)
  - Storage location: `docs/evidence/sessions/` with naming `session_YYYYMMDD_HHMMSS_<label>.{log,json}`
- Security and production-readiness loop required? **Yes**
  - Trigger: credential handling, audit trail, network exposure (upstream HTTP), OSINT data, GDPR alignment
  - Stages 4-6 must run Security and Production-Readiness loop; findings converted to mitigations or explicit risk acceptance before Stage 6 close
- Layered architecture constraint active? **Yes** (Q3-ARCH-01)
  - Trigger: Rust (first-class module, type, and interface support)
  - Interface -> API -> CLI -> GUI hierarchy enforced at Stage 2 spec, Stage 4 build, Stage 5 verify

## 2. Stakeholders and Users

- Sponsor: Lefty (product owner, decision owner, final approver for retained items)
- Product owner: Lefty
- Primary user groups: Solo business operators, individuals opening a new business or evaluating a location
- Secondary user groups: Investors (read-only report consumers)

## 3. Functional Requirements

1. FR-001: User can enter business field/industry, target location, and radius (5/10/20/25/50 km options), run analysis, and receive a sorted competitor list.
2. FR-002: For each competitor, collect and present available profile fields: name, website, address, phone, email, business hours/schedules, service types, studio/facility details, pricing where public.
3. FR-003: Generate both on-screen (terminal) and PDF reports from the same run, including ranking cues (distance, keyword-relevance, search-visibility score) and source references for each data point.
4. FR-004: Keyword-relevance and search-visibility metrics included per competitor.
5. FR-005: Credentials stored encrypted; reused securely across runs; user-controlled.
6. FR-006: Intentional pacing (5–15s randomized delay per source request) to reduce ban risk.
7. FR-007: Missing or unavailable data fields marked explicitly in reports; sources that fail are logged with reason codes and do not abort the run.

## 4. Non-Functional Expectations

- Performance: Intentional throttling — 5–15s randomized delay per request. No strict latency target beyond completing within a reasonable user-interactive session window.
- Reliability/availability: Continue with available sources if some fail; mark missing data explicitly; all source-failure reasons logged.
- Security/privacy: Credentials stored encrypted; secrets redacted in all logs and outputs; audit trail maintained; GDPR-aligned (ephemeral competitor data, no persistent storage of scraped data beyond the report file); user-controlled retention and deletion of credential store.
- Scalability: One active run per user/session; no concurrency requirement in v1.
- Observability: OpenTelemetry for structured logs, metrics, and traces; secrets redacted before emission.
- Maintainability: Modular source-adapter architecture; each adapter is independently replaceable without touching core logic.
- Compliance/regulatory: GDPR-aligned; no unauthorized data; auditable access log; user-controlled credentials.

## 4.1 Determinism and Rebuild Constraints

- Deterministic constants that must not drift: Pacing bounds (5s min, 15s max), radius options (5/10/20/25/50 km), source-priority ordering, confidence thresholds, UTC timestamps in all outputs.
- RNG contract: Randomness used only for request-gap jitter. Test runs use deterministic seeds (seed injectable via test harness or env variable). No randomness in ranking logic.
- Tie/ordering policy for ranking flows: Primary sort by distance ascending. Tie-break by keyword-relevance score descending. Secondary tie-break by alphabetical name.
- I/O contract: CLI flags for all input parameters. PDF output named `competitor_spy_report_YYYYMMDD_HHMMSS_UTC.pdf`. All text output in UTF-8. Malformed or missing upstream data treated as absent (marked, not crashed).
- Target runtime environment matrix and support tiers:
  - Tier 1 (required): Linux x86_64 CLI (bash launcher), Windows 11 x86_64 (PowerShell or Git Bash launcher)
  - Tier 2 (not in scope, v1): macOS, GUI, ARM

## 4.2 Acceptance Scenarios (User-Visible)

1. Scenario ID: AS-001
   - Given: User provides valid industry/location/radius
   - When: Run completes with at least one source responding
   - Then: Sorted competitor list produced in both terminal and PDF outputs, each entry includes source references, missing fields marked

2. Scenario ID: AS-002
   - Given: Some upstream sources fail during a run
   - When: Run encounters failures
   - Then: Reports are still generated; failed sources logged with reason codes; missing data sections marked with reason; no crash

3. Scenario ID: AS-003
   - Given: User runs tool with invalid or out-of-range parameters
   - When: CLI parses input
   - Then: Informative error message on terminal, no report generated, clean exit

## 5. Domain Constraints and Assumptions

- Constraint 1: Only public OSINT sources permitted. Login allowed only if accessing publicly visible information (no scraping behind authentication walls that protect private data).
- Constraint 2: Credentials are user-supplied and stored encrypted; the tool never requests credentials it does not need.
- Constraint 3: One center location and one radius per run; no multi-polygon or multi-center support in v1.
- Constraint 4: Competitor data is ephemeral — bound to the current run; not stored beyond the output report file.
- Assumption 1: Upstream source structure and availability can change; adapter replacement must not require core logic changes.
- Assumption 2: Minimum viable report can be produced with partial data (graceful degradation); completeness is best-effort.
- Assumption 3: Licensing and contractual status of specific upstream APIs is unknown at Stage 1 and must be resolved during specification (logged as Unknown U-001).

## 6. Interfaces and Dependencies

- Upstream: Public business directories (e.g., Google Maps public data, OpenStreetMap, Yelp public API), search engines (public search), municipality/government open data, geocoding services.
- Downstream: Local filesystem (PDF report), terminal stdout (on-screen report).
- External APIs: Geocoding (location resolution), routing/distance calculation (competitor distance), public SEO/search-visibility data, optional LLM summarization (off by default, explicitly opt-in).
- Data stores: Encrypted credential store (local, user-controlled); optional run-metadata log (no competitor data persistence).

## 7. Acceptance Criteria

1. AC-001: Valid input produces both terminal and PDF reports, sorted by distance, with source references per data point.
2. AC-002: Source failures handled gracefully; missing fields marked; logs include reason codes; run does not abort on partial failure.
3. AC-003: Pacing enforced (5–15s delay per request); credentials stored encrypted; no competitor data persisted beyond the report; secrets redacted in all logs.

## 8. Risks and Unknowns

- Risk 1: Ban risk from upstream sources if pacing is insufficient or fingerprinting is detected.
- Risk 2: Source-side structural changes breaking one or more adapters (mitigated by modular adapter design).
- Risk 3: GDPR boundary ambiguity — collecting publicly listed business data is generally allowed, but specific fields (personal contact details) may vary by jurisdiction.
- Unknown U-001: Which specific upstream services are contractually and licensing-approved for automated querying. To be resolved during Stage 2 with a source-selection review.
- Unknown U-002: Minimum field-completeness threshold required for an actionable report. To be defined during Stage 2 specification.

## 8.1 Brownfield Legacy Uncertainty Handling

- Not applicable. Mode is Greenfield.

## 8.2 Approval Authority and Delegation

- Delegation mode: Team lead for all stages
- Delegated approver role: Team Lead Agent
- Delegation start stage: Stage 2
- Delegation end condition: All stages including Stage 6 — explicit full delegation by Lefty 2026-03-21
- Consultation protocol: When Team Lead is in doubt on any approval decision, consult Oracle Agent (authoritative/policy questions) or Claire Voyant Agent (risk/scenario questions) before approving

Intra-stage autonomy profile:

- Autonomy level: Balanced
- Allowed without owner approval: Implementation details within approved scope, intra-stage design decisions, adapter selection from approved source list, test strategy execution
- Must escalate to owner even during delegated stages: Scope changes, security/compliance-impacting decisions, new external dependency additions with legal or operational impact
- Assumption policy: No silent assumptions; one-question clarification first, then one explicit working assumption requiring yes/no before continuation

Stage-by-stage approver selection:

- Stage 2 Specify approved by: Team Lead Agent (approved 2026-03-21)
- Stage 3 Plan approved by: Team Lead Agent (delegated)
- Stage 4 Build approved by: Team Lead Agent (delegated)
- Stage 5 Verify approved by: Team Lead Agent (delegated)
- Stage 6 Release approved by: Team Lead Agent (delegated; full delegation explicit by Lefty 2026-03-21)

Owner-retained exceptions (non-stage-approval items only):

- Scope change approvals: Lefty
- Security/compliance-impacting decisions: Lefty
- Dependency additions with legal or operational impact: Lefty

Prototype handback trigger:

- Trigger condition: First end-to-end run complete — CLI accepts input, at least one source returns data, both terminal and PDF outputs produced
- Required handback package:
  - Prototype demo status (pass/partial/fail with notes)
  - Known gaps and risks identified during build
  - Recommendation: continue as planned | rescope | stop

## 9. Stage 1 Approval

- Approved by: Lefty
- Approval date: 2026-03-21
- Notes: All mandatory fields complete. OQ-001 resolved: CLI Rust prototype is the backbone; future web application (possibly different language) will derive from the same domain API surface. Architecture must enforce clean CLI/API separation from day one. Stage 1 closed.

## Stage 1 Open Questions

- OQ-001 (RESOLVED 2026-03-21): Evolution paths — CLI Rust prototype first; likely future web application consuming the same domain API. Rust selected for prototype and developer documentation regardless of final web-layer language choice.

## 1. Project Overview

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
- Compliance and Auditability module active? Yes | No
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
  - Trigger: project uses a language with first-class module, type, and interface support (for example Rust, Python, TypeScript, Go, C#)
  - If yes, Q3-ARCH-01 (Interface -> API -> CLI -> GUI hierarchy) is enforced at Stage 2 spec, Stage 4 build, and Stage 5 verify

Note: Q1 core pack is always active. Q2 stage-unlocked dimensions activate automatically at their named stages. See `07_QUALITY_DIMENSIONS.md`.

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

- Deterministic constants that must not drift:
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

- Delegation mode: Owner only | Team lead for all stages | Team lead with exceptions
- Delegated approver role (if delegated):
- Delegation start stage:
- Delegation end condition:

Intra-stage autonomy profile:

- Autonomy level: Strict | Balanced | High
- Allowed without owner approval:
- Must escalate to owner even during delegated stages:
- Assumption policy: no silent assumptions; one-question clarification first, then one explicit working assumption requiring yes/no before continuation

Stage-by-stage approver selection:

- Stage 2 Specify approved by:
- Stage 3 Plan approved by:
- Stage 4 Build approved by:
- Stage 5 Verify approved by:
- Stage 6 Release approved by:

Owner-retained exceptions:

- Scope change approvals:
- Security/compliance-impacting decisions:
- Dependency additions with legal or operational impact:
- Release approval override rule:

Prototype handback trigger:

- Trigger condition:
- Required handback package:
  - Prototype demo status
  - Known gaps and risks
  - Recommendation (continue | rescope | stop)

## 9. Stage 1 Approval

- Approved by:
- Approval date:
- Notes:
