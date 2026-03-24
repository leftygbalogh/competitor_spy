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

 - Oracle Agent (authoritative fact-checker, policy/standards arbiter)
 - Claire Voyant Agent (risk forecaster, scenario planner)

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

 - For major handoffs, include Claire Voyant Agent's risk forecast and mitigation suggestions.
 - If acceptance criteria or scope are unclear, consult Oracle Agent for policy or standards interpretation before escalating.

Mode-specific handoff requirements:

- Greenfield: include evolution assumptions and architecture tradeoff rationale.
- Brownfield: include behavior parity target, baseline evidence reference, and smallest safe migration unit.


## Work Tracking

- Task progress is recorded against numbered tasks.
- Decision changes are recorded in `memory.md`.
- User prompts are appended to `prompts.md`.
- Oracle Agent and Claire Voyant Agent contributions should be referenced in decision logs when invoked.


## Escalation Rules

- If blocked by ambiguity, ask one clarifying question. If authoritative clarification is needed, invoke Oracle Agent before escalating to the user.
- If blocked by missing dependency, report exact dependency and impact.
- If blocked by conflict in requirements, pause and request resolution.
- If debate exceeds time-box without new evidence, escalate using the escalation output format in `06_COMMAND_CHAIN_AND_PERSONALITY.md`.

## Clarification Routing Protocol

When ambiguity appears, follow this route before escalating to the user:

1. Identify remit-holder peer for the ambiguous area.
2. Ask peer clarification question with concrete options.
3. Record peer response in task progress notes.
4. If unresolved, escalate with unresolved-ambiguity note including what was tried.

Required artifact trail:

- Task record must include clarification note or escalation note reference.
- `memory.md` must include unresolved-ambiguity reason when escalation happens.
