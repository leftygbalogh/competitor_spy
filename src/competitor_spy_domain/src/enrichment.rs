// WebEnrichment, FetchStatus, EnrichmentErrorCode, enrichment_coverage — T-025
// TDD: tests below written conceptually first; coverage metric is RECONSTRUCTION-CRITICAL.

use uuid::Uuid;

// ── EnrichmentErrorCode ───────────────────────────────────────────────────────

/// Reason a website enrichment fetch failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnrichmentErrorCode {
    /// HTTP error response; contains the status code (0 = TLS/connection error).
    HttpError(u16),
    /// Request timed out.
    Timeout,
    /// DNS resolution failed.
    DnsFailure,
    /// Page fetched but HTML could not be parsed.
    ParseError,
    /// Competitor has no website URL in the business profile.
    NoUrl,
}

impl std::fmt::Display for EnrichmentErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EnrichmentErrorCode::HttpError(code) => write!(f, "HTTP_ERROR({code})"),
            EnrichmentErrorCode::Timeout         => write!(f, "TIMEOUT"),
            EnrichmentErrorCode::DnsFailure      => write!(f, "DNS_FAILURE"),
            EnrichmentErrorCode::ParseError      => write!(f, "PARSE_ERROR"),
            EnrichmentErrorCode::NoUrl           => write!(f, "NO_URL"),
        }
    }
}

// ── FetchStatus ───────────────────────────────────────────────────────────────

/// Outcome of the HTTP fetch phase for one competitor website.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FetchStatus {
    Success,
    Failed(EnrichmentErrorCode),
}

impl FetchStatus {
    pub fn is_success(&self) -> bool {
        matches!(self, FetchStatus::Success)
    }
}

// ── WebEnrichment ─────────────────────────────────────────────────────────────

/// Website-derived enrichment data for one competitor.
///
/// Invariants:
/// - When `fetch_status = Failed`, all enrichment fields are `None`.
/// - A successful fetch may still have all fields `None` (no extractable content). This is valid.
#[derive(Debug, Clone)]
pub struct WebEnrichment {
    pub competitor_id: Uuid,
    pub fetch_status: FetchStatus,

    /// Extracted pricing/cost information.  `None` = not found or fetch failed.
    pub pricing: Option<String>,

    /// Lesson or discipline types offered (e.g. ["Reformer", "Mat"]).
    pub lesson_types: Option<Vec<String>>,

    /// Timetable or schedule text.
    pub schedule: Option<String>,

    /// On-site customer testimonials (max 10 items, max 500 chars each).
    pub testimonials: Option<Vec<String>>,

    /// Class/course description passages (max 8 items, max 800 chars each).
    pub class_descriptions: Option<Vec<String>>,
}

impl WebEnrichment {
    /// Construct a failed enrichment record.  All enrichment fields are `None`.
    pub fn failed(competitor_id: Uuid, error: EnrichmentErrorCode) -> Self {
        Self {
            competitor_id,
            fetch_status: FetchStatus::Failed(error),
            pricing: None,
            lesson_types: None,
            schedule: None,
            testimonials: None,
            class_descriptions: None,
        }
    }

    /// Returns `true` if at least one enrichment field was successfully extracted.
    pub fn has_any_field(&self) -> bool {
        self.pricing.is_some()
            || self.lesson_types.is_some()
            || self.schedule.is_some()
            || self.testimonials.is_some()
            || self.class_descriptions.is_some()
    }
}

// ── enrichment_coverage ───────────────────────────────────────────────────────

/// Fraction (0.0–1.0) of enrichments that have at least one extracted field.
/// Returns 0.0 for an empty slice.
pub fn enrichment_coverage(enrichments: &[WebEnrichment]) -> f64 {
    if enrichments.is_empty() {
        return 0.0;
    }
    let enriched = enrichments.iter().filter(|e| e.has_any_field()).count();
    enriched as f64 / enrichments.len() as f64
}

/// Coverage threshold below which a warning is shown in reports.
pub const COVERAGE_THRESHOLD: f64 = 0.60;

// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn any_id() -> Uuid {
        Uuid::new_v4()
    }

    fn enriched_record(id: Uuid) -> WebEnrichment {
        WebEnrichment {
            competitor_id: id,
            fetch_status: FetchStatus::Success,
            pricing: Some("€15 per class".to_string()),
            lesson_types: None,
            schedule: None,
            testimonials: None,
            class_descriptions: None,
        }
    }

    fn failed_record(id: Uuid) -> WebEnrichment {
        WebEnrichment::failed(id, EnrichmentErrorCode::Timeout)
    }

    fn empty_success_record(id: Uuid) -> WebEnrichment {
        WebEnrichment {
            competitor_id: id,
            fetch_status: FetchStatus::Success,
            pricing: None,
            lesson_types: None,
            schedule: None,
            testimonials: None,
            class_descriptions: None,
        }
    }

    // ── has_any_field ─────────────────────────────────────────────────────────

    #[test]
    fn has_any_field_true_when_pricing_some() {
        let id = any_id();
        let e = enriched_record(id);
        assert!(e.has_any_field());
    }

    #[test]
    fn has_any_field_false_when_all_none() {
        let id = any_id();
        let e = empty_success_record(id);
        assert!(!e.has_any_field());
    }

    #[test]
    fn has_any_field_true_for_lesson_types_only() {
        let e = WebEnrichment {
            competitor_id: any_id(),
            fetch_status: FetchStatus::Success,
            pricing: None,
            lesson_types: Some(vec!["Reformer".to_string()]),
            schedule: None,
            testimonials: None,
            class_descriptions: None,
        };
        assert!(e.has_any_field());
    }

    // ── failed constructor ────────────────────────────────────────────────────

    #[test]
    fn failed_constructor_sets_all_fields_none() {
        let id = any_id();
        let e = WebEnrichment::failed(id, EnrichmentErrorCode::NoUrl);
        assert_eq!(e.fetch_status, FetchStatus::Failed(EnrichmentErrorCode::NoUrl));
        assert!(e.pricing.is_none());
        assert!(e.lesson_types.is_none());
        assert!(e.schedule.is_none());
        assert!(e.testimonials.is_none());
        assert!(e.class_descriptions.is_none());
        assert!(!e.has_any_field());
    }

    // ── enrichment_coverage ───────────────────────────────────────────────────

    #[test]
    fn coverage_empty_slice_returns_zero() {
        assert_eq!(enrichment_coverage(&[]), 0.0);
    }

    #[test]
    fn coverage_all_enriched_returns_one() {
        let e1 = enriched_record(any_id());
        let e2 = enriched_record(any_id());
        assert_eq!(enrichment_coverage(&[e1, e2]), 1.0);
    }

    #[test]
    fn coverage_none_enriched_returns_zero() {
        let e1 = failed_record(any_id());
        let e2 = empty_success_record(any_id());
        assert_eq!(enrichment_coverage(&[e1, e2]), 0.0);
    }

    #[test]
    fn coverage_half_enriched_returns_half() {
        let e1 = enriched_record(any_id());
        let e2 = failed_record(any_id());
        let cov = enrichment_coverage(&[e1, e2]);
        assert!((cov - 0.5).abs() < f64::EPSILON, "expected 0.5, got {cov}");
    }

    #[test]
    fn coverage_below_threshold_flag() {
        // 1 out of 3 = 0.333, below COVERAGE_THRESHOLD (0.60)
        let e1 = enriched_record(any_id());
        let e2 = failed_record(any_id());
        let e3 = empty_success_record(any_id());
        let cov = enrichment_coverage(&[e1, e2, e3]);
        assert!(cov < COVERAGE_THRESHOLD);
    }

    // ── EnrichmentErrorCode display ───────────────────────────────────────────

    #[test]
    fn error_code_display() {
        assert_eq!(EnrichmentErrorCode::HttpError(404).to_string(), "HTTP_ERROR(404)");
        assert_eq!(EnrichmentErrorCode::Timeout.to_string(),        "TIMEOUT");
        assert_eq!(EnrichmentErrorCode::DnsFailure.to_string(),     "DNS_FAILURE");
        assert_eq!(EnrichmentErrorCode::ParseError.to_string(),     "PARSE_ERROR");
        assert_eq!(EnrichmentErrorCode::NoUrl.to_string(),          "NO_URL");
    }

    // ── FetchStatus ───────────────────────────────────────────────────────────

    #[test]
    fn fetch_status_is_success() {
        assert!(FetchStatus::Success.is_success());
        assert!(!FetchStatus::Failed(EnrichmentErrorCode::Timeout).is_success());
    }
}
