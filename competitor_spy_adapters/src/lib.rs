// competitor_spy_adapters
//
// Async crate: HTTP I/O only. Implements the SourceAdapter trait per data provider.
// Depends on competitor_spy_domain for domain types. No rendering or credential
// prompting logic here — credential retrieval is delegated to competitor_spy_credentials.

pub mod pacing;
pub mod registry;
pub mod nominatim;
pub mod osm_overpass;
pub mod yelp;
pub mod google_places;
