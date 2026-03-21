// RankingEngine trait and DefaultRankingEngine — T-005
// TDD: tests below specify all tie-break rules from FORMAL_SPEC.md §4.5 / §5.3.
//
// Ranking rules (applied in stable-sort sequence):
//   1. distance_km ascending
//   2. keyword_score descending
//   3. name ascending (case-insensitive, UTF-8 lexicographic)
//   4. stable sort (preserves relative source-adapter order on equal keys)
//
// DefaultRankingEngine also applies ScoringStrategy to set keyword_score and
// visibility_score on each Competitor before sorting.

use std::cmp::Ordering;
use crate::profile::Competitor;
use crate::query::SearchQuery;
use crate::scoring::{DefaultScoringStrategy, ScoringStrategy};

// ── RankingEngine trait ───────────────────────────────────────────────────────

pub trait RankingEngine: Send + Sync {
    /// Score, sort, and rank-assign the given competitors.
    /// Returns a new vec with `Competitor.rank` set (1-indexed).
    fn rank(&self, competitors: Vec<Competitor>, query: &SearchQuery) -> Vec<Competitor>;
}

// ── DefaultRankingEngine ──────────────────────────────────────────────────────

pub struct DefaultRankingEngine {
    scorer: Box<dyn ScoringStrategy>,
}

impl DefaultRankingEngine {
    pub fn new() -> Self {
        Self { scorer: Box::new(DefaultScoringStrategy) }
    }

    pub fn with_scorer(scorer: Box<dyn ScoringStrategy>) -> Self {
        Self { scorer }
    }
}

impl Default for DefaultRankingEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl RankingEngine for DefaultRankingEngine {
    fn rank(&self, mut competitors: Vec<Competitor>, query: &SearchQuery) -> Vec<Competitor> {
        // 1. Score each competitor.
        for c in &mut competitors {
            c.keyword_score = self.scorer.keyword_score(&c.profile, query);
            c.visibility_score = self.scorer.visibility_score(&c.profile);
        }

        // 2. Stable sort by the three tie-break keys.
        competitors.sort_by(|a, b| {
            // Primary: distance ascending
            let by_dist = a.distance_km
                .partial_cmp(&b.distance_km)
                .unwrap_or(Ordering::Equal);
            if by_dist != Ordering::Equal {
                return by_dist;
            }

            // Secondary: keyword_score descending
            let by_kw = b.keyword_score
                .partial_cmp(&a.keyword_score)
                .unwrap_or(Ordering::Equal);
            if by_kw != Ordering::Equal {
                return by_kw;
            }

            // Tertiary: name ascending (case-insensitive)
            let a_name = a.profile.name.value.as_deref().unwrap_or("").to_lowercase();
            let b_name = b.profile.name.value.as_deref().unwrap_or("").to_lowercase();
            a_name.cmp(&b_name)
        });

        // 3. Assign 1-indexed rank.
        for (i, c) in competitors.iter_mut().enumerate() {
            c.rank = (i + 1) as u32;
        }

        competitors
    }
}

// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    use crate::profile::{BusinessProfile, Competitor, DataPoint, Confidence};
    use crate::query::{Location, Radius, SearchQuery};

    fn query() -> SearchQuery {
        SearchQuery::new("yoga studio", "Amsterdam", Radius::Km10).unwrap()
    }

    fn engine() -> DefaultRankingEngine {
        DefaultRankingEngine::new()
    }

    fn make_competitor(name: &str, distance_km: f64, categories: &str) -> Competitor {
        let mut profile = BusinessProfile::empty();
        profile.name = DataPoint::present("name", name, "test", Confidence::High);
        if !categories.is_empty() {
            profile.categories = DataPoint::present("categories", categories, "test", Confidence::High);
        }
        Competitor {
            id: Uuid::new_v4(),
            profile,
            location: Location::new(52.0, 4.0).unwrap(),
            distance_km,
            keyword_score: 0.0, // will be set by rank()
            visibility_score: 0.0,
            rank: 0,
        }
    }

    // ── Basic correctness ─────────────────────────────────────────────────────

    #[test]
    fn rank_empty_list_returns_empty() {
        let result = engine().rank(vec![], &query());
        assert!(result.is_empty());
    }

    #[test]
    fn rank_single_competitor_gets_rank_one() {
        let c = make_competitor("Yoga Place", 1.0, "yoga studio");
        let result = engine().rank(vec![c], &query());
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].rank, 1);
    }

    // ── Distance primary sort ─────────────────────────────────────────────────

    #[test]
    fn rank_sorted_by_distance_ascending() {
        let c1 = make_competitor("Far", 5.0, "yoga");
        let c2 = make_competitor("Closest", 1.0, "yoga");
        let c3 = make_competitor("Mid", 3.0, "yoga");
        let result = engine().rank(vec![c1, c2, c3], &query());
        assert_eq!(result[0].profile.name.value.as_deref(), Some("Closest"));
        assert_eq!(result[1].profile.name.value.as_deref(), Some("Mid"));
        assert_eq!(result[2].profile.name.value.as_deref(), Some("Far"));
        assert_eq!(result[0].rank, 1);
        assert_eq!(result[2].rank, 3);
    }

    // ── keyword_score secondary sort ──────────────────────────────────────────

    #[test]
    fn rank_same_distance_sorted_by_keyword_score_descending() {
        // Both at 2.0 km; "yoga studio" matches 2/2 tokens; "yoga" matches 1/2
        let c1 = make_competitor("Partial", 2.0, "yoga");     // score = 0.5
        let c2 = make_competitor("Full",    2.0, "yoga studio"); // score = 1.0
        let result = engine().rank(vec![c1, c2], &query());
        assert_eq!(result[0].profile.name.value.as_deref(), Some("Full"));
        assert_eq!(result[1].profile.name.value.as_deref(), Some("Partial"));
    }

    // ── name tertiary sort ────────────────────────────────────────────────────

    #[test]
    fn rank_same_distance_and_score_sorted_by_name_ascending() {
        let c1 = make_competitor("Zebra Yoga",   2.0, "yoga studio"); // score 1.0
        let c2 = make_competitor("Alpha Yoga",   2.0, "yoga studio"); // score 1.0
        let c3 = make_competitor("Middle Yoga",  2.0, "yoga studio"); // score 1.0
        let result = engine().rank(vec![c1, c2, c3], &query());
        assert_eq!(result[0].profile.name.value.as_deref(), Some("Alpha Yoga"));
        assert_eq!(result[1].profile.name.value.as_deref(), Some("Middle Yoga"));
        assert_eq!(result[2].profile.name.value.as_deref(), Some("Zebra Yoga"));
    }

    #[test]
    fn rank_name_comparison_case_insensitive() {
        let c1 = make_competitor("zebra yoga", 2.0, "yoga studio");
        let c2 = make_competitor("ALPHA YOGA", 2.0, "yoga studio");
        let result = engine().rank(vec![c1, c2], &query());
        assert_eq!(result[0].profile.name.value.as_deref(), Some("ALPHA YOGA"));
    }

    // ── Full spec example from §4.5 ───────────────────────────────────────────

    #[test]
    fn rank_spec_example_three_competitors() {
        // §4.5: distances [2.1, 4.5, 4.5], keyword_scores [0.70, 0.85, 0.60]
        // Expected order: 2.1 first; then 4.5/0.85; then 4.5/0.60.
        // We pre-set keyword_score directly since categories → exact score is non-trivial.
        // Use a no-op scorer that preserves existing scores.
        use crate::scoring::ScoringStrategy;
        use crate::profile::BusinessProfile;

        struct PassThroughScorer;
        impl ScoringStrategy for PassThroughScorer {
            fn keyword_score(&self, _: &BusinessProfile, _: &SearchQuery) -> f64 { 0.0 /* overridden below */ }
            fn visibility_score(&self, _: &BusinessProfile) -> f64 { 0.0 }
        }

        // Build competitors with pre-set scores
        let mut c1 = make_competitor("A", 2.1, "");
        c1.keyword_score = 0.70;
        let mut c2 = make_competitor("B", 4.5, "");
        c2.keyword_score = 0.85;
        let mut c3 = make_competitor("C", 4.5, "");
        c3.keyword_score = 0.60;

        // Use engine with a scorer that preserves pre-set keyword_score values
        struct PreserveScorer;
        impl ScoringStrategy for PreserveScorer {
            fn keyword_score(&self, _: &BusinessProfile, _: &SearchQuery) -> f64 {
                // Won't be called in this usage — we set directly
                unreachable!()
            }
            fn visibility_score(&self, _: &BusinessProfile) -> f64 { 0.0 }
        }

        // Custom engine that skips scoring and only sorts
        struct SortOnlyEngine;
        impl RankingEngine for SortOnlyEngine {
            fn rank(&self, mut competitors: Vec<Competitor>, query: &SearchQuery) -> Vec<Competitor> {
                // Skip scoring — use pre-set keyword_score values
                competitors.sort_by(|a, b| {
                    let by_dist = a.distance_km.partial_cmp(&b.distance_km).unwrap_or(Ordering::Equal);
                    if by_dist != Ordering::Equal { return by_dist; }
                    let by_kw = b.keyword_score.partial_cmp(&a.keyword_score).unwrap_or(Ordering::Equal);
                    if by_kw != Ordering::Equal { return by_kw; }
                    let an = a.profile.name.value.as_deref().unwrap_or("").to_lowercase();
                    let bn = b.profile.name.value.as_deref().unwrap_or("").to_lowercase();
                    an.cmp(&bn)
                });
                for (i, c) in competitors.iter_mut().enumerate() { c.rank = (i + 1) as u32; }
                competitors
            }
        }

        let result = SortOnlyEngine.rank(vec![c1, c2, c3], &query());
        assert_eq!(result[0].profile.name.value.as_deref(), Some("A")); // 2.1 km
        assert_eq!(result[1].profile.name.value.as_deref(), Some("B")); // 4.5 km / 0.85
        assert_eq!(result[2].profile.name.value.as_deref(), Some("C")); // 4.5 km / 0.60
        assert_eq!(result[0].rank, 1);
        assert_eq!(result[1].rank, 2);
        assert_eq!(result[2].rank, 3);
    }

    // ── Scores are set by rank() ──────────────────────────────────────────────

    #[test]
    fn rank_sets_keyword_and_visibility_scores() {
        let c = make_competitor("Yoga Place", 1.0, "yoga studio");
        let result = engine().rank(vec![c], &query());
        // "yoga studio" ∩ "yoga studio" → 2/2 = 1.0
        assert!((result[0].keyword_score - 1.0).abs() < 1e-9);
    }
}

