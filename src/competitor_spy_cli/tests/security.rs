// security.rs — T-038, T-041
// Security regression tests for the competitor_spy_cli runner.
//
// T-038 (SEC-006): credential_store_path() must not silently fall back to a
//   CWD-relative path when APPDATA / HOME environment variables are unset.
//   FORMAL_SPEC.md §6.4: location is %APPDATA%\competitor-spy\credentials (Windows)
//   or ~/.config/competitor-spy/credentials (Linux).  CWD is not an acceptable fallback.
//
// T-041 (SEC-005): run_with_urls() must reject --output-dir values that contain
//   path traversal sequences (../) and return exit code 1.
//   07_QUALITY_DIMENSIONS.md §6: "Input validation at every boundary."
//
// RED STATE NOTES
//   T-038: credential_store_path() currently returns a CWD-relative path when
//     env vars are absent → path.is_absolute() is false → assertion FAILS (red).
//     When the fix changes the return type to Result<PathBuf, E>, update this
//     test to call .expect_err("...") on the result instead.
//   T-041: no traversal guard exists → run_with_urls() reaches PDF writing,
//     the PDF write fails silently, and exit code 0 is returned →
//     assert_eq!(exit, 1) FAILS (red).

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

use competitor_spy_cli::runner::{credential_store_path, run_with_urls, AdapterUrls};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

// ── Env-var lock ──────────────────────────────────────────────────────────────
// Env-var mutation must be serialised across threads to avoid races with other
// tests that run in the same process.
static ENV_LOCK: Mutex<()> = Mutex::new(());

// ── Mock response helpers ─────────────────────────────────────────────────────

fn geocode_ok_body() -> serde_json::Value {
    serde_json::json!([{
        "lat": "52.3676",
        "lon": "4.9041",
        "display_name": "Amsterdam, Netherlands",
        "osm_id": 271110,
        "osm_type": "relation",
        "name": "Amsterdam",
        "addresstype": "city",
        "importance": 0.9
    }])
}

fn overpass_empty() -> serde_json::Value {
    serde_json::json!({ "elements": [] })
}

fn yelp_empty() -> serde_json::Value {
    serde_json::json!({ "businesses": [], "total": 0 })
}

fn google_empty() -> serde_json::Value {
    serde_json::json!({ "places": [] })
}

fn all_at(server: &MockServer) -> AdapterUrls {
    AdapterUrls {
        nominatim: server.uri(),
        osm_overpass: format!("{}/interpreter", server.uri()),
        yelp: server.uri(),
        google_places: server.uri(),
    }
}

// ── T-038 ─────────────────────────────────────────────────────────────────────

/// SEC-006 / T-038 — FORMAL_SPEC.md §6.4
///
/// When both the `APPDATA` (Windows) and `HOME` (Unix) environment variables
/// are absent, `credential_store_path()` must NOT silently fall back to a
/// CWD-relative path.  Writing the credential store to `.` risks exposing it
/// in a world-readable working directory.
///
/// RED STATE: returns PathBuf::from(".\competitor-spy\credentials") (relative)
///   → path.is_absolute() is false → assertion FAILS.
///
/// GREEN STATE: fix changes return type to Result<PathBuf, E>; update test to:
///   `credential_store_path().expect_err("should fail when home vars absent");`
///
/// SAFETY of env-var mutation: guarded by ENV_LOCK (single writer at a time).
/// Run with `cargo test -- --test-threads=1` if other tests also mutate env.
#[test]
fn credential_store_path_does_not_fall_back_to_cwd_when_home_vars_absent() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    let saved_appdata = std::env::var("APPDATA").ok();
    let saved_home = std::env::var("HOME").ok();

    // SAFETY: guarded by ENV_LOCK; no other thread mutates APPDATA/HOME concurrently.
    unsafe {
        std::env::remove_var("APPDATA");
        std::env::remove_var("HOME");
    }

    let result = credential_store_path();

    // Restore immediately — must happen before any assertion so that a panic
    // here does not permanently corrupt the env for the rest of the test run.
    unsafe {
        if let Some(v) = saved_appdata {
            std::env::set_var("APPDATA", v);
        }
        if let Some(v) = saved_home {
            std::env::set_var("HOME", v);
        }
    }

    assert!(
        result.is_err(),
        "SEC-006: credential_store_path() returned Ok({:?}) instead of Err when \
         APPDATA and HOME were both unset.\n\
         Hint: use .map_err() to propagate a descriptive error when env var lookup fails.",
        result.as_ref().map(|p| p.display().to_string()).unwrap_or_default()
    );
}

// ── T-041 ─────────────────────────────────────────────────────────────────────

/// SEC-005 / T-041 — 07_QUALITY_DIMENSIONS.md §6
///
/// `--output-dir` values containing path traversal sequences (`../`) must be
/// rejected at the CLI boundary before any I/O is performed.  `run_with_urls()`
/// must return exit code 1 when such a path is supplied.
///
/// RED STATE: no traversal guard exists → geocoding and adapter calls complete
///   successfully, PDF write to the traversal path is attempted (and fails
///   silently), run returns exit code 0 → assert_eq!(exit, 1) FAILS.
///
/// GREEN STATE: fix adds an early validation step at the top of run_with_urls()
///   that detects traversal components in the resolved output path and returns
///   1 immediately, before any network calls.
///
/// The mock server is set up to allow the full run to complete in the red state
/// so the test only fails on the exit-code assertion, not on a network error.
#[tokio::test]
async fn output_dir_traversal_path_is_rejected() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/search"))
        .respond_with(ResponseTemplate::new(200).set_body_json(geocode_ok_body()))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/interpreter"))
        .respond_with(ResponseTemplate::new(200).set_body_json(overpass_empty()))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/v3/businesses/search"))
        .respond_with(ResponseTemplate::new(200).set_body_json(yelp_empty()))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/v1/places:searchText"))
        .respond_with(ResponseTemplate::new(200).set_body_json(google_empty()))
        .mount(&server)
        .await;

    // Canonical traversal paths for each platform.
    #[cfg(windows)]
    let traversal_dir = PathBuf::from("..\\..\\Windows\\Temp");
    #[cfg(not(windows))]
    let traversal_dir = PathBuf::from("../../tmp");

    let exit = run_with_urls(
        "yoga studio",
        "Amsterdam, Netherlands",
        10,
        Some(traversal_dir),
        false, // no_pdf = false: force PDF output path to be evaluated
        true,  // detail
        all_at(&server),
        HashMap::new(),
        true,  // no_enrichment: avoid live HTTP enrichment calls
        false, // allow_insecure_tls
        15,    // enrichment_timeout_secs
        None,  // pacing_seed
    )
    .await;

    assert_eq!(
        exit, 1,
        "SEC-005: run_with_urls() accepted a path-traversal output-dir and returned exit 0.\n\
         Hint: validate --output-dir with a canonicalize() / component scan at the START \
         of run_with_urls() and return 1 on any path containing '..' components."
    );
}
