# Deliverables Manifest

Purpose: define what an industrial-grade software delivery should include beyond source code.

## Core Deliverables (always expected)

| Deliverable | Typical Location | Why it matters |
|---|---|---|
| Source code | src/ | product behavior |
| Tests (unit/integration/e2e) | tests/ | correctness evidence |
| Build and dependency config | pyproject.toml / package files / CI config | reproducible builds |
| Project Brief | PROJECT_BRIEF.md | intent and scope authority |
| Formal Spec | FORMAL_SPEC.md | behavior contract |
| Task List | TASK_LIST.md | planned execution record |
| Implementation Chronicle | IMPLEMENTATION_CHRONICLE.md | reconstruction and decision audit |
| Release Checklist | RELEASE_CHECKLIST.md | gate completion proof |
| Runbook (known failures) | RUNBOOK_KNOWN_FAILURES.md | operator response steps |
| Operations and rollback plan | OPERATIONS_AND_ROLLBACK.md | safe deployment/reversion |
| Post-release monitoring plan | POST_RELEASE_MONITORING.md | production health controls |
| Changelog | CHANGELOG.md | version and change trace |
| Getting started / setup guide | GETTING_STARTED.md | onboarding and reproducibility |
| End-user documentation | docs/user/README.md | installation guide and user guide for non-developer end users; include only applicable sections |
| Admin and operations guide | docs/admin/ | configuration reference, environment variables, operator runbook, and monitoring guide |
| Feedback proposals | templates/feedback.json (project-local) | process improvement capture |
| Pair programming session log | docs/evidence/pair-programming-log.md | auditable driver/navigator collaboration evidence |
| Architecture Decision Records (ADRs) | docs/adr/ | significant design decisions |
| API contract docs | docs/api/ | external or internal API surface |
| API usage guide | docs/api/guide.md | annotated API guide with worked examples for integrators |
| Traceability report (FR/NFR -> tests/evidence) | docs/evidence/traceability.md | regulated/high-assurance projects |
| Release remote proof snapshot | docs/evidence/release-remote-proof.md | prevents template or wrong-repository pushes by recording verified publish target |
| Security review report | docs/security/ | security-sensitive or public deployment |
| Threat model | docs/security/threat-model.md | internet-facing or sensitive data scope |
| Performance benchmark report | docs/perf/ | latency/throughput constraints |
| Environment validation matrix | docs/evidence/environment-matrix.md | terminal/UI/environment-sensitive projects |
| UAT/manual test report | docs/evidence/manual-test-report.md | user-facing workflows |
| SBOM/license report | docs/compliance/ | dependency/compliance governance |

## Delivery Readiness Rule

A release is not considered complete unless required deliverables exist, are current, and are linked from release evidence.
