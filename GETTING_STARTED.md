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

## Repository Identity Setup (Mandatory)

1. Immediately after cloning a project from this template, run `git remote -v`.
2. Confirm the intended application repository URL for fetch and push.
3. If `origin` still points to the template repository, repoint it before any branch work:
   - `git remote set-url origin <application-repository-url>`
4. Record this verification in release evidence before first publish.

No push is allowed until repository identity is explicitly verified.

## Linux Compliance Setup

1. Ensure `.gitattributes` enforces LF (`* text=auto eol=lf`).
2. Ensure `.editorconfig` sets `end_of_line = lf`.
3. Use POSIX-style paths (`/`) in governance examples unless platform-specific behavior is being documented.
4. Prefer Linux-compatible shell command examples for shared runbooks and onboarding docs.

## Required Logs

- Append every user prompt to `prompts.md`.
- Keep `memory.md` updated with status, decisions, blockers, and next step.
