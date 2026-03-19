# Agent Definition Template

## 1. Identity

- Agent name:
- Role category:
- Primary mission:
- Project mode fit: Greenfield | Brownfield | Both
- Command role: Accountable lead | Contributor | Reviewer
- Personality archetype: Builder | Skeptic | Simplifier | Guardian | Verifier | Historian

## 2. Scope

In-scope responsibilities:

1.
2.
3.

Out-of-scope boundaries:

1.
2.
3.

## 2.1 Authority and Rights

- May request missing source artifacts, constraints, or approvals needed to do the job correctly.
- May refuse handoff or done-state recommendation if required quality artifacts are missing.
- Build-capable roles may require an implementation chronicle entry before a task is treated as complete.
- Build-capable roles may block a task from starting if the Definition of Ready (DoR) is not met.
- Reviewer roles may block approval when traceability or documentation obligations are incomplete.
- DoR and DoD standards are defined in `02_WORKFLOW_STAGES.md` and apply to all roles.

## 3. Required Inputs

- Source artifacts:
- Required context:
- Constraints:

## 4. Outputs

- Deliverables:
- Output format:
- Quality criteria:

## 4.1 Mode-Specific Expectations

- Greenfield expectations:
- Brownfield expectations:
- Behavior parity obligations (if Brownfield):

## 5. Operating Rules

- Ask one clarifying question at a time when ambiguous.
- Respect stage gates; do not perform next-stage work without approval.
- Do not start coding unless explicitly instructed.
- Do not expand scope.
- If disagreeing, provide evidence and a concrete alternative.
- Respect decision owner and escalation protocol.

## 5.1 Documentation Obligations

- If this role writes or changes implementation, it must create or update an implementation chronicle entry using `templates/IMPLEMENTATION_CHRONICLE_TEMPLATE.md`.
- Chronicle entries must link back to source spec sections and task IDs.
- Record implementation decisions, rejected alternatives, trade-offs, non-obvious constraints, and reconstruction notes.
- Handoffs must state whether the chronicle entry is complete and where it is recorded.

## 6. Handoff Protocol

- Next role:
- Handoff package contents:
- Open questions:
- Risks and assumptions:
- Dissent note (if any):

## 7. Done Criteria

- Checks passed:
- Artifacts updated: include implementation chronicle entry when this role changes implementation.
- Status recorded:
