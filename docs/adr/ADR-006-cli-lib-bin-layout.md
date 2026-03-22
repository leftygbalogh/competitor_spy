# ADR-006: CLI Lib+Bin Layout for Acceptance-Testable Entry Point

**Date:** 2026-03-22
**Status:** Accepted
**Spec reference:** T-016, T-017; IMPLEMENTATION_CHRONICLE.md CHR-CSPY-016

---

## Context

The acceptance tests (AS-001 through AS-005) need to exercise the full run path — argument validation, geocoding, adapter collection, normalisation, ranking, terminal render, PDF render — in-process, without spawning the binary as a subprocess.

A standard single-binary `[[bin]]` Cargo target cannot be imported by integration tests. Spawning the binary adds process-start overhead and makes URL injection for mock servers impossible.

## Decision

Organise `competitor_spy_cli` as both a library and a binary:

```toml
[[bin]]
name = "competitor-spy"
path = "src/main.rs"

[lib]
name = "competitor_spy_cli"
path = "src/lib.rs"
```

- `src/lib.rs` — re-exports `AdapterUrls` and `run_with_urls()`
- `src/runner.rs` — contains all run logic; `run_with_urls()` accepts injectable adapter URLs
- `src/main.rs` — thin clap wrapper; calls `runner::run_with_urls()` with `AdapterUrls::production()`

## Rationale

`AdapterUrls` is a plain struct with four URL fields — one per adapter. In acceptance tests, all four are pointed at a local wiremock `MockServer`. In production, `AdapterUrls::production()` returns the real upstream URLs.

This design means: zero test-specific code in production paths; same code under both mock and live execution; no subprocess overhead in tests.

## Consequences

- Acceptance tests import `competitor_spy_cli::run_with_urls` directly.
- Adding a new adapter requires adding its URL field to `AdapterUrls`, updating `production()`, and wiring it in `runner.rs`. Tests automatically cover it via the existing acceptance test infrastructure.
- `src/main.rs` has no logic to test — it is an explicit boundary, not a business logic layer.

## Alternatives Rejected

- **Spawn binary as subprocess in tests** — requires binary to be built before tests; no in-process URL injection; slow. Rejected.
- **Trait-based URL injection in production code** — adds abstraction for a concern only relevant in testing. Rejected in favour of simple struct injection.
