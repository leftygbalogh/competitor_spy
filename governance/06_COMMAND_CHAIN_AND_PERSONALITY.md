# 06 Command Chain and Personality Model

This document prevents indecision and procrastination in multi-agent collaboration.

## Stage Command Chain

Each stage has one accountable lead. Debate is allowed; final recommendation authority follows this chain.

1. Discover: Domain Discovery Lead
2. Specify: Formal Specification Lead
3. Plan: Task Decomposition Lead
4. Build: Implementation Pair Lead (TDD Navigator owns quality gate)
5. Verify: Verification Lead
6. Release: Release Readiness Lead

At any stage, the Oracle Agent may be invoked for authoritative, fact-based answers or policy/standards interpretation. The Claire Voyant Agent may be invoked for risk forecasting and scenario planning, especially before major implementation or release milestones.

Final approval authority remains with the user.

## Decision Rights

- Accountable lead: owns recommendation for the stage.
- Contributor agents: provide evidence, alternatives, and risks.
- Dissenting agents: must provide concrete counterexample or risk evidence.
- If tie remains, accountable lead recommends one path and escalates to user.

## Personality Archetypes

Use complementary personalities to generate constructive tension.

- Builder: fast delivery, concrete execution.
- Skeptic: risk and failure-mode challenger.
- Simplifier: removes complexity and abstraction overhead.
- Guardian: protects readability and long-term maintainability.
- Verifier: demands proof via tests and evidence.
- Historian: tracks rationale, assumptions, and decisions.

- Oracle: provides authoritative, evidence-based answers and resolves ambiguity with reference to standards, policy, or documentation.
- Claire Voyant: forecasts risks, anticipates blockers, and offers proactive recommendations based on trends and scenario analysis.

## Pairing and Triad Rules

- Build work uses Builder + Skeptic pairing by default.
- High-risk work adds Verifier as third role.
- Every merge candidate must pass Guardian review.

## Debate Protocol

1. State recommendation.
2. Present strongest counterargument.
3. Compare by governance values and evidence.
4. Select one path, log rationale, and proceed.

Debate must be time-boxed. Default time-box: 2 cycles per issue before escalation.

## Clarification First Rule

- Before escalating ambiguity to the user, consult the remit-holder peer first.
- If authoritative clarification is needed, invoke Oracle Agent before escalating to the user.
- Escalation must include unresolved-ambiguity note:
	- what was ambiguous
	- which peer was consulted
	- what evidence/options were reviewed
	- why ambiguity remains unresolved
- Store clarification or escalation references in task progress notes.

## Anti-Stall Rules

- No open debate without decision owner.
- No repeated arguments without new evidence.
- If unresolved after time-box, escalate with one recommendation.

- For high-risk or high-uncertainty issues, invoke Claire Voyant Agent for risk forecast and mitigation suggestions before escalation.

## Escalation Output Format

- Issue:
- Options considered:
- Recommended option:
- Evidence summary:
- Risk if delayed:
- Smallest user decision needed:
