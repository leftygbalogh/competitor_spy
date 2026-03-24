# Governance Mode

## Current Mode

- Mode: Project
- Effective date: 2026-03-20
- Owner: Lefty

## Meaning

this repository is operating as a live software delivery project and stage-by-stage project execution is the active concern.

## Agent Routing Rules

When Mode is `Template Development`, agents should:

1. Improve governance docs, templates, and personas.
2. Process `templates/feedback.json` outputs from real projects.
3. Propose and implement template hardening updates.
4. Avoid starting a real project discovery flow in this repository unless explicitly instructed.

When Mode is `Project`, agents should:

1. Run mode-first project discovery for that project.
2. Execute stages against project artifacts in stage order.
3. Treat this governance folder as the active working governance directory.

## Suggested First Response by Mode

- Template Development: propose next template-improvement task or governance review.
- Project: start Stage 1 Discover with mode selection (Greenfield/Brownfield).

## Switching Mode

To switch mode, update this file:

1. In the **Current Mode** block: comment out the active `Mode` line and uncomment the target one.
2. Set `Project name` if switching to Project mode.
3. Update `Effective date`.
4. Record the switch reason in `memory.md`.
5. Commit the mode switch before any stage work continues.

## Notes

- This file is the first routing check before discovery-order execution.
- Stage gates and approval rules remain active in both modes.
