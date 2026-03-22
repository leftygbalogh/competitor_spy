// runner.rs — T-016/T-017
// Core run logic, separated from argument parsing so acceptance tests can call
// run_with_urls() in-process against WireMock servers.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use chrono::Utc;

use competitor_spy_adapters::{
    adapter::{Geocoder, GeocodingError},
    nominatim::{NominatimAdapter, NominatimGeocoder},
    osm_overpass::OsmOverpassAdapter,
    registry::SourceRegistry,
    yelp::YelpAdapter,
    google_places::GooglePlacesAdapter,
};
use competitor_spy_credentials::store::CredentialStore;
use competitor_spy_domain::{
    normalizer,
    profile::deduplicate,
    query::{Location, Radius, SearchQuery},
    ranking::{DefaultRankingEngine, RankingEngine},
    run::{SearchRun, AdapterResultStatus},
};
use competitor_spy_output::{pdf, terminal};

// ── URL configuration (injectable for tests) ──────────────────────────────────

/// Holds all adapter/geocoder base URLs. Production uses real endpoints;
/// acceptance tests inject mock-server URIs.
pub struct AdapterUrls {
    pub nominatim: String,
    pub osm_overpass: String,
    pub yelp: String,
    pub google_places: String,
}

impl AdapterUrls {
    pub fn production() -> Self {
        Self {
            nominatim: "https://nominatim.openstreetmap.org".to_string(),
            osm_overpass: "https://overpass-api.de/api/interpreter".to_string(),
            yelp: "https://api.yelp.com".to_string(),
            google_places: "https://maps.googleapis.com".to_string(),
        }
    }
}

// ── Core run logic ────────────────────────────────────────────────────────────

/// Core run logic with injectable URLs. Called from `main` (production URLs)
/// and from acceptance tests (mock-server URLs).
pub async fn run_with_urls(
    industry: &str,
    location_input: &str,
    radius_km: u32,
    output_dir: &PathBuf,
    no_pdf: bool,
    urls: AdapterUrls,
    extra_credentials: HashMap<String, String>,
) -> i32 {
    // 1. Validate query
    let radius = match Radius::try_from(radius_km) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("error: {e}");
            return 1;
        }
    };

    let query = match SearchQuery::new(industry, location_input, radius) {
        Ok(q) => q,
        Err(e) => {
            eprintln!("error: {e}");
            return 1;
        }
    };

    // 2. Build SearchRun state machine
    let mut run = SearchRun::new(query.clone(), Utc::now());
    run.start_validating();

    // 3. Geocode location
    run.start_geocoding();
    let geocoder = NominatimGeocoder::new(&urls.nominatim);
    let location: Location = match geocoder.geocode(location_input).await {
        Ok(loc) => loc,
        Err(GeocodingError::NoResults) => {
            eprintln!("error: no results found for location '{location_input}'");
            return 1;
        }
        Err(e) => {
            eprintln!("error: geocoding failed: {e}");
            return 1;
        }
    };
    run.set_location(location.clone());

    // 4. Load credentials
    let cred_path = credential_store_path();
    let mut credentials: HashMap<String, String> = extra_credentials;

    if cred_path.exists() {
        let passphrase = std::env::var("CSPY_CREDENTIAL_PASSPHRASE").unwrap_or_default();
        match CredentialStore::open(cred_path, passphrase) {
            Ok(store) => {
                for adapter_id in &["yelp", "google_places"] {
                    if let Ok(Some(secret)) = store.retrieve(adapter_id) {
                        if let Ok(s) = secret.as_str() {
                            credentials
                                .entry(adapter_id.to_string())
                                .or_insert_with(|| s.to_string());
                        }
                    }
                }
            }
            Err(e) => {
                log::warn!("Failed to open credential store: {e}; proceeding without credentials");
            }
        }
    }

    // 5. Build registry and run all adapters concurrently
    let mut registry = SourceRegistry::new();
    registry.register(Arc::new(NominatimAdapter::new(&urls.nominatim)));
    registry.register(Arc::new(OsmOverpassAdapter::new(&urls.osm_overpass)));
    registry.register(Arc::new(YelpAdapter::new(&urls.yelp)));
    registry.register(Arc::new(GooglePlacesAdapter::new(&urls.google_places)));

    let source_results = registry
        .collect_all(&query, location.clone(), radius, &credentials)
        .await;

    // 6. Record results in run state machine
    let mut any_failed = false;
    let mut raw_records = Vec::new();
    for result in source_results {
        if matches!(result.status, AdapterResultStatus::Failed(_)) {
            any_failed = true;
        }
        raw_records.extend(result.records.clone());
        run.add_source_result(result);
    }

    // 7. Normalise → deduplicate → rank
    run.start_ranking();
    let competitors = normalizer::normalize(raw_records, &location);
    let competitors = deduplicate(competitors);
    let competitors = DefaultRankingEngine::new().rank(competitors, &query);
    run.set_competitors(competitors);

    // 8. Complete the run
    let completed_at = Utc::now();
    if any_failed {
        run.complete_with_warning(completed_at);
    } else {
        run.complete(completed_at);
    }

    // 9. Terminal output
    if let Err(e) = terminal::render_stdout(&run) {
        eprintln!("error: failed to write terminal output: {e}");
        return 1;
    }

    // 10. PDF output (unless --no-pdf)
    if !no_pdf {
        match pdf::render_to_dir(&run, output_dir) {
            Ok(path) => {
                log::info!("PDF written to {}", path.display());
            }
            Err(e) => {
                log::warn!("Failed to write PDF: {e}");
            }
        }
    }

    0
}

// ── Helpers ───────────────────────────────────────────────────────────────────

pub fn credential_store_path() -> PathBuf {
    #[cfg(windows)]
    {
        let appdata = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(appdata)
            .join("competitor-spy")
            .join("credentials")
    }
    #[cfg(not(windows))]
    {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home)
            .join(".config")
            .join("competitor-spy")
            .join("credentials")
    }
}
