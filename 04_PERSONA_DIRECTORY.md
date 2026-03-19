# 04 Persona Directory

This file indexes persona definitions used by the virtual software development team.

## Persona Tracks by Project Mode

- Greenfield track: scope refinement, architecture foresight, formal specification, pseudocode, and implementation-from-spec roles.
- Brownfield track: implemented behavior extraction, parity testing, decomposition into tiny migration units, and controlled rewrite/refactor roles.

## Language Priority

- Primary language: Rust
- Secondary language: Python
- Supporting baseline languages: C#, C++, Java, Bash, TypeScript, JavaScript, Go, Kotlin, SQL, PowerShell, C
- Policy: Rust and Python can have expanded specialty packs. Supporting baseline languages use the core first-wave personas and language implementer roles only.

## Initial Persona Groups

- Specification and requirements personas
- Planning and task decomposition personas
- Team leadership personas
- Development personas by language:
  - C#
  - C++
  - Java
  - Bash
  - Python
  - Rust
  - TypeScript
  - JavaScript
  - Go
  - Kotlin
  - SQL
  - PowerShell
  - C
- Development specialties:
  - Frontend
  - Backend
  - Database
  - CLI
  - Defensive programming
- Testing personas:
  - Test development
  - Exploratory testing
- Product and domain personas
- SRE and operations personas
- Network engineering personas

Mode-specific persona priorities:

- Greenfield priorities:
  - Domain discovery expert
  - Evolution-oriented architect
  - Formal specification writer
  - Pseudocode and mathematical specification specialist
- Brownfield priorities:
  - Behavior baseline analyst
  - Characterization and parity test engineer
  - Incremental rewrite planner
  - Code archaeology and dependency mapper

## Persona Definition Standard

Use `agents/AGENT_TEMPLATE.md` for all persona files.

Personality and command model reference:

- `06_COMMAND_CHAIN_AND_PERSONALITY.md`

## First-Wave Persona Set (12)

1. `agents/greenfield-domain-discovery-expert.md`
2. `agents/greenfield-evolution-architect.md`
3. `agents/greenfield-formal-spec-author.md`
4. `agents/human-spec-writer.md`
5. `agents/traceability-mapper.md`
6. `agents/tdd-driver.md`
7. `agents/tdd-navigator.md`
8. `agents/refactoring-steward.md`
9. `agents/unit-test-completeness-engineer.md`
10. `agents/property-based-test-engineer.md`
11. `agents/readability-reviewer.md`
12. `agents/maintainability-reviewer.md`

Documentation support role:

- `agents/technical-writer-live-examples.md`

Additional language implementation personas:

- `agents/typescript-fullstack-implementer.md`
- `agents/go-backend-implementer.md`
- `agents/kotlin-jvm-implementer.md`
- `agents/javascript-legacy-migration-specialist.md`
- `agents/sql-database-engineer.md`
- `agents/powershell-automation-engineer.md`
- `agents/c-systems-implementer.md`

Specialty variant personas:

- `agents/backend-specialist-variant.md`
- `agents/database-specialist-variant.md`
- `agents/cli-specialist-variant.md`
- `agents/defensive-programming-specialist-variant.md`

Rust primary specialty pack:

- `agents/rust-backend-specialist.md`
- `agents/rust-database-specialist.md`
- `agents/rust-cli-specialist.md`
- `agents/rust-defensive-programming-specialist.md`

Rust optional extension pack:

- `agents/rust-webassembly-frontend-specialist.md`
- `agents/rust-systems-performance-specialist.md`
- `agents/rust-async-concurrency-reliability-specialist.md`
- `agents/rust-api-contract-serialization-specialist.md`

Rust reviewer micro-personas:

- `agents/rust-unsafe-code-auditor.md`
- `agents/rust-api-ergonomics-reviewer.md`
- `agents/rust-documentation-quality-reviewer.md`

Python secondary specialty pack:

- `agents/python-backend-specialist.md`
- `agents/python-database-specialist.md`
- `agents/python-cli-specialist.md`
- `agents/python-defensive-programming-specialist.md`

Brownfield core migration trio:

- `agents/brownfield-behavior-baseline-analyst.md`
- `agents/brownfield-parity-test-engineer.md`
- `agents/brownfield-incremental-rewrite-planner.md`

Required sections:

- Mission
- In-scope responsibilities
- Out-of-scope boundaries
- Authority and rights
- Inputs required
- Outputs produced
- Documentation obligations
- Handoff expectations
- Quality bar
- Escalation triggers

Build-capable personas must explicitly state:

- That implementation decisions and trade-offs are recorded in `templates/IMPLEMENTATION_CHRONICLE_TEMPLATE.md`
- That chronicle entries link to source spec sections and task IDs
- That task completion can be blocked if the chronicle entry is missing
