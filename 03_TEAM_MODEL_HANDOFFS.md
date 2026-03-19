# 03 Team Model and Handoffs

## Delivery Modes

- Greenfield mode: design-forward delivery with domain discovery, architecture foresight, and specification-first implementation.
- Brownfield mode: behavior-preserving modernization with code-verified baseline extraction and tiny-iteration transformation.

## Team Roles

Core roles include:

- Requirement interviewer and brief author
- Formal specification author
- Task decomposition planner
- Team lead
- Developers by language and specialty
- Test developers and exploratory testers
- Product and domain specialists
- SRE, operations, and network engineering roles

Mode-focused role emphasis:

- Greenfield emphasis: domain experts, architects, formal spec writers, pseudocode and formal methods specialists.
- Brownfield emphasis: behavior baseline analysts, parity testers, reverse-engineering specialists, and fine-grained migration planners.

## Command and Accountability

- Apply stage command chain from `06_COMMAND_CHAIN_AND_PERSONALITY.md`.
- Each stage has one accountable lead to prevent indecision.
- Contributor roles provide options and evidence; accountable lead recommends one path.

## Handoff Contract

Every handoff must include:

- Source artifact reference
- Scope in and out
- Numbered acceptance criteria
- Risks, assumptions, and blockers
- Required clarifications

Mode-specific handoff requirements:

- Greenfield: include evolution assumptions and architecture tradeoff rationale.
- Brownfield: include behavior parity target, baseline evidence reference, and smallest safe migration unit.

## Work Tracking

- Task progress is recorded against numbered tasks.
- Decision changes are recorded in `memory.md`.
- User prompts are appended to `prompts.md`.

## Escalation Rules

- If blocked by ambiguity, ask one clarifying question.
- If blocked by missing dependency, report exact dependency and impact.
- If blocked by conflict in requirements, pause and request resolution.
- If debate exceeds time-box without new evidence, escalate using the escalation output format in `06_COMMAND_CHAIN_AND_PERSONALITY.md`.
