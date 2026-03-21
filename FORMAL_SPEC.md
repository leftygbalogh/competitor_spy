
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
- Approval authority source: Team Lead Agent (delegated; brief §8.2); U-001 source licensing is owner-retained (Lefty)
- Status: Draft — awaiting Stage 2 approval
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
+-- yelp                      -- Yelp Fusion API (credential required; U-001 pending)
+-- google_places             -- Google Places API (credential required; U-001 pending)

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

## 11. Open Items Blocking Stage 3

- **U-001 — Source licensing (OWNER-RETAINED, Lefty):** Before Stage 3 Plan approval, confirm which candidate adapters (OSM/Overpass, Nominatim, Yelp Fusion, Google Places) are approved for automated querying under their terms of service. Adapters for unapproved sources must be excluded from the Stage 3 task list.
- **U-002 — Field-completeness threshold:** Define minimum percentage of profile fields that constitutes an "actionable" report, for use in zero-competitor and low-data warning messages. Can be decided by Team Lead Agent during Stage 3 planning.

---

## 12. Stage 2 Approval

- Approved by: TBD (Team Lead Agent delegated — pending U-001 source licensing decision from Lefty)
- Approval date: TBD
- Notes: Spec is complete and internally consistent. U-001 is the sole owner-retained blocking item before Stage 3.
