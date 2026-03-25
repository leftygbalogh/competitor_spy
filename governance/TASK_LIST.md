
# Task List — Competitor Spy

## Planning Metadata

- Plan ID: CSPY-PLAN-001
- Source spec: `FORMAL_SPEC.md` (CSPY-SPEC-001 v1.0, approved 2026-03-21)
- Mode: Greenfield
- Date: 2026-03-21 UTC
- Approver: Team Lead Agent (full delegation for all stages by Lefty 2026-03-21; consult Oracle or Claire Voyant when in doubt)
- Status: APPROVED — Team Lead Agent, 2026-03-21

---

## Conventions

- Tasks are ordered for sequential implementation (each task's output is an input to the next).
- TDD discipline: failing test written before any implementation code.
- Each task has an evidence requirement — a verifiable artifact produced before the task is marked Done.
- Chronicle entry required for every task marked with [C].
- T-000 must complete before any other task starts (provides the workspace all other tasks build on).

---

## Backlog

### T-000: Project skeleton and capture scripts

- Source: FORMAL_SPEC.md §7.1 (layered architecture), §9.5 (CLI diagnostics scripts)
- Status: **DONE — 2026-03-21**
- Output:
  - `Cargo.toml` (workspace root, members declared)
  - `competitor_spy_domain/Cargo.toml` + `src/lib.rs`
  - `competitor_spy_adapters/Cargo.toml` + `src/lib.rs`
  - `competitor_spy_output/Cargo.toml` + `src/lib.rs`
  - `competitor_spy_credentials/Cargo.toml` + `src/lib.rs`
  - `competitor_spy_telemetry/Cargo.toml` + `src/lib.rs`
  - `competitor_spy_cli/Cargo.toml` + `src/main.rs`
  - `scripts/capture_session.sh` (Linux/Bash)
  - `scripts/capture_session.ps1` (Windows/PowerShell)
  - `docs/evidence/sessions/.gitkeep`
- Evidence: `cargo build` succeeded; `target\debug\competitor-spy.exe` present. Zero errors, zero warnings.
- Chronicle: See IMPLEMENTATION_CHRONICLE.md — T-000

---

### T-001: Domain — SearchQuery, Location, Radius (with validation)

- Source: FORMAL_SPEC.md §3.2, §3.3, §4.2, §9.1 (domain::query unit tests), §9.7 (canonical test vector)
- Status: DONE — 2026-03-21
- Dependencies: T-000
- Output:
  - `competitor_spy_domain/src/query.rs` — `SearchQuery`, `Location`, `Radius` types with validation
  - Unit tests covering: empty industry, invalid radius, empty location string, valid canonical input
- Evidence: `cargo test -p competitor_spy_domain` — 12 passed, 0 failed. Canonical input `{ industry: "yoga studio", location_input: "Amsterdam, Netherlands", radius: Km10 }` constructs without error.
- Chronicle: CHR-CSPY-001

---

### T-002: Domain — BusinessProfile, DataPoint, Confidence, Competitor

- Source: FORMAL_SPEC.md §3.2, §3.3, §4.4, §9.1 (domain::profile unit tests)
- Status: DONE — 2026-03-21
- Dependencies: T-001
- Output:
  - `competitor_spy_domain/src/profile.rs` — `BusinessProfile`, `DataPoint`, `Confidence`, `Competitor` types
  - Deduplication logic (within 50m + case-insensitive name match; merge priority: highest-confidence DataPoint per field)
  - Unit tests covering: normalization, deduplication, Absent DataPoint, merge priority
- Evidence: `cargo test -p competitor_spy_domain` — 30 passed, 0 failed. Deduplication test: two records within 50m with the same name produce one merged Competitor.
- Chronicle: CHR-CSPY-002

---

### T-003: Domain — SearchRun, SourceResult, RunStatus

- Source: FORMAL_SPEC.md §3.2, §3.4, §4.1 (run lifecycle statechart)
- Status: DONE — 2026-03-21
- Dependencies: T-002
- Output:
  - `competitor_spy_domain/src/run.rs` — `SearchRun`, `SourceResult`, `RunStatus`, `ReasonCode`, `FailureReason`, `AdapterResultStatus`, `RawRecord`
  - `SearchRun` is the aggregate root; transition guards match the statechart in §4.1
  - Unit tests covering each valid state transition and adapter failure flow
- Evidence: `cargo test -p competitor_spy_domain` — 40 passed, 0 failed. Adapter-failure test confirms run continues to Ranking after Failed SourceResult.
- Chronicle: CHR-CSPY-003

---

### T-004: Domain — ScoringStrategy trait and default implementation

- Source: FORMAL_SPEC.md §3.5, §4.5, §9.1 (domain::scoring unit tests), §9.7 (deterministic test vectors)
- Status: Not started
- Dependencies: T-003
- Output:
  - `competitor_spy_domain/src/scoring.rs` — `ScoringStrategy` trait + `DefaultScoringStrategy`
  - `keyword_score`: token overlap between competitor categories and query industry keywords, normalised to [0.0, 1.0]
  - `visibility_score`: composite of review count (if present) and profile completeness (fields-present / total-defined)
  - Unit tests at 0.0, 1.0, and midpoints with known inputs
- Status: DONE — 2026-03-21
- Evidence: `cargo test -p competitor_spy_domain` — 51 passed. Known-input tests reproduce scores exactly. See CHR-CSPY-004 for algorithm specification.
- Chronicle: CHR-CSPY-004

---

### T-005: Domain — RankingEngine trait and default implementation

- Source: FORMAL_SPEC.md §3.5, §4.5, §5.3 (tie-break table), §9.1 (domain::ranking unit tests), §9.7 (ranking vector)
- Status: Not started
- Dependencies: T-004
- Output:
  - `competitor_spy_domain/src/ranking.rs` — `RankingEngine` trait + `DefaultRankingEngine`
  - Ranking rules: distance asc → keyword_score desc → name asc (case-insensitive UTF-8) → stable
  - Unit tests: all tie-break combinations from §5.3; empty input; single competitor; canonical spec example from §4.5
- Status: DONE — 2026-03-21
- Evidence: `cargo test -p competitor_spy_domain` — 59 passed. §4.5 spec example [A(2.1km), B(4.5km/0.85), C(4.5km/0.60)] ranked correctly. Name tie-break case-insensitive verified.
- Chronicle: CHR-CSPY-005

---

### T-006: PacingPolicy

- Source: FORMAL_SPEC.md §4.8, §5.5, §9.1 (PacingPolicy unit tests), §9.7 (pacing determinism)
- Status: DONE — 2026-03-21 — commit pending (CHR-CSPY-006; seed=42 → [8,8,13]s)
- Dependencies: T-000
- Output:
  - `competitor_spy_adapters/src/pacing.rs` (or shared crate location TBD in chronicle) — `PacingPolicy`
  - Normal mode: uniform random delay in [5, 15] seconds
  - Seeded mode (`CSPY_PACING_SEED`): deterministic sequence; zero-delay allowed in tests
  - Unit test: `PacingPolicy::from_seed(42)` first three values documented in chronicle
- Evidence: `cargo test` (pacing tests) passes. Seeded policy produces a reproducible sequence across runs.
- Chronicle: [C] RNG choice; seed-to-delay mapping; first three values for seed=42. RECONSTRUCTION-CRITICAL.

---

### T-007: Credential store

- Source: FORMAL_SPEC.md §3.2, §4.7, §5.4, §6.4, §9.1, §9.2 (credential integration tests)
- Status: DONE -- 2026-03-21 -- commit pending (CHR-CSPY-007; 17 tests green)
- Dependencies: T-000
- Output:
  - `competitor_spy_credentials/src/lib.rs` — `CredentialStore`: store / retrieve / delete, `age`-encrypted, per adapter_id
  - File location: Linux `~/.config/competitor-spy/credentials`; Windows `%APPDATA%\competitor-spy\credentials`
  - In-memory zero-on-drop for decrypted values
  - Prompt logic: stderr, echo-disabled, skip-adapter on empty input
  - Unit tests: encrypt/decrypt round-trip; retrieve-absent = None; write-failure warns without crash
  - Integration test on real temp filesystem
- Evidence: `cargo test -p competitor_spy_credentials` passes. Decrypted value confirmed zero'd after drop (via debug assertion or MIRI in CI if feasible).
- Chronicle: [C] Key derivation and storage format choices; passphrase scheme.

---

### T-008: Telemetry initialisation and secret redaction filter

- Source: FORMAL_SPEC.md §6.3, §7.1, §9.2 (telemetry event coverage test)
- Status: DONE -- 2026-03-21 -- commit pending (CHR-CSPY-008; 15 tests green)
- Dependencies: T-003
- Output:
  - `competitor_spy_telemetry/src/lib.rs` — OTel initialisation; pre-emit secret redaction filter
  - Minimum event set: all events listed in §6.3
  - Integration test: emit a log event containing a simulated secret token; confirm token absent from captured output
- Evidence: `cargo test -p competitor_spy_telemetry` passes. Secret-redaction test passes.
- Chronicle: [C] OTel exporter config; redaction filter implementation; event schema.

---

### T-009: SourceAdapter trait and Nominatim adapter (geocoding)

- Source: FORMAL_SPEC.md §3.5, §4.2, §4.3, §7.1 (SourceAdapter trait), §5.1 (failure handling), §9.2 (adapter integration tests)
- Status: DONE
- Output: adapter.rs (Geocoder + SourceAdapter traits), nominatim.rs (NominatimGeocoder + NominatimAdapter), 15 new tests (19 total in crate)
- Evidence: `cargo test -p competitor_spy_adapters` 19 passed, 0 failed. Chronicle: CHR-CSPY-009.

---

### T-010: OSM/Overpass adapter

- Source: FORMAL_SPEC.md §4.3, §4.4, §5.1, §5.2, §9.2
- Status: DONE
- Output: osm_overpass.rs — OsmOverpassAdapter; QL query builder; sanitize; urlencoded; field-tag mapping; coordinate resolution (node direct / way center). 14 new tests (33 total in crate).
- Evidence: `cargo test -p competitor_spy_adapters` 33 passed, 0 failed. Chronicle: CHR-CSPY-010.

---

### T-011: Yelp Fusion adapter

- Source: FORMAL_SPEC.md §4.3, §4.4, §5.1, §5.4, §9.2; U-001 resolved 2026-03-21
- Status: DONE
- Output: yelp.rs — YelpAdapter; credential-absent -> AdapterConfigMissing; Bearer auth; field mapping (display_phone preferred, categories titles+aliases, radius clamp 40km). 12 new tests (45 total).
- Evidence: `cargo test -p competitor_spy_adapters` 45 passed, 0 failed. Chronicle: CHR-CSPY-011.

---

### T-012: Google Places adapter

- Source: FORMAL_SPEC.md §4.3, §4.4, §5.1, §5.4, §9.2; U-001 resolved 2026-03-21
- Status: DONE
- Output: google_places.rs — GooglePlacesAdapter; POST NearbySearch; X-Goog-Api-Key header; field mask; credential-absent guard; empty-body handling. 13 new tests (58 total).
- Evidence: `cargo test -p competitor_spy_adapters` 58 passed, 0 failed. Chronicle: CHR-CSPY-012.

---

### T-013: SourceRegistry and concurrent collection orchestration

- Source: FORMAL_SPEC.md §3.5, §4.1 (Collecting state), §4.3
- Status: DONE
- Dependencies: T-010, T-011, T-012
- Output:
  - `competitor_spy_adapters/src/registry.rs` — `SourceRegistry`: ordered adapter list; concurrent execution via Tokio `JoinSet`
  - Each adapter executes independently; failures isolated per §5.1
  - Integration test: two mock adapters, one succeeds and one times out → two `SourceResult` entries; run continues
- Evidence: `cargo test -p competitor_spy_adapters` passes (registry tests). Timeout isolation confirmed.
- Chronicle: [C] Concurrency model; adapter failure isolation design.

---

### T-014: Terminal report renderer

- Source: FORMAL_SPEC.md §4.6, §6.5, §9.2 (terminal snapshot test)
- Status: DONE
- Dependencies: T-005
- Output:
  - `competitor_spy_output/src/terminal.rs` — formats `SearchRun` to stdout
  - Table: Rank | Name | Distance | Address | Phone | Website | Keyword% | Visibility%
  - Absent fields rendered as `--`; footer lists failed sources with reason codes
  - Snapshot test against fixed `SearchRun`; output matches expected string
- Evidence: `cargo test -p competitor_spy_output` passes. Snapshot test passes with zero-diff against expected output.
- Chronicle: [C] Table layout choices; column widths; truncation rules for long values.

---

### T-015: PDF report renderer

- Source: FORMAL_SPEC.md §4.6, §6.2, §6.5, §9.2 (PDF parse test)
- Status: DONE
- Dependencies: T-014
- Output:
  - `competitor_spy_output/src/pdf.rs` — formats `SearchRun` to PDF via `printpdf`; A4 portrait
  - Filename: `competitor_spy_report_YYYYMMDD_HHMMSS_UTC.pdf`
  - Test: file produced, non-zero bytes, passes PDF parse check (lopdf or similar)
  - PDF write failure handled: warning on stderr; terminal output still produced; exit code 0
- Evidence: `cargo test -p competitor_spy_output` passes (PDF test). Produced PDF opens in a PDF viewer and matches terminal table structure.
- Chronicle: [C] Table layout approach; pagination strategy; font/style choices; failure handling design.

---

### T-016: CLI entry point and argument parsing

- Source: FORMAL_SPEC.md §6.1, §7.1 (CLI-to-API mapping), §4.1 (run lifecycle)
- Status: **DONE** — commit `1ae0cf9`
- Dependencies: T-008, T-013, T-014, T-015
- Output:
  - `competitor_spy_cli/src/main.rs` — argument parsing (clap); no domain logic; maps args to domain API
  - `--industry`, `--location`, `--radius` → `SearchQuery::new()` → `domain::run::execute()`
  - `--output-dir`, `--no-pdf`, `--log-level`, `--pacing-seed` wired per §6.1
  - Exit codes: 0 on success or source failures; 1 on fatal errors (validation, geocoding, render failure)
- Evidence: `cargo build --release` succeeds. `competitor-spy --help` outputs expected flag descriptions. All five acceptance scenarios (AS-001 through AS-005) pass against mock adapters.
- Chronicle: [C] CLI-to-API mapping decisions; clap configuration.

---

### T-017: End-to-end acceptance tests (AS-001 through AS-005)

- Source: FORMAL_SPEC.md §9.3 (acceptance test table)
- Status: **DONE** — commit `91deed3`
- Dependencies: T-016
- Output:
  - `tests/acceptance/` — 5 acceptance test scenarios driven by CLI binary against mock HTTP servers
  - AS-001: valid input, mock adapters → both outputs, exit 0
  - AS-002: one adapter OK, one timeout → both outputs, failed source in footer, exit 0
  - AS-003: invalid radius → stderr message, no report, exit 1
  - AS-004: geocoding no results → stderr message, no report, exit 1
  - AS-005: all adapters fail → both reports with zero competitors, warning footer, exit 0
- Evidence: All five acceptance tests pass. Terminal output artifacts and PDF files from AS-001 stored in `docs/evidence/sessions/`.
- Chronicle: N/A (acceptance tests trace to spec directly).

---

### T-018: Live end-to-end run (prototype handback)

- Source: PROJECT_BRIEF.md §7 (prototype handback trigger: first end-to-end run with both outputs)
- Status: **DONE** — live run 2026-03-22 UTC, exit 0
- Dependencies: T-017
- Output:
  - One successful live run against OSM/Overpass with a real location and industry query
  - Terminal output displayed on screen (captured via `capture_session.sh` / `capture_session.ps1`)
  - PDF artifact saved and verified to open correctly
  - Session log and state log stored in `docs/evidence/sessions/`
- Evidence: Session log + PDF artifact committed to `docs/evidence/`. Run completes with exit code 0.
- Chronicle: [C] Live run command, query parameters used, any unexpected behavior observed.

---

## Done

_(none yet)_

6. T-005: Implement error handling and edge cases
	- Source: [Reference requirements or spec sections]
	- Status: [Not started/In progress/Done]
	- Evidence: [Describe expected behavior or output]
	- Chronicle: [Link to implementation chronicle entry]

7. T-006: Implement success/failure conditions
	- Source: [Reference requirements or spec sections]
	- Status: [Not started/In progress/Done]
	- Evidence: [Describe expected behavior or output]
	- Chronicle: [Link to implementation chronicle entry]

8. T-007: Implement policy or business logic
	- Source: [Reference requirements or spec sections]
	- Status: [Not started/In progress/Done]
	- Evidence: [Describe expected behavior or output]
	- Chronicle: [Link to implementation chronicle entry]

9. T-008: Platform or environment compatibility
	- Source: [Reference requirements or spec sections]
	- Status: [Not started/In progress/Done]
	- Evidence: [Describe expected behavior or output]
	- Chronicle: [Link to implementation chronicle entry]

## Stage 3 Approval

- Approved by: Team Lead Agent (full delegation per PROJECT_BRIEF.md §8.2)
- Approval date: 2026-03-22
- Notes: V1 tasks T-000 through T-018 complete. V2 tasks T-019 through T-024 planned and authorized for build. TDD discipline applies throughout; no implementation before the failing test.

---

# V2 Task List — Competitor Spy v2.0

## V2 Planning Metadata

- Plan ID: CSPY-PLAN-002
- Source spec: `FORMAL_SPEC.md` CSPY-SPEC-002 v2.0, approved 2026-03-22
- Mode: Brownfield extension
- Date: 2026-03-22 UTC
- Approver: Team Lead Agent (full delegation)
- Status: APPROVED — Team Lead Agent, 2026-03-22

---

## V2 Conventions

- All V1 tasks remain done. V2 tasks are numbered T-019 onwards.
- TDD: failing test written before implementation code, per V1 discipline.
- Per V1 feedback lesson: adapter unit tests must assert on outgoing request headers/body, not just response parsing.
- Per V1 feedback lesson: before any live E2E handover, rebuild release binary from HEAD and confirm binary timestamp post-dates last commit.

---

### T-019: Extend BusinessProfile and PlaceReview domain types

- Source: FORMAL_SPEC.md V2 §V2.3
- Status: Not started
- Dependencies: T-002 (BusinessProfile), T-003 (SearchRun)
- Output:
  - `competitor_spy_domain/src/profile.rs` — add `opening_hours: DataPoint<Vec<String>>`, `price_level: DataPoint<Option<u8>>`, `editorial_summary: DataPoint<String>`, `reviews: DataPoint<Vec<PlaceReview>>`, `rating: DataPoint<f64>`, `user_rating_count: DataPoint<u32>`, `place_types: DataPoint<Vec<String>>`
  - New value type `PlaceReview { text: String, rating: u8, relative_time: String }` in `profile.rs`
  - Unit tests: construct profile with all new fields present; construct with all absent; verify Absent DataPoints
- Evidence: `cargo test -p competitor_spy_domain` — all prior tests still pass; new field tests pass
- Chronicle: [C]

---

### T-020: Expand Google Places adapter — field mask and response parsing

- Source: FORMAL_SPEC.md V2 §V2.4
- Status: Not started
- Dependencies: T-019, T-012 (existing google_places adapter)
- Output:
  - `competitor_spy_adapters/src/google_places.rs`:
    - `X-Goog-FieldMask` header extended with `regularOpeningHours.weekdayDescriptions`, `priceLevel`, `editorialSummary`, `reviews`
    - `GooglePlace` struct gains `regular_opening_hours`, `price_level`, `editorial_summary`, `reviews` fields (all `Option<_>`)
    - `place_to_record()` maps new fields into `BusinessProfile` as DataPoints
    - Price level string enum mapped to `u8` (1–4) per §V2.4.3
  - Unit tests (per V1 feedback — header and body assertions required):
    - `google_places_request_contains_expanded_field_mask` — assert `X-Goog-FieldMask` header value in outgoing request contains all new field paths
    - `google_places_parses_opening_hours`
    - `google_places_parses_price_level_string_to_u8`
    - `google_places_parses_reviews_up_to_five`
    - `google_places_absent_new_fields_yield_absent_datapoints`
- Evidence: `cargo test -p competitor_spy_adapters` — all prior + all new tests pass
- Chronicle: [C]

---

### T-021: Add `--detail` CLI flag

- Source: FORMAL_SPEC.md V2 §V2.5
- Status: Not started
- Dependencies: T-016 (CLI wiring)
- Output:
  - `competitor_spy_cli/src/main.rs` — `--detail` boolean flag added to `Args` struct via clap derive
  - Flag threaded through to `RunConfig` or equivalent and passed to both renderers
  - Unit test: `detail_flag_defaults_to_false`; `detail_flag_parsed_true_when_present`
- Evidence: `cargo test -p competitor_spy_cli` — flag tests pass; `competitor-spy --help` shows `--detail`
- Chronicle: [C]

---

### T-022: Terminal renderer — per-competitor detail panel

- Source: FORMAL_SPEC.md V2 §V2.6.1
- Status: Not started
- Dependencies: T-019, T-021, T-014 (terminal renderer)
- Output:
  - `competitor_spy_output/src/terminal.rs` — when `detail=true`, render indented detail panel below each ranked row; absent fields omitted from panel
  - Unit tests:
    - `terminal_detail_panel_renders_opening_hours`
    - `terminal_detail_panel_renders_price_level_symbol`
    - `terminal_detail_panel_renders_reviews`
    - `terminal_detail_panel_omits_absent_fields`
    - `terminal_no_detail_flag_output_unchanged_from_v1`
- Evidence: `cargo test -p competitor_spy_output` — all tests pass; snapshot of detail panel output matches spec §V2.6.1 format
- Chronicle: [C]

---

### T-023: PDF renderer — extended competitor section

- Source: FORMAL_SPEC.md V2 §V2.6.2
- Status: Not started
- Dependencies: T-019, T-021, T-015 (PDF renderer)
- Output:
  - `competitor_spy_output/src/pdf.rs` — when `detail=true`, each competitor block gains opening hours, price level, description, rating, reviews, type tags; absent fields omitted
  - Unit tests:
    - `pdf_detail_section_renders_when_flag_set`
    - `pdf_detail_section_absent_when_flag_not_set`
    - `pdf_absent_fields_omitted_from_detail_section`
- Evidence: `cargo test -p competitor_spy_output` — all tests pass; manually verified generated PDF contains detail section in at least one competitor block (wiremock fixture)
- Chronicle: [C]

---

### T-024: V2 acceptance test and live E2E

- Source: FORMAL_SPEC.md V2 §V2.7.3 and §V2.7.4
- Status: Not started
- Dependencies: T-019 through T-023
- Output:
  - `competitor_spy_cli/tests/acceptance.rs` — AS-006: `--detail` flag with wiremock fixture containing opening hours and one review; assert detail panel appears in stdout capture; assert PDF written
  - Live E2E: rebuild release binary from HEAD, confirm timestamp, run `competitor-spy --industry pilates --location "Neulengbach, Austria" --radius 50 --detail`; confirm at least one competitor shows non-empty opening hours or review in terminal output (`outcome="success" record_count>0` for google_places)
- Evidence: `cargo test` — 200+ tests pass including AS-006; live run log showing detail panel with real data
- Chronicle: [C]

---

## V2 Stage 3 Approval

- Approved by: Team Lead Agent (delegated)
- Approval date: 2026-03-22
- Notes: Six tasks T-019 through T-024 cover domain extension, adapter expansion, CLI flag, two renderers, and acceptance + live E2E. No code work begins until this approval is recorded. V1 output contract preserved when `--detail` is absent.

---

## V3 Task Backlog (Website Enrichment)

Source spec: `FORMAL_SPEC.md` §13 (CSPY-SPEC-003, approved 2026-03-24)
Plan ID: CSPY-PLAN-003
Approver: Team Lead Agent (delegated by Lefty 2026-03-24)
Status: APPROVED — Team Lead Agent, 2026-03-24

Convention: Tasks are ordered for sequential implementation. All V1/V2 tasks remain done and unmodified. V3 tasks are numbered T-025 onwards.

### T-025: Domain — WebEnrichment entity, FetchStatus, EnrichmentErrorCode, coverage metric

- Source: FORMAL_SPEC.md §13.1 (entities), §13.2.9 (FR-V3-008), §13.2.10 (FR-V3-009)
- Status: NOT STARTED
- Dependencies: T-001 (domain crate), T-002 (BusinessProfile)
- Output:
  - `competitor_spy_domain/src/enrichment.rs` — `WebEnrichment`, `FetchStatus`, `EnrichmentErrorCode` types
  - `competitor_spy_domain/src/run.rs` (extended) — `SearchRun` gains `enrichments: Vec<WebEnrichment>`, `enrichment_coverage: f64`
  - `enrichment_coverage()` function: `count(≥1 extracted field) / total`, returns `f64`
  - Unit tests: coverage = 0 when empty; coverage = 1.0 when all enriched; coverage = 0.5 for mixed; all fields None when FetchStatus::Failed
- Evidence: `cargo test -p competitor_spy_domain` — all existing plus new enrichment tests pass
- Chronicle: [C]

---

### T-026: Extractor — Pricing

- Source: FORMAL_SPEC.md §13.2.4 (FR-V3-003)
- Status: NOT STARTED
- Dependencies: T-025
- Output:
  - `competitor_spy_adapters/src/extractors/pricing.rs` — pure function `extract_pricing(html: &str) -> Option<String>`
  - Test fixture: `tests/fixtures/enrichment/fixture_pricing_table_de.html`
  - Test fixture: `tests/fixtures/enrichment/fixture_no_content.html`
  - Unit tests: pricing table with `€` → Some(text); list items with price notation → Some(text); no price elements → None; malformed HTML → None
- Evidence: `cargo test -p competitor_spy_adapters -- extractors::pricing` — all pass including nil case
- Chronicle: [C]

---

### T-027: Extractor — Lesson Types

- Source: FORMAL_SPEC.md §13.2.5 (FR-V3-004)
- Status: NOT STARTED
- Dependencies: T-025
- Output:
  - `competitor_spy_adapters/src/extractors/lesson_types.rs` — pure function `extract_lesson_types(html: &str) -> Option<Vec<String>>`
  - Unit tests: nav with vocabulary tokens → correct deduplicated vec; no vocabulary → None; duplicates suppressed; case-insensitive match
- Evidence: `cargo test -p competitor_spy_adapters -- extractors::lesson_types` — all pass
- Chronicle: [C]

---

### T-028: Extractor — Schedule

- Source: FORMAL_SPEC.md §13.2.6 (FR-V3-005)
- Status: NOT STARTED
- Dependencies: T-025
- Output:
  - `competitor_spy_adapters/src/extractors/schedule.rs` — pure function `extract_schedule(html: &str) -> Option<String>`
  - Test fixture: `tests/fixtures/enrichment/fixture_schedule_table_de.html`
  - Unit tests: German day-header table → Some(text); `.stundenplan` div → Some(text); time pattern + day combination → Some(text); no schedule → None
- Evidence: `cargo test -p competitor_spy_adapters -- extractors::schedule` — all pass
- Chronicle: [C]

---

### T-029: Extractor — Testimonials

- Source: FORMAL_SPEC.md §13.2.7 (FR-V3-006)
- Status: NOT STARTED
- Dependencies: T-025
- Output:
  - `competitor_spy_adapters/src/extractors/testimonials.rs` — pure function `extract_testimonials(html: &str) -> Option<Vec<String>>`
  - Test fixture: `tests/fixtures/enrichment/fixture_testimonials_blockquote.html`
  - Unit tests: `<blockquote>` elements → vec; `.testimonial` class → vec; `„...„` quoted paragraph → vec; > 10 items → capped at 10; all empty → None; items > 500 chars → truncated
- Evidence: `cargo test -p competitor_spy_adapters -- extractors::testimonials` — all pass
- Chronicle: [C]

---

### T-030: Extractor — Class Descriptions

- Source: FORMAL_SPEC.md §13.2.8 (FR-V3-007)
- Status: NOT STARTED
- Dependencies: T-025, T-027 (shares lesson-type vocabulary)
- Output:
  - `competitor_spy_adapters/src/extractors/class_descriptions.rs` — pure function `extract_class_descriptions(html: &str) -> Option<Vec<String>>`
  - Unit tests: heading with vocabulary + sibling `<p>` → vec; `.kurs` section with `<p>` children → vec; deep heading + para → vec; no qualifying context → None; items > 800 chars → truncated; > 8 items → capped at 8
- Evidence: `cargo test -p competitor_spy_adapters -- extractors::class_descriptions` — all pass
- Chronicle: [C]

---

### T-031: WebEnricher adapter — fetch, pace, orchestrate extractors

- Source: FORMAL_SPEC.md §13.2.3 (FR-V3-002), §13.3.1 (failure handling), §13.3.2 (pacing), §13.4.3 (audit log)
- Status: NOT STARTED
- Dependencies: T-025, T-026, T-027, T-028, T-029, T-030
- Output:
  - `competitor_spy_adapters/src/web_enricher.rs` — `WebEnricher` struct with method `enrich(competitors: &[Competitor], pacing: &PacingPolicy) -> Vec<WebEnrichment>`
  - Fetches root page per competitor URL (reqwest GET, 15s timeout, up to 3 redirects, User-Agent header)
  - Calls all 5 extractors on successful fetch; records per-field results
  - On fetch failure: records `FetchStatus::Failed(reason_code)`; all fields None
  - Applies pacing delay after each fetch (including NoUrl competitors: no delay)
  - Emits audit log events: `enrichment_start`, `enrichment_fetch_attempt`, `enrichment_fetch_result`, `enrichment_complete`
  - `Cargo.toml` dependency addition: `scraper = "0.22"` (or current stable) to `competitor_spy_adapters`
  - Integration tests: mock HTTP server (wiremock) returning pricing fixture → correct WebEnrichment; 404 → Failed; timeout → Failed; NoUrl competitor → no HTTP request, Failed(NoUrl)
  - Deterministic pacing test: with `CSPY_PACING_SEED`, delay sequence reproducible
- Evidence: `cargo test -p competitor_spy_adapters -- web_enricher` — all integration tests pass
- Chronicle: [C]

---

### T-032: Run lifecycle extension — Enriching state

- Source: FORMAL_SPEC.md §13.2.1 (run lifecycle extension)
- Status: NOT STARTED
- Dependencies: T-031
- Output:
  - `competitor_spy_domain/src/run.rs` — `RunStatus` gains `Enriching` variant; transition logic: `Collecting -> Enriching -> Ranking`
  - `domain::run::execute()` calls `WebEnricher::enrich()` after collection completes, before ranking
  - Unit tests: run with mock enricher; assert `enrichments` populated; assert `enrichment_coverage` computed; assert ranking still runs after enrichment
- Evidence: `cargo test -p competitor_spy_domain -- run` — all pass including enrichment lifecycle tests
- Chronicle: [C]

---

### T-033: CLI — `--no-enrichment`, `--allow-insecure-tls`, `--enrichment-timeout` flags

- Source: FORMAL_SPEC.md §13.4.1
- Status: NOT STARTED
- Dependencies: T-032
- Output:
  - `competitor_spy_cli/src/main.rs` — three new optional flags parsed and passed to run configuration
  - `--no-enrichment`: skips Enriching state entirely; `SearchRun.enrichments = []`
  - `--enrichment-timeout <secs>`: validates range [5, 60]; default 15
  - `--allow-insecure-tls`: passes flag through to `WebEnricher`
  - Unit tests: flag parsing with valid and out-of-range timeout values
- Evidence: `cargo test -p competitor_spy_cli` — all existing + new CLI tests pass; `competitor-spy --help` shows all three flags
- Chronicle: [C]

---

### T-034: Terminal renderer — Website Enrichment section

- Source: FORMAL_SPEC.md §13.4.2 (report contract — terminal format)
- Status: NOT STARTED
- Dependencies: T-025, T-014 (existing terminal renderer)
- Output:
  - `competitor_spy_output/src/terminal.rs` (extended) — renders `Website Enrichment` section per competitor after the existing competitor table
  - Format: titled block per studio; each field on its own line; `[unavailable]` for None; testimonials and class_descriptions as numbered indented list
  - Footer: coverage line; below-threshold warning if `enrichment_coverage < 0.60`
  - Snapshot test: fixed `SearchRun` with known enrichment data → output matches expected string; verify `[unavailable]` appears for None fields
- Evidence: `cargo test -p competitor_spy_output -- terminal` — all pass including enrichment snapshot
- Chronicle: [C]

---

### T-035: PDF renderer — Website Enrichment subsections

- Source: FORMAL_SPEC.md §13.4.2 (report contract — PDF format)
- Status: NOT STARTED
- Dependencies: T-025, T-015 (existing PDF renderer)
- Output:
  - `competitor_spy_output/src/pdf.rs` (extended) — new subsection per studio for enrichment data
  - Nil fields rendered in italic as `[unavailable]`; testimonials and class_descriptions as numbered paragraphs
  - Footer additions: coverage line; below-threshold warning if applicable
  - Test: PDF file produced, non-zero bytes, passes PDF parse check; coverage warning text appears in byte content when threshold not met
- Evidence: `cargo test -p competitor_spy_output -- pdf` — all pass including enrichment PDF test
- Chronicle: [C]

---

### T-036: Acceptance and live E2E tests (V3)

- Source: FORMAL_SPEC.md §13.6.3 (AS-V3-001, AS-V3-002, AS-V3-003)
- Status: NOT STARTED
- Dependencies: T-025 through T-035
- Output:
  - `competitor_spy_cli/tests/acceptance.rs` — AS-V3-001 (mock run: enrichment section present; ≥1 non-nil field)
  - `competitor_spy_cli/tests/acceptance.rs` — AS-V3-002 (mock run: 200 OK but no extractable content; all `[unavailable]`; exit 0)
  - `competitor_spy_cli/tests/acceptance.rs` — AS-V3-003 (mock run: all URLs return 503; coverage 0%; below-threshold warning in output; exit 0)
  - Live E2E: rebuild release binary from current HEAD; verify binary timestamp post-dates last commit; run `competitor-spy --industry pilates --location "Neulengbach, Austria" --radius 50`; confirm terminal shows enrichment section for ≥1 studio; PDF produced and non-empty; session log captured via `scripts/capture_session.ps1 v3-e2e`
- Evidence: `cargo test` — all tests (200+ existing + new V3 acceptance) pass; live E2E session log at `docs/evidence/sessions/session_<timestamp>_v3-e2e.log`; PDF artifact at `reports/`
- Chronicle: [C]

---

## V3 Stage 3 Approval

- Approved by: Team Lead Agent (delegated by Lefty 2026-03-24)
- Approval date: 2026-03-24
- Notes: Twelve tasks T-025 through T-036 cover domain extension (T-025), five pure extractors (T-026–T-030), web enricher adapter (T-031), lifecycle extension (T-032), CLI flags (T-033), two renderer extensions (T-034–T-035), and acceptance + live E2E (T-036). TDD discipline applies throughout. No implementation code before the corresponding failing test. V1/V2 behavior preserved throughout; `--no-enrichment` flag restores pre-V3 output contract.

