// SourceAdapter trait + Geocoder trait — T-009
// TDD: trait contracts defined first; concrete tests via Nominatim in nominatim.rs.

use async_trait::async_trait;
use thiserror::Error;
use competitor_spy_domain::query::{Location, Radius, SearchQuery};
use competitor_spy_domain::run::SourceResult;

// ── GeocodingError ────────────────────────────────────────────────────────────

#[derive(Debug, Error, PartialEq)]
pub enum GeocodingError {
    #[error("no results found for location string")]
    NoResults,
    #[error("HTTP error: status {0}")]
    Http(u16),
    #[error("response parse error: {0}")]
    Parse(String),
}

// ── Geocoder ──────────────────────────────────────────────────────────────────

/// Resolves a human-readable location string to geographic coordinates.
///
/// If multiple candidates are returned, the implementation selects the
/// highest-confidence match (first entry in ordered results) and should emit an
/// audit event noting the selection.
#[async_trait]
pub trait Geocoder: Send + Sync {
    async fn geocode(&self, location_string: &str) -> Result<Location, GeocodingError>;
}

// ── SourceAdapter ─────────────────────────────────────────────────────────────

/// Contract for every data-source adapter (Nominatim, OSM Overpass, Yelp, etc.).
///
/// Adapters are stateless with respect to individual collect calls; all shared
/// state (HTTP client, base URL) is held in `self` behind interior mutability
/// if required. Each adapter enforces its own pacing via [`PacingPolicy`].
#[async_trait]
pub trait SourceAdapter: Send + Sync {
    /// Stable identifier for this adapter, e.g. `"nominatim"`, `"yelp"`.
    fn adapter_id(&self) -> &str;

    /// Whether this adapter requires a credential from the CredentialStore.
    fn requires_credential(&self) -> bool;

    /// Execute one collection pass.
    ///
    /// `credential` is `Some(plaintext)` when `requires_credential()` is true
    /// and the store contains an entry; `None` otherwise.
    async fn collect(
        &self,
        query: &SearchQuery,
        location: Location,
        radius: Radius,
        credential: Option<&str>,
    ) -> SourceResult;
}
