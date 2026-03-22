# ADR-001: Rust Cargo Workspace with One Crate per Architectural Layer

**Date:** 2026-03-21
**Status:** Accepted
**Spec reference:** FORMAL_SPEC.md §7, Decision 1

---

## Context

Competitor Spy requires strict separation between domain logic, data adapters, output rendering, credential management, telemetry, and the CLI entry point. The boundary between these layers must be enforced at the toolchain level, not by convention alone.

The project is designed to serve as the backbone for a future web consumer. That future consumer must be able to import the domain crate without pulling in CLI rendering or HTTP adapter code.

## Decision

Use a Rust Cargo workspace with one crate per architectural layer:

- `competitor_spy_domain` — pure domain logic; zero I/O, async, or rendering dependencies
- `competitor_spy_adapters` — HTTP adapters; implements `SourceAdapter` trait
- `competitor_spy_output` — terminal and PDF rendering; no domain logic
- `competitor_spy_credentials` — age-encrypted credential store; file I/O only
- `competitor_spy_telemetry` — OpenTelemetry structured logging with secret redaction
- `competitor_spy_cli` — thin entry point; argument parsing only; no domain logic

## Rationale

The Rust compiler enforces crate boundaries at the type and visibility level. A downstream crate that attempts to import `competitor_spy_output` into `competitor_spy_domain` will fail to build. This makes the boundary violation detectable without code review.

This is stronger than module-level separation within a single crate, where `pub(super)` and `pub(crate)` can be bypassed by placing code at the same level.

## Consequences

- Any business logic found in `competitor_spy_cli` or `competitor_spy_output` is an architecture violation and a build blocker.
- A future web server crate can depend on `competitor_spy_domain` and `competitor_spy_adapters` without pulling in printpdf, clap, or any rendering dependency.
- Adding a new source adapter means adding a module to `competitor_spy_adapters` only — domain and output crates are not touched.

## Alternatives Rejected

- **Single crate with modules** — module boundaries are advisory, not enforced by the compiler. Rejected.
