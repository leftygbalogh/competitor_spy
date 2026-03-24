# Requirements and Specification Manifest

Purpose: canonical index of requirement/specification artifacts, where they live, what they contain, and why they exist.

| Title | Location | Core Contents | Purpose |
|---|---|---|---|
| Project Brief | PROJECT_BRIEF.md | problem statement, scope, stakeholders, FR/NFR inputs, approval delegation, risks | source intent and stage authority baseline |
| Formal Specification | FORMAL_SPEC.md | behavior contracts, architecture constraints, data/interface contracts, traceability seed | language-agnostic correctness contract |
| Task List | TASK_LIST.md | ordered work backlog, dependencies, TDD notes, ownership, DoR/DoD status | execution plan and progress ledger |
| Implementation Chronicle | IMPLEMENTATION_CHRONICLE.md | implementation decisions, alternatives, invariants, reconstruction notes | preserve why/how for rebuild and audits |
| Release Checklist | RELEASE_CHECKLIST.md | verify and release gate completion evidence | release readiness proof |
| Runbook Known Failures | RUNBOOK_KNOWN_FAILURES.md | known failure modes, triage steps, mitigations | operational recovery playbook |
| Operations and Rollback | OPERATIONS_AND_ROLLBACK.md | deployment/rollback commands and criteria | safe release/revert control |
| Post Release Monitoring | POST_RELEASE_MONITORING.md | alerts, metrics, watch windows, incident triggers | production feedback loop |
| Feedback File | templates/feedback.json (project-local copy) | stage-tagged governance improvement proposals | continuous template hardening input |
| Requirements/Spec Manifest | REQUIREMENTS_SPEC_MANIFEST.md | this index | prevents artifact omission and drift |

## Usage Rules

1. Keep this file updated whenever a spec artifact is added, renamed, or retired.
2. Stage 2 cannot close if required spec artifacts are missing from this manifest.
3. Stage 6 cannot close if manifest entries do not match actual delivered artifact set.
