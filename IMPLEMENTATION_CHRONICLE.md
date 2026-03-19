# Implementation Chronicle

Plain-language purpose:

- This file explains why something was implemented the way it was.
- It records decisions that are not obvious from the final code or docs.
- It helps another person rebuild the same result later if the code is lost.
- In short:
  - Project brief = what we want
  - Formal spec = how it should behave
  - Implementation chronicle = how we actually chose to build it and what we learned

## 1. Chronicle Metadata

- Chronicle ID: CHR-GOV-STARTUP
- Source task ID: T-001
- Source spec sections: FORMAL_SPEC.md section 4 (FR-001)
- Source requirements: FR-001
- Module / component name: Startup discovery flow
- Implementation language: Markdown policy artifacts
- Author: GitHub Copilot
- Date: 2026-03-19
- Status: Final

## 2. Intent to Implementation Mapping

- What this unit implements from the behavioral spec:
  - Mode-first startup behavior where the first discovery question is always project mode (Greenfield or Brownfield).
- What must remain functionally equivalent across languages:
  - Startup order: read governance, ask mode first, record mode, then continue discovery questions.
- What is intentionally language-specific in this implementation:
  - Rule implementation is via markdown governance docs and instruction files.

## 3. Implementation Decisions

- Data structures chosen and why:
  - Not applicable for document-only governance change. TODO for coding projects.
- Algorithms chosen and why:
  - Not applicable for document-only governance change. TODO for greenfield coding projects.
- Control-flow structure chosen and why:
  - Added explicit "first question" rule in workflow and bootstrap docs.
- Boundary and interface decisions:
  - Behavioral contract examples kept in formal spec FR-001.
- Error-handling strategy:
  - If mode answer ambiguous, ask one clarifying question (already in FR-001).
- Performance or memory trade-offs accepted:
  - N/A.

## 4. Alternatives Considered

- Alternative 1: Keep mode-first only in formal spec.
- Why rejected:
  - Operational docs might drift; runtime behavior would remain implicit.
- Alternative 2: Add separate startup policy file.
- Why rejected:
  - Increases surface area; existing governance files already own this behavior.

## 5. Derived Invariants and Constraints

- Invariant 1:
  - No domain-specific discovery question is asked before mode is selected.
- Invariant 2:
  - Mode is recorded before project brief details are collected.
- Constraints inherited from the spec:
  - Stage-gated progression and explicit approvals remain mandatory.
- Additional implementation constraints introduced:
  - Startup transcript examples must exist for both Greenfield and Brownfield paths.

## 6. Divergences and Clarifications

- Where the spec was ambiguous:
  - Mode-first behavior existed in FR-001 but was not explicit in all bootstrap docs.
- How the ambiguity was resolved in code:
  - Added explicit first-question wording to workflow and bootstrap instructions.
- Any controlled divergence from the spec:
  - None.
- Follow-up needed in the spec or task list:
  - None for T-001.

## 7. Testing Notes

- Unit tests added:
  - Not executable; validated by policy text consistency checks.
- Integration tests added:
  - Not executable; validated startup flow wording across workflow, README, and copilot instructions.
- Property-based tests added:
  - None.
- Edge cases covered:
  - Ambiguous mode answer flow retained in FR-001.
- Failure modes exercised:
  - Missing governance files still block startup before project questions.

## 8. Reconstruction Notes

- If the code were lost, what another implementer must know to rebuild this unit faithfully:
  - Re-implement mode-first startup as explicit rule in workflow + bootstrap docs and include dual startup transcripts in FR-001.
- Order of implementation steps that mattered:
  - Update normative policy docs first, then spec examples, then task status.
- Non-obvious pitfalls discovered during implementation:
  - Behavior can look implemented in spec but still be inconsistent operationally if bootstrap docs lag.
- What not to change without updating the behavioral spec:
  - Do not reorder startup questions to ask domain details before mode selection.

## 9. Known Limitations

- Limitation 1:
  - No executable test harness for startup behavior exists yet.
- Reason accepted:
  - Current project phase is governance artifact build-out.
- Revisit trigger:
  - Build of automation layer and policy checks in later scope.

## 10. Approval / Review

- Reviewed by: Lefty
- Review date: 2026-03-19
- Notes:
  - Linked to task T-001 and requirement FR-001.

---

## Chronicle Entry: CHR-GOV-STAGE-GATE

### 1. Chronicle Metadata

- Chronicle ID: CHR-GOV-STAGE-GATE
- Source task ID: T-002
- Source spec sections: FORMAL_SPEC.md section 4 (FR-002)
- Source requirements: FR-002, NFR-001
- Module / component name: Stage transition gate checks
- Implementation language: Markdown policy artifacts
- Author: GitHub Copilot
- Date: 2026-03-19
- Status: Final

### 2. Intent to Implementation Mapping

- What this unit implements from the behavioral spec:
  - Explicit approval and stage-completion commit checks before any stage transition.
- What must remain functionally equivalent across languages:
  - Transition denial conditions and denial-reason logging must be consistent.
- What is intentionally language-specific in this implementation:
  - Enforcement represented as governance policy text.

### 3. Implementation Decisions

- Data structures chosen and why:
  - Not applicable for document-only governance change. TODO for coding projects.
- Algorithms chosen and why:
  - Not applicable for document-only governance change. TODO for greenfield coding projects.

- Control-flow structure chosen and why:
  - Added denial-reason logging requirement to both policy (`01_DECISION_POLICY.md`) and enforcement (`02_WORKFLOW_STAGES.md`) to avoid single-file drift.
- Error-handling strategy:
  - FR-002 now specifies denial logging for both missing explicit approval and missing stage-completion commit.

### 4. Alternatives Considered

- Alternative 1: Keep denial logging only in formal spec.
- Why rejected:
  - Operational enforcement can be missed without policy-level wording.

### 5. Derived Invariants and Constraints

- Invariant 1:
  - Stage transition denial always includes a recorded reason.
- Invariant 2:
  - Transition cannot proceed without explicit approval and required stage-completion commit.

### 6. Divergences and Clarifications

- Where the spec was ambiguous:
  - Denial behavior existed, but denial-reason recording location was not explicit.
- How the ambiguity was resolved in code:
  - Required logging to `memory.md` and active stage artifact.

### 7. Testing Notes

- Unit tests added:
  - Not executable; policy and spec consistency validated manually.
- Integration tests added:
  - Not executable; cross-document consistency check performed.

### 8. Reconstruction Notes

- If the code were lost, what another implementer must know to rebuild this unit faithfully:
  - Re-add denial-reason logging requirements in both policy and stage enforcement, then align FR-002 error handling.

### 9. Known Limitations

- Limitation 1:
  - No automated lint rule yet to fail on missing denial-reason logging.
- Revisit trigger:
  - Add automation checks during Automation Layer implementation.

### 10. Approval / Review

- Reviewed by: Lefty
- Review date: 2026-03-19
- Notes:
  - Linked to task T-002 and requirements FR-002/NFR-001.

---

## Chronicle Entry: CHR-GOV-TASK-DISCIPLINE

### 1. Chronicle Metadata

- Chronicle ID: CHR-GOV-TASK-DISCIPLINE
- Source task ID: T-003
- Source spec sections: FORMAL_SPEC.md section 4 (FR-003)
- Source requirements: FR-003
- Module / component name: Practical task status flow
- Implementation language: Markdown policy artifacts
- Author: GitHub Copilot
- Date: 2026-03-19
- Status: Final

### 2. Intent to Implementation Mapping

- What this unit implements from the behavioral spec:
  - Clear per-task status flow with mandatory blocker reasons and done-state traceability links.
- What must remain functionally equivalent across languages:
  - Status transition rules and required status metadata must be consistent.
- What is intentionally language-specific in this implementation:
  - Enforcement represented in markdown workflow and task templates.

### 3. Implementation Decisions

- Data structures chosen and why:
  - Not applicable for document-only governance change. TODO for coding projects.
- Algorithms chosen and why:
  - Not applicable for document-only governance change. TODO for greenfield coding projects.
- Control-flow structure chosen and why:
  - Defined explicit transition paths: Not started -> In progress -> (Blocked | Done), with guarded transitions for blocked/done states.
- Boundary and interface decisions:
  - Status rules live in `02_WORKFLOW_STAGES.md`; per-task capture fields live in `templates/TASK_LIST_TEMPLATE.md`.
- Error-handling strategy:
  - Invalid status transitions are denied and reasons must be recorded in task notes.

### 4. Alternatives Considered

- Alternative 1: Keep status rules implicit in DoR/DoD only.
- Why rejected:
  - Teams can still drift without explicit transition rules and required status fields.

### 5. Derived Invariants and Constraints

- Invariant 1:
  - Blocked status always includes blocker reason.
- Invariant 2:
  - Done status always includes traceability links.

### 6. Divergences and Clarifications

- Where the spec was ambiguous:
  - FR-003 did not explicitly state blocker-reason and traceability-link requirements.
- How the ambiguity was resolved in code:
  - FR-003 updated, workflow transition rules added, task template fields added.

### 7. Testing Notes

- Unit tests added:
  - Not executable; validated rule/field consistency across spec, workflow, and task template.
- Integration tests added:
  - Not executable; sample task lifecycle recorded in task list.

### 8. Reconstruction Notes

- If the code were lost, what another implementer must know to rebuild this unit faithfully:
  - Recreate explicit status transitions in workflow docs and required blocker/traceability fields in task templates.

### 9. Known Limitations

- Limitation 1:
  - No automated linter yet to enforce missing blocker or traceability fields.
- Revisit trigger:
  - Add policy checks during automation-layer implementation.

### 10. Approval / Review

- Reviewed by: Lefty
- Review date: 2026-03-19
- Notes:
  - Linked to task T-003 and requirement FR-003.

---

## Chronicle Entry: CHR-GOV-COLLAB

### 1. Chronicle Metadata

- Chronicle ID: CHR-GOV-COLLAB
- Source task ID: T-004
- Source spec sections: FORMAL_SPEC.md section 4 (FR-004)
- Source requirements: FR-004
- Module / component name: Cross-agent clarification routing
- Implementation language: Markdown policy artifacts
- Author: GitHub Copilot
- Date: 2026-03-19
- Status: Final

### 2. Intent to Implementation Mapping

- What this unit implements from the behavioral spec:
  - Peer-first clarification route with escalation only when unresolved.
- What must remain functionally equivalent across languages:
  - Clarification then escalation sequence and unresolved-note requirement.
- What is intentionally language-specific in this implementation:
  - Enforced via governance process text and task-note conventions.

### 3. Implementation Decisions

- Data structures chosen and why:
  - Not applicable for document-only governance change. TODO for coding projects.
- Algorithms chosen and why:
  - Not applicable for document-only governance change. TODO for greenfield coding projects.
- Control-flow structure chosen and why:
  - Added 4-step clarification routing protocol in handoff rules.
- Boundary and interface decisions:
  - Handoff contract defines route; command-chain file defines escalation payload; spec defines behavioral contract.
- Error-handling strategy:
  - Escalation without unresolved-ambiguity note is denied and returned for completion.

### 4. Alternatives Considered

- Alternative 1: Allow direct escalation without peer-first consultation.
- Why rejected:
  - Increases sponsor bottlenecks and reduces use of remit expertise.

### 5. Derived Invariants and Constraints

- Invariant 1:
  - Ambiguity must be routed to remit-holder peer before user escalation.
- Invariant 2:
  - Escalation must include unresolved-ambiguity note with evidence summary.

### 6. Divergences and Clarifications

- Where the spec was ambiguous:
  - FR-004 required peer-first behavior but did not fully define escalation payload fields.
- How the ambiguity was resolved in code:
  - Added unresolved-note payload requirements to command-chain and FR-004 error handling.

### 7. Testing Notes

- Unit tests added:
  - Not executable; validated consistency across handoff, command-chain, and formal spec documents.
- Integration tests added:
  - Not executable; sample clarification and escalation records added in task progress notes.

### 8. Reconstruction Notes

- If the code were lost, what another implementer must know to rebuild this unit faithfully:
  - Recreate peer-first route and unresolved-note fields in both workflow docs and spec behavior.

### 9. Known Limitations

- Limitation 1:
  - No automated checker yet enforces presence of unresolved-ambiguity note fields.
- Revisit trigger:
  - Add policy lint checks in automation layer scope.

### 10. Approval / Review

- Reviewed by: Lefty
- Review date: 2026-03-19
- Notes:
  - Linked to task T-004 and requirement FR-004.

---

## Chronicle Entry: CHR-GOV-BROWNFIELD-GATE

### 1. Chronicle Metadata

- Chronicle ID: CHR-GOV-BROWNFIELD-GATE
- Source task ID: T-005
- Source spec sections: FORMAL_SPEC.md section 4 (FR-005)
- Source requirements: FR-005
- Module / component name: Brownfield confidence gate template
- Implementation language: Markdown policy artifacts
- Author: GitHub Copilot
- Date: 2026-03-19
- Status: Final

### 2. Intent to Implementation Mapping

- What this unit implements from the behavioral spec:
  - Explicit Brownfield commitment gate using baseline evidence + confidence rating.
- What must remain functionally equivalent across languages:
  - Brownfield commitments blocked when evidence is missing or confidence is low.
- What is intentionally language-specific in this implementation:
  - Decision gate represented in markdown decision table.

### 3. Implementation Decisions

- Data structures chosen and why:
  - Not applicable for document-only governance change. TODO for coding projects.
- Algorithms chosen and why:
  - Not applicable for document-only governance change. TODO for greenfield coding projects.
- Control-flow structure chosen and why:
  - Added compact decision table to FR-005 for unambiguous outcome mapping.
- Error-handling strategy:
  - Missing baseline evidence and low-confidence paths explicitly block feature commitment.

### 4. Alternatives Considered

- Alternative 1: Keep brownfield gate as prose only.
- Why rejected:
  - Decision-table form is less ambiguous under pressure.

### 5. Derived Invariants and Constraints

- Invariant 1:
  - Brownfield commitment cannot proceed without baseline evidence.
- Invariant 2:
  - Low confidence requires one of the predefined fallback actions.

### 6. Divergences and Clarifications

- Where the spec was ambiguous:
  - FR-005 outcomes were clear, but condition/result mapping was not tabulated.
- How the ambiguity was resolved in code:
  - Added explicit decision table to FR-005.

### 7. Testing Notes

- Unit tests added:
  - Not executable; validated decision-table completeness.

### 8. Reconstruction Notes

- If the code were lost, what another implementer must know to rebuild this unit faithfully:
  - Restore decision table with three condition bands: evidence complete + confidence level, plus blocked path for missing evidence.

### 9. Known Limitations

- Limitation 1:
  - Confidence scoring remains process-guided, not automatically measured.
- Revisit trigger:
  - Add measurable confidence rubric in later automation/analytics layers.

### 10. Approval / Review

- Reviewed by: Lefty
- Review date: 2026-03-19
- Notes:
  - Linked to task T-005 and requirement FR-005.

---

## Chronicle Entry: CHR-GOV-AUDIT

### 1. Chronicle Metadata

- Chronicle ID: CHR-GOV-AUDIT
- Source task ID: T-006
- Source spec sections: FORMAL_SPEC.md section 5 (NFR-002, NFR-003)
- Source requirements: NFR-002, NFR-003, NFR-004
- Module / component name: Traceability and maintainability audit flow
- Implementation language: Markdown policy artifacts
- Author: GitHub Copilot
- Date: 2026-03-19
- Status: Final

### 2. Intent to Implementation Mapping

- What this unit implements from the behavioral spec:
  - Operational auditability for traceability completeness and maintainability trend capture.
- What must remain functionally equivalent across languages:
  - Missing links are blockers; trend metric captured every cycle.
- What is intentionally language-specific in this implementation:
  - Audit requirements represented in Verify-stage done criteria and NFR validation text.

### 3. Implementation Decisions

- Data structures chosen and why:
  - Not applicable for document-only governance change. TODO for coding projects.
- Algorithms chosen and why:
  - Not applicable for document-only governance change. TODO for greenfield coding projects.
- Control-flow structure chosen and why:
  - Added explicit Verify-stage checks to force audits before stage closure.
- Boundary and interface decisions:
  - NFR validation methods define metrics; Stage 5 defines when they are captured.

### 4. Alternatives Considered

- Alternative 1: Keep traceability and trend capture as optional reviewer guidance.
- Why rejected:
  - Optional checks are skipped under schedule pressure.

### 5. Derived Invariants and Constraints

- Invariant 1:
  - Stage cannot close with unresolved traceability gaps.
- Invariant 2:
  - Maintainability trend metric must be captured each cycle.

### 6. Divergences and Clarifications

- Where the spec was ambiguous:
  - NFR validation existed but stage-level operational enforcement was implicit.
- How the ambiguity was resolved in code:
  - Added explicit Verify-stage done bullets and clarified NFR validation methods.

### 7. Testing Notes

- Unit tests added:
  - Not executable; validated consistency between NFR mapping and stage done criteria.

### 8. Reconstruction Notes

- If the code were lost, what another implementer must know to rebuild this unit faithfully:
  - Re-add mandatory traceability blocker check and maintainability trend capture in Verify stage.

### 9. Known Limitations

- Limitation 1:
  - Audit is still manual, not enforced by automated linter.
- Revisit trigger:
  - Add automated traceability checker in automation layer.

### 10. Approval / Review

- Reviewed by: Lefty
- Review date: 2026-03-19
- Notes:
  - Linked to task T-006 and requirements NFR-002/NFR-003/NFR-004.
