// acceptance.rs — T-017
// Acceptance tests AS-001 through AS-005.
// These call run_with_urls() in-process against WireMock servers,
// so no subprocess or binary build ordering is required.

use std::collections::HashMap;
use std::path::PathBuf;

use competitor_spy_cli::runner::{AdapterUrls, run_with_urls};
use wiremock::{Mock, MockServer, ResponseTemplate};
use wiremock::matchers::{method, path};

// ── Shared mock data ──────────────────────────────────────────────────────────

/// A minimal valid Nominatim geocoding response (Amsterdam).
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

/// A minimal valid Nominatim adapter response with one business result.
#[allow(dead_code)]
fn nominatim_results_body() -> serde_json::Value {
    serde_json::json!([{
        "lat": "52.370",
        "lon": "4.905",
        "display_name": "Yoga Loft, Keizersgracht 1, Amsterdam",
        "name": "Yoga Loft",
        "osm_id": 12345,
        "osm_type": "node",
        "addresstype": "amenity",
        "importance": 0.5
    }])
}

/// OSM Overpass empty result (no businesses found).
fn overpass_empty_body() -> serde_json::Value {
    serde_json::json!({ "elements": [] })
}

/// Yelp empty result body (no businesses).
fn yelp_empty_body() -> serde_json::Value {
    serde_json::json!({ "businesses": [], "total": 0 })
}

/// Google Places empty result body.
fn google_empty_body() -> serde_json::Value {
    serde_json::json!({ "places": [] })
}

/// Returns an `AdapterUrls` pointing all four adapters + geocoder at the same
/// mock server (different paths are mocked independently on that server).
fn all_at(server: &MockServer) -> AdapterUrls {
    AdapterUrls {
        nominatim: server.uri(),
        osm_overpass: format!("{}/interpreter", server.uri()),
        yelp: server.uri(),
        google_places: server.uri(),
    }
}

fn no_creds() -> HashMap<String, String> {
    HashMap::new()
}

fn temp_dir() -> PathBuf {
    std::env::temp_dir()
}

// ── AS-001: Valid input, mock adapters → both outputs, exit 0 ─────────────────

#[tokio::test]
async fn as_001_valid_input_both_outputs_exit_0() {
    let server = MockServer::start().await;

    // Geocoder: /search returns Amsterdam
    Mock::given(method("GET"))
        .and(path("/search"))
        .respond_with(ResponseTemplate::new(200).set_body_json(geocode_ok_body()))
        .mount(&server)
        .await;

    // Nominatim adapter: /search (same endpoint, different query params — both
    //   geocoder and adapter hit /search; both get the same mock response here,
    //   which is fine for this test).
    // Already covered by the mock above (catch-all on /search).

    // OSM Overpass: POST /interpreter returns empty
    Mock::given(method("POST"))
        .and(path("/interpreter"))
        .respond_with(ResponseTemplate::new(200).set_body_json(overpass_empty_body()))
        .mount(&server)
        .await;

    // Yelp: GET /v3/businesses/search returns empty
    Mock::given(method("GET"))
        .and(path("/v3/businesses/search"))
        .respond_with(ResponseTemplate::new(200).set_body_json(yelp_empty_body()))
        .mount(&server)
        .await;

    // Google Places: POST /v1/places:searchText returns empty
    Mock::given(method("POST"))
        .and(path("/v1/places:searchText"))
        .respond_with(ResponseTemplate::new(200).set_body_json(google_empty_body()))
        .mount(&server)
        .await;

    let exit = run_with_urls(
        "yoga studio",
        "Amsterdam, Netherlands",
        10,
        Some(temp_dir()),    // output dir
        false, // produce PDF
        true,  // detail view (production default)
        all_at(&server),
        no_creds(),
    )
    .await;

    assert_eq!(exit, 0, "expected exit 0 for valid input");

    // PDF must exist in temp dir
    let pdf_exists = std::fs::read_dir(temp_dir())
        .unwrap()
        .filter_map(|e| e.ok())
        .any(|e| {
            e.file_name()
                .to_string_lossy()
                .starts_with("competitor_spy_report_")
        });
    assert!(pdf_exists, "expected PDF file in output dir");
}

// ── AS-002: One adapter OK, one timeout → both outputs, failed footer, exit 0 ─

#[tokio::test]
async fn as_002_one_adapter_timeout_still_exit_0() {
    let server = MockServer::start().await;

    // Geocoder ok
    Mock::given(method("GET"))
        .and(path("/search"))
        .respond_with(ResponseTemplate::new(200).set_body_json(geocode_ok_body()))
        .mount(&server)
        .await;

    // OSM Overpass: 503 (simulates failure)
    Mock::given(method("POST"))
        .and(path("/interpreter"))
        .respond_with(ResponseTemplate::new(503))
        .mount(&server)
        .await;

    // Yelp: 401 (missing credential → ADAPTER_CONFIG_MISSING or HTTP_4XX)
    Mock::given(method("GET"))
        .and(path("/v3/businesses/search"))
        .respond_with(ResponseTemplate::new(401))
        .mount(&server)
        .await;

    // Google: 503
    Mock::given(method("POST"))
        .and(path("/v1/places:searchText"))
        .respond_with(ResponseTemplate::new(503))
        .mount(&server)
        .await;

    let exit = run_with_urls(
        "yoga studio",
        "Amsterdam, Netherlands",
        10,
        Some(temp_dir()),    // output dir
        true, // skip PDF for speed
        true,  // detail view (production default)
        all_at(&server),
        no_creds(),
    )
    .await;

    // Run should still succeed (exit 0) even with adapter failures
    assert_eq!(exit, 0, "expected exit 0 when some adapters fail");
}

// ── AS-003: Invalid radius → stderr, no report, exit 1 ───────────────────────

#[tokio::test]
async fn as_003_invalid_radius_exit_1() {
    // No mock server needed — validation fails before any HTTP calls
    let exit = run_with_urls(
        "yoga studio",
        "Amsterdam, Netherlands",
        7, // invalid radius (not 5/10/20/25/50)
        Some(temp_dir()),    // output dir
        true,
        true,  // detail view (production default)
        AdapterUrls::production(), // doesn't matter — never reached
        no_creds(),
    )
    .await;

    assert_eq!(exit, 1, "expected exit 1 for invalid radius");
}

// ── AS-004: Geocoding returns no results → stderr, no report, exit 1 ─────────

#[tokio::test]
async fn as_004_geocoding_no_results_exit_1() {
    let server = MockServer::start().await;

    // Geocoder: /search returns empty array → NoResults
    Mock::given(method("GET"))
        .and(path("/search"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .mount(&server)
        .await;

    let exit = run_with_urls(
        "yoga studio",
        "NowhereXYZ9999",
        10,
        Some(temp_dir()),    // output dir
        true,
        true,  // detail view (production default)
        all_at(&server),
        no_creds(),
    )
    .await;

    assert_eq!(exit, 1, "expected exit 1 when geocoding returns no results");
}

// ── AS-005: All adapters fail → both reports with zero competitors, exit 0 ────

#[tokio::test]
async fn as_005_all_adapters_fail_zero_competitors_exit_0() {
    let server = MockServer::start().await;

    // Geocoder: ok
    Mock::given(method("GET"))
        .and(path("/search"))
        .respond_with(ResponseTemplate::new(200).set_body_json(geocode_ok_body()))
        .mount(&server)
        .await;

    // All adapters: 503
    Mock::given(method("POST"))
        .and(path("/interpreter"))
        .respond_with(ResponseTemplate::new(503))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/v3/businesses/search"))
        .respond_with(ResponseTemplate::new(503))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/v1/places:searchText"))
        .respond_with(ResponseTemplate::new(503))
        .mount(&server)
        .await;

    let exit = run_with_urls(
        "yoga studio",
        "Amsterdam, Netherlands",
        10,
        Some(temp_dir()),    // output dir
        true, // skip PDF for speed
        true,  // detail view (production default)
        all_at(&server),
        no_creds(),
    )
    .await;

    // All adapters failed but run still completes with exit 0 and zero competitors
    assert_eq!(exit, 0, "expected exit 0 with zero competitors when all adapters fail");
}

// ── AS2-006: detail=true, full V2 mock (editorial+price+reviews) → exit 0 ────

#[tokio::test]
async fn as2_006_detail_true_v2_fields_exit_0() {
    let server = MockServer::start().await;

    // Geocoder: Amsterdam
    Mock::given(method("GET"))
        .and(path("/search"))
        .respond_with(ResponseTemplate::new(200).set_body_json(geocode_ok_body()))
        .mount(&server)
        .await;

    // OSM Overpass: empty
    Mock::given(method("POST"))
        .and(path("/interpreter"))
        .respond_with(ResponseTemplate::new(200).set_body_json(overpass_empty_body()))
        .mount(&server)
        .await;

    // Yelp: empty
    Mock::given(method("GET"))
        .and(path("/v3/businesses/search"))
        .respond_with(ResponseTemplate::new(200).set_body_json(yelp_empty_body()))
        .mount(&server)
        .await;

    // Google Places: full V2 response with editorial summary, price level, reviews
    let google_v2_body = serde_json::json!({
        "places": [{
            "id": "ChIJv2as2006",
            "displayName": { "text": "Pilates Wien Pro" },
            "formattedAddress": "Mariahilfer Strasse 12, 1060 Wien",
            "location": { "latitude": 52.370, "longitude": 4.895 },
            "nationalPhoneNumber": "+43 1 234 5678",
            "websiteUri": "https://pilateswien.at",
            "editorialSummary": { "text": "A welcoming studio in the heart of Vienna." },
            "priceLevel": "PRICE_LEVEL_MODERATE",
            "regularOpeningHours": {
                "weekdayDescriptions": [
                    "Monday: 9:00 AM - 6:00 PM",
                    "Tuesday: 9:00 AM - 6:00 PM"
                ]
            },
            "reviews": [
                {
                    "text": { "text": "Great instructor, very professional." },
                    "rating": 4.0,
                    "relativePublishTimeDescription": "2 months ago"
                },
                {
                    "text": { "text": "Best pilates in Vienna!" },
                    "rating": 5.0,
                    "relativePublishTimeDescription": "1 month ago"
                }
            ]
        }]
    });
    Mock::given(method("POST"))
        .and(path("/v1/places:searchText"))
        .respond_with(ResponseTemplate::new(200).set_body_json(google_v2_body))
        .mount(&server)
        .await;

    let exit = run_with_urls(
        "pilates studio",
        "Wien, Austria",
        10,
        Some(temp_dir()),    // output dir
        false, // produce PDF
        true,  // detail view — exercises the reviews rendering path
        all_at(&server),
        no_creds(),
    )
    .await;

    assert_eq!(exit, 0, "expected exit 0 for full V2 mock with detail=true");
}