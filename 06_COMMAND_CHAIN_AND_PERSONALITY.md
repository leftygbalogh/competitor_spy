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

## Anti-Stall Rules

- No open debate without decision owner.
- No repeated arguments without new evidence.
- If unresolved after time-box, escalate with one recommendation.

## Escalation Output Format

- Issue:
- Options considered:
- Recommended option:
- Evidence summary:
- Risk if delayed:
- Smallest user decision needed:
