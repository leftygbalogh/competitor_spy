# Formal Specification Template

## 1. Specification Metadata

Layer metadata: Layer 2 of the three-layer documentation stack (Commander's Intent -> Behavioral Specification -> Implementation Chronicle).
Required linkage: cite source `PROJECT_BRIEF.md` sections for intent and map implementation plans to `IMPLEMENTATION_CHRONICLE.md` entries.

- Spec ID:
- Version:
- Project mode: Greenfield | Brownfield
- Declared implementation language(s) from brief:
- Language-specific constraints captured in this spec:
- Source brief:
- Approval authority source (from project brief delegation section):
- Status:
- Author:
- Reviewers:
- Active Q3 modules: (from project brief — Data Quality | Compliance & Auditability | None)

## 1.1 Mode Constraints

- Greenfield constraints:
- Brownfield parity scope and allowed deltas:

## 1.2 Behavioral Specification Approach

All specifications use the default tooling: statecharts + design by contract + decision tables. See `07_QUALITY_DIMENSIONS.md` for full guidance.

- Statechart coverage: (list states and key transitions modeled)
- Contract coverage: (list operations with pre/post conditions defined)
- Decision table coverage: (list conditional logic areas covered by tables)
- Escalation to mathematical verification? Yes | No
  - If yes, method selected: (TLA+ | Alloy | B-method)
  - If yes, rationale and scope of verification:

## 2. Scope

- In scope:
- Out of scope:

## 3. Domain Model

- Ubiquitous language:
- Core entities:
- Value objects:
- Aggregates:
- Domain services:

## 4. Functional Behavior

For each requirement from brief, lead with one concrete real-world example before abstract rules:

- Requirement ID:
- Example: (one concrete scenario from the target domain or use case; orients the reader before rules are stated)
- Preconditions:
- Trigger:
- Expected behavior:
- Postconditions:
- Error handling:

## 5. Non-Functional Requirements Mapping

Map each NFR to measurable criteria. Targets defined here are validated at Stage 5 Verify.

- NFR ID:
- Dimension: (from 07_QUALITY_DIMENSIONS.md — e.g., Performance & Efficiency, Reliability & Resilience)
- Metric:
- Target:
- Validation method:

## 6. Data and Interface Contracts

- Input schemas/contracts:
- Output schemas/contracts:
- API or protocol definitions:
- Versioning strategy:

## 6.1 Exact Data Formats (required when files/log streams are part of behavior)

- File/log grammar (field order, delimiter, timestamp format, encoding):
- Record limits and ordering rules:
- Malformed-row or malformed-payload handling:
- Render-frame/output format contract (if user-visible rendering exists):

## 6.2 API Signature and Side-Effect Contract

- Stable signatures for key operations (inputs, outputs, error contract):
- Side-effect declaration per operation (files, network, stdout/stderr, persistence):
- Purity declaration (pure vs side-effecting):
- Async/runtime context constraints (for calls sensitive to event-loop or threading context):

## 7. Architecture and Design Decisions

- Decision:
- Rationale:
- Alternatives considered:
- Consequences:

### 7.1 Layered Architecture (required if Q3-ARCH-01 is active)

- Module interface definitions: (list each module and its formally defined interface or trait/protocol)
- API surface: (describe operations callable from inside the application, from an external caller, and from a CLI invocation)
- CLI-to-API mapping: (for each CLI command or entry point, state which API call it delegates to)
- GUI-to-API mapping: (for each GUI action, state which API call it delegates to — omit if no GUI)
- Business logic placement constraint: No logic belonging to the domain may reside exclusively in the CLI or GUI layer. Any business logic found there during review is an architecture violation and a build blocker.

## 8. Quality Dimension Targets (Q2 Pack)

Targets defined here are mandatory for Stage 2 closure and are validated at Stage 5 Verify.

- Performance and efficiency targets (latency, throughput, memory budget, startup budget):
- Reliability and resilience targets (failure modes, degradation behavior, recovery constraints):
- Maintainability over time targets (change seams, extension points, deprecation/refactor constraints):
- Not-applicable declarations (if any dimension is N/A, state rationale and approval reference):

## 9. Test Strategy (TDD-aligned)

- Unit test approach:
- Integration test approach:
- Acceptance test approach:
- Exploratory test focus areas:
- Interactive CLI diagnostics approach (required if project has interactive CLI UX):
  - Screen-state capture method:
  - Application-state capture method:
  - Manual session execution path (how testers run app through capture helpers):
  - Artifact storage path and naming convention:

## 9.1 Downstream Implementation Chronicle Expectations

- Required chronicle entries or modules:
- Implementation constraints that must be recorded by coders:
- Areas where implementation alternatives are expected and should be justified:
- Reconstruction-critical details future coders must preserve:

## 9.2 Deterministic Test Vector Appendix

- Canonical deterministic vectors (fixed seeds, fixed inputs, expected outputs):
- Expected persistence snapshots for terminal/end-state scenarios:
- Expected rendered/output snapshots for key user-visible states:
- Replay procedure for regression confirmation:

## 10. Traceability Matrix

Map requirements to spec sections, tests, and implementation chronicles.

- FR/NFR ID:
- Spec section:
- Planned tests:
- Planned implementation chronicle entry:

## 11. Stage Approval

- Approved by:
- Approval date:
- Notes:
