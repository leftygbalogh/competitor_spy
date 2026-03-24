// NominatimGeocoder + NominatimAdapter — T-009
// TDD: tests written alongside implementation; WireMock used to mock HTTP.
//
// Nominatim docs: https://nominatim.org/release-docs/develop/api/Search/
// Geocoder endpoint: GET /search?q=<encoded>&format=jsonv2&limit=5
// Adapter endpoint:  GET /search?q=<industry>&near=<lat,lon>&radius=<km>&format=jsonv2
//
// Geocoder selects the first result (highest position = highest confidence).
// Adapter produces one RawRecord per result item.

use async_trait::async_trait;
use serde::Deserialize;
use chrono::Utc;
use tracing::{info, warn};

use competitor_spy_domain::query::{Location, Radius, SearchQuery};
use competitor_spy_domain::run::{AdapterResultStatus, RawRecord, ReasonCode, SourceResult};

use crate::adapter::{Geocoder, GeocodingError, SourceAdapter};
use crate::pacing::PacingPolicy;

// ── Nominatim JSON response ────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct NominatimPlace {
    /// Latitude as a string in Nominatim's jsonv2 format.
    lat: String,
    /// Longitude as a string in Nominatim's jsonv2 format.
    lon: String,
    display_name: String,
    #[serde(rename = "osm_id")]
    osm_id: Option<u64>,
    #[serde(rename = "osm_type")]
    osm_type: Option<String>,
    #[serde(rename = "name")]
    name: Option<String>,
    #[serde(rename = "addresstype")]
    address_type: Option<String>,
    #[serde(rename = "importance")]
    importance: Option<f64>,
    // Extra tags map for extended info (present when addressdetails=1)
    #[serde(rename = "extratags")]
    extra_tags: Option<std::collections::HashMap<String, String>>,
}

// ── NominatimGeocoder ─────────────────────────────────────────────────────────

/// Geocodes a location string using the Nominatim Search API.
///
/// The base URL is configurable so tests can point at a WireMock server.
pub struct NominatimGeocoder {
    client: reqwest::Client,
    /// Base URL, e.g. `"https://nominatim.openstreetmap.org"` or
    /// `"http://localhost:8080"` in tests.
    base_url: String,
    pacing: PacingPolicy,
}

impl NominatimGeocoder {
    pub fn new(base_url: impl Into<String>) -> Self {
        let client = reqwest::Client::builder()
            .user_agent("competitor-spy/1.0 contact:competitor-spy@pm.me")
            .build()
            .expect("failed to build reqwest client");
        Self {
            client,
            base_url: base_url.into(),
            pacing: PacingPolicy::new(),
        }
    }

    /// Constructor for tests: accepts an external client and zero-delay pacing.
    #[cfg(test)]
    pub fn with_client(client: reqwest::Client, base_url: impl Into<String>) -> Self {
        Self {
            client,
            base_url: base_url.into(),
            pacing: PacingPolicy::from_seed(0, true),
        }
    }
}

#[async_trait]
impl Geocoder for NominatimGeocoder {
    async fn geocode(&self, location_string: &str) -> Result<Location, GeocodingError> {
        self.pacing.pace().await;

        let url = format!("{}/search", self.base_url);
        info!(event = "geocoding_attempt", location = location_string);

        let response = self.client
            .get(&url)
            .query(&[
                ("q", location_string),
                ("format", "jsonv2"),
                ("limit", "5"),
            ])
            .send()
            .await
            .map_err(|_| GeocodingError::Http(0))?;

        let status = response.status();
        if !status.is_success() {
            warn!(event = "geocoding_result", outcome = "http_error", status = status.as_u16());
            return Err(GeocodingError::Http(status.as_u16()));
        }

        let places: Vec<NominatimPlace> = response
            .json()
            .await
            .map_err(|e| GeocodingError::Parse(e.to_string()))?;

        if places.is_empty() {
            warn!(event = "geocoding_result", outcome = "no_results", location = location_string);
            return Err(GeocodingError::NoResults);
        }

        if places.len() > 1 {
            info!(
                event = "geocoding_result",
                outcome = "multiple_candidates",
                count = places.len(),
                selected = %places[0].display_name
            );
        }

        let best = &places[0];
        let lat = best.lat.parse::<f64>().map_err(|e| GeocodingError::Parse(e.to_string()))?;
        let lon = best.lon.parse::<f64>().map_err(|e| GeocodingError::Parse(e.to_string()))?;

        info!(
            event = "geocoding_result",
            outcome = "success",
            location = location_string,
            lat,
            lon
        );

        Ok(Location { latitude: lat, longitude: lon })
    }
}

// ── NominatimAdapter ──────────────────────────────────────────────────────────

/// Source adapter that queries Nominatim for businesses matching the industry
/// keyword near the resolved coordinates within the given radius.
///
/// Nominatim is a geocoding API and not a business directory; the search here
/// uses the free-text `q` parameter with the industry keyword + coordinates,
/// which surfaces tagged OSM nodes (amenity, shop, etc.) in the area.
/// Results are approximate and lower-fidelity than dedicated business APIs.
///
/// Does not require a credential.
pub struct NominatimAdapter {
    client: reqwest::Client,
    base_url: String,
    pacing: PacingPolicy,
}

impl NominatimAdapter {
    pub fn new(base_url: impl Into<String>) -> Self {
        let client = reqwest::Client::builder()
            .user_agent("competitor-spy/1.0 contact:competitor-spy@pm.me")
            .build()
            .expect("failed to build reqwest client");
        Self {
            client,
            base_url: base_url.into(),
            pacing: PacingPolicy::new(),
        }
    }

    #[cfg(test)]
    pub fn with_client(client: reqwest::Client, base_url: impl Into<String>) -> Self {
        Self {
            client,
            base_url: base_url.into(),
            pacing: PacingPolicy::from_seed(0, true),
        }
    }
}

#[async_trait]
impl SourceAdapter for NominatimAdapter {
    fn adapter_id(&self) -> &str {
        "nominatim"
    }

    fn requires_credential(&self) -> bool {
        false
    }

    async fn collect(
        &self,
        query: &SearchQuery,
        location: Location,
        radius: Radius,
        _credential: Option<&str>,
    ) -> SourceResult {
        self.pacing.pace().await;

        let url = format!("{}/search", self.base_url);
        let search_term = format!(
            "{} near {},{} radius {}km",
            query.industry,
            location.latitude,
            location.longitude,
            radius.km_value()
        );

        // Audit: log URL as hostname+path only — query params omitted per §6.3
        info!(
            event = "adapter_request",
            adapter_id = "nominatim",
            url = %url    // just host+path; no query params in span
        );

        let response = match self.client
            .get(&url)
            .query(&[
                ("q", search_term.as_str()),
                ("format", "jsonv2"),
                ("limit", "50"),
                ("addressdetails", "1"),
                ("extratags", "1"),
            ])
            .send()
            .await
        {
            Ok(r) => r,
            Err(_) => {
                warn!(event = "adapter_result", adapter_id = "nominatim", outcome = "timeout");
                return failed_result("nominatim", ReasonCode::Timeout);
            }
        };

        let status = response.status();
        if status.is_client_error() {
            warn!(event = "adapter_result", adapter_id = "nominatim", outcome = "http_4xx", code = status.as_u16());
            return failed_result("nominatim", ReasonCode::Http4xx);
        }
        if status.is_server_error() {
            warn!(event = "adapter_result", adapter_id = "nominatim", outcome = "http_5xx", code = status.as_u16());
            return failed_result("nominatim", ReasonCode::Http5xx);
        }

        let places: Vec<NominatimPlace> = match response.json().await {
            Ok(p) => p,
            Err(e) => {
                warn!(event = "adapter_result", adapter_id = "nominatim", outcome = "parse_error", error = %e);
                return failed_result("nominatim", ReasonCode::ParseError);
            }
        };

        let records: Vec<RawRecord> = places
            .into_iter()
            .map(|p| place_to_record(p))
            .collect();

        let status = if records.is_empty() {
            AdapterResultStatus::Success   // zero results is valid
        } else {
            AdapterResultStatus::Success
        };

        info!(
            event = "adapter_result",
            adapter_id = "nominatim",
            outcome = "success",
            record_count = records.len()
        );

        SourceResult {
            adapter_id: "nominatim".to_string(),
            status,
            records,
            retrieved_at: Utc::now(),
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn failed_result(adapter_id: &str, code: ReasonCode) -> SourceResult {
    SourceResult {
        adapter_id: adapter_id.to_string(),
        status: AdapterResultStatus::Failed(code),
        records: vec![],
        retrieved_at: Utc::now(),
    }
}

fn place_to_record(p: NominatimPlace) -> RawRecord {
    let mut fields = std::collections::HashMap::new();
    fields.insert("adapter_id".to_string(), "nominatim".to_string());
    fields.insert("lat".to_string(), p.lat);
    fields.insert("lon".to_string(), p.lon);
    fields.insert("display_name".to_string(), p.display_name);
    if let Some(name) = p.name {
        fields.insert("name".to_string(), name);
    }
    if let Some(osm_id) = p.osm_id {
        fields.insert("osm_id".to_string(), osm_id.to_string());
    }
    if let Some(osm_type) = p.osm_type {
        fields.insert("osm_type".to_string(), osm_type);
    }
    if let Some(addr_type) = p.address_type {
        fields.insert("address_type".to_string(), addr_type);
    }
    if let Some(importance) = p.importance {
        fields.insert("importance".to_string(), importance.to_string());
    }
    if let Some(extra) = p.extra_tags {
        for (k, v) in extra {
            fields.insert(format!("extra_{k}"), v);
        }
    }
    RawRecord {
        adapter_id: "nominatim".to_string(),
        fields,
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::{MockServer, Mock, ResponseTemplate};
    use wiremock::matchers::{method, path, query_param};
    use competitor_spy_domain::query::{Location, Radius, SearchQuery};
    use competitor_spy_domain::run::AdapterResultStatus;

    fn make_client() -> reqwest::Client {
        reqwest::Client::builder()
            .user_agent("test-agent")
            .build()
            .unwrap()
    }

    fn make_query() -> SearchQuery {
        SearchQuery::new("yoga studio", "Amsterdam", Radius::Km10).unwrap()
    }

    fn amsterdam() -> Location {
        Location { latitude: 52.3676, longitude: 4.9041 }
    }

    // ── Geocoder tests ────────────────────────────────────────────────────────

    #[tokio::test]
    async fn geocoder_returns_location_on_success() {
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/search"))
            .and(query_param("q", "Amsterdam, Netherlands"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
                {
                    "lat": "52.3676",
                    "lon": "4.9041",
                    "display_name": "Amsterdam, Netherlands",
                    "osm_id": 271110,
                    "osm_type": "relation",
                    "name": "Amsterdam",
                    "addresstype": "city",
                    "importance": 0.9
                }
            ])))
            .mount(&mock_server)
            .await;

        let geocoder = NominatimGeocoder::with_client(make_client(), mock_server.uri());
        let result = geocoder.geocode("Amsterdam, Netherlands").await;
        assert!(result.is_ok(), "expected Ok, got {result:?}");
        let loc = result.unwrap();
        assert!((loc.latitude - 52.3676).abs() < 0.0001);
        assert!((loc.longitude - 4.9041).abs() < 0.0001);
    }

    #[tokio::test]
    async fn geocoder_selects_first_of_multiple_candidates() {
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/search"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
                { "lat": "52.3676", "lon": "4.9041", "display_name": "Amsterdam, Netherlands", "importance": 0.9 },
                { "lat": "40.7128", "lon": "-74.0060", "display_name": "Amsterdam, NY, USA", "importance": 0.7 }
            ])))
            .mount(&mock_server)
            .await;

        let geocoder = NominatimGeocoder::with_client(make_client(), mock_server.uri());
        let loc = geocoder.geocode("Amsterdam").await.unwrap();
        // First result selected
        assert!((loc.latitude - 52.3676).abs() < 0.0001);
    }

    #[tokio::test]
    async fn geocoder_returns_no_results_error_on_empty_array() {
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/search"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
            .mount(&mock_server)
            .await;

        let geocoder = NominatimGeocoder::with_client(make_client(), mock_server.uri());
        let result = geocoder.geocode("Nowhere Land XYZ1234").await;
        assert_eq!(result, Err(GeocodingError::NoResults));
    }

    #[tokio::test]
    async fn geocoder_returns_http_error_on_4xx() {
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/search"))
            .respond_with(ResponseTemplate::new(429))
            .mount(&mock_server)
            .await;

        let geocoder = NominatimGeocoder::with_client(make_client(), mock_server.uri());
        let result = geocoder.geocode("Amsterdam").await;
        assert_eq!(result, Err(GeocodingError::Http(429)));
    }

    #[tokio::test]
    async fn geocoder_returns_http_error_on_5xx() {
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/search"))
            .respond_with(ResponseTemplate::new(503))
            .mount(&mock_server)
            .await;

        let geocoder = NominatimGeocoder::with_client(make_client(), mock_server.uri());
        let result = geocoder.geocode("Amsterdam").await;
        assert_eq!(result, Err(GeocodingError::Http(503)));
    }

    #[tokio::test]
    async fn geocoder_returns_parse_error_on_bad_lat() {
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/search"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
                { "lat": "not-a-number", "lon": "4.9041", "display_name": "Amsterdam" }
            ])))
            .mount(&mock_server)
            .await;

        let geocoder = NominatimGeocoder::with_client(make_client(), mock_server.uri());
        let result = geocoder.geocode("Amsterdam").await;
        assert!(matches!(result, Err(GeocodingError::Parse(_))));
    }

    // ── Adapter tests ─────────────────────────────────────────────────────────

    #[tokio::test]
    async fn adapter_id_is_nominatim() {
        let adapter = NominatimAdapter::with_client(make_client(), "http://localhost");
        assert_eq!(adapter.adapter_id(), "nominatim");
    }

    #[tokio::test]
    async fn adapter_does_not_require_credential() {
        let adapter = NominatimAdapter::with_client(make_client(), "http://localhost");
        assert!(!adapter.requires_credential());
    }

    #[tokio::test]
    async fn adapter_returns_records_on_success() {
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/search"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
                {
                    "lat": "52.3700",
                    "lon": "4.8900",
                    "display_name": "Yoga Studio A, Amsterdam",
                    "osm_id": 111,
                    "osm_type": "node",
                    "name": "Yoga Studio A",
                    "addresstype": "amenity",
                    "importance": 0.5
                },
                {
                    "lat": "52.3750",
                    "lon": "4.8950",
                    "display_name": "Yoga Studio B, Amsterdam",
                    "osm_id": 222,
                    "osm_type": "node",
                    "name": "Yoga Studio B",
                    "addresstype": "amenity",
                    "importance": 0.4
                }
            ])))
            .mount(&mock_server)
            .await;

        let adapter = NominatimAdapter::with_client(make_client(), mock_server.uri());
        let result = adapter.collect(&make_query(), amsterdam(), Radius::Km10, None).await;

        assert_eq!(result.adapter_id, "nominatim");
        assert!(matches!(result.status, AdapterResultStatus::Success));
        assert_eq!(result.records.len(), 2);
        assert_eq!(result.records[0].fields["name"], "Yoga Studio A");
        assert_eq!(result.records[1].fields["name"], "Yoga Studio B");
    }

    #[tokio::test]
    async fn adapter_returns_success_with_zero_records_on_empty_response() {
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/search"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
            .mount(&mock_server)
            .await;

        let adapter = NominatimAdapter::with_client(make_client(), mock_server.uri());
        let result = adapter.collect(&make_query(), amsterdam(), Radius::Km10, None).await;

        assert!(matches!(result.status, AdapterResultStatus::Success));
        assert_eq!(result.records.len(), 0);
    }

    #[tokio::test]
    async fn adapter_returns_failed_http4xx_on_429() {
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/search"))
            .respond_with(ResponseTemplate::new(429))
            .mount(&mock_server)
            .await;

        let adapter = NominatimAdapter::with_client(make_client(), mock_server.uri());
        let result = adapter.collect(&make_query(), amsterdam(), Radius::Km10, None).await;

        assert!(matches!(result.status, AdapterResultStatus::Failed(ReasonCode::Http4xx)));
        assert!(result.records.is_empty());
    }

    #[tokio::test]
    async fn adapter_returns_failed_http5xx_on_503() {
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/search"))
            .respond_with(ResponseTemplate::new(503))
            .mount(&mock_server)
            .await;

        let adapter = NominatimAdapter::with_client(make_client(), mock_server.uri());
        let result = adapter.collect(&make_query(), amsterdam(), Radius::Km10, None).await;

        assert!(matches!(result.status, AdapterResultStatus::Failed(ReasonCode::Http5xx)));
    }

    #[tokio::test]
    async fn adapter_returns_failed_parse_error_on_invalid_json() {
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/search"))
            .respond_with(ResponseTemplate::new(200).set_body_string("not json at all"))
            .mount(&mock_server)
            .await;

        let adapter = NominatimAdapter::with_client(make_client(), mock_server.uri());
        let result = adapter.collect(&make_query(), amsterdam(), Radius::Km10, None).await;

        assert!(matches!(result.status, AdapterResultStatus::Failed(ReasonCode::ParseError)));
    }

    #[tokio::test]
    async fn adapter_record_has_adapter_id_tag() {
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/search"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
                { "lat": "52.3700", "lon": "4.8900", "display_name": "Some Place" }
            ])))
            .mount(&mock_server)
            .await;

        let adapter = NominatimAdapter::with_client(make_client(), mock_server.uri());
        let result = adapter.collect(&make_query(), amsterdam(), Radius::Km10, None).await;

        assert_eq!(result.records[0].fields["adapter_id"], "nominatim");
        assert_eq!(result.records[0].adapter_id, "nominatim");
    }

    #[tokio::test]
    async fn adapter_ignores_credential_argument() {
        // Nominatim is free; passing a credential must not break anything
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/search"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
            .mount(&mock_server)
            .await;

        let adapter = NominatimAdapter::with_client(make_client(), mock_server.uri());
        let result = adapter.collect(&make_query(), amsterdam(), Radius::Km10, Some("ignored_key")).await;
        assert!(matches!(result.status, AdapterResultStatus::Success));
    }
}
