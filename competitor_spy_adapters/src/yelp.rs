// YelpAdapter — T-011
// TDD: WireMock used to mock Yelp Fusion API responses.
//
// Yelp Fusion API v3: https://api.yelp.com/v3/businesses/search
// Authentication: Bearer token in Authorization header
// Fields: name, url, phone, location (address), categories, rating, review_count
//
// Requires credential: yes (Yelp API key stored in CredentialStore under "yelp")
// If credential absent: returns Failed(AdapterConfigMissing)

use async_trait::async_trait;
use serde::Deserialize;
use chrono::Utc;
use tracing::{info, warn};

use competitor_spy_domain::query::{Location, Radius, SearchQuery};
use competitor_spy_domain::run::{AdapterResultStatus, RawRecord, ReasonCode, SourceResult};

use crate::adapter::SourceAdapter;
use crate::pacing::PacingPolicy;

// ── Yelp Fusion API response types ───────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct YelpSearchResponse {
    businesses: Vec<YelpBusiness>,
}

#[derive(Debug, Deserialize)]
struct YelpBusiness {
    id: String,
    name: String,
    #[serde(default)]
    url: Option<String>,
    #[serde(default)]
    phone: Option<String>,
    #[serde(default)]
    display_phone: Option<String>,
    #[serde(default)]
    rating: Option<f64>,
    #[serde(default)]
    review_count: Option<u64>,
    #[serde(default)]
    categories: Vec<YelpCategory>,
    #[serde(default)]
    location: Option<YelpLocation>,
    #[serde(default)]
    coordinates: Option<YelpCoordinates>,
    #[serde(default)]
    distance: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct YelpCategory {
    alias: String,
    title: String,
}

#[derive(Debug, Deserialize)]
struct YelpLocation {
    address1: Option<String>,
    address2: Option<String>,
    city: Option<String>,
    state: Option<String>,
    zip_code: Option<String>,
    country: Option<String>,
    #[serde(default)]
    display_address: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct YelpCoordinates {
    latitude: Option<f64>,
    longitude: Option<f64>,
}

// ── YelpAdapter ───────────────────────────────────────────────────────────────

pub struct YelpAdapter {
    client: reqwest::Client,
    base_url: String,
    pacing: PacingPolicy,
}

impl YelpAdapter {
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
impl SourceAdapter for YelpAdapter {
    fn adapter_id(&self) -> &str {
        "yelp"
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
                warn!(event = "adapter_result", adapter_id = "yelp", outcome = "missing_credential");
                return failed_result(ReasonCode::AdapterConfigMissing);
            }
        };

        self.pacing.pace().await;

        let url = format!("{}/v3/businesses/search", self.base_url);
        // Audit: log URL as hostname+path only — no query params (§6.3)
        info!(event = "adapter_request", adapter_id = "yelp", url = %url);

        // Yelp accepts radius in metres (max 40000m = 40km).
        // Clamp to Yelp's maximum.
        let radius_m = (radius.km_value() * 1000).min(40_000);

        let response = match self.client
            .get(&url)
            .header("Authorization", format!("Bearer {api_key}"))
            .query(&[
                ("term", query.industry.as_str()),
                ("latitude", &location.latitude.to_string()),
                ("longitude", &location.longitude.to_string()),
                ("radius", &radius_m.to_string()),
                ("limit", "50"),
                ("sort_by", "distance"),
            ])
            .send()
            .await
        {
            Ok(r) => r,
            Err(_) => {
                warn!(event = "adapter_result", adapter_id = "yelp", outcome = "timeout");
                return failed_result(ReasonCode::Timeout);
            }
        };

        let status = response.status();
        if status.is_client_error() {
            warn!(event = "adapter_result", adapter_id = "yelp", outcome = "http_4xx", code = status.as_u16());
            return failed_result(ReasonCode::Http4xx);
        }
        if status.is_server_error() {
            warn!(event = "adapter_result", adapter_id = "yelp", outcome = "http_5xx", code = status.as_u16());
            return failed_result(ReasonCode::Http5xx);
        }

        let yelp_response: YelpSearchResponse = match response.json().await {
            Ok(r) => r,
            Err(e) => {
                warn!(event = "adapter_result", adapter_id = "yelp", outcome = "parse_error", error = %e);
                return failed_result(ReasonCode::ParseError);
            }
        };

        let records: Vec<RawRecord> = yelp_response.businesses
            .into_iter()
            .map(business_to_record)
            .collect();

        info!(
            event = "adapter_result",
            adapter_id = "yelp",
            outcome = "success",
            record_count = records.len()
        );

        SourceResult {
            adapter_id: "yelp".to_string(),
            status: AdapterResultStatus::Success,
            records,
            retrieved_at: Utc::now(),
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn failed_result(code: ReasonCode) -> SourceResult {
    SourceResult {
        adapter_id: "yelp".to_string(),
        status: AdapterResultStatus::Failed(code),
        records: vec![],
        retrieved_at: Utc::now(),
    }
}

fn business_to_record(b: YelpBusiness) -> RawRecord {
    let mut fields = std::collections::HashMap::new();
    fields.insert("adapter_id".to_string(), "yelp".to_string());
    fields.insert("yelp_id".to_string(), b.id);
    fields.insert("name".to_string(), b.name);

    if let Some(url) = b.url {
        fields.insert("website".to_string(), url);
    }
    // Prefer display_phone (formatted) over raw phone
    if let Some(dp) = b.display_phone.filter(|s| !s.is_empty()) {
        fields.insert("phone".to_string(), dp);
    } else if let Some(p) = b.phone.filter(|s| !s.is_empty()) {
        fields.insert("phone".to_string(), p);
    }
    if let Some(r) = b.rating {
        fields.insert("rating".to_string(), r.to_string());
    }
    if let Some(rc) = b.review_count {
        fields.insert("review_count".to_string(), rc.to_string());
    }
    if let Some(d) = b.distance {
        fields.insert("distance_m".to_string(), d.to_string());
    }

    // Categories: comma-separated titles and aliases
    if !b.categories.is_empty() {
        let titles: Vec<&str> = b.categories.iter().map(|c| c.title.as_str()).collect();
        let aliases: Vec<&str> = b.categories.iter().map(|c| c.alias.as_str()).collect();
        fields.insert("categories".to_string(), titles.join(", "));
        fields.insert("category_aliases".to_string(), aliases.join(", "));
    }

    // Location
    if let Some(loc) = b.location {
        if let Some(a) = loc.address1.filter(|s| !s.is_empty()) {
            fields.insert("address_street".to_string(), a);
        }
        if let Some(a2) = loc.address2.filter(|s| !s.is_empty()) {
            fields.insert("address_street2".to_string(), a2);
        }
        if let Some(city) = loc.city.filter(|s| !s.is_empty()) {
            fields.insert("address_city".to_string(), city);
        }
        if let Some(state) = loc.state.filter(|s| !s.is_empty()) {
            fields.insert("address_state".to_string(), state);
        }
        if let Some(zip) = loc.zip_code.filter(|s| !s.is_empty()) {
            fields.insert("address_postcode".to_string(), zip);
        }
        if let Some(country) = loc.country.filter(|s| !s.is_empty()) {
            fields.insert("address_country".to_string(), country);
        }
        if !loc.display_address.is_empty() {
            fields.insert("display_address".to_string(), loc.display_address.join(", "));
        }
    }

    // Coordinates
    if let Some(coords) = b.coordinates {
        if let Some(lat) = coords.latitude {
            fields.insert("lat".to_string(), lat.to_string());
        }
        if let Some(lon) = coords.longitude {
            fields.insert("lon".to_string(), lon.to_string());
        }
    }

    RawRecord {
        adapter_id: "yelp".to_string(),
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

    fn two_businesses_response() -> serde_json::Value {
        serde_json::json!({
            "businesses": [
                {
                    "id": "yelp-id-alpha",
                    "name": "Yoga Studio Alpha",
                    "url": "https://yelp.com/biz/alpha",
                    "phone": "+31201234567",
                    "display_phone": "+31 20 123 4567",
                    "rating": 4.5,
                    "review_count": 120,
                    "distance": 800.0,
                    "categories": [
                        { "alias": "yoga", "title": "Yoga" },
                        { "alias": "fitness", "title": "Fitness" }
                    ],
                    "location": {
                        "address1": "Prinsengracht 1",
                        "city": "Amsterdam",
                        "state": "NH",
                        "zip_code": "1015",
                        "country": "NL",
                        "display_address": ["Prinsengracht 1", "Amsterdam, NL 1015"]
                    },
                    "coordinates": { "latitude": 52.370, "longitude": 4.895 }
                },
                {
                    "id": "yelp-id-beta",
                    "name": "Yoga Studio Beta",
                    "url": null,
                    "phone": "",
                    "display_phone": "",
                    "rating": 4.0,
                    "review_count": 45,
                    "distance": 1200.0,
                    "categories": [{ "alias": "yoga", "title": "Yoga" }],
                    "location": {
                        "address1": "Jordaan 10",
                        "city": "Amsterdam",
                        "zip_code": "1016",
                        "country": "NL",
                        "display_address": ["Jordaan 10", "Amsterdam, NL"]
                    },
                    "coordinates": { "latitude": 52.375, "longitude": 4.900 }
                }
            ],
            "total": 2
        })
    }

    #[tokio::test]
    async fn adapter_id_is_yelp() {
        let a = YelpAdapter::with_client(make_client(), "http://localhost");
        assert_eq!(a.adapter_id(), "yelp");
    }

    #[tokio::test]
    async fn adapter_requires_credential() {
        let a = YelpAdapter::with_client(make_client(), "http://localhost");
        assert!(a.requires_credential());
    }

    #[tokio::test]
    async fn adapter_returns_missing_credential_when_no_api_key() {
        let mock = MockServer::start().await;
        let a = YelpAdapter::with_client(make_client(), mock.uri());
        let result = a.collect(&make_query(), amsterdam(), Radius::Km10, None).await;
        assert!(matches!(result.status, AdapterResultStatus::Failed(ReasonCode::AdapterConfigMissing)));
        assert!(result.records.is_empty());
    }

    #[tokio::test]
    async fn adapter_returns_missing_credential_when_empty_key() {
        let mock = MockServer::start().await;
        let a = YelpAdapter::with_client(make_client(), mock.uri());
        let result = a.collect(&make_query(), amsterdam(), Radius::Km10, Some("")).await;
        assert!(matches!(result.status, AdapterResultStatus::Failed(ReasonCode::AdapterConfigMissing)));
    }

    #[tokio::test]
    async fn adapter_sends_bearer_token_in_authorization_header() {
        let mock = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v3/businesses/search"))
            .and(header("Authorization", "Bearer test-api-key"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "businesses": [], "total": 0
            })))
            .mount(&mock)
            .await;

        let a = YelpAdapter::with_client(make_client(), mock.uri());
        let result = a.collect(&make_query(), amsterdam(), Radius::Km10, Some("test-api-key")).await;
        assert!(matches!(result.status, AdapterResultStatus::Success));
    }

    #[tokio::test]
    async fn adapter_returns_records_on_success() {
        let mock = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v3/businesses/search"))
            .respond_with(ResponseTemplate::new(200).set_body_json(two_businesses_response()))
            .mount(&mock)
            .await;

        let a = YelpAdapter::with_client(make_client(), mock.uri());
        let result = a.collect(&make_query(), amsterdam(), Radius::Km10, Some("key")).await;

        assert!(matches!(result.status, AdapterResultStatus::Success));
        assert_eq!(result.records.len(), 2);
        assert_eq!(result.records[0].fields["name"], "Yoga Studio Alpha");
        assert_eq!(result.records[1].fields["name"], "Yoga Studio Beta");
    }

    #[tokio::test]
    async fn adapter_extracts_all_business_fields() {
        let mock = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v3/businesses/search"))
            .respond_with(ResponseTemplate::new(200).set_body_json(two_businesses_response()))
            .mount(&mock)
            .await;

        let a = YelpAdapter::with_client(make_client(), mock.uri());
        let result = a.collect(&make_query(), amsterdam(), Radius::Km10, Some("key")).await;
        let rec = &result.records[0];

        assert_eq!(rec.fields["yelp_id"], "yelp-id-alpha");
        assert_eq!(rec.fields["website"], "https://yelp.com/biz/alpha");
        assert_eq!(rec.fields["phone"], "+31 20 123 4567");  // display_phone preferred
        assert_eq!(rec.fields["rating"], "4.5");
        assert_eq!(rec.fields["review_count"], "120");
        assert_eq!(rec.fields["categories"], "Yoga, Fitness");
        assert_eq!(rec.fields["category_aliases"], "yoga, fitness");
        assert_eq!(rec.fields["address_street"], "Prinsengracht 1");
        assert_eq!(rec.fields["address_city"], "Amsterdam");
        assert_eq!(rec.fields["address_postcode"], "1015");
        assert_eq!(rec.fields["adapter_id"], "yelp");
    }

    #[tokio::test]
    async fn adapter_record_has_adapter_id_tag() {
        let mock = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v3/businesses/search"))
            .respond_with(ResponseTemplate::new(200).set_body_json(two_businesses_response()))
            .mount(&mock)
            .await;

        let a = YelpAdapter::with_client(make_client(), mock.uri());
        let result = a.collect(&make_query(), amsterdam(), Radius::Km10, Some("key")).await;

        assert_eq!(result.records[0].adapter_id, "yelp");
        assert_eq!(result.records[0].fields["adapter_id"], "yelp");
    }

    #[tokio::test]
    async fn adapter_returns_success_with_zero_records() {
        let mock = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v3/businesses/search"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "businesses": [], "total": 0
            })))
            .mount(&mock)
            .await;

        let a = YelpAdapter::with_client(make_client(), mock.uri());
        let result = a.collect(&make_query(), amsterdam(), Radius::Km10, Some("key")).await;
        assert!(matches!(result.status, AdapterResultStatus::Success));
        assert_eq!(result.records.len(), 0);
    }

    #[tokio::test]
    async fn adapter_returns_failed_http4xx_on_401() {
        let mock = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v3/businesses/search"))
            .respond_with(ResponseTemplate::new(401))
            .mount(&mock)
            .await;

        let a = YelpAdapter::with_client(make_client(), mock.uri());
        let result = a.collect(&make_query(), amsterdam(), Radius::Km10, Some("bad-key")).await;
        assert!(matches!(result.status, AdapterResultStatus::Failed(ReasonCode::Http4xx)));
    }

    #[tokio::test]
    async fn adapter_returns_failed_http5xx_on_503() {
        let mock = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v3/businesses/search"))
            .respond_with(ResponseTemplate::new(503))
            .mount(&mock)
            .await;

        let a = YelpAdapter::with_client(make_client(), mock.uri());
        let result = a.collect(&make_query(), amsterdam(), Radius::Km10, Some("key")).await;
        assert!(matches!(result.status, AdapterResultStatus::Failed(ReasonCode::Http5xx)));
    }

    #[tokio::test]
    async fn adapter_returns_failed_parse_error_on_invalid_json() {
        let mock = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v3/businesses/search"))
            .respond_with(ResponseTemplate::new(200).set_body_string("not json"))
            .mount(&mock)
            .await;

        let a = YelpAdapter::with_client(make_client(), mock.uri());
        let result = a.collect(&make_query(), amsterdam(), Radius::Km10, Some("key")).await;
        assert!(matches!(result.status, AdapterResultStatus::Failed(ReasonCode::ParseError)));
    }
}
