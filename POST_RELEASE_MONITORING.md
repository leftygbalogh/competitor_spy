# Post-Release Monitoring Plan — Competitor Spy v1.0

---

## Monitoring Windows

| Window | Duration | Focus |
|---|---|---|
| Immediate | First 3 live runs after release | Data quality, adapter health, credential flow |
| Short-term | First 2 weeks | Nominatim/Overpass availability, PDF output on varied inputs |
| Ongoing | Per run | Exit codes, adapter failure rates, report completeness |

---

## Signals to Monitor

| Signal | Threshold | Action |
|---|---|---|
| Nominatim HTTP 403 | Any occurrence | Investigate user-agent; update if Nominatim policy changed |
| Overpass HTTP 404 | Any occurrence | Verify `/api/interpreter` endpoint URL still correct |
| Overpass HTTP 503 | > 2 per session | Check Overpass API status; consider alternate mirror |
| Yelp/Google HTTP 429 | Any occurrence | Key may be rate-limited; check quota; increase pacing |
| Exit code 1 on valid input | Any occurrence | Investigate geocoding or render failure; file defect |
| PDF file size < 500 bytes | Any occurrence | printpdf regression or empty write; check output |
| Zero competitors on known-good location | Any occurrence | Adapter degradation; run diagnostics |

---

## Defect Capture and Response

1. Capture full session log: `scripts/capture_session.ps1` (Windows) or `scripts/capture_session.sh` (Linux).
2. Store in `docs/evidence/sessions/` with descriptive label.
3. File defect: describe symptom, expected, actual, session log path.
4. Convert defect to regression test before closing (per Stage 5 escaped-defect rule).
5. Link regression test to defect ID in `docs/evidence/traceability.md`.

---

## Linux / Untested Environment Monitoring

Linux x86_64 was not validated in v1.0. When first deployed on Linux:
1. Run the full test suite: `cargo test --workspace` — must be 198/198.
2. Run canonical validation: yoga studio / Amsterdam / 10 km.
3. Record result in `docs/evidence/environment-matrix.md`.
4. If failures: file defect, follow defect capture procedure above.

---

## Observability Notes

- All runs emit OpenTelemetry structured logs. Use `--log-level debug` to see per-adapter request/response detail.
- Audit log events include: `run_start`, `geocoding_attempt`, `geocoding_result`, `adapter_start`, `adapter_result`, `normalization_complete`, `ranking_complete`, `render_terminal_complete`, `render_pdf_complete` / `render_pdf_failed`, `run_complete`.
- Secrets are redacted before log emission. Never appear in any log or report.
