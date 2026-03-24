// SearchRun, RunStatus, SourceResult, RawRecord, ReasonCode — T-003
// TDD: tests written first; state-machine transitions tested explicitly.

use chrono::{DateTime, Utc};
use uuid::Uuid;
use crate::query::{Location, SearchQuery};
use crate::profile::Competitor;

// ── ReasonCode ────────────────────────────────────────────────────────────────

/// Standard failure reason codes for source adapter outcomes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReasonCode {
    Http4xx,
    Http5xx,
    Timeout,
    ParseError,
    AdapterConfigMissing,
}

impl std::fmt::Display for ReasonCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReasonCode::Http4xx              => write!(f, "HTTP_4XX"),
            ReasonCode::Http5xx              => write!(f, "HTTP_5XX"),
            ReasonCode::Timeout              => write!(f, "TIMEOUT"),
            ReasonCode::ParseError           => write!(f, "PARSE_ERROR"),
            ReasonCode::AdapterConfigMissing => write!(f, "ADAPTER_CONFIG_MISSING"),
        }
    }
}

// ── AdapterResultStatus ───────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum AdapterResultStatus {
    Success,
    PartialSuccess,
    Failed(ReasonCode),
}

impl AdapterResultStatus {
    pub fn is_failed(&self) -> bool {
        matches!(self, AdapterResultStatus::Failed(_))
    }
}

// ── RawRecord ─────────────────────────────────────────────────────────────────

/// Raw key-value map as returned by an adapter before normalization.
#[derive(Debug, Clone)]
pub struct RawRecord {
    pub adapter_id: String,
    pub fields: std::collections::HashMap<String, String>,
}

// ── SourceResult ──────────────────────────────────────────────────────────────

/// The complete outcome of one adapter invocation for one run.
#[derive(Debug, Clone)]
pub struct SourceResult {
    pub adapter_id: String,
    pub status: AdapterResultStatus,
    pub records: Vec<RawRecord>,
    pub retrieved_at: DateTime<Utc>,
}

// ── FailureReason ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum FailureReason {
    ValidationError(String),
    GeocodingError(String),
    RenderError(String),
}

impl std::fmt::Display for FailureReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FailureReason::ValidationError(m) => write!(f, "validation error: {m}"),
            FailureReason::GeocodingError(m)  => write!(f, "geocoding error: {m}"),
            FailureReason::RenderError(m)     => write!(f, "render error: {m}"),
        }
    }
}

// ── RunStatus ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum RunStatus {
    Idle,
    Validating,
    Geocoding,
    Collecting,
    Ranking,
    Rendering,
    Done,
    DoneWithWarning,
    Failed(FailureReason),
}

// ── SearchRun ─────────────────────────────────────────────────────────────────

/// Aggregate root for one complete competitor-spy execution.
pub struct SearchRun {
    pub id: Uuid,
    pub query: SearchQuery,
    pub resolved_location: Option<Location>,
    pub competitors: Vec<Competitor>,
    pub source_results: Vec<SourceResult>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub status: RunStatus,
}

impl SearchRun {
    /// Create a new run in Idle state.
    pub fn new(query: SearchQuery, started_at: DateTime<Utc>) -> Self {
        Self {
            id: Uuid::new_v4(),
            query,
            resolved_location: None,
            competitors: Vec::new(),
            source_results: Vec::new(),
            started_at,
            completed_at: None,
            status: RunStatus::Idle,
        }
    }

    /// Idle → Validating.
    pub fn start_validating(&mut self) {
        debug_assert_eq!(self.status, RunStatus::Idle);
        self.status = RunStatus::Validating;
    }

    /// Validating → Geocoding.
    pub fn start_geocoding(&mut self) {
        debug_assert_eq!(self.status, RunStatus::Validating);
        self.status = RunStatus::Geocoding;
    }

    /// Geocoding → Collecting; stores the resolved location.
    pub fn set_location(&mut self, location: Location) {
        debug_assert_eq!(self.status, RunStatus::Geocoding);
        self.resolved_location = Some(location);
        self.status = RunStatus::Collecting;
    }

    /// Record one adapter outcome during Collecting.
    pub fn add_source_result(&mut self, result: SourceResult) {
        debug_assert_eq!(self.status, RunStatus::Collecting);
        self.source_results.push(result);
    }

    /// Collecting → Ranking.
    pub fn start_ranking(&mut self) {
        debug_assert_eq!(self.status, RunStatus::Collecting);
        self.status = RunStatus::Ranking;
    }

    /// Ranking → Rendering; stores ranked competitors.
    pub fn set_competitors(&mut self, competitors: Vec<Competitor>) {
        debug_assert_eq!(self.status, RunStatus::Ranking);
        self.competitors = competitors;
        self.status = RunStatus::Rendering;
    }

    /// Rendering → Done.
    pub fn complete(&mut self, completed_at: DateTime<Utc>) {
        debug_assert_eq!(self.status, RunStatus::Rendering);
        self.completed_at = Some(completed_at);
        self.status = RunStatus::Done;
    }

    /// Rendering → DoneWithWarning (e.g., PDF failed but terminal succeeded).
    pub fn complete_with_warning(&mut self, completed_at: DateTime<Utc>) {
        debug_assert_eq!(self.status, RunStatus::Rendering);
        self.completed_at = Some(completed_at);
        self.status = RunStatus::DoneWithWarning;
    }

    /// Any state → Failed.
    pub fn fail(&mut self, reason: FailureReason, completed_at: DateTime<Utc>) {
        self.completed_at = Some(completed_at);
        self.status = RunStatus::Failed(reason);
    }

    /// True when the run terminated (Done, DoneWithWarning, or Failed).
    pub fn is_terminal(&self) -> bool {
        matches!(
            self.status,
            RunStatus::Done | RunStatus::DoneWithWarning | RunStatus::Failed(_)
        )
    }

    /// All SourceResults where the adapter failed.
    pub fn failed_source_results(&self) -> Vec<&SourceResult> {
        self.source_results
            .iter()
            .filter(|r| r.status.is_failed())
            .collect()
    }
}

// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::{Radius, SearchQuery};

    fn make_query() -> SearchQuery {
        SearchQuery::new("yoga studio", "Amsterdam, Netherlands", Radius::Km10).unwrap()
    }

    fn now() -> DateTime<Utc> {
        Utc::now()
    }

    fn make_run() -> SearchRun {
        SearchRun::new(make_query(), now())
    }

    // ── ReasonCode ────────────────────────────────────────────────────────────

    #[test]
    fn reason_code_display_matches_spec() {
        assert_eq!(ReasonCode::Http4xx.to_string(),              "HTTP_4XX");
        assert_eq!(ReasonCode::Http5xx.to_string(),              "HTTP_5XX");
        assert_eq!(ReasonCode::Timeout.to_string(),              "TIMEOUT");
        assert_eq!(ReasonCode::ParseError.to_string(),           "PARSE_ERROR");
        assert_eq!(ReasonCode::AdapterConfigMissing.to_string(), "ADAPTER_CONFIG_MISSING");
    }

    // ── AdapterResultStatus ───────────────────────────────────────────────────

    #[test]
    fn adapter_result_status_is_failed_correct() {
        assert!(!AdapterResultStatus::Success.is_failed());
        assert!(!AdapterResultStatus::PartialSuccess.is_failed());
        assert!(AdapterResultStatus::Failed(ReasonCode::Timeout).is_failed());
    }

    // ── RunStatus / SearchRun state machine ───────────────────────────────────

    #[test]
    fn searchrun_initial_status_is_idle() {
        let run = make_run();
        assert_eq!(run.status, RunStatus::Idle);
        assert!(run.resolved_location.is_none());
        assert!(run.competitors.is_empty());
        assert!(run.source_results.is_empty());
        assert!(run.completed_at.is_none());
    }

    #[test]
    fn searchrun_happy_path_transitions() {
        let t = now();
        let mut run = SearchRun::new(make_query(), t);

        run.start_validating();
        assert_eq!(run.status, RunStatus::Validating);

        run.start_geocoding();
        assert_eq!(run.status, RunStatus::Geocoding);

        let loc = Location::new(52.3676, 4.9041).unwrap();
        run.set_location(loc.clone());
        assert_eq!(run.status, RunStatus::Collecting);
        assert_eq!(run.resolved_location.as_ref().unwrap().latitude, loc.latitude);

        let src = SourceResult {
            adapter_id: "osm".to_string(),
            status: AdapterResultStatus::Success,
            records: vec![],
            retrieved_at: t,
        };
        run.add_source_result(src);
        assert_eq!(run.source_results.len(), 1);

        run.start_ranking();
        assert_eq!(run.status, RunStatus::Ranking);

        run.set_competitors(vec![]);
        assert_eq!(run.status, RunStatus::Rendering);

        run.complete(t);
        assert_eq!(run.status, RunStatus::Done);
        assert!(run.completed_at.is_some());
        assert!(run.is_terminal());
    }

    #[test]
    fn searchrun_fail_from_validating() {
        let mut run = make_run();
        run.start_validating();
        run.fail(FailureReason::ValidationError("bad radius".to_string()), now());
        assert!(matches!(run.status, RunStatus::Failed(FailureReason::ValidationError(_))));
        assert!(run.is_terminal());
    }

    #[test]
    fn searchrun_fail_from_geocoding() {
        let mut run = make_run();
        run.start_validating();
        run.start_geocoding();
        run.fail(FailureReason::GeocodingError("no result".to_string()), now());
        assert!(matches!(run.status, RunStatus::Failed(FailureReason::GeocodingError(_))));
    }

    #[test]
    fn searchrun_adapter_failure_does_not_abort_run() {
        let t = now();
        let mut run = SearchRun::new(make_query(), t);
        run.start_validating();
        run.start_geocoding();
        run.set_location(Location::new(52.3676, 4.9041).unwrap());

        // One success, one failure
        run.add_source_result(SourceResult {
            adapter_id: "osm".to_string(),
            status: AdapterResultStatus::Success,
            records: vec![],
            retrieved_at: t,
        });
        run.add_source_result(SourceResult {
            adapter_id: "yelp".to_string(),
            status: AdapterResultStatus::Failed(ReasonCode::Timeout),
            records: vec![],
            retrieved_at: t,
        });

        // Run still proceeds to Ranking
        run.start_ranking();
        assert_eq!(run.status, RunStatus::Ranking);
        assert_eq!(run.source_results.len(), 2);
        assert_eq!(run.failed_source_results().len(), 1);
        assert_eq!(run.failed_source_results()[0].adapter_id, "yelp");
    }

    #[test]
    fn searchrun_done_with_warning_is_terminal() {
        let t = now();
        let mut run = SearchRun::new(make_query(), t);
        run.start_validating();
        run.start_geocoding();
        run.set_location(Location::new(52.3676, 4.9041).unwrap());
        run.start_ranking();
        run.set_competitors(vec![]);
        run.complete_with_warning(t);
        assert_eq!(run.status, RunStatus::DoneWithWarning);
        assert!(run.is_terminal());
    }

    #[test]
    fn searchrun_is_not_terminal_mid_run() {
        let mut run = make_run();
        assert!(!run.is_terminal());
        run.start_validating();
        assert!(!run.is_terminal());
        run.start_geocoding();
        assert!(!run.is_terminal());
    }

    #[test]
    fn failure_reason_display() {
        let v = FailureReason::ValidationError("x".to_string());
        assert!(v.to_string().contains("validation error"));
        let g = FailureReason::GeocodingError("y".to_string());
        assert!(g.to_string().contains("geocoding error"));
    }
}

