# Copilot Instructions for AI Governance Template

## Discovery and Order

At session start in projects using this template:

1. Read `GOVERNANCE_MODE.md`
2. Read `00_INTERACTION_GUARDRAILS.md`
3. Read `01_DECISION_POLICY.md`
4. Read `02_WORKFLOW_STAGES.md`
5. Read `03_TEAM_MODEL_HANDOFFS.md`
6. Read `04_PERSONA_DIRECTORY.md`
7. Read `05_IDLE_AUTOMATION.md`
8. Read `06_COMMAND_CHAIN_AND_PERSONALITY.md`
9. Read `07_QUALITY_DIMENSIONS.md`
10. Use templates in `templates/` as needed, including `IMPLEMENTATION_CHRONICLE_TEMPLATE.md` during Build

## Core Behavior

- Keep responses concise and direct.
- Use judgment to provide more detail only when complexity or risk requires it.
- In multi-line complex paragraphs, use clear punctuation for readability.
- For more complex responses, default to: direct answer, key reasoning, then open question or next decision.
- If ambiguous, ask one clarifying question at a time.
- Ask no more than 12 clarification questions per cycle.
- Stop clarifying once one explicit working assumption can be proposed; request yes/no confirmation before proceeding.

**When asking for approval, clarification, or presenting a decision point (e.g., 'Do you approve F001?'), always provide a brief, up-to-date summary of the relevant context and a concrete suggestion or recommended action. Never ask for approval or input using only a code or reference without context.**
- If the user asks multiple questions, turn them into a short task list and process one by one.
- For each list item, answer or ask clarification first, mark the item done, then move to the next item.
- Do not proceed to next stage until current stage approval.
- Stage approval requires an explicit yes; silence is not approval.

**After Stage 1 (Discover) approval, the agent must explicitly ask the user: "Can we start work now, boss?" before proceeding. If the user says yes and the team lead is the assigned approver, the team lead agent takes over coordination for the next stage.**
- Do not start code work unless explicitly requested.
- Do not take independent scope-expanding actions.
- Routine remit tasks can proceed without explicit permission.
- Permission-gated actions require explicit approval: legal/compliance risk, third-party code/repo download or execution, unsolicited refactoring, long-term strategy changes, or unclear remit.
- Before any push operation, run `git remote -v` and explicitly confirm which remote and branch should receive the push; if uncertain, ask and wait for confirmation.
- **For compiled languages: before any live E2E test, exploratory testing handover, or user-facing demo, always rebuild the release binary from current HEAD and verify the binary timestamp post-dates the last commit. Never run a live test against a binary whose build predates recent commits.**
- When governance values conflict, present the tradeoff briefly and recommend one path; the user may accept or redirect.
- When blocked, report: blocker, impact, what was attempted, and the smallest user decision or input needed.

## Record Keeping

- Append each user prompt to `prompts.md`.
- Keep `memory.md` updated with status, decisions, blockers, and next step.

## Engineering Principles

When in implementation stages, follow:

- TDD
- Domain-driven design
- XP principles
- Maintain implementation chronicle entries for each significant task or module; link them to source spec sections and task IDs

## New Project Bootstrap Rules

For projects initialized from this template:

- Ensure git repository is initialized.
- Keep governance folder project-specific and complete the document Q&A.
- Use the copied project-specific governance folder as the active working governance directory for all project governance decisions and artifacts.
- Treat this template as the master governance source; project-local improvements should be fed back here when approved.
- Use simple manual versioning and short changelog updates for the master template.
- Select project mode (Greenfield or Brownfield) at the start and carry that through brief, spec, planning, and role assignment.
- Ask project mode as the first discovery question before asking any project-domain requirement details.
- For template-clone projects, verify repository identity immediately after clone (`git remote -v`) and repoint `origin` before any publish if needed.
- **When `GOVERNANCE_MODE.md` mode is `Project` (not `Template`): Stage 1 discovery must include an explicit step — show the user the current `git remote -v` output, ask for the target remote repository URL for this project, and require the user to confirm or replace the origin before any commits are pushed. Do not proceed past Stage 1 without this confirmation.**
- Re-run full discovery and stage gates for each new project; prior template maturity does not bypass project-specific discovery.
- Create and approve project brief before formal specification.
- Create and approve formal specification before task planning.
- Create and approve numbered task list before implementation.
