// runner.rs — T-016/T-017
// Core run logic, separated from argument parsing so acceptance tests can call
// run_with_urls() in-process against WireMock servers.

use std::collections::HashMap;
use std::io::{self, IsTerminal};
use std::path::PathBuf;
use std::sync::Arc;

use chrono::Utc;
use rpassword::prompt_password;

use competitor_spy_adapters::{
    adapter::{Geocoder, GeocodingError},
    nominatim::{NominatimAdapter, NominatimGeocoder},
    osm_overpass::OsmOverpassAdapter,
    pacing::PacingPolicy,
    registry::SourceRegistry,
    web_enricher::{EnricherConfig, WebEnricher},
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

const CREDENTIAL_ADAPTER_IDS: [&str; 2] = ["yelp", "google_places"];

impl AdapterUrls {
    pub fn production() -> Self {
        Self {
            nominatim: "https://nominatim.openstreetmap.org".to_string(),
            osm_overpass: "https://overpass-api.de/api/interpreter".to_string(),
            yelp: "https://api.yelp.com".to_string(),
            google_places: "https://places.googleapis.com".to_string(),
        }
    }

    fn is_production(&self) -> bool {
        self.nominatim == "https://nominatim.openstreetmap.org"
            && self.osm_overpass == "https://overpass-api.de/api/interpreter"
            && self.yelp == "https://api.yelp.com"
            && self.google_places == "https://places.googleapis.com"
    }
}

// ── Core run logic ────────────────────────────────────────────────────────────

/// Core run logic with injectable URLs. Called from `main` (production URLs)
/// and from acceptance tests (mock-server URLs).
pub async fn run_with_urls(
    industry: &str,
    location_input: &str,
    radius_km: u32,
    output_dir: Option<PathBuf>,
    no_pdf: bool,
    detail: bool,
    urls: AdapterUrls,
    extra_credentials: HashMap<String, String>,
    no_enrichment: bool,
    allow_insecure_tls: bool,
    enrichment_timeout_secs: u64,
    pacing_seed: Option<u64>,
) -> i32 {
    // Guard (SEC-005): reject --output-dir values containing path traversal sequences.
    if let Some(ref dir) = output_dir {
        if dir.components().any(|c| c == std::path::Component::ParentDir) {
            eprintln!("error: --output-dir must not contain path traversal sequences (..)");
            return 1;
        }
    }

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
    let mut credentials: HashMap<String, String> = extra_credentials;
    match credential_store_path() {
        Ok(cred_path) => {
            load_stored_credentials(&cred_path, &mut credentials, urls.is_production());
        }
        Err(e) => {
            log::warn!("{e}; credential-backed adapters will be skipped");
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

    // 7. Normalise → deduplicate → [enrich →] rank
    let competitors_pre_rank = normalizer::normalize(raw_records, &location);
    let competitors_pre_rank = deduplicate(competitors_pre_rank);

    if !no_enrichment {
        run.start_enriching();
        let pacing = match pacing_seed {
            Some(seed) => PacingPolicy::from_seed(seed, false),
            None => PacingPolicy::new(),
        };
        let cfg = EnricherConfig {
            timeout_secs: enrichment_timeout_secs,
            allow_insecure_tls,
        };
        // reqwest::blocking must not run on the async executor thread — offload to
        // the blocking thread pool so the runtime remains responsive.
        let competitors_for_enrich = competitors_pre_rank.clone();
        let enrichments = tokio::task::spawn_blocking(move || {
            match WebEnricher::new(cfg) {
                Ok(enricher) => enricher.enrich(&competitors_for_enrich, &pacing),
                Err(e) => {
                    log::warn!("Failed to initialise WebEnricher: {e}; proceeding without enrichment");
                    vec![]
                }
            }
        })
        .await
        .unwrap_or_else(|e| {
            log::warn!("Enrichment task panicked: {e}; proceeding without enrichment");
            vec![]
        });
        run.set_enrichments(enrichments);
    }

    run.start_ranking();
    let competitors = DefaultRankingEngine::new().rank(competitors_pre_rank, &query);
    run.set_competitors(competitors);

    // 8. Complete the run
    let completed_at = Utc::now();
    if any_failed {
        run.complete_with_warning(completed_at);
    } else {
        run.complete(completed_at);
    }

    // 9. Terminal output
    if let Err(e) = terminal::render_stdout(&run, detail) {
        eprintln!("error: failed to write terminal output: {e}");
        return 1;
    }

    // 10. PDF output (unless --no-pdf)
    if !no_pdf {
        let resolved_dir = match output_dir {
            Some(d) => d,
            None => default_output_dir(),
        };
        match pdf::render_to_dir(&run, detail, &resolved_dir) {
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

/// Resolve the default PDF output directory.
/// Returns the `reports/` directory anchored to the project root.
///
/// The project root is derived from the running binary's location:
/// `<exe>` lives at `target/{debug,release}/competitor-spy[.exe]`, so
/// three `.parent()` calls reach `Competitor_Spy_V2/`.
/// Falls back to `./reports` relative to CWD if the binary path cannot
/// be resolved (e.g. unusual install layouts).
pub fn default_output_dir() -> PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        // exe → target/{debug,release}/ → target/ → project root
        if let Some(project_root) = exe.parent().and_then(|p| p.parent()).and_then(|p| p.parent()) {
            let dir = project_root.join("reports");
            if std::fs::create_dir_all(&dir).is_ok() {
                return dir;
            }
        }
    }
    // Fallback: relative to CWD
    let dir = PathBuf::from("reports");
    if std::fs::create_dir_all(&dir).is_ok() {
        dir
    } else {
        PathBuf::from(".")
    }
}

/// SEC-006: returns `Err` when neither `APPDATA` (Windows) nor `HOME` (Unix)
/// is set.  Callers must never silently fall back to a CWD-relative path.
pub fn credential_store_path() -> Result<PathBuf, String> {
    #[cfg(windows)]
    {
        let appdata = std::env::var("APPDATA").map_err(|_| {
            "APPDATA environment variable is not set; cannot locate the credential store"
                .to_string()
        })?;
        Ok(PathBuf::from(appdata)
            .join("competitor-spy")
            .join("credentials"))
    }
    #[cfg(not(windows))]
    {
        let home = std::env::var("HOME").map_err(|_| {
            "HOME environment variable is not set; cannot locate the credential store"
                .to_string()
        })?;
        Ok(PathBuf::from(home)
            .join(".config")
            .join("competitor-spy")
            .join("credentials"))
    }
}

fn load_stored_credentials(
    cred_path: &PathBuf,
    credentials: &mut HashMap<String, String>,
    allow_prompt: bool,
) {
    if !cred_path.exists() {
        return;
    }

    let passphrase = match credential_store_passphrase(allow_prompt) {
        Some(passphrase) => passphrase,
        None => {
            log::warn!(
                "Credential store exists at {} but no passphrase is available; credential-backed adapters will be skipped",
                cred_path.display()
            );
            return;
        }
    };

    let store = match CredentialStore::open(cred_path.clone(), passphrase) {
        Ok(store) => store,
        Err(error) => {
            log::warn!("Failed to open credential store: {error}; proceeding without credentials");
            return;
        }
    };

    for adapter_id in CREDENTIAL_ADAPTER_IDS {
        if credentials.contains_key(adapter_id) {
            continue;
        }

        match store.retrieve(adapter_id) {
            Ok(Some(secret)) => {
                if let Ok(value) = secret.as_str() {
                    credentials.insert(adapter_id.to_string(), value.to_string());
                }
            }
            Ok(None) => {}
            Err(error) => {
                log::warn!(
                    "Failed to decrypt credential for adapter '{}' from store: {error}",
                    adapter_id
                );
            }
        }
    }
}

fn credential_store_passphrase(allow_prompt: bool) -> Option<String> {
    match std::env::var("CSPY_CREDENTIAL_PASSPHRASE") {
        Ok(passphrase) if !passphrase.trim().is_empty() => Some(passphrase),
        _ if allow_prompt && stdin_is_interactive() => prompt_password("Credential store passphrase: ")
            .ok()
            .filter(|value| !value.trim().is_empty()),
        _ => None,
    }
}

fn stdin_is_interactive() -> bool {
    io::stdin().is_terminal()
}
