// SourceRegistry — T-013
// TDD: uses mock adapters (test doubles) to verify concurrent execution and
//      failure isolation without HTTP.
//
// Design:
// - Registry holds Arc<dyn SourceAdapter> instances.
// - collect_all() spawns each adapter in a tokio::task::JoinSet.
// - Each adapter runs concurrently; panics or cancellations in one task do not
//   affect others (join handles are awaited independently).
// - Returns exactly one SourceResult per adapter, always (§4.3: failures are
//   recorded and the run continues).

use std::sync::Arc;
use tokio::task::JoinSet;
use tracing::warn;

use competitor_spy_domain::query::{Location, Radius, SearchQuery};
use competitor_spy_domain::run::{AdapterResultStatus, ReasonCode, SourceResult};
use chrono::Utc;

use crate::adapter::SourceAdapter;

// ── SourceRegistry ────────────────────────────────────────────────────────────

/// Manages an ordered list of source adapters and runs them concurrently.
pub struct SourceRegistry {
    adapters: Vec<Arc<dyn SourceAdapter>>,
}

impl SourceRegistry {
    pub fn new() -> Self {
        Self { adapters: vec![] }
    }

    /// Register a source adapter. Order determines source_result ordering.
    pub fn register(&mut self, adapter: Arc<dyn SourceAdapter>) {
        self.adapters.push(adapter);
    }

    /// Execute all adapters concurrently for the given query.
    ///
    /// `credentials`: maps adapter_id -> plaintext credential (already
    /// decrypted by the caller from CredentialStore).
    ///
    /// Returns one `SourceResult` per registered adapter, in registration order.
    /// Adapter panics produce a `Failed(PARSE_ERROR)` sentinel rather than
    /// propagating to the caller.
    pub async fn collect_all(
        &self,
        query: &SearchQuery,
        location: Location,
        radius: Radius,
        credentials: &std::collections::HashMap<String, String>,
    ) -> Vec<SourceResult> {
        let mut set: JoinSet<(usize, SourceResult)> = JoinSet::new();

        for (idx, adapter) in self.adapters.iter().enumerate() {
            let adapter = Arc::clone(adapter);
            let query = query.clone();
            let location = location.clone();
            let radius = radius;
            let credential = credentials.get(adapter.adapter_id()).cloned();

            set.spawn(async move {
                let result = adapter
                    .collect(&query, location, radius, credential.as_deref())
                    .await;
                (idx, result)
            });
        }

        // Collect results; preserve registration order
        let mut results: Vec<Option<SourceResult>> = (0..self.adapters.len()).map(|_| None).collect();

        while let Some(join_result) = set.join_next().await {
            match join_result {
                Ok((idx, source_result)) => {
                    results[idx] = Some(source_result);
                }
                Err(join_err) => {
                    // Task panicked — record a failed result; we don't know which
                    // adapter caused it, so just note it as a parse error sentinel.
                    warn!(event = "adapter_result", outcome = "task_panic", error = %join_err);
                    // We can't recover the idx from a panic JoinError, so push a sentinel
                    // at any remaining None slot (order lost on panic — acceptable).
                    for slot in results.iter_mut() {
                        if slot.is_none() {
                            *slot = Some(SourceResult {
                                adapter_id: "unknown".to_string(),
                                status: AdapterResultStatus::Failed(ReasonCode::ParseError),
                                records: vec![],
                                retrieved_at: Utc::now(),
                            });
                            break;
                        }
                    }
                }
            }
        }

        results.into_iter().flatten().collect()
    }

    pub fn adapter_count(&self) -> usize {
        self.adapters.len()
    }
}

impl Default for SourceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use competitor_spy_domain::query::{Location, Radius, SearchQuery};
    use competitor_spy_domain::run::{AdapterResultStatus, RawRecord, ReasonCode, SourceResult};
    use std::collections::HashMap;

    fn make_query() -> SearchQuery {
        SearchQuery::new("yoga studio", "Amsterdam", Radius::Km10).unwrap()
    }

    fn amsterdam() -> Location {
        Location { latitude: 52.3676, longitude: 4.9041 }
    }

    // ── Test-double adapters ──────────────────────────────────────────────────

    struct SuccessAdapter {
        id: String,
        record_count: usize,
    }

    #[async_trait]
    impl SourceAdapter for SuccessAdapter {
        fn adapter_id(&self) -> &str { &self.id }
        fn requires_credential(&self) -> bool { false }
        async fn collect(&self, _: &SearchQuery, _: Location, _: Radius, _: Option<&str>) -> SourceResult {
            let records: Vec<RawRecord> = (0..self.record_count).map(|i| {
                let mut fields = HashMap::new();
                fields.insert("name".to_string(), format!("Business {i}"));
                fields.insert("adapter_id".to_string(), self.id.clone());
                RawRecord { adapter_id: self.id.clone(), fields }
            }).collect();
            SourceResult {
                adapter_id: self.id.clone(),
                status: AdapterResultStatus::Success,
                records,
                retrieved_at: Utc::now(),
            }
        }
    }

    struct FailingAdapter {
        id: String,
        code: ReasonCode,
    }

    #[async_trait]
    impl SourceAdapter for FailingAdapter {
        fn adapter_id(&self) -> &str { &self.id }
        fn requires_credential(&self) -> bool { false }
        async fn collect(&self, _: &SearchQuery, _: Location, _: Radius, _: Option<&str>) -> SourceResult {
            SourceResult {
                adapter_id: self.id.clone(),
                status: AdapterResultStatus::Failed(self.code.clone()),
                records: vec![],
                retrieved_at: Utc::now(),
            }
        }
    }

    struct CredentialAdapter {
        id: String,
    }

    #[async_trait]
    impl SourceAdapter for CredentialAdapter {
        fn adapter_id(&self) -> &str { &self.id }
        fn requires_credential(&self) -> bool { true }
        async fn collect(&self, _: &SearchQuery, _: Location, _: Radius, credential: Option<&str>) -> SourceResult {
            let mut fields = HashMap::new();
            fields.insert("credential_present".to_string(), credential.unwrap_or("none").to_string());
            fields.insert("adapter_id".to_string(), self.id.clone());
            SourceResult {
                adapter_id: self.id.clone(),
                status: AdapterResultStatus::Success,
                records: vec![RawRecord { adapter_id: self.id.clone(), fields }],
                retrieved_at: Utc::now(),
            }
        }
    }

    // ── Tests ─────────────────────────────────────────────────────────────────

    #[test]
    fn registry_starts_empty() {
        let r = SourceRegistry::new();
        assert_eq!(r.adapter_count(), 0);
    }

    #[test]
    fn register_increases_count() {
        let mut r = SourceRegistry::new();
        r.register(Arc::new(SuccessAdapter { id: "a".to_string(), record_count: 2 }));
        r.register(Arc::new(SuccessAdapter { id: "b".to_string(), record_count: 0 }));
        assert_eq!(r.adapter_count(), 2);
    }

    #[tokio::test]
    async fn collect_all_returns_one_result_per_adapter() {
        let mut r = SourceRegistry::new();
        r.register(Arc::new(SuccessAdapter { id: "a1".to_string(), record_count: 3 }));
        r.register(Arc::new(SuccessAdapter { id: "a2".to_string(), record_count: 1 }));

        let results = r.collect_all(&make_query(), amsterdam(), Radius::Km10, &HashMap::new()).await;
        assert_eq!(results.len(), 2);
    }

    #[tokio::test]
    async fn collect_all_results_are_in_registration_order() {
        let mut r = SourceRegistry::new();
        r.register(Arc::new(SuccessAdapter { id: "first".to_string(), record_count: 2 }));
        r.register(Arc::new(SuccessAdapter { id: "second".to_string(), record_count: 0 }));
        r.register(Arc::new(SuccessAdapter { id: "third".to_string(), record_count: 1 }));

        let results = r.collect_all(&make_query(), amsterdam(), Radius::Km10, &HashMap::new()).await;
        assert_eq!(results[0].adapter_id, "first");
        assert_eq!(results[1].adapter_id, "second");
        assert_eq!(results[2].adapter_id, "third");
    }

    #[tokio::test]
    async fn collect_all_run_continues_when_one_adapter_fails() {
        let mut r = SourceRegistry::new();
        r.register(Arc::new(SuccessAdapter { id: "good".to_string(), record_count: 2 }));
        r.register(Arc::new(FailingAdapter { id: "bad".to_string(), code: ReasonCode::Timeout }));

        let results = r.collect_all(&make_query(), amsterdam(), Radius::Km10, &HashMap::new()).await;

        assert_eq!(results.len(), 2);
        let good = results.iter().find(|r| r.adapter_id == "good").unwrap();
        let bad = results.iter().find(|r| r.adapter_id == "bad").unwrap();
        assert!(matches!(good.status, AdapterResultStatus::Success));
        assert!(matches!(bad.status, AdapterResultStatus::Failed(ReasonCode::Timeout)));
        assert_eq!(good.records.len(), 2);
        assert!(bad.records.is_empty());
    }

    #[tokio::test]
    async fn collect_all_empty_registry_returns_empty() {
        let r = SourceRegistry::new();
        let results = r.collect_all(&make_query(), amsterdam(), Radius::Km10, &HashMap::new()).await;
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn collect_all_passes_credential_to_adapter() {
        let mut r = SourceRegistry::new();
        r.register(Arc::new(CredentialAdapter { id: "cred_adapter".to_string() }));

        let mut creds = HashMap::new();
        creds.insert("cred_adapter".to_string(), "secret-api-key".to_string());

        let results = r.collect_all(&make_query(), amsterdam(), Radius::Km10, &creds).await;
        assert_eq!(results.len(), 1);
        let rec = &results[0].records[0];
        assert_eq!(rec.fields["credential_present"], "secret-api-key");
    }

    #[tokio::test]
    async fn collect_all_passes_none_credential_when_absent_from_map() {
        let mut r = SourceRegistry::new();
        r.register(Arc::new(CredentialAdapter { id: "cred_adapter".to_string() }));

        let results = r.collect_all(&make_query(), amsterdam(), Radius::Km10, &HashMap::new()).await;
        let rec = &results[0].records[0];
        assert_eq!(rec.fields["credential_present"], "none");
    }

    #[tokio::test]
    async fn collect_all_all_adapters_fail_returns_all_failed_results() {
        let mut r = SourceRegistry::new();
        r.register(Arc::new(FailingAdapter { id: "f1".to_string(), code: ReasonCode::Http5xx }));
        r.register(Arc::new(FailingAdapter { id: "f2".to_string(), code: ReasonCode::ParseError }));

        let results = r.collect_all(&make_query(), amsterdam(), Radius::Km10, &HashMap::new()).await;

        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.status.is_failed()));
    }
}
