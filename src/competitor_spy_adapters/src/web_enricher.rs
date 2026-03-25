// WebEnricher — T-031
// Spec: FORMAL_SPEC.md §13.2.3 (FR-V3-002), §13.3.1, §13.3.2
//
// Fetches each competitor's website (reqwest HTTP GET) and runs all 5 extractors.
// Applies pacing delay after each fetch. Sequential (one at a time) to reduce detection risk.
// Returns Vec<WebEnrichment> — one entry per competitor in the same order.

use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use std::time::Duration;

use competitor_spy_domain::enrichment::{
    EnrichmentErrorCode, FetchStatus, WebEnrichment,
};
use competitor_spy_domain::profile::Competitor;

use crate::extractors::class_descriptions::extract_class_descriptions;
use crate::extractors::lesson_types::extract_lesson_types;
use crate::extractors::pricing::extract_pricing;
use crate::extractors::schedule::extract_schedule;
use crate::extractors::testimonials::extract_testimonials;
use crate::pacing::PacingPolicy;

const DEFAULT_TIMEOUT_SECS: u64 = 15;
const MAX_REDIRECTS: usize = 3;

/// Configuration for the web enricher.
pub struct EnricherConfig {
    /// Per-fetch timeout in seconds (range [5, 60]).  Default: 15.
    pub timeout_secs: u64,
    /// Whether to accept self-signed or expired TLS certificates.
    pub allow_insecure_tls: bool,
}

impl Default for EnricherConfig {
    fn default() -> Self {
        Self {
            timeout_secs: DEFAULT_TIMEOUT_SECS,
            allow_insecure_tls: false,
        }
    }
}

/// Orchestrates website enrichment for a list of competitors.
pub struct WebEnricher {
    client: Client,
    config: EnricherConfig,
}

impl WebEnricher {
    /// Construct a new enricher with the given configuration.
    ///
    /// Returns an error if the HTTP client cannot be initialised (e.g. bad TLS config).
    pub fn new(config: EnricherConfig) -> Result<Self, reqwest::Error> {
        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static(
                "Mozilla/5.0 (compatible; CompetitorSpy/3.0; +https://github.com/local)",
            ),
        );

        let timeout_secs = config.timeout_secs;
        let allow_insecure_tls = config.allow_insecure_tls;

        let client = Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .redirect(reqwest::redirect::Policy::limited(MAX_REDIRECTS))
            .default_headers(headers)
            .danger_accept_invalid_certs(allow_insecure_tls)
            .build()?;

        // SEC-004: alert the operator when insecure TLS is active.
        if allow_insecure_tls {
            tracing::warn!(
                "--allow-insecure-tls active: TLS certificate validation disabled. \
                Do not use in production."
            );
        }

        Ok(Self {
            client,
            config: EnricherConfig {
                timeout_secs,
                allow_insecure_tls,
            },
        })
    }

    /// Run enrichment for all competitors.  Returns one `WebEnrichment` per competitor,
    /// in the same order.  Never panics; failures are recorded as `FetchStatus::Failed`.
    pub fn enrich(
        &self,
        competitors: &[Competitor],
        pacing: &PacingPolicy,
    ) -> Vec<WebEnrichment> {
        competitors
            .iter()
            .map(|c| self.enrich_one(c, pacing))
            .collect()
    }

    fn enrich_one(&self, competitor: &Competitor, pacing: &PacingPolicy) -> WebEnrichment {
        let url = match competitor.profile.website.value.as_deref() {
            Some(u) if !u.is_empty() => u.to_string(),
            _ => {
                // No URL: no HTTP request, no pacing delay.
                return WebEnrichment::failed(competitor.id, EnrichmentErrorCode::NoUrl);
            }
        };

        let result = self.fetch_html(&url);

        // Apply pacing delay after the fetch attempt (success or failure).
        pacing.wait();

        match result {
            Err(e) => WebEnrichment::failed(competitor.id, e),
            Ok(html) => {
                let pricing = extract_pricing(&html);
                let lesson_types = extract_lesson_types(&html);
                let schedule = extract_schedule(&html);
                let testimonials = extract_testimonials(&html);
                let class_descriptions = extract_class_descriptions(&html);

                WebEnrichment {
                    competitor_id: competitor.id,
                    fetch_status: FetchStatus::Success,
                    pricing,
                    lesson_types,
                    schedule,
                    testimonials,
                    class_descriptions,
                }
            }
        }
    }

    fn fetch_html(&self, url: &str) -> Result<String, EnrichmentErrorCode> {
        let response = self.client.get(url).send().map_err(|e| {
            if e.is_timeout() {
                EnrichmentErrorCode::Timeout
            } else if e.is_connect() || e.is_request() {
                // Distinguish DNS from TLS/connect errors as best we can.
                // reqwest doesn't expose DNS errors directly; map to DnsFailure heuristically.
                let msg = e.to_string().to_lowercase();
                if msg.contains("dns") || msg.contains("resolve") || msg.contains("lookup") {
                    EnrichmentErrorCode::DnsFailure
                } else {
                    EnrichmentErrorCode::HttpError(0)
                }
            } else {
                EnrichmentErrorCode::HttpError(0)
            }
        })?;

        let status = response.status();
        if !status.is_success() {
            return Err(EnrichmentErrorCode::HttpError(status.as_u16()));
        }

        response.text().map_err(|_| EnrichmentErrorCode::ParseError)
    }

    pub fn timeout_secs(&self) -> u64 {
        self.config.timeout_secs
    }
}

// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use competitor_spy_domain::enrichment::EnrichmentErrorCode;
    use competitor_spy_domain::profile::{BusinessProfile, Competitor, DataPoint, Confidence};
    use competitor_spy_domain::query::Location;
    use uuid::Uuid;

    fn make_competitor(website: Option<&str>) -> Competitor {
        let mut profile = BusinessProfile::empty();
        if let Some(url) = website {
            profile.website = DataPoint::present("website", url, "test", Confidence::High);
        }
        let location = Location::new(48.0, 16.0).unwrap();
        Competitor {
            id: Uuid::new_v4(),
            profile,
            location,
            distance_km: 1.0,
            keyword_score: 0.5,
            visibility_score: 0.5,
            rank: 1,
        }
    }

    fn zero_delay_pacing() -> PacingPolicy {
        PacingPolicy::from_seed(42, true)
    }

    #[test]
    fn competitor_without_url_produces_no_url_error() {
        let enricher = WebEnricher::new(EnricherConfig::default()).unwrap();
        let competitor = make_competitor(None);
        let pacing = zero_delay_pacing();
        let result = enricher.enrich(&[competitor], &pacing);
        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0].fetch_status,
            FetchStatus::Failed(EnrichmentErrorCode::NoUrl)
        );
        assert!(!result[0].has_any_field());
    }

    #[test]
    fn enricher_config_default_values() {
        let cfg = EnricherConfig::default();
        assert_eq!(cfg.timeout_secs, 15);
        assert!(!cfg.allow_insecure_tls);
    }

    /// SEC-004 / T-040 — 07_QUALITY_DIMENSIONS.md §6
    ///
    /// When `allow_insecure_tls` is `true`, `WebEnricher::new()` must emit a
    /// WARN-level tracing event so that operators are alerted that TLS
    /// certificate validation is disabled.  Silent activation of an insecure
    /// mode violates the "secure defaults" quality dimension.
    ///
    /// RED STATE: no `tracing::warn!` call exists in `WebEnricher::new()` →
    ///   no event is captured → assertion `output.contains("allow-insecure-tls")`
    ///   FAILS (red).
    ///
    /// GREEN STATE: fix adds `if allow_insecure_tls { tracing::warn!("..."); }`
    ///   inside `WebEnricher::new()` → event captured → assertion passes.
    #[test]
    fn insecure_tls_emits_warn_event() {
        use std::sync::{Arc, Mutex};
        use tracing_subscriber::prelude::*;

        // ── Captured writer ──────────────────────────────────────────────────
        let buf: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::new()));

        struct CaptureWriter(Arc<Mutex<Vec<u8>>>);

        impl std::io::Write for CaptureWriter {
            fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
                self.0.lock().unwrap().extend_from_slice(b);
                Ok(b.len())
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }

        impl tracing_subscriber::fmt::MakeWriter<'_> for CaptureWriter {
            type Writer = CaptureWriter;
            fn make_writer(&self) -> Self::Writer {
                CaptureWriter(Arc::clone(&self.0))
            }
        }

        // Subscribe with a captured writer for this test only.
        // with_default() avoids polluting the global subscriber.
        let subscriber = tracing_subscriber::registry().with(
            tracing_subscriber::fmt::layer()
                .with_ansi(false)
                .with_writer(CaptureWriter(Arc::clone(&buf))),
        );

        tracing::subscriber::with_default(subscriber, || {
            let _enricher = WebEnricher::new(EnricherConfig {
                allow_insecure_tls: true,
                timeout_secs: 15,
            })
            .expect("WebEnricher::new should succeed even with insecure TLS flag");
        });

        let output = String::from_utf8(buf.lock().unwrap().clone()).unwrap();

        assert!(
            output.to_ascii_lowercase().contains("allow-insecure-tls")
                || output.to_ascii_lowercase().contains("insecure"),
            "SEC-004: no WARN event emitted when allow_insecure_tls=true.\n\
             Hint: add `tracing::warn!(\"--allow-insecure-tls active: TLS \
             certificate validation disabled. Do not use in production.\")` \
             inside WebEnricher::new() when allow_insecure_tls is true.\n\
             Captured output:\n{output}"
        );
    }

    // Integration tests using a live mock HTTP server are in tests/integration/.
    // They require a running wiremock instance and are guarded by --test flag.
}
