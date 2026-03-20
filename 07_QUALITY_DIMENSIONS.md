# 07 Quality Dimensions

## Purpose

This document defines what good software quality depends on, how dimensions are prioritized, and when each pack is active during delivery.

This is the master quality reference for this template. Every agent persona, stage gate, and review checklist derives its quality standards from this document. When dimensions conflict or resources are constrained, this document determines what gets addressed first.

---

## The 15 Dimensions

### 1. Problem Understanding
- Correct and complete requirements (what the system must do)
- Formal or semi-formal specification (what correctness means, provably)
- Domain model accuracy (code reflects how the domain actually works)
- Stakeholder alignment (who needs what, and why)
- Explicit non-goals (what the system deliberately will not do)

### 2. Architecture & Design
- Appropriate architectural style for the problem (event-driven, layered, microservices, monolith, etc.)
- Separation of concerns (each module has one clear responsibility)
- Dependency direction discipline (dependencies point inward, not outward)
- Avoidance of premature abstraction
- Explicit boundary definitions (what crosses the boundary, in what format)
- Evolution plan (can the system grow without rewriting)
- Fit for non-functional requirements (latency, throughput, storage, resilience)

### 3. Code Quality
- Naming reveals intent, not implementation
- Function/method size and cognitive load (small, single-purpose units)
- Cyclomatic complexity (shallow decision trees)
- Duplication elimination where duplication is genuinely harmful
- Error handling discipline (explicit, consistent, no silent failures)
- Language idiom adherence (code looks natural in its language)
- Coding standard compliance (formatting, style, lint rules)
- Dependency management (minimal, explicit, versioned)
- Absence of dead code
- No magic numbers or unexplained constants

### 4. Behavioral Specification Rigor
- Specification is language-agnostic: precise on behavior, configuration, and algorithmic flows; silent on implementation language and idiom
- The test of a good spec: two independent teams working in different languages produce functionally equivalent programs from the document alone
- Default tooling: statecharts (states, transitions, events, guards, actions) + design by contract (pre/post conditions, invariants) + decision tables (complex conditional logic)
- Pre/post condition contracts on all non-trivial operations
- Invariant definitions for core data structures and domain rules
- Property definitions for property-based testing
- Type system usage to carry correctness guarantees where language permits
- Mathematical verification methods (TLA+, Alloy, B-method) are an optional escalation for safety-critical or concurrent-protocol work only — never required by default

### 5. Testing
- Unit test coverage (logic tested in isolation)
- Integration test coverage (components interact correctly)
- End-to-end test coverage (system behaves correctly from user perspective)
- Negative-path coverage (illegal states, malformed inputs, and unexpected user actions are asserted deliberately)
- Boundary and edge-case coverage (off-by-one, empty inputs, max values, concurrent access points)
- Orchestration-path coverage (flows where input parsing, output rendering, and persistence combine are tested end-to-end, not component-isolated)
- Interactive CLI reproducibility coverage (manual sessions capture screen-state and application-state artifacts so observed defects can be replayed and triaged)
- Property-based tests (invariants hold across generated inputs)
- Mutation testing (tests actually detect faults, not just execute)
- Regression test suite (previously fixed bugs do not return; escaped defects are converted into permanent regression tests)
- Performance tests (latency and throughput stay within bounds)
- Security tests (known vulnerability classes are actively probed)
- Contract tests (API agreements between services are verified)
- Test independence (tests do not depend on each other's state)
- Test readability (tests are documentation of intended behavior)

### 6. Security
- Input validation at every boundary
- Authentication and authorization correctness
- Least privilege principle applied throughout
- No secrets in code or logs
- Dependency vulnerability scanning
- Cryptographic correctness (right algorithm, right key management)
- Secure defaults (system is secure out of the box, not by configuration)
- Audit logging of sensitive operations

### 7. Performance & Efficiency
- Latency within defined bounds under expected load
- Throughput within defined bounds
- Memory usage within defined bounds
- Efficient algorithms and data structures for the problem size
- No unnecessary blocking or contention
- Resource cleanup (connections, file handles, memory)

### 8. Reliability & Resilience
- Defined failure modes (what happens when X fails)
- Graceful degradation (partial failure does not cause total failure)
- Retry and backoff discipline
- Timeout discipline (no unbounded waits)
- Circuit breaker or bulkhead patterns where appropriate
- Data durability guarantees (what survives a crash)
- Recovery time objective met

### 9. Observability
- Structured logs (machine-readable, queryable)
- Metrics (counters, gauges, histograms on key operations)
- Distributed tracing (requests traceable end-to-end)
- Health endpoints
- Alerting on meaningful signals (not noise)
- Runbooks for known failure scenarios
- For interactive CLI projects, screen-state and application-state capture artifacts are retained with clear naming and path conventions for post-failure analysis

### 10. Documentation
- Architecture decision records (why decisions were made, not just what)
- API documentation (contract, inputs, outputs, errors, examples)
- Implementation chronicle records (how each significant unit was implemented, why those choices were made, and how to reconstruct it if code is lost)
- In-code comments where intent is non-obvious (not where it is obvious)
- Runbook / operational guide
- Getting-started guide for contributors
- Glossary of domain terms
- Changelog

### 11. Developer Experience
- Build time acceptable
- Local development setup simple and reproducible
- Test suite runs fast and reliably locally
- CI feedback loop short
- Contribution guidelines clear
- Onboarding time to first useful contribution low

### 12. Process & Workflow
- Stage-gated delivery (no code before spec, no ship before verify)
- Test-driven development (tests written before implementation)
- Continuous integration (every commit tested)
- Continuous delivery readiness (main branch always shippable)
- Peer review (no single-author merge to main)
- Refactoring discipline (technical debt addressed continuously)
- Escaped-defect conversion (any defect found in production or at release time must be converted into a permanent regression test and a process improvement before stage closure)
- Retrospective and improvement loop

### 13. Data Quality
- Schema correctness and version discipline
- Migration safety (backward compatible changes)
- Data validation at ingestion
- Referential integrity enforced at the right layer
- Retention and deletion policy defined

### 14. Compliance & Auditability
- Regulatory requirements met (GDPR, HIPAA, SOC2, financial, etc.)
- Audit trail completeness
- Data residency requirements met
- License compliance for dependencies

### 15. Maintainability Over Time
- Seams for change exist (new behavior added without rewriting old)
- External dependencies isolated behind abstractions
- Version upgrade path exists for core dependencies
- Bus factor greater than 1 (no single person holds critical knowledge)

---

## Priority Grid

### Q1 — Core Pack (Urgent + Important)

Always active. No opt-out. Enforced from Stage 1 Discover onward.

| # | Dimension | Rationale |
|---|-----------|-----------|
| 1 | Problem Understanding | Wrong requirements cannot be recovered from. Nothing downstream fixes this. |
| 2 | Architecture & Design | Structural decisions made here cast long shadows. Change costs compound. |
| 3 | Code Quality | Technical debt accrues from line one. Standards must exist before first commit. |
| 5 | Testing | TDD means tests precede code. Cannot be retrofitted; must be designed in. |
| 6 | Security | Security defects from early design are the most expensive to fix. Must be designed in. |
| 12 | Process & Workflow | The scaffold everything else hangs from. Stage gates, TDD, CI — urgent on every project. |

---

### Q2 — Stage-Unlocked Pack (Important, Not Universally Urgent)

Present in the template. Activated at the named stage gate. Agent checklists and stage done criteria expand to include these dimensions when their stage is reached.

| # | Dimension | Activated at stage |
|---|-----------|-------------------|
| 4 | Behavioral Specification Rigor | Stage 2: Specify — behavioral contract produced using statecharts, design by contract, and decision tables |
| 7 | Performance & Efficiency | Stage 2: Specify (define targets); Stage 5: Verify (validate against targets) |
| 8 | Reliability & Resilience | Stage 2: Specify (design for failure); Stage 5: Verify (validate failure paths) |
| 9 | Observability | Stage 4: Build (instrument hooks); Stage 6: Release (confirm operational readiness) |
| 10 | Documentation | Stage 4: Build (ADRs, in-code, API docs); Stage 6: Release (runbooks, guides, changelog) |
| 11 | Developer Experience | Stage 3: Plan (CI setup, contribution standards); Stage 4: Build (iterative improvement) |
| 15 | Maintainability Over Time | Stage 2: Specify (design for change seams); Stage 5: Verify (confirm seams exist and work) |

---

### Q3 — Project-Triggered Modules (Urgent When Project Requires It)

Present in the template as optional plug-in governance. Inactive by default. Declared during Stage 1 Discover. Once declared, they behave as core expectations for the project.

| # | Dimension | Activation trigger |
|---|-----------|-------------------|
| 13 | Data Quality | Project declares persistent storage, schemas, or migrations |
| 14 | Compliance & Auditability | Project declares regulatory scope (GDPR, HIPAA, SOC2, financial, healthcare, etc.) |

---

## Pack Definitions

**Core pack (Q1):** Always active, beginning at Stage 1 Discover. No project may bypass these dimensions. These are the baseline expectation for every agent, every review, and every stage gate in every project.

**Stage-unlocked pack (Q2):** Defined at project start. Each dimension has a named activation stage. When that stage is reached, the relevant agent checklists, done criteria, and review expectations expand to include the dimension. If Q2 targets are not defined by the end of Stage 2, treat that as a spec gap and block stage-gate approval.

**Project-triggered modules (Q3):** Declared in Stage 1 Discover Q&A. If the project declares a Q3 trigger, that dimension joins Q1 as an always-active expectation for this project. The relevant specialist agents from `04_PERSONA_DIRECTORY.md` are activated.

---

## Three-Layer Documentation Stack

Every project should be reconstructible from documentation alone.

| Layer | Name | Purpose |
|-------|------|---------|
| 1 | Commander's intent | Why the system exists, what outcome it serves, what tradeoffs matter |
| 2 | Behavioral specification | What the system must do, language-agnostic, precise on behavior, configuration, and algorithmic flows |
| 3 | Implementation chronicle | How the implementation was built, module by module, including decisions, rejected alternatives, trade-offs, and reconstruction notes |

The reconstruction test is simple: if code is lost, a new team should be able to rebuild the system from these three layers and converge on materially equivalent behavior.

Implementation chronicle entries are required during Stage 4 Build for each significant task or module. They are not a replacement for code comments or ADRs; they sit between architecture decisions and source code.

---

## Behavioral Specification Rigor (Q2 Dimension 4)

### Default tooling (required on every project at Stage 2)

| Tool | Role in the spec |
|------|------------------|
| Statecharts (Harel / UML state machines) | Model all states, transitions, events, guards, and actions. Language-agnostic. Representable as text (PlantUML, Mermaid, SCXML) or diagrams. |
| Design by contract (pre/post conditions + invariants) | Each operation: what must be true before, what is guaranteed after. Invariants: properties that always hold across state transitions. |
| Decision tables | Where multiple conditions combine to determine outcomes. Unambiguous, testable, language-agnostic. Replaces complex conditional prose. |

Together these produce a behavioral contract precise enough that two teams working in different languages converge on functionally equivalent programs.

### Escalation tooling (optional, project-declared)

Activate when the project requires provable correctness, not just behavioral precision.

| Method | When to escalate |
|--------|------------------|
| TLA+ | Concurrent or distributed systems, protocol correctness |
| Alloy | Structural data model correctness, relational invariants |
| B-method | Safety-critical systems requiring machine-checked proofs |

### Per-language notes

- **Rust (primary):** type-driven design encodes invariants in the type system by default; `proptest` for property-based test coverage. Statecharts and contracts still required in spec.
- **Python (secondary):** `hypothesis` for property-based testing; mypy for type-level guarantees.
- **All other languages:** statecharts + design by contract + decision tables as spec standard; no additional tooling required unless project escalates.

---

## Per-Language Coding Standards

Language-specific standards define what good looks like for code quality (Q1 Dimension 3) and behavioral specification rigor (Q2 Dimension 4) in each language.

| Language | Tier | Standards document |
|----------|------|--------------------|
| Rust | Primary | See `agents/rust-*` persona files; clippy + rustfmt enforced; proptest default |
| Python | Secondary | See `agents/python-*` persona files; ruff + mypy enforced; hypothesis default |
| TypeScript | Baseline | See `agents/typescript-fullstack-implementer.md` |
| Go | Baseline | See `agents/go-backend-implementer.md` |
| Kotlin | Baseline | See `agents/kotlin-jvm-implementer.md` |
| JavaScript | Baseline | See `agents/javascript-legacy-migration-specialist.md` |
| SQL | Baseline | See `agents/sql-database-engineer.md` |
| PowerShell | Baseline | See `agents/powershell-automation-engineer.md` |
| C | Baseline | See `agents/c-systems-implementer.md` |
| Java | Baseline | See language implementer persona when activated |
| C++ | Baseline | See language implementer persona when activated |
| Bash | Baseline | See language implementer persona when activated |
| C# | Baseline | See language implementer persona when activated |

---

## Agent Coverage Map

Each agent persona enforces the quality dimensions within its scope:

- **Q1 dimensions:** enforced by all agents at all times.
- **Q2 dimensions:** enforced by specialist agents when the dimension is unlocked at its named stage.
- **Q3 dimensions:** enforced by specialist agents when declared active in Stage 1 Discover.

Reviewer micro-personas (e.g., `rust-unsafe-code-auditor`, `rust-api-ergonomics-reviewer`) may be activated for high-risk or high-stakes dimensions.

See `04_PERSONA_DIRECTORY.md` for the full index of available agents and tiers.

---

## Changelog

| Version | Date | Change |
|---------|------|--------|
| 1.0 | 2026-03-19 | Initial version. 15 dimensions, Q1/Q2/Q3 grid, formal methods table, per-language standards map, agent coverage map. |
| 1.1 | 2026-03-19 | Dimension 4 renamed from Formal Correctness to Behavioral Specification Rigor. Default tooling set to statecharts + design by contract + decision tables. Mathematical methods demoted to optional escalation only. |
| 1.2 | 2026-03-19 | Added three-layer documentation stack and implementation chronicle as required documentation depth during Build. |
