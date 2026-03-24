// ScoringStrategy trait and DefaultScoringStrategy — T-004
// TDD: tests below drive the algorithm contracts.
//
// Algorithm decisions (also in chronicle CHR-CSPY-004):
//   keyword_score  = |query_tokens ∩ category_tokens| / |query_tokens|
//                    tokens = whitespace-split, lowercased; 0.0 if no query tokens
//   visibility_score = 0.5 * completeness + 0.5 * review_score
//                    review_score = min(parsed_count, 200) / 200  (saturates at 200 reviews)
//                    if review_count_text absent: visibility_score = completeness

use crate::profile::{BusinessProfile, Confidence};
use crate::query::SearchQuery;

// ── ScoringStrategy trait ─────────────────────────────────────────────────────

pub trait ScoringStrategy: Send + Sync {
    fn keyword_score(&self, profile: &BusinessProfile, query: &SearchQuery) -> f64;
    fn visibility_score(&self, profile: &BusinessProfile) -> f64;
}

// ── DefaultScoringStrategy ────────────────────────────────────────────────────

pub struct DefaultScoringStrategy;

impl ScoringStrategy for DefaultScoringStrategy {
    fn keyword_score(&self, profile: &BusinessProfile, query: &SearchQuery) -> f64 {
        let query_tokens: Vec<String> = query
            .industry
            .split_whitespace()
            .map(|t| t.to_lowercase())
            .collect();

        if query_tokens.is_empty() {
            return 0.0;
        }

        let categories_text = profile
            .categories
            .value
            .as_deref()
            .unwrap_or("")
            .to_lowercase();

        let matched = query_tokens
            .iter()
            .filter(|t| categories_text.contains(t.as_str()))
            .count();

        matched as f64 / query_tokens.len() as f64
    }

    fn visibility_score(&self, profile: &BusinessProfile) -> f64 {
        let completeness = profile.completeness();

        let review_score = if profile.review_count_text.confidence == Confidence::Absent {
            // No review data: use completeness alone
            return completeness;
        } else {
            let count: f64 = profile
                .review_count_text
                .value
                .as_deref()
                .unwrap_or("0")
                .parse::<f64>()
                .unwrap_or(0.0)
                .max(0.0);
            (count / 200.0).min(1.0)
        };

        (0.5 * completeness + 0.5 * review_score).min(1.0)
    }
}

// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profile::{BusinessProfile, DataPoint};
    use crate::query::{Radius, SearchQuery};

    fn query(industry: &str) -> SearchQuery {
        SearchQuery::new(industry, "Amsterdam", Radius::Km10).unwrap()
    }

    fn scorer() -> DefaultScoringStrategy {
        DefaultScoringStrategy
    }

    // ── keyword_score ─────────────────────────────────────────────────────────

    #[test]
    fn keyword_score_empty_categories_is_zero() {
        let profile = BusinessProfile::empty(); // categories = Absent
        assert_eq!(scorer().keyword_score(&profile, &query("yoga studio")), 0.0);
    }

    #[test]
    fn keyword_score_all_tokens_match_is_one() {
        let mut profile = BusinessProfile::empty();
        profile.categories = DataPoint::present(
            "categories",
            "yoga studio pilates",
            "osm",
            crate::profile::Confidence::High,
        );
        let score = scorer().keyword_score(&profile, &query("yoga studio"));
        assert!((score - 1.0).abs() < 1e-9, "expected 1.0, got {score}");
    }

    #[test]
    fn keyword_score_partial_match() {
        let mut profile = BusinessProfile::empty();
        profile.categories = DataPoint::present(
            "categories",
            "yoga pilates",
            "osm",
            crate::profile::Confidence::High,
        );
        // query has 2 tokens: "yoga" (matches), "studio" (no match) → 0.5
        let score = scorer().keyword_score(&profile, &query("yoga studio"));
        assert!((score - 0.5).abs() < 1e-9, "expected 0.5, got {score}");
    }

    #[test]
    fn keyword_score_no_match_is_zero() {
        let mut profile = BusinessProfile::empty();
        profile.categories = DataPoint::present(
            "categories",
            "restaurant bar",
            "osm",
            crate::profile::Confidence::High,
        );
        let score = scorer().keyword_score(&profile, &query("yoga studio"));
        assert_eq!(score, 0.0);
    }

    #[test]
    fn keyword_score_case_insensitive() {
        let mut profile = BusinessProfile::empty();
        profile.categories = DataPoint::present(
            "categories",
            "YOGA STUDIO",
            "osm",
            crate::profile::Confidence::High,
        );
        let score = scorer().keyword_score(&profile, &query("yoga studio"));
        assert!((score - 1.0).abs() < 1e-9, "expected 1.0, got {score}");
    }

    #[test]
    fn keyword_score_empty_industry_is_zero() {
        // SearchQuery rejects empty industry, so we test the trait directly
        // by constructing any profile and overriding the query industry check indirectly.
        // Edge: if trait is called with industry "  " normalised to zero tokens.
        // We test by constructing a SearchQuery via the raw struct fields.
        let profile = BusinessProfile::empty();
        // Simulate: a scoring strategy receiving a query where industry produces no tokens
        // (empty string is invalid at validation, but we test the scoring code's guard)
        let score = DefaultScoringStrategy.keyword_score(
            &profile,
            &SearchQuery {
                industry: String::new(),
                location_input: "Amsterdam".to_string(),
                radius: Radius::Km10,
            },
        );
        assert_eq!(score, 0.0);
    }

    // ── visibility_score ──────────────────────────────────────────────────────

    #[test]
    fn visibility_score_all_absent_is_zero() {
        let profile = BusinessProfile::empty();
        assert_eq!(scorer().visibility_score(&profile), 0.0);
    }

    #[test]
    fn visibility_score_full_completeness_no_reviews_equals_completeness() {
        let mut profile = BusinessProfile::empty();
        let fill = |n: &str| DataPoint::present(n, "v", "s", Confidence::Low);
        profile.name = fill("name");
        profile.address = fill("address");
        profile.phone = fill("phone");
        profile.website = fill("website");
        profile.categories = fill("categories");
        profile.opening_hours = fill("opening_hours");
        profile.email = fill("email");
        profile.description = fill("description");
        profile.rating_text = fill("rating_text");
        profile.editorial_summary = fill("editorial_summary");
        profile.price_level = fill("price_level");
        // review_count_text is Absent
        let score = scorer().visibility_score(&profile);
        // completeness = 11/12; review absent -> returns completeness
        let expected = 11.0_f64 / 12.0_f64;
        assert!((score - expected).abs() < 1e-9, "expected {expected}, got {score}");
    }

    #[test]
    fn visibility_score_with_200_reviews_and_full_completeness_is_one() {
        let mut profile = BusinessProfile::empty();
        let fill = |n: &str| DataPoint::present(n, "v", "s", Confidence::Low);
        profile.name = fill("name");
        profile.address = fill("address");
        profile.phone = fill("phone");
        profile.website = fill("website");
        profile.categories = fill("categories");
        profile.opening_hours = fill("opening_hours");
        profile.email = fill("email");
        profile.description = fill("description");
        profile.rating_text = fill("rating_text");
        profile.editorial_summary = fill("editorial_summary");
        profile.price_level = fill("price_level");
        profile.review_count_text =
            DataPoint::present("review_count_text", "200", "yelp", Confidence::High);
        let score = scorer().visibility_score(&profile);
        // completeness=1.0, review_score=1.0 -> 0.5*1.0 + 0.5*1.0 = 1.0
        assert!((score - 1.0).abs() < 1e-9, "expected 1.0, got {score}");
    }

    #[test]
    fn visibility_score_with_100_reviews_and_no_other_fields() {
        let mut profile = BusinessProfile::empty();
        profile.review_count_text =
            DataPoint::present("review_count_text", "100", "yelp", Confidence::High);
        let score = scorer().visibility_score(&profile);
        // completeness = 1/12; review_score = 100/200 = 0.5
        // -> 0.5*(1/12) + 0.5*0.5
        let expected = 0.5 * (1.0_f64 / 12.0_f64) + 0.5 * 0.5;
        assert!((score - expected).abs() < 1e-9, "expected {expected}, got {score}");
    }

    #[test]
    fn visibility_score_always_in_0_to_1() {
        // With absurd review count (e.g. 99999), score should cap at 1.0
        let mut profile = BusinessProfile::empty();
        profile.review_count_text =
            DataPoint::present("review_count_text", "99999", "yelp", Confidence::High);
        let score = scorer().visibility_score(&profile);
        assert!(score <= 1.0, "expected <=1.0, got {score}");
        assert!(score >= 0.0, "expected >=0.0, got {score}");
    }
}

