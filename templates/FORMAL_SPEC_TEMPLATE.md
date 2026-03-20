# Formal Specification Template

## 1. Specification Metadata

- Spec ID:
- Version:
- Project mode: Greenfield | Brownfield
- Declared implementation language(s) from brief:
- Language-specific constraints captured in this spec:
- Source brief:
- Approval authority source (from brief section 8.2):
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

## 7. Architecture and Design Decisions

- Decision:
- Rationale:
- Alternatives considered:
- Consequences:

## 8. Test Strategy (TDD-aligned)

- Unit test approach:
- Integration test approach:
- Acceptance test approach:
- Exploratory test focus areas:

## 8.1 Downstream Implementation Chronicle Expectations

- Required chronicle entries or modules:
- Implementation constraints that must be recorded by coders:
- Areas where implementation alternatives are expected and should be justified:
- Reconstruction-critical details future coders must preserve:

## 9. Traceability Matrix

Map requirements to spec sections, tests, and implementation chronicles.

- FR/NFR ID:
- Spec section:
- Planned tests:
- Planned implementation chronicle entry:

## 10. Stage Approval

- Approved by:
- Approval date:
- Notes:
