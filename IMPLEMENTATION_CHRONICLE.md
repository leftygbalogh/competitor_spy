# Implementation Chronicle

## Entry CHR-SNK-001

- Task: T-001
- Requirement: FR-001, FR-006, NFR-004
- Decision: enforce startup minimum size check before any curses init.
- Why: guarantees required print+log+exit behavior even when rendering cannot start.
- Evidence: `startup_size_check` and `log_size_crash` implementation.

## Entry CHR-SNK-002

- Task: T-002
- Requirement: FR-001
- Decision: center head at `fixed_width // 2`, `fixed_height // 2`; pre-start loop waits for any key.
- Why: exact startup placement and wait behavior from brief.
- Evidence: pre-start state in `run_game`.

## Entry CHR-SNK-003

- Task: T-003
- Requirement: FR-002
- Decision: constant-tick movement with direction updates from arrow keys only.
- Why: player controls direction only; no speed control.
- Evidence: `KEY_TO_DIR`, movement loop, fixed `TICK_SECONDS`.

## Entry CHR-SNK-004

- Task: T-004
- Requirement: FR-003
- Decision: when apple eaten, insert old head behind new head: `[new_head, old_head] + rest`.
- Why: implements growth directly behind head exactly as specified.
- Evidence: apple-eat branch in game loop.

## Entry CHR-SNK-005

- Task: T-005
- Requirement: FR-004
- Decision: out-of-bounds and body-hit end game immediately.
- Why: strict wall/self collision loss semantics, no wraparound.
- Evidence: collision checks before snake update finalization.

## Entry CHR-SNK-006

- Task: T-006
- Requirement: FR-005
- Decision: win on full board after final apple consumption.
- Why: exact explicit win contract from brief.
- Evidence: `if len(snake) == fixed_width * fixed_height` after growth.

## Entry CHR-SNK-007

- Task: T-007
- Requirement: FR-007
- Decision: only write leaderboard when `score > previous_high`.
- Why: brief prohibits non-record writes and prompts.
- Evidence: conditional name prompt and write path in `main`.

## Entry CHR-SNK-008

- Task: T-008
- Requirement: runtime terminal support in Git Bash
- Decision: Bash launcher uses `winpty` on MSYS if available.
- Why: enables curses compatibility under Git Bash on Windows.
- Evidence: `run_snake.sh` conditional execution path.

## Stage 4 Approval

- Approved by: Team Lead Agent (delegated)
- Approval date: 2026-03-20
- Notes: Build complete with chronicle links for all tasks.

---

# Competitor Spy Implementation Chronicle

## Entry CHR-CSPY-000

- Task: T-000
- Date: 2026-03-21
- Requirement: CSPY-PLAN-001
- Spec ref: FORMAL_SPEC.md §Q3-ARCH-01 (layered architecture)

### Decision: 6-crate workspace with compiler-enforced layer isolation

All 6 roles (domain, adapters, output, credentials, telemetry, cli) are separate Cargo crates in a single workspace. The dependency graph is:

```
competitor_spy_domain          (no I/O, no async, no rendering deps)
  ↑
competitor_spy_adapters        (async HTTP; depends on domain)
competitor_spy_credentials     (sync file I/O; depends on nothing except age/serde)
competitor_spy_telemetry       (async OTel; depends on nothing domain-specific)
competitor_spy_output          (sync rendering; depends on domain)
  ↑
competitor_spy_cli             (entry point; depends on all 5 above)
```

Why separate crates: If domain accidentally imports tokio or reqwest, the compiler rejects it. No lint rule or code-review custom needed — the dep graph is the enforcement. Adapter crates cannot import output or credentials, enforced by the same mechanism.

### Decision: resolver = "2" and workspace-level dependency pinning

`[workspace.dependencies]` declares every shared version once. All crate `Cargo.toml` files use `{ workspace = true }` to inherit. This prevents version drift between crates and makes audits and upgrades a single-location change.

Key pins:
- tokio 1, features = full (gives all runtime primitives; trimmed in specific crates if needed)
- reqwest 0.12, no-default-features, features = [json, rustls-tls] (avoids openssl C dep on Windows)
- age 0.10 (passphrase-based encryption for credential store)
- opentelemetry 0.27 + tracing-opentelemetry 0.28 (compatible pair; OTel 0.x API stability)
- clap 4, features = [derive] (ergonomic CLI; derive macro means no manual arg builder)
- wiremock 0.6 (dev-only; HTTP mock server for adapter tests)

### Decision: CSPY_STATE_LOG env-var pattern for capture scripts

Capture scripts (`capture_session.sh` and `capture_session.ps1`) write all stdout + stderr to a session log file at path given by env var `CSPY_STATE_LOG`. If the var is not set, a default timestamped path is constructed. Output is simultaneously teed to the terminal. The binary is invoked with `--log-level trace` to ensure maximum diagnostic data. Exit code is propagated faithfully (PIPESTATUS on bash; `$process.ExitCode` on PS).

### Evidence

- `cargo build` output: 335+ packages downloaded, all 6 crates compiled, zero errors.
- Binary confirmed: `target\debug\competitor-spy.exe` (Test-Path = True).
- All 6 crates: `cargo build --quiet` returns no warnings relevant to stub structure.
- TASK_LIST.md: T-000 status = DONE, evidence-date = 2026-03-21.
## Entry CHR-CSPY-004  [RECONSTRUCTION-CRITICAL]

- Task: T-004
- Date: 2026-03-21
- Requirement: FORMAL_SPEC.md §3.5 (ScoringStrategy), §4.5 (scoring algorithms must be deterministic and chronicled)

### keyword_score algorithm (EXACT — used in ranking and reports)

```
query_tokens = whitespace_split(query.industry.to_lowercase())
if query_tokens.is_empty() → return 0.0
categories_text = profile.categories.value.as_deref().unwrap_or("").to_lowercase()
matched = |{ t in query_tokens : categories_text.contains(t) }|
score = matched / len(query_tokens)  ∈ [0.0, 1.0]
```

This is a substring-containment token overlap (not full-word boundary matching). Token "yoga" matches "yoga" or "yoga-studio" or "iyoga". Boundary-matching was considered but rejected: source tags are free-form and substring matching is more forgiving.

### visibility_score algorithm (EXACT — used in ranking and reports)

```
completeness = profile.completeness()  // non-Absent fields / 10

if profile.review_count_text.confidence == Absent:
    return completeness  // No review data; use completeness alone

review_count = parse_f64(profile.review_count_text.value).unwrap_or(0).max(0)
review_score = min(review_count / 200.0, 1.0)  // saturates at 200 reviews

visibility = min(0.5 * completeness + 0.5 * review_score, 1.0)
```

The 200-review saturation cap was chosen to normalise typical SMB review ranges (0–200) without artificially inflating large chains. This value is an implementation decision and may be revisited in v2.

### Known test vectors (for cross-check)

| Industry | Categories | keyword_score |
|---|---|---|
| "yoga studio" | "yoga studio pilates" | 1.0 |
| "yoga studio" | "yoga pilates" | 0.5 |
| "yoga studio" | "restaurant bar" | 0.0 |
| "yoga studio" | Absent | 0.0 |

| completeness | review_count | visibility_score |
|---|---|---|
| 0.0 | Absent | 0.0 |
| 0.9 | Absent | 0.9 |
| 1.0 | 200 | 1.0 |
| 0.1 | 100 | 0.30 |

### Evidence

- `cargo test -p competitor_spy_domain` — 51 passed, 0 failed.

## Entry CHR-CSPY-005  [RECONSTRUCTION-CRITICAL]

- Task: T-005
- Date: 2026-03-21
- Requirement: FORMAL_SPEC.md §3.5 (RankingEngine), §4.5 (ranking rules), §5.3 (tie-break table)

### Ranking algorithm (EXACT — determines report order)

Sort key (stable, applied in order):
1. `distance_km` ascending (smaller distance = higher rank)
2. `keyword_score` descending (higher relevance = higher rank on distance tie)
3. `profile.name.value.as_deref().unwrap_or("").to_lowercase()` ascending UTF-8 lexicographic (alphabetical on double tie)

After sort: `rank = index + 1` (1-indexed).

### DefaultRankingEngine scores before sorting

`DefaultRankingEngine` holds an injected `ScoringStrategy` (default: `DefaultScoringStrategy`). Scoring is applied to all Competitor objects before the sort. This makes the rank deterministic given fixed inputs and a fixed scorer.

### sort_by with partial_cmp fallback

`f64` is not `Ord`, so `partial_cmp().unwrap_or(Ordering::Equal)` is used for `distance_km` and `keyword_score`. NaN values are treated as equal to anything (sorts stably). NaN should never appear in practice (validated at construction/scoring).

### Spec example §4.5 verified

| Name | distance_km | keyword_score | expected rank |
|---|---|---|---|
| A | 2.1 | 0.70 | 1 |
| B | 4.5 | 0.85 | 2 |
| C | 4.5 | 0.60 | 3 |

B and C are at the same distance; B has higher keyword_score → ranked 2nd.

### Evidence

- `cargo test -p competitor_spy_domain` — 59 passed, 0 failed.
- Spec §4.5 example reproduced exactly in `rank_spec_example_three_competitors` test.
- Name tie-break case-insensitive test: "ALPHA YOGA" < "zebra yoga" alphabetically → correct.
- All known-vector tests pass with tolerance < 1e-9.## Entry CHR-CSPY-003

- Task: T-003
- Date: 2026-03-21
- Requirement: FORMAL_SPEC.md §3.2 (SearchRun, SourceResult), §3.4 (aggregate root), §4.1 (run lifecycle statechart), §4.3 (adapter failure semantics)

### Decision: SearchRun state transitions via named methods, not a generic transition()

Each state transition is a named method (`start_validating`, `start_geocoding`, `set_location`, `add_source_result`, `start_ranking`, `set_competitors`, `complete`, `complete_with_warning`, `fail`). Guards use `debug_assert_eq!` — they panic in dev/test if an invalid transition is attempted, but are elided in release for performance. This aligns with the "adapter failure does not abort run" invariant: `add_source_result()` is always valid in Collecting state regardless of the result's status.

### Decision: ReasonCode::AdapterConfigMissing added alongside spec codes

Spec defines HTTP_4XX, HTTP_5XX, TIMEOUT, PARSE_ERROR, ADAPTER_CONFIG_MISSING. All five are implemented. Display matches the spec strings exactly (used in report footer and audit log).

### Decision: FailureReason carries a human-readable message

FailureReason variants (ValidationError, GeocodingError, RenderError) carry a String message. This message is what gets written to stderr on failure. The RankedState is separate from the Done state so downstream auditing can distinguish clean runs from warned-runs without parsing output.

### Evidence

- `cargo test -p competitor_spy_domain` — 40 passed, 0 failed.
- Adapter-failure test: run with OSM Success + Yelp Failed(Timeout) reaches Ranking state (status = Ranking) with source_results.len() = 2; failed_source_results() = ["yelp"].
- Happy-path test: complete transition sequence Idle → Validating → Geocoding → Collecting → Ranking → Rendering → Done all assert correct status at each step.## Entry CHR-CSPY-002

- Task: T-002
- Date: 2026-03-21
- Requirement: FORMAL_SPEC.md §3.2 (Competitor, BusinessProfile), §3.3 (DataPoint, Confidence), §4.4 (normalization, deduplication), U-002 (40% completeness threshold = 4 of 10 fields)

### Decision: BusinessProfile has exactly 10 named fields

Fields: `name, address, phone, website, categories, opening_hours, email, description, rating_text, review_count_text`. This satisfies U-002 ("~10 defined fields" — threshold is 40% = 4 of 10 non-Absent).

Every field is always a `DataPoint`; absent data is `DataPoint::absent(field_name)` with `Confidence::Absent`. No Option<DataPoint> anywhere in the model. This eliminates null-checks downstream and matches the spec requirement "No null values in the domain model."

### Decision: Confidence ordering by declaration (Absent < Low < Medium < High)

`Confidence` derives `PartialOrd + Ord` from declaration order. This makes merge priority a one-liner: `if incoming.confidence > base.confidence { *base = incoming }`. Equal confidence keeps the existing (base) DataPoint — first source wins on ties.

### Decision: Haversine formula for 50m deduplication

Coordinates are WGS-84 lat/lon. Haversine with Earth radius 6,371,000m provides accurate distance at all latitudes. The 50m threshold is exact — no approximation. Using Euclidean deg-to-m conversion was rejected: error grows at high latitudes.

### Decision: O(n²) deduplication — acceptable for bounded n

The deduplication loop is O(n²) over competitors. The maximum practical n is bounded by what the 4 adapters return (hundreds, not thousands). An indexed approach (geohash buckets) adds complexity not justified for v1.

### Evidence

- `cargo test -p competitor_spy_domain` — 30 passed, 0 failed (includes 18 new profile tests + 12 from T-001).
- haversine test: 0.0004° lat ≈ 44m (< 50) confirmed; 0.001° lat ≈ 111m (> 50) confirmed.
- Deduplication merge test: two Iron Temple records at 52.0000/52.0001, phone Low vs High → result has phone "+31-high" (High confidence wins).
## Entry CHR-CSPY-001

- Task: T-001
- Date: 2026-03-21
- Requirement: FORMAL_SPEC.md §3.2 (SearchQuery, Location, Radius), §3.3 (value objects), §4.2 (validation preconditions)

### Decision: Radius as closed enum with TryFrom<u32>

`Radius` is an enum with five variants (`Km5 | Km10 | Km20 | Km25 | Km50`). Conversion from u32 is done via `TryFrom<u32>` which rejects any value not in {5, 10, 20, 25, 50}. The `km_value()` method returns the fixed numeric constant. Enum prevents any invalid radius value from ever existing in the type system after construction.

### Decision: Location validation at construction, not at use site

`Location::new(lat, lon)` validates lat in [-90.0, 90.0] and lon in [-180.0, 180.0] and returns a typed error differentiating which axis is out of range. Boundary values (±90.0, ±180.0) are accepted. Once a `Location` exists it is always valid — no downstream guard checks needed.

### Decision: SearchQuery trims whitespace to detect empty fields

`SearchQuery::new()` calls `.trim().is_empty()` on both `industry` and `location_input`. A string of only spaces is treated as empty. The raw (untrimmed) string is stored in the struct fields so that the original user input is preserved for audit logging.

### Evidence

- `cargo test -p competitor_spy_domain` — 12 passed, 0 failed.
- Tests cover: Radius valid/invalid/km_value, Location valid/boundary/lat-out/lon-out, SearchQuery valid/empty-industry/whitespace-industry/empty-location/whitespace-location.
- Canonical vector FORMAL_SPEC.md §9.7 confirmed: `SearchQuery::new("yoga studio", "Amsterdam, Netherlands", Radius::Km10)` returns Ok.
