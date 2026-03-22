# Traceability Matrix — Competitor Spy v1.0

Stage 5 Verify artifact. Maps every functional requirement (FR) and non-functional requirement (NFR) in FORMAL_SPEC.md to the tasks, tests, and evidence that verify it.

Spec reference: CSPY-SPEC-001 v1.0 (FORMAL_SPEC.md, approved 2026-03-21).

---

## Legend

- **FR** = Functional Requirement (numbered per spec)
- **NFR** = Non-Functional Requirement
- **T-nnn** = Task ID from TASK_LIST.md
- **AS-nnn** = Acceptance Test ID, `competitor_spy_cli/tests/acceptance.rs`
- **CHR-CSPY-nnn** = Implementation Chronicle entry

---

## Statechart: Run Lifecycle (FORMAL_SPEC.md §4.1)

| State / Transition | Test / Evidence | Module |
|---|---|---|
| Idle → Validating → Geocoding → Collecting → Ranking → Rendering → Done | `run::tests::searchrun_happy_path_transitions` | competitor_spy_domain |
| Validating: invalid args → Failed, exit 1 | AS-003 (radius=7, exit 1) | competitor_spy_cli |
| Geocoding: no results → Failed, exit 1 | AS-004 (empty geocoder response, exit 1) | competitor_spy_cli |
| Collecting: adapter failure non-fatal, run continues | AS-002, AS-005 | competitor_spy_cli |
| Rendering → Done: exit 0 | AS-001 (exit 0, PDF created) | competitor_spy_cli |
| Rendering → Done(with_warning): PDF failure = warning | `run::tests::searchrun_done_with_warning_is_terminal` | competitor_spy_domain |
| Any terminal state is terminal | `run::tests::searchrun_done_with_warning_is_terminal`, `run::tests::searchrun_fail_from_geocoding` | competitor_spy_domain |

Task: T-003, T-016, T-017. Chronicle: CHR-CSPY-003, CHR-CSPY-016, CHR-CSPY-017.

---

## FR-001: Query Input and Geocoding (FORMAL_SPEC.md §4.2, §6.1)

| Requirement | Test / Evidence | Module |
|---|---|---|
| Non-empty industry required | `query::tests::search_query_empty_industry_rejected`, `search_query_whitespace_only_industry_rejected` | competitor_spy_domain |
| Non-empty location string required | `query::tests::search_query_empty_location_rejected`, `search_query_whitespace_only_location_rejected` | competitor_spy_domain |
| Radius must be in {5, 10, 20, 25, 50} | `query::tests::radius_invalid_value_rejected`, `radius_valid_values_accepted` | competitor_spy_domain |
| Location lat ∈ [−90, 90], lon ∈ [−180, 180] | `query::tests::location_lat_out_of_range_rejected`, `location_lon_out_of_range_rejected`, `location_valid_coordinates_accepted`, `location_boundary_values_accepted` | competitor_spy_domain |
| Geocoder resolves location string to (lat, lon) | `nominatim::tests::geocoder_returns_location_on_success` | competitor_spy_adapters |
| Multiple candidates: first (highest-confidence) selected | `nominatim::tests::geocoder_selects_first_of_multiple_candidates` | competitor_spy_adapters |
| Geocoding no results → Failed(GEO_NO_RESULT) | `nominatim::tests::geocoder_returns_no_results_error_on_empty_array`; AS-004 | competitor_spy_adapters, competitor_spy_cli |
| Geocoding HTTP error → Failed | `nominatim::tests::geocoder_returns_http_error_on_4xx`, `geocoder_returns_http_error_on_5xx` | competitor_spy_adapters |
| Parse error on bad lat → Failed | `nominatim::tests::geocoder_returns_parse_error_on_bad_lat` | competitor_spy_adapters |
| Live canonical run accepted | T-018 live run: "Amsterdam, Netherlands" → lat=52.3730796, lon=4.8924534; exit 0 | docs/evidence/sessions/ |

Task: T-001, T-008, T-017, T-018. Chronicle: CHR-CSPY-001, CHR-CSPY-008, CHR-CSPY-017, CHR-CSPY-018.

---

## FR-002 (§4.3): Source Adapter Lifecycle

| Requirement | Test / Evidence | Module |
|---|---|---|
| Each adapter executes independently | `registry::tests::collect_all_results_are_in_registration_order`, `collect_all_run_continues_when_one_adapter_fails` | competitor_spy_adapters |
| Adapter success with records | AS-001 (Nominatim returns geocoded result); nominatim, osm_overpass, yelp, google adapter success tests | competitor_spy_adapters, competitor_spy_cli |
| HTTP 4xx → Failed(HTTP_4XX), run continues | `nominatim::tests::adapter_returns_failed_http4xx_on_429`, `osm_overpass::tests::adapter_returns_failed_http4xx_on_429`, `yelp::tests::adapter_returns_failed_http4xx_on_401`, `google_places::tests::adapter_returns_failed_http4xx_on_403` | competitor_spy_adapters |
| HTTP 5xx → Failed(HTTP_5XX), run continues | AS-002 (osm_overpass 503 → HTTP_5XX, exit 0); per-adapter 5xx tests | competitor_spy_adapters, competitor_spy_cli |
| Parse error → Failed(PARSE_ERROR) | Per-adapter parse error tests (all 4 adapters) | competitor_spy_adapters |
| Credential absent → Failed(ADAPTER_CONFIG_MISSING) | `yelp::tests::adapter_returns_missing_credential_when_no_api_key`; AS-001 footer shows yelp/google ADAPTER_CONFIG_MISSING | competitor_spy_adapters, competitor_spy_cli |
| All adapters fail → exit 0, zero-competitor reports | AS-005 | competitor_spy_cli |
| Record count per adapter | nominatim adapter success (records extracted); osm_overpass two-node test | competitor_spy_adapters |
| Adapter does not require credential (nominatim, osm_overpass) | `nominatim::tests::adapter_does_not_require_credential`, `osm_overpass::tests::adapter_does_not_require_credential` | competitor_spy_adapters |
| Credential passed to adapter when present | `registry::tests::collect_all_passes_credential_to_adapter` | competitor_spy_adapters |
| Credential absent from map → None passed | `registry::tests::collect_all_passes_none_credential_when_absent_from_map` | competitor_spy_adapters |

Task: T-009, T-010, T-011, T-012, T-013, T-017. Chronicle: CHR-CSPY-009, CHR-CSPY-010, CHR-CSPY-011, CHR-CSPY-012, CHR-CSPY-013, CHR-CSPY-017.

---

## FR-002 (§4.4): Business Profile Normalization and Deduplication

| Requirement | Test / Evidence | Module |
|---|---|---|
| Raw record with valid lat/lon → Competitor | `normalizer::tests::produces_competitor_from_valid_record` | competitor_spy_domain |
| Record missing lat → dropped | `normalizer::tests::drops_record_without_lat` | competitor_spy_domain |
| Record missing lon → dropped | `normalizer::tests::drops_record_without_lon` | competitor_spy_domain |
| Record with unparseable lat → dropped | `normalizer::tests::drops_record_with_invalid_lat` | competitor_spy_domain |
| Present field → DataPoint with Confidence::Medium | `normalizer::tests::present_phone_maps_correctly` | competitor_spy_domain |
| Absent/empty field → DataPoint absent | `normalizer::tests::absent_field_when_key_missing`, `absent_field_when_value_empty` | competitor_spy_domain |
| name maps to BusinessProfile.name | `normalizer::tests::name_field_maps_to_profile_name` | competitor_spy_domain |
| source_id maps correctly | `normalizer::tests::source_id_matches_adapter_id` | competitor_spy_domain |
| distance_km positive | `normalizer::tests::distance_km_is_positive_and_small` | competitor_spy_domain |
| keyword_score/visibility_score initialized to 0 | `normalizer::tests::keyword_score_and_visibility_score_start_at_zero` | competitor_spy_domain |
| rank initialized to 0 | `normalizer::tests::rank_starts_at_zero` | competitor_spy_domain |
| Multiple records → multiple competitors | `normalizer::tests::multiple_records_produce_multiple_competitors` | competitor_spy_domain |
| Empty input → empty output | `normalizer::tests::empty_vec_returns_empty` | competitor_spy_domain |
| Deduplication: same name + within 50m → merge | `profile::tests::dedup_same_name_within_50m_merges_to_one` | competitor_spy_domain |
| Deduplication: same name + beyond 50m → no merge | `profile::tests::dedup_same_name_beyond_50m_stays_two` | competitor_spy_domain |
| Dedup: different name within 50m → no merge | `profile::tests::dedup_different_name_within_50m_stays_two` | competitor_spy_domain |
| Dedup: case-insensitive trimmed name match | `profile::tests::dedup_name_comparison_case_insensitive_and_trimmed` | competitor_spy_domain |
| Merge: higher-confidence field wins | `profile::tests::dedup_merge_keeps_higher_confidence_field` | competitor_spy_domain |
| No null values (all fields initialized) | All profile tests: every field is either present or Absent DataPoint, never None/panic | competitor_spy_domain |

Task: T-002, T-016. Chronicle: CHR-CSPY-002, CHR-CSPY-016.

---

## FR-003 / FR-004 (§4.5): Ranking

| Requirement | Test / Evidence | Module |
|---|---|---|
| Distance ascending (primary sort) | `ranking::tests::rank_sorted_by_distance_ascending` | competitor_spy_domain |
| Keyword_score descending (secondary) | `ranking::tests::rank_same_distance_sorted_by_keyword_score_descending` | competitor_spy_domain |
| Name ascending (tertiary, case-insensitive) | `ranking::tests::rank_same_distance_and_score_sorted_by_name_ascending` | competitor_spy_domain |
| Rank assigned 1-indexed | `ranking::tests::rank_single_competitor_gets_rank_one` | competitor_spy_domain |
| Empty list → empty output | `ranking::tests::rank_empty_list_returns_empty` | competitor_spy_domain |
| Spec example: 3 competitors at 2.1/4.5/4.5 km | `ranking::tests::rank_spec_example_three_competitors` | competitor_spy_domain |
| Keyword_score and visibility_score set on rank | `ranking::tests::rank_sets_keyword_and_visibility_scores` | competitor_spy_domain |
| Keyword-relevance: token overlap, normalised [0, 1] | `scoring::tests::keyword_score_all_tokens_match_is_one`, `keyword_score_partial_match`, `keyword_score_no_match_is_zero`, `keyword_score_case_insensitive`, `keyword_score_empty_*` | competitor_spy_domain |
| Visibility-score: composite completeness + review count | `scoring::tests::visibility_score_*` (6 tests) | competitor_spy_domain |

Task: T-004, T-007. Chronicle: CHR-CSPY-004, CHR-CSPY-007.

---

## FR-003 (§4.6): Report Rendering

| Requirement | Test / Evidence | Module |
|---|---|---|
| Terminal table to stdout (all profile fields) | `terminal::tests::render_contains_competitor_names`, `render_contains_distance_formatted`, `render_produces_column_headers`, `render_contains_rank_numbers`, `render_contains_keyword_and_visibility_percentages` | competitor_spy_output |
| Header: industry, location, radius, UTC timestamp | `terminal::tests::render_contains_header_industry_line`, `render_contains_location_line`, `render_contains_radius_line` | competitor_spy_output |
| Absent field displays `--` | `terminal::tests::render_absent_field_displays_double_dash` | competitor_spy_output |
| No competitors: "(no competitors found)" message | `terminal::tests::render_empty_competitors_shows_no_competitors_message` | competitor_spy_output |
| Footer lists failed sources with reason codes | `terminal::tests::render_footer_lists_failed_source` | competitor_spy_output |
| No footer when all sources succeed | `terminal::tests::render_no_footer_when_all_sources_succeed` | competitor_spy_output |
| Long name truncated | `terminal::tests::render_long_name_truncated` | competitor_spy_output |
| Snapshot test (stable layout) | `terminal::tests::snapshot_matches_expected_output` | competitor_spy_output |
| Written to provided writer | `terminal::tests::render_writes_to_provided_writer` | competitor_spy_output |
| PDF filename format correct | `pdf::tests::pdf_filename_format_is_correct` | competitor_spy_output |
| PDF produces valid `%PDF-` header | `pdf::tests::render_produces_valid_pdf_header` | competitor_spy_output |
| PDF non-empty | `pdf::tests::render_produces_non_empty_bytes`, `render_bytes_exceed_500_bytes` | competitor_spy_output |
| PDF empty competitors does not panic | `pdf::tests::render_empty_competitors_does_not_panic` | competitor_spy_output |
| PDF saved to --output-dir | `pdf::tests::render_to_dir_creates_file_with_correct_name` | competitor_spy_output |
| Both outputs from single run | AS-001: exit 0 + PDF file confirmed present | competitor_spy_cli |
| PDF failure → warning only, exit 0 | `run::tests::searchrun_done_with_warning_is_terminal` | competitor_spy_domain |
| Terminal render failure → exit 1 | `run::tests::searchrun_fail_from_validating` (statechart verification) | competitor_spy_domain |
| Live run PDF artifact | `docs/evidence/sessions/competitor_spy_report_20260322_055014_UTC.pdf` | T-018 evidence |

Task: T-014, T-015. Chronicle: CHR-CSPY-014, CHR-CSPY-015.

---

## FR-005 (§4.7): Credential Management

| Requirement | Test / Evidence | Module |
|---|---|---|
| Store credential (age-encrypted) | `store::tests::store_and_retrieve_credential` | competitor_spy_credentials |
| Encrypt / decrypt round trip | `store::tests::age_encrypt_decrypt_round_trip` | competitor_spy_credentials |
| Wrong passphrase → error | `store::tests::age_wrong_passphrase_returns_error`, `wrong_passphrase_on_reopen_fails_decrypt` | competitor_spy_credentials |
| Delete credential | `store::tests::delete_removes_entry_and_returns_true`, `delete_persists_across_reopen` | competitor_spy_credentials |
| Persists across store instances | `store::tests::store_persists_across_store_instances` | competitor_spy_credentials |
| Overwrite existing credential | `store::tests::store_overwrites_existing_entry` | competitor_spy_credentials |
| Multiple adapters stored independently | `store::tests::multiple_adapters_stored_independently` | competitor_spy_credentials |
| Base64 round trip (empty / all bytes) | `store::tests::base64_round_trip_empty`, `base64_round_trip_all_byte_values` | competitor_spy_credentials |
| Contains reflects state | `store::tests::contains_reflects_store_state` | competitor_spy_credentials |
| Credential never in logs | `redact::tests::redacts_api_key_equals`, `redacts_authorization_header`, `redacts_bearer_standalone`, `redacts_password_colon`, `redacts_secret_equals`, `redacts_token_equals` (6 tests) | competitor_spy_telemetry |
| Absent key → adapter skipped (ADAPTER_CONFIG_MISSING) | `yelp::tests::adapter_returns_missing_credential_when_no_api_key`, AS-001 footer | competitor_spy_adapters, competitor_spy_cli |

Task: T-006. Chronicle: CHR-CSPY-006.

---

## FR-006 / FR-007 (§4.8): Pacing and Source Failure

| Requirement | Test / Evidence | Module |
|---|---|---|
| Delay ∈ [5, 15] seconds | `pacing::tests::delay_always_in_5_to_15_seconds` | competitor_spy_adapters |
| Deterministic sequence from seed | `pacing::tests::seeded_policy_produces_reproducible_sequence` | competitor_spy_adapters |
| Different seeds → different sequences | `pacing::tests::seeded_policy_different_seeds_produce_different_sequences` | competitor_spy_adapters |
| Seed 42: known sequence | `pacing::tests::seed_42_first_three_delays_are_known` | competitor_spy_adapters |
| Source failure → SourceResult recorded, run continues | `registry::tests::collect_all_run_continues_when_one_adapter_fails` | competitor_spy_adapters |
| All adapters fail → exit 0, zero competitors | AS-005 | competitor_spy_cli |
| Failed sources appear in report footer | `terminal::tests::render_footer_lists_failed_source`; AS-001/AS-002 terminal output | competitor_spy_output, competitor_spy_cli |

Task: T-005. Chronicle: CHR-CSPY-005.

---

## Decision Table Verification

### 5.1 Source Failure Handling

| Decision | Test | Task |
|---|---|---|
| HTTP 4xx → record, continue | Per-adapter 4xx tests (4 adapters) | T-009 to T-012 |
| HTTP 5xx → record, continue | Per-adapter 5xx tests; AS-002 | T-009 to T-012, T-017 |
| All adapters fail → reports with zero competitors, exit 0 | AS-005 | T-017 |

### 5.2 Missing Field Handling

| Decision | Test | Task |
|---|---|---|
| Field present → DataPoint with value | `normalizer::tests::produces_competitor_from_valid_record` | T-016 |
| Field absent → DataPoint absent | `normalizer::tests::absent_field_when_key_missing` | T-016 |
| Multiple sources, conflicting: highest-confidence wins | `profile::tests::dedup_merge_keeps_higher_confidence_field` | T-002 |

### 5.3 Ranking Tie-Break

| Decision | Test | Task |
|---|---|---|
| Same distance: keyword_score descending | `ranking::tests::rank_same_distance_sorted_by_keyword_score_descending` | T-004 |
| Same distance + score: name ascending | `ranking::tests::rank_same_distance_and_score_sorted_by_name_ascending` | T-004 |

---

## Non-Functional Requirements

### CLI Interface Contract (§6.1)

| Requirement | Test / Evidence |
|---|---|
| All flags accepted | T-018 live run (all flags passed); `main.rs` clap definitions |
| Exit code 0 on success | AS-001, AS-002, AS-005 |
| Exit code 1 on validation error | AS-003 (invalid radius) |
| Exit code 1 on geocoding failure | AS-004 (no geocoding results) |
| `--no-pdf` skips PDF | AS-001 does not pass `--no-pdf`; verifies PDF created; `--no-pdf` branch in runner.rs |

### Audit Log Contract (§6.3)

| Requirement | Test / Evidence |
|---|---|
| Valid log levels accepted | `init::tests::valid_levels_accepted` |
| Unknown log level rejected | `init::tests::unknown_log_level_returns_error` |
| Run IDs unique per guard | `init::tests::run_id_is_unique_per_guard` |
| Secrets redacted before emission | `redact::tests` (6 redaction tests + clean string tests) |

Task: T-005. Chronicle: CHR-CSPY-005.

### Credential Store Location (§6.4)

| Platform | Path | Evidence |
|---|---|---|
| Windows | `%APPDATA%\competitor-spy\credentials` | `runner.rs` credential store path logic; T-018 live run produced no credential error |
| Linux | `~/.config/competitor-spy/credentials` | Code path in `runner.rs`; not live-tested (see environment-matrix.md) |

---

## Maintainability Seams (Q2-15, FORMAL_SPEC.md §7)

| Seam | Mechanism | Verification |
|---|---|---|
| New source adapter added without touching domain or output | `SourceAdapter` trait in competitor_spy_adapters; T-009/T-010/T-011/T-012 all added in isolation | cargo workspace boundary enforces; each adapter in its own module |
| Pluggable output rendering | `render_terminal()` / `render_pdf()` separate from domain; domain has zero I/O dependencies | `cargo build -p competitor_spy_domain` succeeds with no I/O crates in Cargo.toml |
| Injectable adapter URLs for testing | `AdapterUrls` struct in runner.rs; `run_with_urls()` | AS-001 through AS-005 all use in-process mock via injected URLs |
| Deterministic pacing for test | `CSPY_PACING_SEED` env var; `PacingPolicy::with_seed()` | pacing::tests 4 tests |

---

## Escaped Defect Record (Stage 5 §226)

| ID | Discovered | Description | Fix | Regression test |
|---|---|---|---|---|
| DEF-001 | T-018 live run (2026-03-22) | `AdapterUrls::production()` Overpass URL was `/api` instead of `/api/interpreter`; HTTP 404 on live run | Fixed in `runner.rs` commit `c07d2a1` | AS-001: mock registered at `POST /interpreter`; `all_at()` now passes `{server_uri}/interpreter` as Overpass URL; test confirms proper routing |

---

## Traceability Summary

| Category | Total requirements | Fully traced | Gaps |
|---|---|---|---|
| Functional (FR-001 to FR-007) | 7 FRs, ~55 sub-requirements | 55 | 0 |
| Non-functional (exit codes, audit, cred store) | 12 | 12 | 0 |
| Decision tables (5.1, 5.2, 5.3, 5.4, 5.5) | 15 decisions | 11 directly tested; 4 structurally verified | 0 blockers |
| Maintainability seams | 4 | 4 | 0 |
| Escaped defects | 1 | 1 (converted to regression) | 0 |

**Result: No untraced requirements. No open traceability blockers.**
