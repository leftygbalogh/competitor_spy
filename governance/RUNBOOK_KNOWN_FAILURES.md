# Runbook — Known Failure Scenarios — Competitor Spy v1.0

---

## RUN-001: Exit Code 1 — Invalid radius

**Symptom:** `error: invalid value '7' for '--radius <5|10|20|25|50>'`

**Cause:** Radius must be one of 5, 10, 20, 25, or 50 km.

**Resolution:** Re-run with a valid radius. Example: `--radius 10`

---

## RUN-002: Exit Code 1 — Geocoding failure (GEO_NO_RESULT)

**Symptom:** `[ERROR] geocoding failed: no results returned for location "..."`

**Cause:** Nominatim could not resolve the location string.

**Resolution:**
1. Check the location string is spelled correctly and is a real place.
2. Try a more specific location: city + country (e.g. "Berlin, Germany").
3. Check internet connectivity.
4. Verify Nominatim is reachable: `curl -A "competitor-spy/1.0" "https://nominatim.openstreetmap.org/search?q=Amsterdam&format=json"`

---

## RUN-003: Nominatim HTTP 403

**Symptom:** `adapter_result adapter_id="nominatim" outcome="http_4xx" code=403`

**Cause:** Nominatim rejected the User-Agent. Nominatim policy prohibits `example.com` addresses. This was fixed in v1.0 (user-agent changed to `competitor-spy/1.0 contact:competitor-spy@pm.me`). If 403 recurs after an update, the user-agent may have been inadvertently changed.

**Resolution:** Verify user-agent in `competitor_spy_adapters/src/nominatim.rs` — both `NominatimGeocoder::new()` and `NominatimAdapter::new()` — is not using an `example.com` address.

---

## RUN-004: OSM Overpass HTTP 404

**Symptom:** `adapter_result adapter_id="osm_overpass" outcome="http_4xx" code=404`

**Cause:** The Overpass endpoint URL is wrong. The correct URL is `https://overpass-api.de/api/interpreter` (not `/api`). This was fixed in v1.0 (DEF-001).

**Resolution:** Verify `AdapterUrls::production()` in `competitor_spy_cli/src/runner.rs` has `osm_overpass: "https://overpass-api.de/api/interpreter"`.

---

## RUN-005: All adapters show ADAPTER_CONFIG_MISSING

**Symptom:** Footer shows `yelp: ADAPTER_CONFIG_MISSING` and `google_places: ADAPTER_CONFIG_MISSING`. Exit code is still 0.

**Cause:** No API credentials are stored for Yelp/Google. OSM/Nominatim do not require credentials, so these always attempt.

**Resolution:** This is expected behaviour for initial runs without credentials. Results from OSM/Nominatim will still appear. To add Yelp/Google credentials, set `CSPY_CREDENTIAL_PASSPHRASE` and run — you will be prompted on stderr.

---

## RUN-006: No competitors found ("(no competitors found)")

**Symptom:** Report renders with `(no competitors found)`. Exit code 0.

**Cause:** All adapters returned zero matching records, or all adapters failed.

**Resolution:**
1. Check the failed sources footer for adapter error codes.
2. Try a broader industry term (e.g. "gym" instead of "martial arts gym").
3. Try a larger radius (`--radius 20` or `--radius 50`).
4. Verify internet access and adapter URLs (see RUN-003, RUN-004).

---

## RUN-007: PDF write failure

**Symptom:** `[WARN] PDF render failed: ...`. Terminal output still produced. Exit code 0.

**Cause:** `--output-dir` path does not exist or is not writable.

**Resolution:**
1. Verify the path exists: `Test-Path <output-dir>` (PowerShell) or `ls <output-dir>` (Bash).
2. Create the directory if missing: `New-Item -ItemType Directory <output-dir>`.
3. Check disk space and write permissions.

---

## RUN-008: Credential store unreadable

**Symptom:** `[WARN] credential store unreadable; skipping credential-requiring adapters`

**Cause:** The credential store file exists but is corrupted, or the wrong passphrase is supplied.

**Resolution:**
1. Verify `CSPY_CREDENTIAL_PASSPHRASE` is set correctly.
2. If the store is corrupted, delete it and re-enter credentials:
   - Windows: `Remove-Item "$env:APPDATA\competitor-spy\credentials"`
   - Linux: `rm ~/.config/competitor-spy/credentials`

---

## RUN-009: Overpass name-query timeout (public API congestion)

**Symptom:** Log lines:
```
WARN event="overpass_query" label="name" outcome="timeout"
WARN event="osm_overpass_name_query" outcome="ignored" reason=Timeout
```
Exit code 0. Tag-query results still appear (may be empty if the industry has no standard OSM tag value).

**Cause:** The public Overpass API (`overpass-api.de`) queues requests during high-load periods (typically European business hours and weekends). The name-regex query has a 35 s client-side deadline; if the server has not responded by then, the query is abandoned. This is a transient infrastructure condition.

**Impact:** Industries found only by business name (e.g., `"pilates"`, `"yoga studio"` in rural areas) may show 0 results when Overpass is congested, even if OSM data exists.

**Resolution:**
1. Retry during off-peak hours (late night UTC).
2. If the industry has a standard OSM tag value (e.g., `cafe`, `gym`), the tag query succeeds regardless — only the name fallback is affected.
3. For consistent sub-20 s latency in production, consider self-hosting an Overpass instance pointing at a local planet extract.

---

## RUN-010: No OSM coverage for niche industries in rural areas

**Symptom:** `record_count=0` even after retrying at off-peak hours.

**Cause:** OpenStreetMap community coverage varies by region. Rural towns in central/eastern Europe often lack detailed POI tagging. Businesses may exist on Google Maps or Yelp but have never been added to OSM.

**Impact:** Tools relying solely on OSM (no API credentials set) will report `(no competitors found)` for valid industries.

**Resolution:**
1. Validate OSM coverage directly at `https://www.openstreetmap.org` by searching the area.
2. Configure Yelp or Google Places credentials (see RUN-005) to add commercial-data sources that cover these gaps.
3. As a workaround, try the industry in English + the country language (e.g., `"pilates"` → OSM uses English names even in Austria where OSM coverage for pilates studios is sparse).

---

## Known Environment Gaps (see `docs/evidence/environment-matrix.md`)

| Environment | Status | Risk |
|---|---|---|
| Linux x86_64 | NOT TESTED in v1.0 | Medium — pure-Rust deps, no system binary calls; expected to work; validate before production use |
| macOS | OUT OF SCOPE | Not supported in v1.0 |

**Post-release Linux validation steps:**
1. `cargo build --release -p competitor_spy_cli`
2. `./target/release/competitor-spy --industry "yoga studio" --location "Amsterdam, Netherlands" --radius 10`
3. Confirm exit 0, PDF created, terminal output rendered.
4. Update `docs/evidence/environment-matrix.md` with result.
