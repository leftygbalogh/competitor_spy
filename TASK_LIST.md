
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
- Status: Not started
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
- Status: Not started
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
- Status: Not started
- Dependencies: T-006, T-007, T-008
- Output:
  - `competitor_spy_adapters/src/lib.rs` — `SourceAdapter` trait (defined here per §7.1)
  - `competitor_spy_adapters/src/nominatim.rs` — geocoding only; no credential required
  - Integration tests via mock HTTP server (wiremock): success, 4xx, 5xx, timeout, malformed JSON → correct `SourceResult`
- Evidence: `cargo test -p competitor_spy_adapters` passes (Nominatim tests). Geocoding of "Amsterdam, Netherlands" against live Nominatim returns a result within 0.01 degrees of (52.3676, 4.9041) — captured as live evidence artifact.
- Chronicle: [C] Response schema interpretation; confidence level assignment for geocoding results.

---

### T-010: OSM/Overpass adapter

- Source: FORMAL_SPEC.md §4.3, §4.4, §5.1, §5.2, §9.2
- Status: Not started
- Dependencies: T-009
- Output:
  - `competitor_spy_adapters/src/osm_overpass.rs` — no credential required
  - Fields extracted: name, address, phone, website, hours, amenity/category tags
  - Integration tests: success, timeout, parse error → correct `SourceResult`
  - At least one known query against live Overpass API captured as evidence artifact before prototype handback
- Evidence: `cargo test -p competitor_spy_adapters` passes (Overpass tests). Live evidence artifact present.
- Chronicle: [C] Overpass QL query template; field extraction; confidence assignment per field.

---

### T-011: Yelp Fusion adapter

- Source: FORMAL_SPEC.md §4.3, §4.4, §5.1, §5.4, §9.2; U-001 resolved 2026-03-21
- Status: Not started
- Dependencies: T-009
- Output:
  - `competitor_spy_adapters/src/yelp.rs` — credential required; prompts via `CredentialStore`
  - Fields extracted: name, address, phone, website, categories, rating, review_count
  - Integration tests: success, 4xx (invalid key), timeout, parse error → correct `SourceResult`
- Evidence: `cargo test -p competitor_spy_adapters` passes (Yelp tests). Credential-absent path produces `ADAPTER_CONFIG_MISSING`.
- Chronicle: [C] Yelp Fusion API endpoint; response schema; field-to-DataPoint mapping; confidence assignment.

---

### T-012: Google Places adapter

- Source: FORMAL_SPEC.md §4.3, §4.4, §5.1, §5.4, §9.2; U-001 resolved 2026-03-21
- Status: Not started
- Dependencies: T-009
- Output:
  - `competitor_spy_adapters/src/google_places.rs` — credential required; prompts via `CredentialStore`
  - Fields extracted: name, address, phone, website, types/categories, rating, user_ratings_total
  - Integration tests: success, 4xx (invalid key), timeout, parse error → correct `SourceResult`
- Evidence: `cargo test -p competitor_spy_adapters` passes (Google tests). Credential-absent path produces `ADAPTER_CONFIG_MISSING`.
- Chronicle: [C] Google Places API endpoint; response schema; field-to-DataPoint mapping; confidence assignment.

---

### T-013: SourceRegistry and concurrent collection orchestration

- Source: FORMAL_SPEC.md §3.5, §4.1 (Collecting state), §4.3
- Status: Not started
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
- Status: Not started
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
- Status: Not started
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
- Status: Not started
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
- Status: Not started
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
- Status: Not started
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

- Approved by: [Team Lead Agent or designated approver]
- Approval date: [YYYY-MM-DD UTC]
- Notes: Planning complete; tasks authorized for build.
