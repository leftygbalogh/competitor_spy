// GooglePlacesAdapter — T-012
// TDD: WireMock used to mock Google Places API responses.
//
// Google Places API (New) Text Search:
//   POST https://places.googleapis.com/v1/places:searchText
//   Authorization: api_key in X-Goog-Api-Key header
//   Field mask via X-Goog-FieldMask header
//
// Text Search is used (not Nearby Search) because it accepts a free-text
// textQuery, which maps naturally onto the user's industry keyword.
// Nearby Search requires a type from the Places API table, which does not
// cover niche industries like "pilates".
//
// Response: { "places": [ { "id", "displayName.text", "formattedAddress",
//   "nationalPhoneNumber", "websiteUri", "types", "rating", "userRatingCount",
//   "location": {"latitude","longitude"} } ] }
//
// Requires credential: yes (Google Places API key stored under "google_places")
// If credential absent: returns Failed(AdapterConfigMissing)

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use chrono::Utc;
use tracing::{info, warn};

use competitor_spy_domain::query::{Location, Radius, SearchQuery};
use competitor_spy_domain::run::{AdapterResultStatus, RawRecord, ReasonCode, SourceResult};

use crate::adapter::SourceAdapter;
use crate::pacing::PacingPolicy;

// ── Google Places API request/response types ─────────────────────────────────

#[derive(Debug, Serialize)]
struct TextSearchRequest {
    #[serde(rename = "textQuery")]
    text_query: String,
    #[serde(rename = "maxResultCount")]
    max_result_count: u32,
    #[serde(rename = "locationBias")]
    location_bias: LocationBias,
    #[serde(rename = "rankPreference")]
    rank_preference: String,
}

#[derive(Debug, Serialize)]
struct LocationBias {
    circle: Circle,
}

#[derive(Debug, Serialize)]
struct Circle {
    center: LatLng,
    radius: f64,
}

#[derive(Debug, Serialize)]
struct LatLng {
    latitude: f64,
    longitude: f64,
}

#[derive(Debug, Deserialize)]
struct NearbySearchResponse {
    #[serde(default)]
    places: Vec<GooglePlace>,
}

#[derive(Debug, Deserialize)]
struct GooglePlace {
    #[serde(default)]
    id: Option<String>,
    #[serde(rename = "displayName")]
    display_name: Option<DisplayName>,
    #[serde(rename = "formattedAddress", default)]
    formatted_address: Option<String>,
    #[serde(rename = "nationalPhoneNumber", default)]
    national_phone_number: Option<String>,
    #[serde(rename = "websiteUri", default)]
    website_uri: Option<String>,
    #[serde(default)]
    types: Vec<String>,
    #[serde(default)]
    rating: Option<f64>,
    #[serde(rename = "userRatingCount", default)]
    user_rating_count: Option<u64>,
    #[serde(default)]
    location: Option<GoogleLatLng>,
}

#[derive(Debug, Deserialize)]
struct DisplayName {
    text: String,
}

#[derive(Debug, Deserialize)]
struct GoogleLatLng {
    latitude: f64,
    longitude: f64,
}

// ── GooglePlacesAdapter ───────────────────────────────────────────────────────

pub struct GooglePlacesAdapter {
    client: reqwest::Client,
    base_url: String,
    pacing: PacingPolicy,
}

impl GooglePlacesAdapter {
    pub fn new(base_url: impl Into<String>) -> Self {
        let client = reqwest::Client::builder()
            .user_agent("competitor-spy/1.0 (contact: admin@example.com)")
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
impl SourceAdapter for GooglePlacesAdapter {
    fn adapter_id(&self) -> &str {
        "google_places"
    }

    fn requires_credential(&self) -> bool {
        true
    }

    async fn collect(
        &self,
        query: &SearchQuery,
        location: Location,
        radius: Radius,
        credential: Option<&str>,
    ) -> SourceResult {
        let api_key = match credential {
            Some(k) if !k.is_empty() => k,
            _ => {
                warn!(event = "adapter_result", adapter_id = "google_places", outcome = "missing_credential");
                return failed_result(ReasonCode::AdapterConfigMissing);
            }
        };

        self.pacing.pace().await;

        let url = format!("{}/v1/places:searchText", self.base_url);
        // Audit: log URL as hostname+path only (§6.3) — api key is in header, not URL
        info!(event = "adapter_request", adapter_id = "google_places", url = %url);

        let request_body = build_request_body(
            &query.industry,
            location.latitude,
            location.longitude,
            radius.km_value() as f64 * 1000.0,
        );

        let response = match self.client
            .post(&url)
            .header("X-Goog-Api-Key", api_key)
            .header(
                "X-Goog-FieldMask",
                "places.id,places.displayName,places.formattedAddress,places.nationalPhoneNumber,places.websiteUri,places.types,places.rating,places.userRatingCount,places.location",
            )
            .json(&request_body)
            .send()
            .await
        {
            Ok(r) => r,
            Err(_) => {
                warn!(event = "adapter_result", adapter_id = "google_places", outcome = "timeout");
                return failed_result(ReasonCode::Timeout);
            }
        };

        let status = response.status();
        if status.is_client_error() {
            warn!(event = "adapter_result", adapter_id = "google_places", outcome = "http_4xx", code = status.as_u16());
            return failed_result(ReasonCode::Http4xx);
        }
        if status.is_server_error() {
            warn!(event = "adapter_result", adapter_id = "google_places", outcome = "http_5xx", code = status.as_u16());
            return failed_result(ReasonCode::Http5xx);
        }

        let google_response: NearbySearchResponse = match response.json().await {
            Ok(r) => r,
            Err(e) => {
                warn!(event = "adapter_result", adapter_id = "google_places", outcome = "parse_error", error = %e);
                return failed_result(ReasonCode::ParseError);
            }
        };

        let records: Vec<RawRecord> = google_response.places
            .into_iter()
            .map(place_to_record)
            .collect();

        info!(
            event = "adapter_result",
            adapter_id = "google_places",
            outcome = "success",
            record_count = records.len()
        );

        SourceResult {
            adapter_id: "google_places".to_string(),
            status: AdapterResultStatus::Success,
            records,
            retrieved_at: Utc::now(),
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn failed_result(code: ReasonCode) -> SourceResult {
    SourceResult {
        adapter_id: "google_places".to_string(),
        status: AdapterResultStatus::Failed(code),
        records: vec![],
        retrieved_at: Utc::now(),
    }
}

/// Build the Text Search request body.
/// textQuery is the user's industry keyword — works for any free-text term
/// including niche categories like "pilates" that have no Google place type.
/// locationBias biases results toward the circle; results may extend slightly
/// beyond the radius but are ranked by distance.
fn build_request_body(industry: &str, lat: f64, lon: f64, radius_m: f64) -> TextSearchRequest {
    // Max radius: 50000m; clamp to 50km.
    let radius_clamped = radius_m.min(50_000.0);

    TextSearchRequest {
        text_query: industry.to_string(),
        max_result_count: 20,
        location_bias: LocationBias {
            circle: Circle {
                center: LatLng { latitude: lat, longitude: lon },
                radius: radius_clamped,
            },
        },
        rank_preference: "DISTANCE".to_string(),
    }
}

fn place_to_record(p: GooglePlace) -> RawRecord {
    let mut fields = std::collections::HashMap::new();
    fields.insert("adapter_id".to_string(), "google_places".to_string());

    if let Some(id) = p.id {
        fields.insert("google_place_id".to_string(), id);
    }
    if let Some(dn) = p.display_name {
        fields.insert("name".to_string(), dn.text);
    }
    if let Some(addr) = p.formatted_address {
        fields.insert("address".to_string(), addr);
    }
    if let Some(phone) = p.national_phone_number {
        fields.insert("phone".to_string(), phone);
    }
    if let Some(web) = p.website_uri {
        fields.insert("website".to_string(), web);
    }
    if !p.types.is_empty() {
        fields.insert("types".to_string(), p.types.join(", "));
        // Use first type as category
        fields.insert("category".to_string(), p.types[0].clone());
    }
    if let Some(r) = p.rating {
        fields.insert("rating".to_string(), r.to_string());
    }
    if let Some(rc) = p.user_rating_count {
        fields.insert("review_count".to_string(), rc.to_string());
    }
    if let Some(loc) = p.location {
        fields.insert("lat".to_string(), loc.latitude.to_string());
        fields.insert("lon".to_string(), loc.longitude.to_string());
    }

    RawRecord {
        adapter_id: "google_places".to_string(),
        fields,
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::{MockServer, Mock, ResponseTemplate};
    use wiremock::matchers::{method, path, header};
    use competitor_spy_domain::query::{Location, Radius};
    use competitor_spy_domain::run::AdapterResultStatus;

    fn make_client() -> reqwest::Client {
        reqwest::Client::builder().user_agent("test").build().unwrap()
    }

    fn make_query() -> SearchQuery {
        SearchQuery::new("yoga studio", "Amsterdam", Radius::Km10).unwrap()
    }

    fn amsterdam() -> Location {
        Location { latitude: 52.3676, longitude: 4.9041 }
    }

    fn two_places_response() -> serde_json::Value {
        serde_json::json!({
            "places": [
                {
                    "id": "ChIJplace1",
                    "displayName": { "text": "Yoga Studio Alpha" },
                    "formattedAddress": "Prinsengracht 1, Amsterdam, Netherlands",
                    "nationalPhoneNumber": "+31 20 123 4567",
                    "websiteUri": "https://alpha.example.com",
                    "types": ["yoga_studio", "gym", "establishment"],
                    "rating": 4.7,
                    "userRatingCount": 210,
                    "location": { "latitude": 52.370, "longitude": 4.895 }
                },
                {
                    "id": "ChIJplace2",
                    "displayName": { "text": "Yoga Studio Beta" },
                    "formattedAddress": "Jordaan 10, Amsterdam, Netherlands",
                    "types": ["yoga_studio"],
                    "rating": 4.1,
                    "userRatingCount": 80,
                    "location": { "latitude": 52.375, "longitude": 4.900 }
                }
            ]
        })
    }

    #[tokio::test]
    async fn adapter_id_is_google_places() {
        let a = GooglePlacesAdapter::with_client(make_client(), "http://localhost");
        assert_eq!(a.adapter_id(), "google_places");
    }

    #[tokio::test]
    async fn adapter_requires_credential() {
        let a = GooglePlacesAdapter::with_client(make_client(), "http://localhost");
        assert!(a.requires_credential());
    }

    #[tokio::test]
    async fn adapter_returns_missing_credential_when_no_api_key() {
        let mock = MockServer::start().await;
        let a = GooglePlacesAdapter::with_client(make_client(), mock.uri());
        let result = a.collect(&make_query(), amsterdam(), Radius::Km10, None).await;
        assert!(matches!(result.status, AdapterResultStatus::Failed(ReasonCode::AdapterConfigMissing)));
        assert!(result.records.is_empty());
    }

    #[tokio::test]
    async fn adapter_returns_missing_credential_when_empty_key() {
        let mock = MockServer::start().await;
        let a = GooglePlacesAdapter::with_client(make_client(), mock.uri());
        let result = a.collect(&make_query(), amsterdam(), Radius::Km10, Some("")).await;
        assert!(matches!(result.status, AdapterResultStatus::Failed(ReasonCode::AdapterConfigMissing)));
    }

    #[tokio::test]
    async fn adapter_sends_api_key_in_goog_header() {
        let mock = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/places:searchText"))
            .and(header("X-Goog-Api-Key", "my-test-key"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"places": []})))
            .mount(&mock)
            .await;

        let a = GooglePlacesAdapter::with_client(make_client(), mock.uri());
        let result = a.collect(&make_query(), amsterdam(), Radius::Km10, Some("my-test-key")).await;
        assert!(matches!(result.status, AdapterResultStatus::Success));
    }

    #[tokio::test]
    async fn adapter_returns_records_on_success() {
        let mock = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/places:searchText"))
            .respond_with(ResponseTemplate::new(200).set_body_json(two_places_response()))
            .mount(&mock)
            .await;

        let a = GooglePlacesAdapter::with_client(make_client(), mock.uri());
        let result = a.collect(&make_query(), amsterdam(), Radius::Km10, Some("key")).await;

        assert!(matches!(result.status, AdapterResultStatus::Success));
        assert_eq!(result.records.len(), 2);
        assert_eq!(result.records[0].fields["name"], "Yoga Studio Alpha");
        assert_eq!(result.records[1].fields["name"], "Yoga Studio Beta");
    }

    #[tokio::test]
    async fn adapter_extracts_all_place_fields() {
        let mock = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/places:searchText"))
            .respond_with(ResponseTemplate::new(200).set_body_json(two_places_response()))
            .mount(&mock)
            .await;

        let a = GooglePlacesAdapter::with_client(make_client(), mock.uri());
        let result = a.collect(&make_query(), amsterdam(), Radius::Km10, Some("key")).await;
        let rec = &result.records[0];

        assert_eq!(rec.fields["google_place_id"], "ChIJplace1");
        assert_eq!(rec.fields["address"], "Prinsengracht 1, Amsterdam, Netherlands");
        assert_eq!(rec.fields["phone"], "+31 20 123 4567");
        assert_eq!(rec.fields["website"], "https://alpha.example.com");
        assert_eq!(rec.fields["types"], "yoga_studio, gym, establishment");
        assert_eq!(rec.fields["category"], "yoga_studio");
        assert_eq!(rec.fields["rating"], "4.7");
        assert_eq!(rec.fields["review_count"], "210");
        assert_eq!(rec.fields["lat"], "52.37");
        assert_eq!(rec.fields["adapter_id"], "google_places");
    }

    #[tokio::test]
    async fn adapter_record_has_adapter_id_tag() {
        let mock = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/places:searchText"))
            .respond_with(ResponseTemplate::new(200).set_body_json(two_places_response()))
            .mount(&mock)
            .await;

        let a = GooglePlacesAdapter::with_client(make_client(), mock.uri());
        let result = a.collect(&make_query(), amsterdam(), Radius::Km10, Some("key")).await;

        assert_eq!(result.records[0].adapter_id, "google_places");
        assert_eq!(result.records[0].fields["adapter_id"], "google_places");
    }

    #[tokio::test]
    async fn adapter_returns_success_with_zero_places() {
        let mock = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/places:searchText"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"places": []})))
            .mount(&mock)
            .await;

        let a = GooglePlacesAdapter::with_client(make_client(), mock.uri());
        let result = a.collect(&make_query(), amsterdam(), Radius::Km10, Some("key")).await;
        assert!(matches!(result.status, AdapterResultStatus::Success));
        assert_eq!(result.records.len(), 0);
    }

    #[tokio::test]
    async fn adapter_handles_empty_places_field_in_response() {
        // Google returns {} (no "places" key) when there are no results
        let mock = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/places:searchText"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
            .mount(&mock)
            .await;

        let a = GooglePlacesAdapter::with_client(make_client(), mock.uri());
        let result = a.collect(&make_query(), amsterdam(), Radius::Km10, Some("key")).await;
        assert!(matches!(result.status, AdapterResultStatus::Success));
        assert_eq!(result.records.len(), 0);
    }

    #[tokio::test]
    async fn adapter_returns_failed_http4xx_on_403() {
        let mock = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/places:searchText"))
            .respond_with(ResponseTemplate::new(403))
            .mount(&mock)
            .await;

        let a = GooglePlacesAdapter::with_client(make_client(), mock.uri());
        let result = a.collect(&make_query(), amsterdam(), Radius::Km10, Some("bad-key")).await;
        assert!(matches!(result.status, AdapterResultStatus::Failed(ReasonCode::Http4xx)));
    }

    #[tokio::test]
    async fn adapter_returns_failed_http5xx_on_503() {
        let mock = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/places:searchText"))
            .respond_with(ResponseTemplate::new(503))
            .mount(&mock)
            .await;

        let a = GooglePlacesAdapter::with_client(make_client(), mock.uri());
        let result = a.collect(&make_query(), amsterdam(), Radius::Km10, Some("key")).await;
        assert!(matches!(result.status, AdapterResultStatus::Failed(ReasonCode::Http5xx)));
    }

    #[tokio::test]
    async fn adapter_returns_failed_parse_error_on_invalid_json() {
        let mock = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/places:searchText"))
            .respond_with(ResponseTemplate::new(200).set_body_string("not json"))
            .mount(&mock)
            .await;

        let a = GooglePlacesAdapter::with_client(make_client(), mock.uri());
        let result = a.collect(&make_query(), amsterdam(), Radius::Km10, Some("key")).await;
        assert!(matches!(result.status, AdapterResultStatus::Failed(ReasonCode::ParseError)));
    }
}
