# Changelog — Competitor Spy

## v1.0.0 — 2026-03-22

Initial release. Greenfield Rust CLI prototype complete.

### What's included

- **CLI entry point** (`competitor-spy`) — clap 4 with derive; flags: `--industry`, `--location`, `--radius`, `--output-dir`, `--no-pdf`, `--log-level`, `--pacing-seed`
- **Geocoding** — Nominatim public API; resolves location string to (lat, lon); selects highest-confidence candidate
- **Source adapters** — OSM Nominatim (places), OSM Overpass (QL), Yelp Fusion, Google Places; independently concurrent via Tokio; each with intentional [5, 15]s pacing jitter
- **Normalisation** — raw records → `Competitor` domain entities; absent fields clearly marked; haversine distance from resolved origin
- **Deduplication** — same name (case-insensitive, trimmed) + within 50 m → merged profile; highest-confidence field wins per merge
- **Ranking** — distance ascending → keyword-relevance descending → name ascending (UTF-8 lexicographic, case-insensitive)
- **Scoring** — keyword-relevance: token overlap with query industry, normalised [0, 1]; search-visibility: composite profile completeness + review count, normalised [0, 1]
- **Terminal renderer** — structured table: Rank | Name | Distance | Address | Phone | Website | Keyword% | Visibility%; failed-sources footer
- **PDF renderer** — A4 portrait; same logical structure as terminal; filename `competitor_spy_report_YYYYMMDD_HHMMSS_UTC.pdf`; pure Rust (printpdf 0.7)
- **Credential store** — age passphrase-encrypted; Windows `%APPDATA%\competitor-spy\credentials`; Linux `~/.config/competitor-spy/credentials`; secrets redacted in all logs
- **Telemetry** — OpenTelemetry structured logging; configurable log level; secrets pre-emit redaction; unique run IDs
- **Session capture scripts** — `scripts/capture_session.ps1` (Windows), `scripts/capture_session.sh` (Linux)

### Known limitations / release risks

- Linux x86_64 not validated at release; documented in `docs/evidence/environment-matrix.md`
- Yelp and Google Places require user-supplied API keys
- OSM sources return limited data for some industries/locations (expected OSINT limitation)
- No background scheduling, multi-user support, or web interface (post-v1)

### Test coverage at release

- 193 unit tests + 5 acceptance tests = 198 total, all green
- Acceptance tests: AS-001 (happy path), AS-002 (adapter failure non-fatal), AS-003 (invalid radius, exit 1), AS-004 (geocoding no results, exit 1), AS-005 (all adapters fail, exit 0)

### Defects fixed before release

- DEF-001: `AdapterUrls::production()` Overpass URL was `/api` instead of `/api/interpreter` — discovered during T-018 live run; fixed and regressed in commit `c07d2a1`

---

## v1.0.1 — 2026-03-22 (post-release patch)

### Defects fixed

- **DEF-002** (`cd8c443`): Overpass returned 0 results for dense cities (e.g. Amsterdam cafes). Root cause: regex QL query timed out on Overpass public API. Fix: switched to exact-match indexed queries using `[amenity=cafe]` style filters.
- **DEF-003** (`7e69048`): Niche industries with no OSM tag (e.g. "pilates") returned 0 results from Overpass. Fix: concurrent `tokio::join!` running tag query + name-regex query; name-regex failures are best-effort (silently ignored on timeout).
- **DEF-004** (Google Places domain URL): `AdapterUrls::production()` used `maps.googleapis.com` — incorrect domain for the New Places API. Corrected to `places.googleapis.com`.
- **DEF-005** (`9ac0f37`): Google Places returned HTTP 400 on every request. Root cause: the adapter called `places:searchNearby` with `includedTypes: ["establishment"]` — "establishment" is a legacy type not in Table A of the New Places API. Deeper cause: `searchNearby` cannot filter by keyword at all; the industry string was silently discarded (`let _ = industry`). Fix: switched to `places:searchText` with `textQuery` set to the industry keyword and `locationBias` (circle) for geographic weighting.

### Live E2E result after patch

```
adapter_id="google_places" outcome="success" record_count=20
adapter_id="osm_overpass"  outcome="success" record_count=9
29 competitors returned for "pilates / Neulengbach, Austria / 50 km"
PDF written to docs/evidence/sessions/competitor_spy_report_20260322_115413_UTC.pdf
```

### Credentials management added

- New `credentials` subcommand: `competitor-spy credentials set <adapter>`, `delete <adapter>`, `list`
- Setup script for Windows: `scripts/setup_google_places.ps1`

