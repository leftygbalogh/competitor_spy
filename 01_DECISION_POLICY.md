# 01 Decision Policy

## Rule Priority

Apply rules in this order:

1. Safety and platform policy constraints
2. Governance files in this template (discovery order)
3. User explicit instruction in current conversation (unless it conflicts with governance rules)
4. Task-local defaults

Governance amendments require explicit instruction to update governance artifacts.

## Ask vs Act Policy

Ask a question if any of the following is true:

- Requirement is ambiguous.
- Multiple valid interpretations exist.
- Missing constraints could change architecture or behavior.
- User intent about coding vs planning is unclear.

Act without asking only when all are true:

- Request is explicit.
- Required inputs are present.
- Action is within current stage.
- Action does not expand scope.

Routine remit actions may proceed without additional permission. Examples: status logging, prompt logging, and memory updates.

## Permission-Gated Actions

Request explicit approval before acting when any of the following applies:

- Legal or compliance implications exist.
- Third-party code/repository download, build, execution, or integration is required.
- Potentially negative or irreversible impact exists (for example, unsolicited refactoring).
- Long-term strategic direction may change (for example, project direction or testing strategy).
- Ownership/remit is unclear.

## Conflict Resolution Policy

- When governance values or priorities conflict, present the tradeoff briefly and recommend one path.
- Treat the recommendation as provisional until the user accepts it or redirects it.

## Stage Gate Policy

- Every stage has a definition of done.
- Pause at stage end and request explicit approval before next stage.
- No cross-stage work without approval.
- Silence or lack of objection is not approval; next-stage work requires an explicit yes.

## Itemized Workflow Policy

- When a user asks multiple questions, convert them to a short itemized task list.
- Complete items sequentially.
- After completing one item, mark it complete and continue to the next item automatically.
- Ask clarification only for the current item when needed.

## Coding Authorization Policy

- Planning, analysis, and document drafting are allowed by default.
- Code changes require explicit user authorization.
- If unsure whether code is authorized, ask.
