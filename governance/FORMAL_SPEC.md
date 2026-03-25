
# Formal Specification — Competitor Spy

Layer metadata: Layer 2 of the three-layer documentation stack (Commander's Intent → Behavioral Specification → Implementation Chronicle).
Source brief: `PROJECT_BRIEF.md` (approved 2026-03-21 by Lefty).
Downstream: `IMPLEMENTATION_CHRONICLE.md` (Layer 3).

---

## 1. Specification Metadata

- Spec ID: CSPY-SPEC-001
- Version: 1.0
- Project mode: Greenfield
- Declared implementation language(s): Rust (primary), Bash (Linux launcher), PowerShell (Windows launcher)
- Language-specific constraints: Rust workspace with one crate per layer; async via Tokio; HTTP via reqwest; PDF via printpdf; credentials via age encryption; telemetry via opentelemetry crates.
- Source brief: `PROJECT_BRIEF.md`
- Approval authority source: Team Lead Agent (delegated; brief §8.2)
- Status: APPROVED — 2026-03-21
- Author: Formal Specification Lead (Team Lead Agent)
- Reviewers: Domain Discovery Lead, Security Specialist
- Active Q3 modules: Data Quality, Compliance and Auditability, Interactive CLI Diagnostics, Security and Production-Readiness Loop, Q3-ARCH-01 Layered Architecture

### 1.1 Mode Constraints

- Greenfield constraints: No legacy behavior to preserve. Architecture decisions are forward-looking. Primary evolution constraint: clean domain API that can serve a future web consumer without domain code changes.
- Brownfield parity scope: N/A.

### 1.2 Behavioral Specification Approach

- Statechart coverage: Run lifecycle (section 4.1), per-source adapter lifecycle (section 4.3), credential store lifecycle (section 4.7)
- Contract coverage: `run_search`, `collect`, `normalize`, `rank`, `render_terminal`, `render_pdf`, `store_credential`, `retrieve_credential`
- Decision table coverage: source failure handling (section 5.1), missing field handling (section 5.2), ranking tie-break (section 5.3), credential prompt policy (section 5.4), pacing policy (section 5.5)
- Escalation to mathematical verification: No — not safety-critical; statecharts and pre/post conditions are sufficient.

---

## 2. Scope

**In scope:**
- CLI interface accepting industry, location, radius
- Geocoding of user-supplied location to coordinates
- Distance-bounded competitor search via pluggable public source adapters
- Business profile data collection: name, website, address, phone, email, hours, service types, facility details, pricing (all best-effort)
- Keyword-relevance and search-visibility scoring per competitor
- Terminal (stdout) report and PDF report from a single run
- Source failure isolation and graceful degradation
- Encrypted credential store (local, user-controlled)
- Intentional pacing (5-15s randomised delay per source request)
- Structured audit log (OpenTelemetry, secrets redacted)
- Linux x86_64 and Windows 11 x86_64 runtime support

**Out of scope (v1):**
- Private or authenticated data scraping
- Paid/paywalled data sources
- Background monitoring or scheduling
- Multi-user or team accounts
- macOS, GUI, ARM
- Legal advice or completeness guarantees
- Web interface (architecture is web-ready; web interface is post-v1)
- LLM summarization (infrastructure present, off by default, not tested in v1)

---

## 3. Domain Model

### 3.1 Ubiquitous Language

| Term | Definition |
|---|---|
| Search Query | The user's intent: an industry/category, a location string, and a radius. One per run. |
| Location | Resolved coordinates (latitude, longitude) derived from the user's location string via geocoding. |
| Radius | One of the fixed values: 5, 10, 20, 25, or 50 km. |
| Competitor | A business entity discovered within the radius that matches the industry. |
| Business Profile | The collected data fields for one Competitor. Fields are individually present or absent. |
| Data Point | One field-value within a Business Profile, tagged with its source and a confidence level. |
| Source Adapter | A pluggable module responsible for querying one upstream data provider. |
| Source Result | Raw outcome from one Source Adapter for one request. May be empty or failed. |
| Run | One complete execution: one query, all adapter results, one ranked result set, two rendered reports. |
| Ranking | Sorting competitors by distance (primary), keyword-relevance (secondary), name (tertiary). |
| Report | Either the terminal output or the PDF file produced from a Run. Both derived from the same data. |
| Keyword-Relevance Score | Numeric score (0.0-1.0) reflecting how closely a competitor's categories match the query industry. |
| Search-Visibility Score | Numeric score (0.0-1.0) reflecting estimated online search presence. |
| Credential | User-supplied API key or auth token for a source requiring account registration. Stored encrypted. |
| Audit Log | Structured, redacted log of all operations in a run, emitted via OpenTelemetry. |

### 3.2 Core Entities

- `SearchQuery` — immutable; created once per run; fields: industry (String), location_input (String), radius (Radius)
- `Competitor` — mutable during collection; fields: id (Uuid), profile (BusinessProfile), distance_km (f64), keyword_score (f64), visibility_score (f64), rank (u32)
- `SearchRun` — aggregate root; fields: id (Uuid), query (SearchQuery), resolved_location (Location), competitors (Vec<Competitor>), source_results (Vec<SourceResult>), started_at (UTC DateTime), completed_at (Option<UTC DateTime>), status (RunStatus)
- `SourceResult` — per-adapter outcome; fields: adapter_id (String), status (Success | PartialSuccess | Failed(ReasonCode)), records (Vec<RawRecord>), retrieved_at (UTC DateTime)
- `Credential` — fields: adapter_id (String), encrypted_value (Vec<u8>), created_at (UTC DateTime), last_used_at (Option<UTC DateTime>)

### 3.3 Value Objects

- `Location` — (latitude: f64, longitude: f64); validated: lat in [-90.0, 90.0], lon in [-180.0, 180.0]
- `Radius` — enum: Km5 | Km10 | Km20 | Km25 | Km50; numeric value fixed per variant
- `DataPoint` — (field_name: &str, value: Option<String>, source_id: String, confidence: Confidence)
- `Confidence` — enum: High | Medium | Low | Absent

### 3.4 Aggregates

- `SearchRun` is the sole aggregate root. All competitors and source results are owned by the run. Neither exists outside a run.

### 3.5 Domain Services

- `Geocoder` — resolves a location string to a `Location`; returns error if unresolvable
- `SourceRegistry` — maintains the ordered list of active `SourceAdapter` instances for a run
- `RankingEngine` — takes `Vec<Competitor>` and `SearchQuery`; returns sorted `Vec<Competitor>`
- `ReportRenderer` — produces both terminal and PDF outputs from a finalised `SearchRun`
- `CredentialStore` — encrypted key-value store scoped to adapter IDs
- `PacingPolicy` — enforces per-request jitter delay; injectable seed for deterministic testing

---

## 4. Functional Behavior

### 4.1 Run Lifecycle — Statechart

States: Idle -> Validating -> Geocoding -> Collecting -> Ranking -> Rendering -> Done | Failed

```
Idle
  --[user invokes CLI with args]--> Validating

Validating
  --[args valid]--> Geocoding
  --[args invalid]--> Failed(validation_error)      // exit code 1, message on stderr

Geocoding
  --[location resolved]--> Collecting
  --[geocoding fails]--> Failed(geocoding_error)    // exit code 1, message on stderr

Collecting
  --[all adapters complete (success or failed)]--> Ranking
  // Individual adapter failures do NOT abort the run.
  // They produce SourceResult(status=Failed) and the run proceeds.

Ranking
  --[sorted result set produced]--> Rendering

Rendering
  --[both outputs written]--> Done                  // exit code 0
  --[PDF write fails]--> Done(with_warning)         // terminal output still produced; PDF failure = warning only
  --[terminal render fails]--> Failed(render_error) // exit code 1

Done:   exit code 0
Failed: exit code 1, structured error on stderr
```

### 4.2 FR-001: Query Input and Geocoding

- **Example:** User runs `competitor-spy --industry "yoga studio" --location "Amsterdam, Netherlands" --radius 10`. The tool resolves "Amsterdam, Netherlands" to (52.3676, 4.9041) and searches within 10 km.
- Preconditions: CLI receives non-empty industry, non-empty location string, radius in {5, 10, 20, 25, 50}.
- Trigger: CLI invocation.
- Expected behavior: Geocoder resolves location string to (lat, lon). If geocoding returns multiple candidates, the highest-confidence match is selected and noted in the audit log.
- Postconditions: `SearchRun.resolved_location` is set; run transitions to Collecting.
- Error handling: No geocoding results -> `Failed(GEO_NO_RESULT)` with human-readable stderr message and reason code in audit log.

### 4.3 FR-002: Competitor Collection — Source Adapter Lifecycle

States per adapter: Idle -> Requesting -> Throttling -> Done | Failed

```
Idle
  --[run enters Collecting]--> Requesting

Requesting
  --[HTTP response received]--> Throttling
  --[HTTP error / timeout]--> Failed(reason_code)

Throttling
  --[pacing delay elapsed]--> Done(records)

Failed: SourceResult recorded; run continues with remaining adapters.
Done:   SourceResult recorded (Success or PartialSuccess).
```

- **Example:** OSM/Overpass adapter returns 12 business nodes. Yelp adapter times out. Run records one Success and one Failed(TIMEOUT), then continues to ranking with 12 records.
- Preconditions: `SearchRun` in Collecting state; `Location` and `Radius` resolved.
- Expected behavior: Each adapter executes independently (concurrently via Tokio; each enforces its own pacing per request; no shared adapter state). Every returned field is tagged with the adapter's `source_id`.
- Postconditions: `SearchRun.source_results` contains one `SourceResult` per adapter. Zero competitors is a valid outcome; run proceeds to rendering.
- Error handling reason codes: `HTTP_4XX`, `HTTP_5XX`, `TIMEOUT`, `PARSE_ERROR`, `ADAPTER_CONFIG_MISSING`.

### 4.4 FR-002: Business Profile Normalization

- **Example:** Overpass returns a node with `amenity=gym`, `name=Iron Temple`, `addr:street=Rembrandtplein 1`, no phone tag. Produces BusinessProfile with name=High DataPoint, address=Medium DataPoint, phone=Absent DataPoint.
- Preconditions: Raw records available (zero records is valid; produces empty profile set).
- Expected behavior: Each raw record maps to a `BusinessProfile`. Fields absent from the source become Absent DataPoints. Multiple sources for the same business are merged: highest-confidence DataPoint per field wins; all source IDs preserved.
- Postconditions: Every `Competitor` has a fully-initialized `BusinessProfile` with every defined field as a DataPoint (valued or Absent). No null values in the domain model.
- Deduplication rule: Two raw records refer to the same business if their names match (case-insensitive, trimmed) AND their coordinates are within 50 metres. On match: merge profiles as above.

### 4.5 FR-003 and FR-004: Ranking

- **Example:** Three competitors at [2.1, 4.5, 4.5] km with keyword_scores [0.70, 0.85, 0.60]. Order: 2.1 km first; then 4.5 km/0.85; then 4.5 km/0.60.
- Preconditions: Normalised `Vec<Competitor>` available.
- Ranking rules (applied in sequence):
  1. Primary: `distance_km` ascending.
  2. Secondary: `keyword_score` descending.
  3. Tertiary: `name` ascending (case-insensitive, UTF-8 lexicographic).
  4. Quaternary: stable sort (preserves relative source adapter order).
- Keyword-relevance score: token overlap between competitor categories/tags and query industry keywords, normalised to [0.0, 1.0]. Algorithm is an implementation decision; must be deterministic and documented in the implementation chronicle.
- Search-visibility score: normalised composite of review count and profile completeness (fields-present / total-defined-fields). Algorithm is an implementation decision; must be deterministic and documented in the implementation chronicle.
- Postconditions: `SearchRun.competitors` sorted; each `Competitor.rank` set (1-indexed).

### 4.6 FR-003: Report Rendering

- **Example:** Run with 8 competitors produces terminal table (Rank | Name | Distance | Address | Phone | Website | Keyword% | Visibility%) and saves `competitor_spy_report_20260321_143022_UTC.pdf`.
- Preconditions: `SearchRun` in Rendering state; `competitors` ranked.
- Terminal behavior: Structured table to stdout. Header includes query parameters and run UTC timestamp. Each row covers all profile fields; absent = `--`. Footer lists failed sources with reason codes.
- PDF behavior: Same logical structure as terminal report, formatted A4 portrait. Filename: `competitor_spy_report_YYYYMMDD_HHMMSS_UTC.pdf` (run start UTC). Saved to `--output-dir` (default: cwd).
- Postconditions: Both outputs derived from the same `SearchRun.competitors`; logically equivalent (no data in one absent from the other).
- Error handling: PDF library failure -> warning on stderr; terminal output still produced; exit code 0. Terminal render failure -> `Failed(render_error)`; exit code 1.

### 4.7 FR-005: Credential Management

States: Absent -> PromptUser -> Encrypting -> Stored | Absent

- **Example:** On first run, Yelp has no stored key. Prompts on stderr: `Yelp API key required. Enter key (input hidden):`. User enters key. Encrypted and stored. All subsequent runs use stored key silently.
- Preconditions: An adapter with `requires_credential = true` initialised; `CredentialStore` has no entry for its `adapter_id`.
- Expected behavior: Prompt on stderr (never stdout). Input read with echo disabled. Non-empty input -> encrypt and store. Empty input -> skip this adapter with `ADAPTER_CONFIG_MISSING`.
- Postconditions: Credential stored encrypted on disk. Decrypted value in-memory only for the duration of the run; zeroed on drop. Never written to logs, stdout, or PDF.

### 4.8 FR-006 and FR-007: Pacing and Source Failure

- Pacing: After each HTTP request, the adapter waits for a duration drawn uniformly from [5, 15] seconds. In test mode (`CSPY_PACING_SEED` env var set), sequence is deterministic; zero-delay allowed in unit tests.
- Source failure: Any adapter-level failure produces `SourceResult { status: Failed(reason_code) }`. Run continues. Failed sources appear in the report footer. Run never aborts due to adapter failure alone.

---

## 5. Decision Tables

### 5.1 Source Failure Handling

| Condition | Action |
|---|---|
| HTTP 4xx | Record `Failed(HTTP_4XX)`, audit log, continue run |
| HTTP 5xx | Record `Failed(HTTP_5XX)`, audit log, continue run |
| HTTP timeout | Record `Failed(TIMEOUT)`, audit log, continue run |
| Response parse error | Record `Failed(PARSE_ERROR)`, audit log, continue run |
| Credential absent, user skips | Record `Failed(ADAPTER_CONFIG_MISSING)`, skip adapter, continue |
| All adapters fail | Both reports produced with zero competitors; warning footer; exit 0 |

### 5.2 Missing Field Handling

| Condition | Action |
|---|---|
| Field present in source | DataPoint with value; Confidence per adapter ruleset |
| Field absent from source | DataPoint, value = None, Confidence = Absent |
| Field in multiple sources, same value | Highest-confidence source wins; all source IDs preserved |
| Field in multiple sources, conflicting | Highest-confidence source wins; conflict logged to audit trail |
| Field never defined for this adapter | DataPoint = Absent; not displayed as an error |

### 5.3 Ranking Tie-Break

| Condition | Rule |
|---|---|
| Same distance | keyword_score descending |
| Same distance and keyword_score | name ascending (case-insensitive, UTF-8) |
| All three equal | Stable sort; preserves source adapter order |

### 5.4 Credential Prompt Policy

| Condition | Action |
|---|---|
| Credential stored and valid | Use silently; update `last_used_at` |
| Credential absent | Prompt on stderr; store on non-empty input; skip on empty |
| Credential store unreadable | Warn; skip all credential-requiring adapters; continue |
| Credential store write fails | Warn; use in-memory for this run only; not persisted |

### 5.5 Pacing Policy

| Context | Behaviour |
|---|---|
| Normal run | Uniform[5, 15] seconds delay after each HTTP request |
| Test run (`CSPY_PACING_SEED` set) | Deterministic sequence from seed; zero-delay allowed in unit tests |
| Multiple requests to same source | Each request gets its own independent delay |
| Geocoding requests | Pacing applied equally to geocoding HTTP requests |

---

## 6. Data and Interface Contracts

### 6.1 CLI Interface Contract

```
competitor-spy \
  --industry <string>              # required; e.g. "yoga studio"
  --location <string>              # required; e.g. "Amsterdam, Netherlands"
  --radius <5|10|20|25|50>        # required; km
  [--output-dir <path>]            # optional; directory for PDF; default = cwd
  [--no-pdf]                       # optional; skip PDF output
  [--log-level <trace|debug|info|warn|error>]  # optional; default = info
  [--pacing-seed <u64>]            # optional; test/debug only; deterministic pacing
```

Exit codes: 0 = run completed (reports produced, even with source failures or zero competitors). 1 = fatal error (validation, geocoding, terminal render failure, or unrecoverable I/O).

### 6.2 PDF Output Contract

- Filename: `competitor_spy_report_YYYYMMDD_HHMMSS_UTC.pdf` (run start time, UTC)
- Encoding: UTF-8 throughout
- Sections: Header (query parameters, run timestamp, sources attempted/succeeded/failed), Competitor Table (ranked; all profile fields; absent = `--`; source reference per field), Source Summary (per source: status and record count or failure reason code)
- Format: A4, portrait orientation

### 6.3 Audit Log Contract

- Emitted via OpenTelemetry structured logging
- All events include: run_id (Uuid), adapter_id (where applicable), event_type (String), timestamp_utc (ISO 8601)
- Secrets redacted before log emission by a pre-emit filter; no credential value, API key, or token ever appears in any log entry
- Minimum events per run: `run_start`, `geocoding_attempt`, `geocoding_result`, `adapter_start` (per adapter), `adapter_request` (per HTTP call; URL = hostname+path only, no query params captured), `adapter_result`, `normalization_complete`, `ranking_complete`, `render_terminal_complete`, `render_pdf_complete` or `render_pdf_failed`, `run_complete`

### 6.4 Credential Store Contract

- Location: Linux `~/.config/competitor-spy/credentials`; Windows `%APPDATA%\competitor-spy\credentials`
- Format: encrypted binary via `age` (passphrase-based, user-controlled)
- Schema: serialised map of `adapter_id (String) -> encrypted_credential_bytes (Vec<u8>)`
- Never read or written during report rendering phase
- Contents never appear in any log, report, or stdout

### 6.5 Exact Field Formats

- Distances: km, 2 decimal places in reports (e.g. `4.52 km`)
- Scores: 0.0-1.0 internal; integer percentage in reports (e.g. `85%`)
- Timestamps: UTC ISO 8601 in audit log; `YYYYMMDD_HHMMSS_UTC` suffix in PDF filename
- Absent field display: `--` in both terminal and PDF

---

## 7. Architecture and Design Decisions

### Decision 1: Rust Cargo workspace with one crate per architectural layer

- Rationale: compiler-enforced boundary between domain, adapters, output, and CLI. Domain crate has zero I/O or async dependencies — prerequisite for a future web consumer.
- Alternative rejected: single crate with modules — module boundaries are advisory, not enforced.

### Decision 2: Async runtime (Tokio) for all I/O

- Rationale: multiple adapters execute concurrently (each throttling independently), reducing total run time.
- Pacing: each adapter issues one request at a time and delays independently; Adapter A can be throttling while Adapter B is requesting.

### Decision 3: Source adapters as a trait (`SourceAdapter`)

- Rationale: independently replaceable without touching domain or rendering code. New adapters added as new modules.

### Decision 4: PDF via `printpdf` crate (pure Rust, no system binary dependency)

- Alternative rejected: `wkhtmltopdf` — requires system binary; portability rejected.
- Risk: manual table layout; layout decisions documented in implementation chronicle.

### Decision 5: Credential encryption via `age` crate

- Rationale: modern, audited format; first-class Rust implementation; user passphrase-controlled.
- Alternative rejected: OS keychain (`keyring` crate) — cross-platform unreliable for v1.

### 7.1 Layered Architecture (Q3-ARCH-01)

```
competitor_spy_domain         (pure Rust; zero I/O, zero async, zero rendering dependencies)
+-- query                     -- SearchQuery, Location, Radius, input validation
+-- profile                   -- BusinessProfile, DataPoint, Confidence, Competitor, deduplication
+-- run                       -- SearchRun, RunStatus, SourceResult
+-- ranking                   -- RankingEngine trait + default implementation
+-- scoring                   -- ScoringStrategy trait (keyword_score, visibility_score)

competitor_spy_adapters       (async; implements SourceAdapter trait; HTTP I/O only)
+-- trait SourceAdapter       -- defined here; implemented per adapter module
+-- nominatim                 -- OSM geocoding (no credential required)
+-- osm_overpass              -- Overpass API (no credential required)
+-- yelp                      -- Yelp Fusion API (credential required; approved U-001 2026-03-21)
+-- google_places             -- Google Places API (credential required; approved U-001 2026-03-21)

competitor_spy_output         (sync; no domain logic; pure rendering)
+-- terminal                  -- formats SearchRun to stdout
+-- pdf                       -- formats SearchRun to PDF via printpdf

competitor_spy_credentials    (sync; file I/O; age encryption)
+-- CredentialStore           -- store / retrieve / delete encrypted credentials per adapter_id

competitor_spy_telemetry      (async; OpenTelemetry; pre-emit secret redaction filter)

competitor_spy_cli            (entry point only; argument parsing; no domain logic)
+-- main                      -- parse args -> build SearchQuery -> call domain API -> render
```

**Business logic placement constraint:** No domain logic may reside in `competitor_spy_cli` or `competitor_spy_output`. Any such logic found during review is an architecture violation and a build blocker.

**Key trait interfaces:**

```rust
// competitor_spy_domain::ranking
pub trait RankingEngine: Send + Sync {
    fn rank(&self, competitors: Vec<Competitor>, query: &SearchQuery) -> Vec<Competitor>;
}

// competitor_spy_adapters -- implemented per adapter module
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

// competitor_spy_domain::scoring
pub trait ScoringStrategy: Send + Sync {
    fn keyword_score(&self, profile: &BusinessProfile, query: &SearchQuery) -> f64;
    fn visibility_score(&self, profile: &BusinessProfile) -> f64;
}
```

**CLI-to-API mapping:**
- `--industry`, `--location`, `--radius` -> `SearchQuery::new(...)` -> `domain::run::execute(query) -> SearchRun`
- `--output-dir` -> passed to `output::pdf::render(run, output_dir)`
- `--no-pdf` -> `output::terminal::render(run)` only
- `--log-level` -> `telemetry::init(log_level)`
- `--pacing-seed` -> `PacingPolicy::from_seed(seed)` injected into adapter runtime

**GUI-to-API mapping:** N/A for v1. Future web layer calls `domain::run::execute(query)` directly.

---

## 8. Quality Dimension Targets (Q2 Pack)

- **Performance and efficiency targets:**
  - Per-request pacing: 5-15s delay (by design; not a defect).
  - Ranking and normalization: complete within 500ms for up to 500 raw records (no I/O in these phases).
  - PDF write: complete within 10s for up to 500 competitors.
  - Startup to first network request: under 500ms.
  - Peak resident memory: under 256 MB for a standard run (up to 200 results).

- **Reliability and resilience targets:**
  - Any single adapter failure must not abort the run. A test must exist for each failure mode in section 5.1.
  - Geocoding failure -> clean exit code 1 with human-readable stderr message; no panic.
  - PDF write failure -> terminal output still produced; exit code 0 with warning.
  - Credential store unreadable -> credential-requiring adapters skipped; run continues.
  - Empty result set -> valid run; both reports produced with zero competitors and explanatory note; no crash.

- **Maintainability over time targets:**
  - New adapter: create module in `competitor_spy_adapters`, implement `SourceAdapter`, register in `SourceRegistry`. Zero changes to domain, output, or CLI crates.
  - `domain::run::execute` signature is the stable web-consumer surface; breaking changes require an explicit ADR before Stage 4 implementation.
  - `ScoringStrategy` and `RankingEngine` are traits; alternative implementations are swappable without modifying callers.
  - Change seams: domain/adapters, adapters/output, domain/credentials, domain/CLI.

- **Not-applicable declarations:**
  - Concurrent user throughput / SLA: N/A (single-user CLI). Approved by Lefty at Stage 1.
  - HTTP retry logic / circuit breakers: N/A for v1 (pacing only). Deferred to post-v1. Approved by Team Lead.

---

## 9. Test Strategy (TDD-aligned)

### 9.1 Unit Tests

| Module | Coverage targets |
|---|---|
| `domain::ranking` | All tie-break combinations; empty input; single competitor |
| `domain::scoring` | keyword_score and visibility_score at 0.0, 1.0, and midpoint with known inputs |
| `domain::query` | Validation edge cases: empty industry, invalid radius, empty location string |
| `domain::profile` | Normalization; deduplication (within 50m + name match); Absent DataPoint; merge priority |
| `competitor_spy_credentials` | Encrypt/decrypt round-trip; retrieve-absent returns None; write-failure warns without crash |
| `PacingPolicy` | Deterministic seed produces reproducible sequence; zero-delay allowed in unit context |

### 9.2 Integration Tests

| Scope | Coverage targets |
|---|---|
| Each `SourceAdapter` | Mock HTTP server (wiremock): success, 4xx, 5xx, timeout, malformed JSON -> correct `SourceResult` |
| `CredentialStore` | File-based encrypt/decrypt on real temp filesystem |
| `output::terminal` | Snapshot test against a fixed `SearchRun`; output matches expected string |
| `output::pdf` | File produced, non-zero bytes, passes PDF parse check |

### 9.3 Acceptance Tests

| ID | Scenario |
|---|---|
| AS-001 | Valid input + mock adapters -> both outputs produced -> exit 0 |
| AS-002 | One adapter 200 OK, one timeout -> both outputs, failed source in footer -> exit 0 |
| AS-003 | Invalid radius (`--radius 7`) -> stderr message, no report -> exit 1 |
| AS-004 | Geocoding mock returns no results -> stderr message, no report -> exit 1 |
| AS-005 | All adapters fail -> both reports with zero competitors, warning footer -> exit 0 |

### 9.4 Exploratory / Manual Testing

- At least one successful live run against OSM/Overpass (no credentials required) with real output artifacts stored before prototype handback.

### 9.5 Interactive CLI Diagnostics

- Screen-state capture: `scripts/capture_session.sh <label>` (Linux/Bash); `scripts/capture_session.ps1 <label>` (Windows/PowerShell). Both pipe stdout/stderr to a timestamped log file while displaying on terminal via `tee`.
- Application-state capture: run with `--log-level trace`; structured audit log stored alongside screen capture.
- Artifact path: `docs/evidence/sessions/session_YYYYMMDD_HHMMSS_<label>.log` and `..._state.json`.
- Both scripts must be created in T-000 before any other task starts.

### 9.6 Implementation Chronicle Requirements

- Chronicle entry per adapter: fields extracted, confidence level assigned, response schema interpretation.
- Chronicle entry for scoring: exact token-overlap formula for keyword_score; exact completeness formula for visibility_score.
- Chronicle entry for PDF layout: table layout approach, pagination strategy.
- Chronicle entry for credential encryption: key derivation and storage format choices.
- Reconstruction-critical: scoring formulas and ranking tie-break rules must be fully reproducible from the chronicle alone.

### 9.7 Deterministic Test Vectors

- Canonical input: `SearchQuery { industry: "yoga studio", location_input: "Amsterdam, Netherlands", radius: Km10 }`, resolved: `Location { lat: 52.3676, lon: 4.9041 }`.
- Ranking vector: competitors at [2.1, 4.5, 4.5] km, keyword_scores [0.70, 0.85, 0.60], names ["Alpha", "Beta", "Gamma"]. Expected order: [Alpha, Beta, Gamma].
- Name tie-break: competitors at [3.0, 3.0] km, scores [0.5, 0.5], names ["Zeta", "Alpha"]. Expected order: [Alpha, Zeta].
- Empty source result: `SourceResult { adapter_id: "osm_overpass", status: Failed(TIMEOUT), records: [] }` -> zero competitors from this adapter; run continues.
- Pacing determinism: with `CSPY_PACING_SEED=42`, the first three delay values must be documented in the implementation chronicle once the pacing implementation is complete.

---

## 10. Traceability Matrix

| Requirement | Spec section | Planned tests | Chronicle entry |
|---|---|---|---|
| FR-001 (query + geocoding) | section 4.2 | AS-001, AS-003, AS-004 | query module |
| FR-002 (collection + normalization) | sections 4.3, 4.4 | AS-002, AS-005, adapter integration | per-adapter module |
| FR-003 (reports) | section 4.6 | AS-001, terminal snapshot, PDF parse | rendering module |
| FR-004 (scoring) | sections 4.5, 3.5 | scoring unit tests | scoring algorithm |
| FR-005 (credentials) | section 4.7 | credential unit + integration | credential store |
| FR-006 (pacing) | sections 4.8, 5.5 | pacing unit test with seed | pacing policy |
| FR-007 (missing data + failures) | sections 4.8, 5.1, 5.2 | AS-002, AS-005, adapter failure tests | error handling |
| NFR: Security/GDPR | sections 6.4, 8 | credential redaction test; secret-filter audit log test | credential store; telemetry |
| NFR: Observability | sections 6.3, 7.1 | telemetry event coverage integration test | telemetry module |
| NFR: Maintainability | section 7 Decision 3, section 8 | crate dependency boundary test | adapter extension ADR |
| Q3-ARCH-01 | section 7.1 | crate dep graph acyclic; domain has zero I/O deps | architecture decisions |

---

## 11. Open Items

- **U-001 — Source licensing (RESOLVED 2026-03-21, Lefty):** All four candidate adapters approved for automated querying: Nominatim, OSM/Overpass, Yelp Fusion, Google Places. Pacing policy (5-15s per request) satisfies fair-use expectations for all four.
- **U-002 — Field-completeness threshold:** Define minimum percentage of profile fields that constitutes an "actionable" report, for use in zero-competitor and low-data warning messages. Decided by Team Lead Agent: threshold = 40% fields present (i.e., at least 4 of the ~10 defined fields have a non-Absent DataPoint). Warn if below threshold; do not error.

---

## 12. Stage 2 Approval

- Approved by: Team Lead Agent (delegated authority per PROJECT_BRIEF.md §8.2)
- Approval date: 2026-03-21
- Notes: U-001 resolved by Lefty 2026-03-21 — all four adapters approved. U-002 resolved by Team Lead (40% field-completeness threshold). Spec frozen for Stage 3 planning. Breaking changes to `domain::run::execute` signature require an ADR before any Stage 4 implementation.

---

## 13. V3 Enrichment Specification (Addendum)

### 13.0 Addendum Metadata

- Spec ID: CSPY-SPEC-003 (V3 addendum to CSPY-SPEC-001)
- Version: 1.0
- Approval authority: Team Lead Agent (delegated by Lefty 2026-03-24; PROJECT_BRIEF.md §8 + §9)
- Status: APPROVED — 2026-03-24
- Source brief: `PROJECT_BRIEF.md` V3 section (approved 2026-03-24)
- Scope: Website enrichment pipeline appended to the V2 collection stage. Does not alter V1/V2 behavior; additive only.

---

### 13.1 V3 Domain Extensions

#### New Ubiquitous Language Terms

| Term | Definition |
|---|---|
| Enrichment Run | The post-collection phase where each Competitor's website is fetched and parsed for additional fields. |
| WebEnrichment | The set of website-derived enrichment fields for one Competitor. May be fully absent if the site is unreachable or yields no parseable fields. |
| Enrichment Field | One extracted value within a WebEnrichment (e.g. pricing, lesson types). Individually present or absent. |
| Extraction Attempt | One parse operation targeting one Enrichment Field on one fetched HTML document. |
| Enrichment Coverage | The fraction of Competitors in a run for which at least one Enrichment Field was successfully extracted. |
| Nil Marker | The literal string `[unavailable]` displayed in reports when an Enrichment Field could not be extracted. |

#### New Entity: `WebEnrichment`

```rust
pub struct WebEnrichment {
    pub competitor_id: Uuid,             // links to Competitor entity
    pub fetch_status: FetchStatus,       // Success | Failed(reason_code)
    pub fetched_at: Option<UtcDateTime>,
    pub pricing: Option<String>,         // extracted pricing text (cleaned, trimmed)
    pub lesson_types: Option<Vec<String>>, // e.g. ["Reformer", "Mat", "Fusion"]
    pub schedule: Option<String>,        // full timetable text or structured summary
    pub testimonials: Option<Vec<String>>, // list of individual testimonial passages
    pub class_descriptions: Option<Vec<String>>, // list of class/course description passages
}

pub enum FetchStatus {
    Success,
    Failed(EnrichmentErrorCode),
}

pub enum EnrichmentErrorCode {
    HttpError(u16),   // HTTP status code returned
    Timeout,
    DnsFailure,
    ParseError,       // page fetched but HTML unparseable
    NoUrl,            // competitor has no website URL in profile
}
```

**Invariants:**
- A `WebEnrichment` with `fetch_status = Failed(...)` has all enrichment fields set to `None`.
- A `WebEnrichment` with `fetch_status = Success` may still have all enrichment fields set to `None` (site fetched but no target content found). This is valid; not an error.
- `competitor_id` must reference an existing `Competitor` in the same `SearchRun`.

#### Extended Entity: `SearchRun`

Extends existing `SearchRun` with:
```rust
pub enrichments: Vec<WebEnrichment>,  // one entry per Competitor (including failed entries)
pub enrichment_coverage: f64,         // fraction (0.0–1.0) of competitors with ≥1 extracted field
```

---

### 13.2 V3 Functional Behavior

#### 13.2.1 Run Lifecycle Extension (V3)

The V3 run lifecycle extends section 4.1 by inserting an `Enriching` state after `Collecting`:

```
Collecting
  --[all adapters complete]--> Enriching      // NEW V3 STATE

Enriching
  --[all enrichment attempts complete]--> Ranking
  // Individual enrichment failures do NOT abort the run.
  // They produce WebEnrichment(fetch_status=Failed) and the run proceeds.
  // Zero successful enrichments is a valid outcome.

Ranking, Rendering, Done: unchanged from section 4.1
```

#### 13.2.2 FR-V3-001: Enrichment Source Input

- Preconditions: `SearchRun` has transitioned to `Enriching` state; `SearchRun.competitors` is populated (may be empty).
- Trigger: Automatic after `Collecting` completes.
- Expected behavior: For each `Competitor` where `BusinessProfile.website` DataPoint is non-Absent, schedule one enrichment fetch. Competitors with no website URL produce a `WebEnrichment { fetch_status: Failed(NoUrl), ... all fields None }`.
- Postconditions: `SearchRun.enrichments` contains exactly one `WebEnrichment` per `Competitor`, in the same order.

#### 13.2.3 FR-V3-002: Website Content Fetching

- HTTP client: `reqwest` (blocking mode acceptable in the enrichment phase; async acceptable if already in Tokio context).
- Request method: GET, following up to 3 redirects.
- Request headers: `User-Agent: Mozilla/5.0 (compatible; CompetitorSpy/3.0; +https://github.com/local)` to appear as a normal browser visit.
- Timeout: 15 seconds per fetch.
- TLS: required; reject invalid certificates unless `--allow-insecure-tls` flag is passed (CLI flag defined in 13.4.1).
- Pacing: after each fetch, pause for a duration drawn uniformly from [5, 15] seconds (same policy as section 4.8). See section 13.3.2 for the enrichment pacing decision table.
- Fetches are sequential (one at a time), not concurrent, to minimise detection risk.
- Only the root page (`/`) is fetched unless the HTTP response contains a `<link rel="sitemap">` hint pointing to a pricing or schedule sub-page; in that case, the hint URL is fetched as a second request for the same competitor (one additional pacing delay).
- Postconditions: Raw HTML string available for parsing, or `FetchStatus::Failed(reason_code)` recorded.

#### 13.2.4 FR-V3-003: Pricing Extraction

- **Goal:** Extract any pricing, membership, or class-cost information visible on the page.
- **Extraction strategy (in priority order):**
  1. Find `<table>` elements whose headers or caption contain any of: `Preis`, `Preise`, `price`, `pricing`, `tarif`, `kosten`, `euro`, `€`. Extract all cell text from matching tables.
  2. Find `<ul>` or `<ol>` list items containing a `€` character or a 2–4 digit number followed by `€` or `EUR`.
  3. Find `<p>` or `<div>` elements whose text contains both a digit and `€`/`EUR`.
  4. Find elements with CSS class or id containing `preis`, `price`, `tarif`, `cost`.
- **Output:** Concatenated plain text, whitespace-normalised, truncated to 2 000 characters.
- **Nil condition:** No qualifying element found → `pricing = None`.

#### 13.2.5 FR-V3-004: Lesson Type Extraction

- **Goal:** Identify the discipline/modality names offered at the studio.
- **Target vocabulary (case-insensitive):** `Reformer`, `Mat`, `Matwork`, `Fusion`, `Pilates`, `Yoga`, `Barre`, `Aerial`, `Tower`, `Cadillac`, `Chair`, `Barrel`, `Clinical`, `Prenatal`, `Postnatal`, `Hot`, `Yin`, `Vinyasa`, `HIIT`, `Stretch`, `Mobility`, `Fascia`.
- **Extraction strategy:**
  1. Scan all navigation `<nav>` links, `<h1>` through `<h3>` text, and `<li>` list items.
  2. Collect any token from the target vocabulary found in those elements.
  3. Deduplicate; preserve document order.
- **Output:** `Vec<String>` of matched vocabulary tokens.
- **Nil condition:** No target vocabulary token found → `lesson_types = None`.

#### 13.2.6 FR-V3-005: Schedule Extraction

- **Goal:** Extract timetable or class schedule information if present.
- **Extraction strategy (in priority order):**
  1. Look for `<table>` elements whose headers contain day-of-week names (German or English: `Mo`, `Di`, `Mi`, `Do`, `Fr`, `Sa`, `So`, `Mon`, `Tue`, `Wed`, `Thu`, `Fri`, `Sat`, `Sun`). Extract full table text.
  2. Look for `<div>` or `<section>` elements with class/id containing `stundenplan`, `timetable`, `schedule`, `kursplan`, `kurse`. Extract inner text.
  3. Look for time-pattern matches (e.g. `\d{1,2}:\d{2}`) in combination with day-of-week tokens within the same parent element.
- **Output:** Concatenated plain text, whitespace-normalised, truncated to 3 000 characters.
- **Nil condition:** No qualifying element found → `schedule = None`.

#### 13.2.7 FR-V3-006: Testimonial Extraction

- **Goal:** Extract customer testimonials or reviews published on the studio's own site.
- **Extraction strategy (in priority order):**
  1. Find `<blockquote>` elements. Extract text content.
  2. Find elements with class/id containing `testimonial`, `review`, `bewertung`, `kundenstimme`, `erfahrung`. Extract text content.
  3. Find `<p>` elements that begin or end with a `"` or `„` quotation character and are longer than 40 characters.
- **Output:** `Vec<String>`, each item one testimonial passage, trimmed, max 500 characters per item, max 10 items.
- **Nil condition:** No qualifying element found → `testimonials = None`.

#### 13.2.8 FR-V3-007: Class Description Extraction

- **Goal:** Extract class/course descriptions indicating content, level, or target audience.
- **Extraction strategy (in priority order):**
  1. Find `<p>` elements that are direct siblings or children of a heading containing lesson-type vocabulary (section 13.2.5 target vocabulary). Extract those paragraph texts.
  2. Find `<section>` or `<article>` elements with class/id containing `kurs`, `class`, `angebot`, `offer`, `leistung`. Extract `<p>` text within.
  3. Find `<p>` elements longer than 80 characters within 3 DOM levels below any `<h2>` or `<h3>` containing lesson-type vocabulary.
- **Output:** `Vec<String>`, each item one description passage, trimmed, max 800 characters per item, max 8 items.
- **Nil condition:** No qualifying element found → `class_descriptions = None`.

#### 13.2.9 FR-V3-008: Partial Enrichment and Nil Marking

- Each Enrichment Field is extracted and evaluated independently. A failure or nil result for one field does not affect extraction of other fields.
- A `WebEnrichment` is considered partially enriched if `fetch_status = Success` AND at least one field is `Some(...)`.
- A `WebEnrichment` is considered fully absent if `fetch_status = Success` AND all fields are `None`. This is valid; no error is raised.
- In all report outputs, `None` fields are rendered as the literal `[unavailable]`.
- `Vec` fields with an empty or `None` value are rendered as `[unavailable]`.

#### 13.2.10 FR-V3-009: Enrichment Coverage Threshold (U-V3-001, resolved by Team Lead)

- **Coverage metric:** `enrichment_coverage = (count of competitors with ≥1 extracted field) / (total competitors)`.
- **Passing threshold:** ≥ 60% (0.60). If fewer than 60% of studios yield any enrichment data, a warning line is appended to the report footer: `Warning: enrichment coverage below threshold (N/M studios enriched)`.
- **Run outcome:** Coverage below threshold is a warning, never a failure. Exit code is unaffected.
- **Decision authority:** Team Lead Agent, 2026-03-24. Revocable by Lefty.

---

### 13.3 V3 Decision Tables

#### 13.3.1 Enrichment Fetch Failure Handling

| Condition | Action |
|---|---|
| HTTP 4xx response | `Failed(HttpError(status))`, all fields None, audit log, run continues |
| HTTP 5xx response | `Failed(HttpError(status))`, all fields None, audit log, run continues |
| HTTP timeout (>15s) | `Failed(Timeout)`, all fields None, audit log, run continues |
| DNS resolution failure | `Failed(DnsFailure)`, all fields None, audit log, run continues |
| TLS certificate invalid | `Failed(HttpError(0))`, all fields None, audit log, run continues (unless `--allow-insecure-tls`) |
| HTML parse failure | `Failed(ParseError)`, all fields None, audit log, run continues |
| Competitor has no URL | `Failed(NoUrl)`, all fields None, no HTTP request made |
| All enrichments fail | Run proceeds to Ranking; report footer notes zero enrichment coverage; exit 0 |

#### 13.3.2 Enrichment Pacing Policy

| Context | Behaviour |
|---|---|
| Normal enrichment run | Uniform[5, 15] seconds delay after each HTTP fetch |
| Test run (`CSPY_PACING_SEED` set) | Deterministic sequence from seed; zero-delay allowed in unit tests |
| Competitor has no URL | No delay (no request made) |
| Second sub-page fetch (sitemap hint) | Additional pacing delay after the sub-page request |
| Single-competitor debug run | Normal pacing applies; no exception |

#### 13.3.3 Enrichment Field Nil vs Empty

| Condition | Stored value | Report display |
|---|---|---|
| Field extracted with content | `Some(text)` / `Some(vec![...])` | content |
| Field extraction strategy matched no elements | `None` | `[unavailable]` |
| Fetch failed (any reason) | `None` (forced) | `[unavailable]` |
| Vec field extracted but all items empty after trim | `None` | `[unavailable]` |

---

### 13.4 V3 Interface Contracts

#### 13.4.1 CLI Extension

V3 adds the following optional flags to the existing CLI contract (section 6.1):

```
[--no-enrichment]              # skip the enrichment phase; produce V2-equivalent report only
[--allow-insecure-tls]        # accept self-signed or expired TLS certificates during enrichment fetches
[--enrichment-timeout <secs>] # per-fetch timeout; default = 15; min = 5; max = 60
```

#### 13.4.2 Report Enrichment Section Contract

Both terminal and PDF reports gain a new **"Website Enrichment"** section, rendered after the existing competitor table.

**Terminal format (per studio):**

```
--- Website Enrichment: <Studio Name> ---
  Pricing:             <pricing text or [unavailable]>
  Lesson Types:        <comma-separated list or [unavailable]>
  Schedule:            <schedule text or [unavailable]>
  Testimonials:        <count> found  (or [unavailable])
    [1] "..."
    [2] "..."
  Class Descriptions:  <count> found  (or [unavailable])
    [1] "..."
```

**PDF format:** Same logical content as terminal. Each studio's enrichment block is a titled subsection. Testimonials and class descriptions are rendered as numbered indented paragraphs. `[unavailable]` is rendered in italic.

**Footer additions:**
- Enrichment coverage: `Enrichment: N/M studios (X%) had at least one extractable field.`
- Below-threshold warning (if applicable): `Warning: enrichment coverage below 60% threshold.`

#### 13.4.3 Audit Log Extension

Additional minimum events per enrichment run (appended to section 6.3):

| Event | Fields |
|---|---|
| `enrichment_start` | run_id, competitor_count |
| `enrichment_fetch_attempt` | run_id, competitor_id, url_host (no path, no query) |
| `enrichment_fetch_result` | run_id, competitor_id, fetch_status, fields_extracted_count |
| `enrichment_complete` | run_id, coverage_fraction |

URL logging rule: only the hostname is logged, never the path or query string, to avoid leaking PII or tracking tokens.

---

### 13.5 V3 Architecture Extension

New module additions (additive; no existing module changes required for V1/V2 behavior):

```
competitor_spy_domain
+-- enrichment              -- WebEnrichment, FetchStatus, EnrichmentErrorCode, enrichment_coverage()
+-- run (extended)          -- SearchRun gains: enrichments: Vec<WebEnrichment>, enrichment_coverage: f64

competitor_spy_adapters
+-- web_enricher            -- fetches HTML (reqwest), applies pacing, returns raw HTML per competitor
+-- extractors/
    +-- pricing             -- FR-V3-003 extraction logic
    +-- lesson_types        -- FR-V3-004 extraction logic
    +-- schedule            -- FR-V3-005 extraction logic
    +-- testimonials        -- FR-V3-006 extraction logic
    +-- class_descriptions  -- FR-V3-007 extraction logic

competitor_spy_output
+-- terminal (extended)     -- renders WebEnrichment section per competitor
+-- pdf (extended)          -- renders WebEnrichment subsection per competitor
```

**Key crate additions:**
- `scraper = "0.22"` (or latest stable) — HTML parsing via CSS selectors; added to `competitor_spy_adapters` dependencies only.
- `reqwest` — already present; no version change required.

**Architecture constraint:** All extraction logic (`extractors/`) must be pure functions with no I/O. They receive a `&str` (raw HTML) and return the appropriate `Option<T>`. This makes them unit-testable without network access.

---

### 13.6 V3 Test Strategy

#### 13.6.1 Unit Tests (Extraction)

| Module | Coverage targets |
|---|---|
| `extractors::pricing` | HTML with pricing table → correct text; HTML without → None; malformed HTML → None |
| `extractors::lesson_types` | Nav with mixed vocabulary → correct deduplicated vec; no vocabulary → None |
| `extractors::schedule` | German day-header table → correct text; English timetable div → correct text; no schedule → None |
| `extractors::testimonials` | `<blockquote>` → correct vec; `.testimonial` class → correct; > 10 items → capped at 10 |
| `extractors::class_descriptions` | Heading + sibling `<p>` → correct vec; no qualifying context → None |
| `WebEnrichment` | All fields None when FetchStatus::Failed; coverage metric calculation edge cases |

#### 13.6.2 Integration Tests (Enrichment Pipeline)

| Scope | Coverage targets |
|---|---|
| `web_enricher` | Mock HTTP server returns static HTML fixture → correct `WebEnrichment` produced |
| `web_enricher` | Mock HTTP server returns 404 → `FetchStatus::Failed(HttpError(404))`; other fields None |
| `web_enricher` | Mock HTTP server times out → `FetchStatus::Failed(Timeout)` |
| `web_enricher` | Competitor with no URL → `FetchStatus::Failed(NoUrl)`; no HTTP request made |
| Pacing | With `CSPY_PACING_SEED` set, enrichment delay sequence is deterministic and logged |

#### 13.6.3 Acceptance Tests (V3)

| ID | Scenario | Pass Condition |
|---|---|---|
| AS-V3-001 | Live run: `--industry "pilates" --location "Neulengbach, Austria" --radius 50` | Both reports produced; enrichment section present; ≥1 studio has ≥1 non-nil field; exit 0 |
| AS-V3-002 | Mock run: one competitor URL returns 200 but HTML has no extractable fields | `WebEnrichment.fetch_status = Success`; all fields None; report shows `[unavailable]` for all fields; run completes; exit 0 |
| AS-V3-003 | Mock run: all competitor URLs return HTTP 503 | All `WebEnrichment.fetch_status = Failed`; coverage = 0%; below-threshold warning in report footer; exit 0 |

#### 13.6.4 HTML Test Fixtures

Real-world HTML fixture files (anonymised) are stored under `tests/fixtures/enrichment/`. At minimum:
- `fixture_pricing_table_de.html` — German-language pricing table
- `fixture_schedule_table_de.html` — German-language timetable
- `fixture_testimonials_blockquote.html` — blockquote-based testimonials
- `fixture_no_content.html` — minimal valid HTML with no extractable enrichment content

---

### 13.7 V3 Traceability Extension

| Requirement | Spec section | Planned tests | Chronicle entry |
|---|---|---|---|
| FR-V3-001 (input from V2 list) | 13.2.2 | AS-V3-001 | enrichment module |
| FR-V3-002 (HTTP fetch) | 13.2.3 | web_enricher integration | web_enricher module |
| FR-V3-003 (pricing) | 13.2.4 | extractors::pricing unit | pricing extractor |
| FR-V3-004 (lesson types) | 13.2.5 | extractors::lesson_types unit | lesson_types extractor |
| FR-V3-005 (schedule) | 13.2.6 | extractors::schedule unit | schedule extractor |
| FR-V3-006 (testimonials) | 13.2.7 | extractors::testimonials unit | testimonials extractor |
| FR-V3-007 (class descriptions) | 13.2.8 | extractors::class_descriptions unit | class_descriptions extractor |
| FR-V3-008 (partial enrichment + nil marking) | 13.2.9 | AS-V3-002; extractor nil tests | enrichment module |
| FR-V3-009 (coverage threshold, U-V3-001) | 13.2.10 | AS-V3-003; coverage calculation unit | enrichment module |
| NFR: pacing (enrichment) | 13.3.2 | pacing integration with CSPY_PACING_SEED | web_enricher module |
| NFR: audit log (enrichment) | 13.4.3 | telemetry event coverage | telemetry enrichment events |
| NFR: architecture extension | 13.5 | crate dep graph unchanged for existing crates | ADR for scraper crate adoption |

---

### 13.8 V3 Open Items

- **U-V3-001 — Coverage threshold (RESOLVED 2026-03-24, Team Lead):** ≥60% studios must yield ≥1 enrichment field for a passing run. Below threshold = warning only; exit code unaffected.
- **U-V3-002 — Language handling:** Austrian studio sites are predominantly German. Extraction strategies in sections 13.2.4–13.2.8 include German keyword variants (`Preis`, `Preise`, `Stundenplan`, `Kursplan`, `Kundenstimme`, etc.). Multi-language pages are handled implicitly by the same strategies. No translation or NLP required.
- **U-V3-003 — Sub-page depth:** V3 fetches root page only (+ one optional sub-page via sitemap hint). Deep crawling is out of scope unless a follow-on ADR approves it.
- **U-V3-004 — JavaScript-rendered content:** V3 assumes static/server-rendered HTML. Sites requiring JS execution will yield partial or nil enrichment; this is expected and acceptable per FR-V3-008. Headless browser support deferred to post-V3.

---

## 14. V3 Stage 2 Approval

- Approved by: Team Lead Agent (delegated by Lefty 2026-03-24; PROJECT_BRIEF.md §8 + §9)
- Approval date: 2026-03-24
- Notes: All V3 open items resolved or formally deferred. Spec frozen for Stage 3 task planning. V1/V2 sections unchanged. Breaking changes to enrichment domain model require a new U-V3-xxx entry and Team Lead sign-off before Stage 4 implementation.

---

# V2 Specification Addendum — Competitor Spy v2.0

## V2.1 Addendum Metadata

- Spec ID: CSPY-SPEC-002
- Version: 2.0
- Source brief: V2 brief approved by Lefty 2026-03-22
- Mode: Brownfield extension of CSPY-SPEC-001 v1.0
- Status: APPROVED — Team Lead Agent, 2026-03-22
- Author: Team Lead Agent
- V1 baseline: all V1 behaviour preserved; this addendum extends, never replaces
- Track: A (Google Places API enrichment only)

## V2.2 Scope Delta

**Added in V2:**
- Seven additional Google Places fields collected and rendered per competitor: opening hours, price level, editorial summary, reviews (up to 5), rating (rendered — was collected but not shown in V1), user rating count (rendered — was collected but not shown in V1), business type tags (rendered — was collected but not shown in V1)
- New `--detail` CLI flag enabling extended per-competitor output in both terminal and PDF renderers
- Default (no flag) terminal and PDF output unchanged from V1

**Explicitly out of scope for V2:**
- Website scraping (Track B → V3)
- New source adapters
- LLM summarization, social media data
- Scheduling, monitoring, web interface, multi-user

## V2.3 Domain Model Delta

### V2.3.1 Extended BusinessProfile fields

`BusinessProfile` gains the following optional fields. All are `DataPoint` values (present or Absent per V1 convention):

| Field name | Type | Source | Confidence when present |
|---|---|---|---|
| `opening_hours` | `Vec<String>` — Mon–Sun text lines | Google Places | Medium |
| `price_level` | `Option<u8>` — 1 to 4 | Google Places | High |
| `editorial_summary` | `String` | Google Places | Medium |
| `reviews` | `Vec<PlaceReview>` — up to 5 | Google Places | Medium |
| `rating` | `f64` — 1.0–5.0 | Google Places | High |
| `user_rating_count` | `u32` | Google Places | High |
| `place_types` | `Vec<String>` | Google Places | Medium |

### V2.3.2 New value object: PlaceReview

```
PlaceReview {
    text: String,
    rating: u8,             // 1–5
    relative_time: String,  // e.g. "3 months ago"
}
```

All fields best-effort; `text` may be empty (Google sometimes omits review body). If `text` is empty the review is still rendered with star rating and recency.

### V2.3.3 Missing field policy

Identical to V1 §5.2: if a field is absent from the API response, it becomes an Absent DataPoint. It is displayed as `--` in terminal output and omitted from the PDF detail section with a `(not available)` note. No field absence aborts the run.

## V2.4 API Contract Delta — Google Places Adapter

### V2.4.1 Field mask change

Current V1 field mask (in `X-Goog-FieldMask` header):
```
places.id,places.displayName,places.formattedAddress,places.nationalPhoneNumber,
places.websiteUri,places.types,places.rating,places.userRatingCount,places.location
```

V2 field mask (additions in **bold**):
```
places.id,places.displayName,places.formattedAddress,places.nationalPhoneNumber,
places.websiteUri,places.types,places.rating,places.userRatingCount,places.location,
places.regularOpeningHours.weekdayDescriptions,places.priceLevel,
places.editorialSummary,places.reviews
```

### V2.4.2 Response struct additions

`GooglePlace` struct gains:
- `regular_opening_hours: Option<RegularOpeningHours>` — `{ weekday_descriptions: Vec<String> }`
- `price_level: Option<String>` — Google returns string enum: `PRICE_LEVEL_FREE`, `PRICE_LEVEL_INEXPENSIVE`, `PRICE_LEVEL_MODERATE`, `PRICE_LEVEL_EXPENSIVE`, `PRICE_LEVEL_VERY_EXPENSIVE`
- `editorial_summary: Option<EditorialSummary>` — `{ text: String }`
- `reviews: Option<Vec<GoogleReview>>` — `{ text: String, rating: u8, relative_publish_time_description: String }`

### V2.4.3 Price level mapping

| API value | Rendered symbol |
|---|---|
| `PRICE_LEVEL_FREE` | Free |
| `PRICE_LEVEL_INEXPENSIVE` | $ |
| `PRICE_LEVEL_MODERATE` | $$ |
| `PRICE_LEVEL_EXPENSIVE` | $$$ |
| `PRICE_LEVEL_VERY_EXPENSIVE` | $$$$ |
| absent / unknown | `--` |

### V2.4.4 Adapter contract (pre/post)

- Pre: same as V1 — valid API key in credential store
- Post: `GooglePlace` structs may now contain populated `regular_opening_hours`, `price_level`, `editorial_summary`, `reviews`; all fields are optional and their absence does not affect the `outcome="success"` criteria
- Error contract unchanged from V1: HTTP 4xx/5xx handling, timeout, parse error — all produce `Failed` result as before

## V2.5 CLI Contract Delta

### V2.5.1 New flag

```
--detail    Include extended competitor detail in terminal and PDF output.
            Default: off. When off, output is identical to V1.
```

### V2.5.2 Acceptance contract

| Input | Expected output |
|---|---|
| No `--detail` flag | Identical to V1: summary table only |
| `--detail` flag | Summary table + per-competitor detail panel |
| `--detail` + `--no-pdf` | Detail panel in terminal only |
| `--detail` + fields absent for a competitor | Absent fields shown as `--`; run does not abort |

## V2.6 Renderer Contract Delta

### V2.6.1 Terminal detail panel

When `--detail` is present, after each ranked table row, print a detail panel:

```
  Opening hours : Mon 09:00–18:00, Tue 09:00–18:00, …
  Price level   : $$
  Description   : Boutique reformer pilates studio in the historic centre.
  Rating        : 4.7 ★ (132 reviews)
  Reviews:
    ★★★★★ "Amazing instructors, small groups." — 2 months ago
    ★★★★☆ "Great classes but parking is tricky." — 4 months ago
  Types         : gym, health, point_of_interest
```

Panel is indented 2 spaces. Lines with Absent data are omitted entirely (no `--` clutter in the detail panel; `--` is reserved for summary table columns).

### V2.6.2 PDF detail section

Each competitor page/block gains an extended section below the summary fields containing the same information as the terminal detail panel, formatted for A4 portrait layout. Absent fields omitted.

## V2.7 Test Strategy Delta

### V2.7.1 Unit tests (adapter)

- New test: `google_places_request_body_contains_expanded_field_mask` — assert the serialised `X-Goog-FieldMask` header in the outgoing request contains all new field names (per V1 feedback — request body/header assertions must be present)
- New test: `google_places_parses_opening_hours_from_response`
- New test: `google_places_parses_price_level_string_to_symbol`
- New test: `google_places_parses_reviews_up_to_five`
- New test: `google_places_absent_new_fields_produce_absent_datapoints`

### V2.7.2 Unit tests (renderer)

- New test: `terminal_detail_panel_renders_all_present_fields`
- New test: `terminal_detail_panel_omits_absent_fields`
- New test: `terminal_no_detail_flag_output_identical_to_v1`

### V2.7.3 Acceptance test

- AS-006: `--detail` flag produces extended output; at least one competitor has non-empty opening hours or a review (wiremock fixture with populated fields)

### V2.7.4 Live E2E requirement

Before declaring V2 complete: run `competitor-spy --industry pilates --location "Neulengbach, Austria" --radius 50 --detail` with a freshly built release binary and confirm at least one competitor displays non-empty opening hours or a review in the terminal output.

## V2.8 Stage 2 Approval (V2)

- Approved by: Team Lead Agent (delegated; per PROJECT_BRIEF.md §8.2 — full delegation to Team Lead for all stages)
- Approval date: 2026-03-22
- Notes: Spec frozen. Track A only. V1 output unchanged when `--detail` absent. Proceed to Stage 3 task planning.
