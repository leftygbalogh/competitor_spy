# Environment Validation Matrix — Competitor Spy v1.0

Stage 5 Verify artifact. Captures which target runtime environments were validated, with pass/fail status and artifact references. Untested environments are logged as explicit release risks.

---

## Target Environments (per FORMAL_SPEC.md §2)

| OS | Architecture | Shell | Status | Evidence artifact | Notes |
|---|---|---|---|---|---|
| Windows 11 | x86_64 | PowerShell 5.1 | **PASS** | `docs/evidence/sessions/competitor_spy_report_20260322_055014_UTC.pdf` | Live run T-018; exit 0; PDF produced; geocoding resolved; terminal output rendered |
| Linux x86_64 | x86_64 | Bash | **NOT TESTED** | — | Explicit release risk; see risk notes below |
| macOS | any | any | **OUT OF SCOPE** | — | Explicitly excluded per FORMAL_SPEC.md §2 |

---

## Validated Run Details (Windows 11 / PowerShell)

- **Date:** 2026-03-22 UTC
- **Binary:** `target\release\competitor-spy.exe` (built with `cargo build --release -p competitor_spy_cli`)
- **Rust version:** 1.92.0 (stable)
- **Command:**
  ```
  .\target\release\competitor-spy.exe --industry "yoga studio" --location "Amsterdam, Netherlands" --radius 10 --output-dir "docs\evidence\sessions" --log-level info
  ```
- **Exit code:** 0
- **Geocoding:** Nominatim resolved "Amsterdam, Netherlands" → lat=52.3730796, lon=4.8924534
- **Adapter results:** nominatim success (0 records), osm_overpass HTTP_4XX 404 (pre-fix), yelp/google ADAPTER_CONFIG_MISSING
- **Terminal output:** rendered with "(no competitors found)" + failed sources footer
- **PDF:** written to `docs/evidence/sessions/competitor_spy_report_20260322_055014_UTC.pdf`

Note: The Overpass 404 observed in the live run was the known bug (base URL missing `/interpreter`). This was fixed and regressed in commit `c07d2a1` before Stage 5 closure.

---

## Acceptance Test Coverage (all environments — in-process mock)

All acceptance tests (AS-001 through AS-005) run via `cargo test` and exercise the full `run_with_urls()` call path with in-process wiremock. These tests pass on Windows 11 x86_64 and are independent of network or OS-specific rendering.

| Test | Environment | Result |
|---|---|---|
| AS-001 — happy path, both outputs | Windows 11 x86_64 | PASS |
| AS-002 — one adapter fails, exit 0 | Windows 11 x86_64 | PASS |
| AS-003 — invalid radius, exit 1 | Windows 11 x86_64 | PASS |
| AS-004 — geocoding no results, exit 1 | Windows 11 x86_64 | PASS |
| AS-005 — all adapters fail, exit 0 | Windows 11 x86_64 | PASS |

---

## Release Risk: Linux x86_64 Not Tested

**Risk level:** Medium

**Description:** The binary has not been built or executed on Linux x86_64. The codebase uses only cross-platform Rust crates (`reqwest`, `printpdf`, `tokio`, `age`, `clap`). No platform-specific system calls are used. The credential store path resolves via `$HOME/.config/competitor-spy/credentials` on non-Windows targets (handled in `runner.rs`).

**Known portability concerns:**
1. Path separator: `runner.rs` uses `format!()` with `/` for the credential store path on Linux (correct). Windows uses `%APPDATA%` via `env::var("APPDATA")`.
2. PDF output: `printpdf` is pure Rust, no system binary. Expected to work cross-platform.
3. OpenTelemetry subscriber: `tracing_subscriber` is cross-platform.

**Post-release validation steps:**
1. Build on Linux: `cargo build --release -p competitor_spy_cli`
2. Run with canonical query: `./competitor-spy --industry "yoga studio" --location "Amsterdam, Netherlands" --radius 10`
3. Confirm exit 0, PDF written, terminal output rendered.
4. Update this matrix with result and evidence artifact path.
