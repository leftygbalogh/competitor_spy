# Getting Started

## Goal

Use this governance template as the active policy and workflow layer for a new software project.

## Quick Start

1. Copy this folder into your target project.
2. Rename it to a project-specific governance folder.
3. Initialize git repository if missing.
4. Read files in the discovery order listed in `README.md`.
5. First discovery question must select project mode: Greenfield or Brownfield.
6. Complete and approve artifacts in order:
   - `PROJECT_BRIEF.md`
   - `FORMAL_SPEC.md`
   - `TASK_LIST.md`
   - `IMPLEMENTATION_CHRONICLE.md`
7. Enforce stage gates:
   - explicit approval required
   - stage-completion commit required

## Required Logs

- Append every user prompt to `prompts.md`.
- Keep `memory.md` updated with status, decisions, blockers, and next step.
