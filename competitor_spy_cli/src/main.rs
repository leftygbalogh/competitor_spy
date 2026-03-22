// competitor_spy_cli — entry point only.
//
// No domain logic lives here. This binary parses CLI arguments and
// delegates entirely to the domain and adapter crates.
// Implemented in T-016.

// competitor-spy CLI entry point — T-016
// TDD: argument parsing, validation, and lifecycle integration tested via
//      acceptance tests (T-017). This file wires all crates together.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use clap::Parser;
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
    query::{Radius, SearchQuery},
    ranking::{DefaultRankingEngine, RankingEngine},
    run::{SearchRun, AdapterResultStatus},
};
use competitor_spy_output::{pdf, terminal};
use competitor_spy_telemetry::init::init_telemetry;

// ── CLI flags ─────────────────────────────────────────────────────────────────

#[derive(Parser, Debug)]
#[command(
    name = "competitor-spy",
    version,
    about = "Discover and rank competitors within a geographic radius"
)]
struct Cli {
    /// Industry or business category to search for
    #[arg(long)]
    industry: String,

    /// Human-readable location string (e.g. "London, UK")
    #[arg(long)]
    location: String,

    /// Search radius in km: 5, 10, 20, 25, or 50
    #[arg(long)]
    radius: u32,

    /// Directory to write the PDF report (default: current directory)
    #[arg(long, default_value = ".")]
    output_dir: PathBuf,

    /// Skip PDF output; only render to terminal
    #[arg(long, default_value_t = false)]
    no_pdf: bool,

    /// Log verbosity: trace, debug, info, warn, error
    #[arg(long, default_value = "info")]
    log_level: String,

    /// Seed for request pacing (overrides CSPY_PACING_SEED env var)
    #[arg(long)]
    pacing_seed: Option<u64>,
}

// ── Entry point ───────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    std::process::exit(run().await);
}

async fn run() -> i32 {
    let cli = Cli::parse();

    // Resolve pacing seed: CLI flag > env var > none
    let _pacing_seed: Option<u64> = cli.pacing_seed.or_else(|| {
        std::env::var("CSPY_PACING_SEED")
            .ok()
            .and_then(|s| s.parse().ok())
    });

    // 1. Init telemetry — guard must stay alive for the duration of main
    let _guard = match init_telemetry(&cli.log_level) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("error: failed to initialise telemetry: {e}");
            return 1;
        }
    };

    // 2. Validate query
    let radius = match Radius::try_from(cli.radius) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("error: {e}");
            return 1;
        }
    };

    let query = match SearchQuery::new(&cli.industry, &cli.location, radius) {
        Ok(q) => q,
        Err(e) => {
            eprintln!("error: {e}");
            return 1;
        }
    };

    // 3. Build SearchRun state machine
    let mut run = SearchRun::new(query.clone(), Utc::now());
    run.start_validating();

    // 4. Geocode location
    run.start_geocoding();
    let geocoder = NominatimGeocoder::new("https://nominatim.openstreetmap.org");
    let location: competitor_spy_domain::query::Location = match geocoder.geocode(&cli.location).await {
        Ok(loc) => loc,
        Err(GeocodingError::NoResults) => {
            eprintln!("error: no results found for location '{}'", cli.location);
            return 1;
        }
        Err(e) => {
            eprintln!("error: geocoding failed: {e}");
            return 1;
        }
    };
    run.set_location(location.clone());

    // 5. Load credentials from platform-appropriate store location
    let cred_path = credential_store_path();
    let mut credentials: HashMap<String, String> = HashMap::new();

    if cred_path.exists() {
        // Passphrase via env var; empty string = unencrypted store for OSS users
        let passphrase = std::env::var("CSPY_CREDENTIAL_PASSPHRASE").unwrap_or_default();
        match CredentialStore::open(cred_path, passphrase) {
            Ok(store) => {
                for adapter_id in &["yelp", "google_places"] {
                    if let Ok(Some(secret)) = store.retrieve(adapter_id) {
                        if let Ok(s) = secret.as_str() {
                            credentials.insert(adapter_id.to_string(), s.to_string());
                        }
                    }
                }
            }
            Err(e) => {
                // Non-fatal: log warning, continue without credentials
                log::warn!("Failed to open credential store: {e}; proceeding without credentials");
            }
        }
    }

    // 6. Build registry and run all adapters concurrently
    let mut registry = SourceRegistry::new();
    registry.register(Arc::new(NominatimAdapter::new(
        "https://nominatim.openstreetmap.org",
    )));
    registry.register(Arc::new(OsmOverpassAdapter::new(
        "https://overpass-api.de/api",
    )));
    registry.register(Arc::new(YelpAdapter::new("https://api.yelp.com")));
    registry.register(Arc::new(GooglePlacesAdapter::new(
        "https://maps.googleapis.com",
    )));

    let source_results = registry
        .collect_all(&query, location.clone(), radius, &credentials)
        .await;

    // 7. Record results in run state machine
    let mut any_failed = false;
    let mut raw_records = Vec::new();
    for result in source_results {
        if matches!(result.status, AdapterResultStatus::Failed(_)) {
            any_failed = true;
        }
        raw_records.extend(result.records.clone());
        run.add_source_result(result);
    }

    // 8. Normalise → deduplicate → rank
    run.start_ranking();
    let competitors = normalizer::normalize(raw_records, &location);
    let competitors = deduplicate(competitors);
    let competitors = DefaultRankingEngine::new().rank(competitors, &query);
    run.set_competitors(competitors);

    // 9. Complete the run
    let completed_at = Utc::now();
    if any_failed {
        run.complete_with_warning(completed_at);
    } else {
        run.complete(completed_at);
    }

    // 10. Terminal output
    if let Err(e) = terminal::render_stdout(&run) {
        eprintln!("error: failed to write terminal output: {e}");
        return 1;
    }

    // 11. PDF output (unless --no-pdf)
    if !cli.no_pdf {
        match pdf::render_to_dir(&run, &cli.output_dir) {
            Ok(path) => {
                log::info!("PDF written to {}", path.display());
            }
            Err(e) => {
                // Non-fatal: warn and continue; exit 0 per spec §6.4
                log::warn!("Failed to write PDF: {e}");
            }
        }
    }

    0
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn credential_store_path() -> PathBuf {
    #[cfg(windows)]
    {
        let appdata = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(appdata).join("competitor-spy").join("credentials")
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
