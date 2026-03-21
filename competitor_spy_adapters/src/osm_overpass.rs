// OsmOverpassAdapter — T-010
// TDD: WireMock used to mock Overpass API HTTP responses.
//
// Overpass API: https://overpass-api.de/api/interpreter
// Uses QL (Overpass Query Language) to find OSM nodes/ways/relations by tag.
//
// Query template: searches for amenity/shop/leisure nodes near given coordinates
// within the given radius using the "around" filter.
//
// Response format: JSON (output=json, out=body geom)
// Schema: { "elements": [ { "type": "node|way", "id": u64, "lat": f64, "lon": f64,
//           "tags": { "name": String, "phone": String, ... } } ] }
//
// Does not require a credential.

use async_trait::async_trait;
use serde::Deserialize;
use chrono::Utc;
use tracing::{info, warn};

use competitor_spy_domain::query::{Location, Radius, SearchQuery};
use competitor_spy_domain::run::{AdapterResultStatus, RawRecord, ReasonCode, SourceResult};

use crate::adapter::SourceAdapter;
use crate::pacing::PacingPolicy;

// ── Overpass JSON response ────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct OverpassResponse {
    elements: Vec<OverpassElement>,
}

#[derive(Debug, Deserialize)]
struct OverpassElement {
    #[serde(rename = "type")]
    element_type: String,
    id: u64,
    #[serde(default)]
    lat: Option<f64>,
    #[serde(default)]
    lon: Option<f64>,
    #[serde(default)]
    tags: std::collections::HashMap<String, String>,
    // Way/relation centroids (returned with "out center")
    #[serde(default)]
    center: Option<OverpassCenter>,
}

#[derive(Debug, Deserialize)]
struct OverpassCenter {
    lat: f64,
    lon: f64,
}

// ── OsmOverpassAdapter ────────────────────────────────────────────────────────

/// Source adapter for the OSM Overpass API.
///
/// Generates an Overpass QL query that searches for tagged nodes/ways within
/// the given radius around the resolved coordinates. The query attempts to
/// match the industry keyword against common OSM tag categories.
///
/// Does not require a credential.
pub struct OsmOverpassAdapter {
    client: reqwest::Client,
    /// Base URL for the Overpass interpreter endpoint.
    /// Default: `"https://overpass-api.de/api/interpreter"`
    base_url: String,
    pacing: PacingPolicy,
}

impl OsmOverpassAdapter {
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
impl SourceAdapter for OsmOverpassAdapter {
    fn adapter_id(&self) -> &str {
        "osm_overpass"
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

        let radius_m = radius.km_value() * 1000;
        let ql_query = build_overpass_query(&query.industry, location.latitude, location.longitude, radius_m);

        // Audit: log URL as hostname+path only — no query params (§6.3)
        info!(
            event = "adapter_request",
            adapter_id = "osm_overpass",
            url = %self.base_url
        );

        let response = match self.client
            .post(&self.base_url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(format!("data={}", urlencoded(&ql_query)))
            .send()
            .await
        {
            Ok(r) => r,
            Err(_) => {
                warn!(event = "adapter_result", adapter_id = "osm_overpass", outcome = "timeout");
                return failed_result(ReasonCode::Timeout);
            }
        };

        let status = response.status();
        if status.is_client_error() {
            warn!(event = "adapter_result", adapter_id = "osm_overpass", outcome = "http_4xx", code = status.as_u16());
            return failed_result(ReasonCode::Http4xx);
        }
        if status.is_server_error() {
            warn!(event = "adapter_result", adapter_id = "osm_overpass", outcome = "http_5xx", code = status.as_u16());
            return failed_result(ReasonCode::Http5xx);
        }

        let overpass: OverpassResponse = match response.json().await {
            Ok(r) => r,
            Err(e) => {
                warn!(event = "adapter_result", adapter_id = "osm_overpass", outcome = "parse_error", error = %e);
                return failed_result(ReasonCode::ParseError);
            }
        };

        let records: Vec<RawRecord> = overpass.elements
            .into_iter()
            .filter_map(|el| element_to_record(el))
            .collect();

        info!(
            event = "adapter_result",
            adapter_id = "osm_overpass",
            outcome = "success",
            record_count = records.len()
        );

        SourceResult {
            adapter_id: "osm_overpass".to_string(),
            status: AdapterResultStatus::Success,
            records,
            retrieved_at: Utc::now(),
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Build an Overpass QL query searching for the industry keyword near a location.
/// The union searches nodes, ways, and relations with name~<keyword>.
/// Also searches common amenity/shop/leisure tags for catchall discovery.
fn build_overpass_query(industry: &str, lat: f64, lon: f64, radius_m: u32) -> String {
    // Escape the industry string for QL regex (strip QL special chars)
    let keyword = sanitize_ql_string(industry);
    format!(
        r#"[out:json][timeout:25];
(
  node["name"~"{keyword}",i](around:{radius_m},{lat},{lon});
  way["name"~"{keyword}",i](around:{radius_m},{lat},{lon});
  relation["name"~"{keyword}",i](around:{radius_m},{lat},{lon});
);
out body center qt;"#,
        keyword = keyword,
        radius_m = radius_m,
        lat = lat,
        lon = lon
    )
}

/// Strip characters that have special meaning in Overpass QL regex values.
fn sanitize_ql_string(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_alphanumeric() || *c == ' ' || *c == '-' || *c == '_')
        .collect()
}

/// Percent-encode a string for use as a URL-encoded form body value.
///
/// Encodes all characters except unreserved (A-Z, a-z, 0-9, -, _, ., ~).
fn urlencoded(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 2);
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9'
            | b'-' | b'_' | b'.' | b'~' => out.push(b as char),
            b' ' => out.push('+'),
            _ => {
                out.push('%');
                out.push(hex_digit(b >> 4));
                out.push(hex_digit(b & 0x0F));
            }
        }
    }
    out
}

fn hex_digit(n: u8) -> char {
    match n {
        0..=9 => (b'0' + n) as char,
        _ => (b'A' + n - 10) as char,
    }
}

fn failed_result(code: ReasonCode) -> SourceResult {
    SourceResult {
        adapter_id: "osm_overpass".to_string(),
        status: AdapterResultStatus::Failed(code),
        records: vec![],
        retrieved_at: Utc::now(),
    }
}

/// Convert an Overpass element to a RawRecord.
/// Returns None if the element has no coordinates (skips coordinate-less elements).
fn element_to_record(el: OverpassElement) -> Option<RawRecord> {
    // Resolve coordinates: nodes have lat/lon directly; ways/relations use center
    let (lat, lon) = match (el.lat, el.lon, el.center.as_ref()) {
        (Some(lat), Some(lon), _) => (lat, lon),
        (_, _, Some(c)) => (c.lat, c.lon),
        _ => return None,  // no coordinates available — skip
    };

    let mut fields = std::collections::HashMap::new();
    fields.insert("adapter_id".to_string(), "osm_overpass".to_string());
    fields.insert("osm_id".to_string(), el.id.to_string());
    fields.insert("osm_type".to_string(), el.element_type);
    fields.insert("lat".to_string(), lat.to_string());
    fields.insert("lon".to_string(), lon.to_string());

    // Standard OSM tags mapped to canonical field names
    for (k, v) in &el.tags {
        match k.as_str() {
            "name" => { fields.insert("name".to_string(), v.clone()); }
            "phone" | "contact:phone" => { fields.entry("phone".to_string()).or_insert_with(|| v.clone()); }
            "website" | "contact:website" | "url" => { fields.entry("website".to_string()).or_insert_with(|| v.clone()); }
            "addr:street" => { fields.entry("address_street".to_string()).or_insert_with(|| v.clone()); }
            "addr:housenumber" => { fields.entry("address_housenumber".to_string()).or_insert_with(|| v.clone()); }
            "addr:city" => { fields.entry("address_city".to_string()).or_insert_with(|| v.clone()); }
            "addr:postcode" => { fields.entry("address_postcode".to_string()).or_insert_with(|| v.clone()); }
            "amenity" | "shop" | "leisure" | "tourism" | "office" => {
                fields.entry("category".to_string()).or_insert_with(|| v.clone());
                fields.insert(k.clone(), v.clone());
            }
            "opening_hours" => { fields.insert("opening_hours".to_string(), v.clone()); }
            _ => { fields.insert(format!("tag_{k}"), v.clone()); }
        }
    }

    Some(RawRecord {
        adapter_id: "osm_overpass".to_string(),
        fields,
    })
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::{MockServer, Mock, ResponseTemplate};
    use wiremock::matchers::{method, path};
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

    fn overpass_two_nodes() -> serde_json::Value {
        serde_json::json!({
            "elements": [
                {
                    "type": "node",
                    "id": 1001,
                    "lat": 52.370,
                    "lon": 4.895,
                    "tags": {
                        "name": "Yoga Studio Alpha",
                        "amenity": "fitness_centre",
                        "phone": "+31201234567",
                        "website": "https://alpha.example.com"
                    }
                },
                {
                    "type": "node",
                    "id": 1002,
                    "lat": 52.375,
                    "lon": 4.900,
                    "tags": {
                        "name": "Yoga Studio Beta",
                        "amenity": "fitness_centre"
                    }
                }
            ]
        })
    }

    #[tokio::test]
    async fn adapter_id_is_osm_overpass() {
        let a = OsmOverpassAdapter::with_client(make_client(), "http://localhost");
        assert_eq!(a.adapter_id(), "osm_overpass");
    }

    #[tokio::test]
    async fn adapter_does_not_require_credential() {
        let a = OsmOverpassAdapter::with_client(make_client(), "http://localhost");
        assert!(!a.requires_credential());
    }

    #[tokio::test]
    async fn adapter_returns_records_on_success() {
        let mock = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/"))
            .respond_with(ResponseTemplate::new(200).set_body_json(overpass_two_nodes()))
            .mount(&mock)
            .await;

        let a = OsmOverpassAdapter::with_client(make_client(), format!("{}/", mock.uri()));
        let result = a.collect(&make_query(), amsterdam(), Radius::Km10, None).await;

        assert!(matches!(result.status, AdapterResultStatus::Success));
        assert_eq!(result.records.len(), 2);
        assert_eq!(result.records[0].fields["name"], "Yoga Studio Alpha");
    }

    #[tokio::test]
    async fn adapter_extracts_phone_and_website() {
        let mock = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/"))
            .respond_with(ResponseTemplate::new(200).set_body_json(overpass_two_nodes()))
            .mount(&mock)
            .await;

        let a = OsmOverpassAdapter::with_client(make_client(), format!("{}/", mock.uri()));
        let result = a.collect(&make_query(), amsterdam(), Radius::Km10, None).await;

        let rec = &result.records[0];
        assert_eq!(rec.fields["phone"], "+31201234567");
        assert_eq!(rec.fields["website"], "https://alpha.example.com");
    }

    #[tokio::test]
    async fn adapter_record_has_adapter_id_tag() {
        let mock = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/"))
            .respond_with(ResponseTemplate::new(200).set_body_json(overpass_two_nodes()))
            .mount(&mock)
            .await;

        let a = OsmOverpassAdapter::with_client(make_client(), format!("{}/", mock.uri()));
        let result = a.collect(&make_query(), amsterdam(), Radius::Km10, None).await;

        assert_eq!(result.records[0].fields["adapter_id"], "osm_overpass");
        assert_eq!(result.records[0].adapter_id, "osm_overpass");
    }

    #[tokio::test]
    async fn adapter_returns_success_with_zero_records_on_empty_elements() {
        let mock = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({ "elements": [] })))
            .mount(&mock)
            .await;

        let a = OsmOverpassAdapter::with_client(make_client(), format!("{}/", mock.uri()));
        let result = a.collect(&make_query(), amsterdam(), Radius::Km10, None).await;

        assert!(matches!(result.status, AdapterResultStatus::Success));
        assert_eq!(result.records.len(), 0);
    }

    #[tokio::test]
    async fn adapter_skips_elements_without_coordinates() {
        let mock = MockServer::start().await;
        let body = serde_json::json!({
            "elements": [
                // node with coords — included
                { "type": "node", "id": 1, "lat": 52.37, "lon": 4.90, "tags": { "name": "Has Coords" } },
                // relation without lat/lon and no center — excluded
                { "type": "relation", "id": 2, "tags": { "name": "No Coords" } }
            ]
        });
        Mock::given(method("POST"))
            .and(path("/"))
            .respond_with(ResponseTemplate::new(200).set_body_json(body))
            .mount(&mock)
            .await;

        let a = OsmOverpassAdapter::with_client(make_client(), format!("{}/", mock.uri()));
        let result = a.collect(&make_query(), amsterdam(), Radius::Km10, None).await;

        assert_eq!(result.records.len(), 1);
        assert_eq!(result.records[0].fields["name"], "Has Coords");
    }

    #[tokio::test]
    async fn adapter_uses_way_center_coordinates() {
        let mock = MockServer::start().await;
        let body = serde_json::json!({
            "elements": [
                {
                    "type": "way",
                    "id": 99,
                    "center": { "lat": 52.380, "lon": 4.910 },
                    "tags": { "name": "Way Business" }
                }
            ]
        });
        Mock::given(method("POST"))
            .and(path("/"))
            .respond_with(ResponseTemplate::new(200).set_body_json(body))
            .mount(&mock)
            .await;

        let a = OsmOverpassAdapter::with_client(make_client(), format!("{}/", mock.uri()));
        let result = a.collect(&make_query(), amsterdam(), Radius::Km10, None).await;

        assert_eq!(result.records.len(), 1);
        assert_eq!(result.records[0].fields["lat"], "52.38");
        assert_eq!(result.records[0].fields["lon"], "4.91");
    }

    #[tokio::test]
    async fn adapter_returns_failed_http4xx_on_429() {
        let mock = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/"))
            .respond_with(ResponseTemplate::new(429))
            .mount(&mock)
            .await;

        let a = OsmOverpassAdapter::with_client(make_client(), format!("{}/", mock.uri()));
        let result = a.collect(&make_query(), amsterdam(), Radius::Km10, None).await;
        assert!(matches!(result.status, AdapterResultStatus::Failed(ReasonCode::Http4xx)));
    }

    #[tokio::test]
    async fn adapter_returns_failed_http5xx_on_503() {
        let mock = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/"))
            .respond_with(ResponseTemplate::new(503))
            .mount(&mock)
            .await;

        let a = OsmOverpassAdapter::with_client(make_client(), format!("{}/", mock.uri()));
        let result = a.collect(&make_query(), amsterdam(), Radius::Km10, None).await;
        assert!(matches!(result.status, AdapterResultStatus::Failed(ReasonCode::Http5xx)));
    }

    #[tokio::test]
    async fn adapter_returns_failed_parse_error_on_invalid_json() {
        let mock = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/"))
            .respond_with(ResponseTemplate::new(200).set_body_string("not json"))
            .mount(&mock)
            .await;

        let a = OsmOverpassAdapter::with_client(make_client(), format!("{}/", mock.uri()));
        let result = a.collect(&make_query(), amsterdam(), Radius::Km10, None).await;
        assert!(matches!(result.status, AdapterResultStatus::Failed(ReasonCode::ParseError)));
    }

    // ── Unit tests for helpers ────────────────────────────────────────────────

    #[test]
    fn sanitize_ql_string_removes_special_chars() {
        assert_eq!(sanitize_ql_string("yoga studio"), "yoga studio");
        assert_eq!(sanitize_ql_string("yoga/studio"), "yogastudio");  // slash stripped
        // é is Unicode alphanumeric so it passes through
        assert_eq!(sanitize_ql_string("café"), "café");
        assert_eq!(sanitize_ql_string("gym-fitness_centre"), "gym-fitness_centre");
        // Overpass QL meta chars stripped
        assert_eq!(sanitize_ql_string("studio[type]"), "studiotype");
        assert_eq!(sanitize_ql_string("yoga;shop"), "yogashop");
    }

    #[test]
    fn urlencoded_encodes_special_chars() {
        let q = "data=[out:json]";
        let encoded = urlencoded(q);
        assert!(encoded.contains("%5B"));  // [ -> %5B
        assert!(encoded.contains("%3A"));  // : -> %3A
        assert!(encoded.contains("%5D"));  // ] -> %5D
    }

    #[test]
    fn build_overpass_query_contains_keyword_and_radius() {
        let q = build_overpass_query("yoga studio", 52.3676, 4.9041, 10000);
        assert!(q.contains("yoga studio"));
        assert!(q.contains("10000"));
        assert!(q.contains("52.3676"));
        assert!(q.contains("4.9041"));
        assert!(q.contains("[out:json]"));
    }
}
