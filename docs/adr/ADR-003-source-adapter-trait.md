# ADR-003: Source Adapters as a Pluggable Trait

**Date:** 2026-03-21
**Status:** Accepted
**Spec reference:** FORMAL_SPEC.md §7, Decision 3

---

## Context

Competitor Spy uses multiple public data sources today (OSM Nominatim, OSM Overpass, Yelp, Google Places). New sources will be added in future versions without changing the domain or rendering layers.

## Decision

Define a `SourceAdapter` trait in `competitor_spy_adapters`:

```rust
#[async_trait]
pub trait SourceAdapter: Send + Sync {
    fn adapter_id(&self) -> &str;
    fn requires_credential(&self) -> bool;
    async fn collect(
        &self,
        query: &SearchQuery,
        location: Location,
        radius: Radius,
        credential: Option<&str>,
    ) -> SourceResult;
}
```

All adapters implement this trait. The `SourceRegistry` holds `Arc<dyn SourceAdapter>` and executes them uniformly.

## Rationale

A trait-based design means:
- Adding a new adapter = adding one new module in `competitor_spy_adapters`, implementing the trait, and registering it in `runner.rs`. No other code changes.
- Each adapter is independently testable with a mock HTTP server (wiremock in unit tests).
- The domain crate never imports an adapter directly — it only sees `SourceResult` (a domain type).

## Consequences

- Each adapter module is self-contained: HTTP client construction, URL building, response parsing, error mapping, and pacing are all internal.
- `requires_credential()` drives the prompt-on-first-run flow in the CLI runner.
- Acceptance tests inject mock adapter URLs via `AdapterUrls` struct — same trait implementations used in tests and production.

## Alternatives Rejected

- **Hard-coded adapter calls in runner** — tightly couples adapter logic to CLI; makes adding a new source a multi-file change. Rejected.
