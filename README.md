# AI Governance Template

This folder defines AI interaction rules, stage gates, team model, and project artifacts.

## Baseline Status

- Baseline version: 1.0
- Baseline state: Approved
- Approved on: 2026-03-19

## Discovery Order

Read and apply files in this order:

1. `GOVERNANCE_MODE.md`
2. `00_INTERACTION_GUARDRAILS.md`
3. `01_DECISION_POLICY.md`
4. `02_WORKFLOW_STAGES.md`
5. `03_TEAM_MODEL_HANDOFFS.md`
6. `04_PERSONA_DIRECTORY.md`
7. `05_IDLE_AUTOMATION.md`
8. `06_COMMAND_CHAIN_AND_PERSONALITY.md`
9. `07_QUALITY_DIMENSIONS.md`
10. `templates/PROJECT_BRIEF_TEMPLATE.md`
11. `templates/FORMAL_SPEC_TEMPLATE.md`
12. `templates/TASK_LIST_TEMPLATE.md`
13. `templates/IMPLEMENTATION_CHRONICLE_TEMPLATE.md`
14. `.github/copilot-instructions.md`
15. `REQUIREMENTS_SPEC_MANIFEST.md`
16. `DELIVERABLES_MANIFEST.md`

## Non-Negotiable Operating Rules

- Clarify ambiguity through questions, one at a time, up to 12 questions per clarification cycle.
- Do not proceed to the next stage until the current stage is complete and approved.
- After each approved stage completion, save and create a clear stage-completion commit before starting next-stage work.
- Prefer smaller, behavior-scoped commits to keep rollback points frequent.
- Do not write code unless explicitly instructed to do so.
- Record every user prompt in `prompts.md`.
- Maintain `memory.md` with current status, decisions, blockers, and next step.
- Linux compliance baseline applies to all template artifacts:
  - Text files must use LF line endings.
  - Path examples should use POSIX-style `/` separators by default.
  - Shell examples should be Linux-compatible unless a Windows-specific path is explicitly required.
- For new projects based on this template:
  - Initialize a git repository.
  - Copy this folder as project-specific governance folder.
  - Use that copied project-specific governance folder as the active working governance directory for the project.
  - Run a guided Q&A to customize all required documents.

## Usage in New Projects

1. Copy this folder into the new project.
2. Rename it to project-specific governance folder name.
3. Initialize git if not already initialized.
4. Complete Q&A customization in this order:
  - Read `GOVERNANCE_MODE.md` first and confirm whether this repo is in Template Development mode or Project mode
  - First question: determine project mode (Greenfield or Brownfield) before any domain-specific questions
   - Project brief
   - Formal specification
   - Task list
  - Implementation chronicle template usage rules
   - Agent/persona additions
5. Re-run the same discovery and stage-gated workflow in the new project context; do not skip stages because this template already exists.

## Living Template Model

- This folder is the master template.
- Each new project should copy this template into its own project-specific governance folder.
- Improvements discovered in project-local governance should be fed back into the master template when approved.
- Subsequent projects should start from the improved master template.
- Use simple manual versioning and short changelog updates as the master template evolves.
- This creates a repeatable loop: template -> project discovery/spec/planning -> improvements -> template.

## Notes on Automation

Idle-time actions (save/update/commit after inactivity) are policy requirements. They should be enforced with editor hooks or automation scripts in each project environment.

## Release Readiness Artifacts

- Release checklist: `RELEASE_CHECKLIST.md`
- Operational notes and rollback: `OPERATIONS_AND_ROLLBACK.md`
- Post-release monitoring plan: `POST_RELEASE_MONITORING.md`
- Failure runbook: `RUNBOOK_KNOWN_FAILURES.md`
- Getting-started guide: `GETTING_STARTED.md`
- Changelog: `CHANGELOG.md`
- Requirements/spec index: `REQUIREMENTS_SPEC_MANIFEST.md`
- Deliverables set baseline: `DELIVERABLES_MANIFEST.md`
