# ADR-002: Async Runtime — Tokio for All I/O

**Date:** 2026-03-21
**Status:** Accepted
**Spec reference:** FORMAL_SPEC.md §7, Decision 2

---

## Context

Competitor Spy queries multiple data sources per run (Nominatim, Overpass, Yelp, Google Places). Each adapter enforces intentional pacing of [5, 15]s per request. If adapters were executed sequentially, a 4-adapter run would take a minimum of 20–60 seconds of pacing time alone, plus network latency.

## Decision

Use Tokio as the async runtime for all I/O operations. All adapters are implemented as `async fn` on an `#[async_trait]` trait.

## Rationale

Tokio enables concurrent adapter execution: while one adapter is in its pacing delay, another can be making its HTTP request. Total wall-clock time is dominated by the slowest adapter rather than the sum of all adapters.

Each adapter manages its own pacing independently, so Adapter A can be throttling while Adapter B is requesting. There is no shared throttle state between adapters.

## Consequences

- The `competitor_spy_adapters` crate requires Tokio and reqwest.
- The `competitor_spy_domain` crate has zero async dependencies (pure sync logic). This preserves its suitability as a future web backend dependency.
- Acceptance tests use `#[tokio::test]` to exercise the full async run path in-process.

## Alternatives Rejected

- **Sequential synchronous execution** — would multiply pacing delays; unacceptable UX for multi-source runs. Rejected.
- **std::thread per adapter** — possible but noisier than async Tokio, and reqwest's async client is idiomatic. Rejected.
