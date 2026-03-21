// BusinessProfile, DataPoint, Confidence, Competitor, deduplication — T-002
// TDD: test module written before implementation.

use uuid::Uuid;
use crate::query::Location;

// ── Confidence ────────────────────────────────────────────────────────────────

/// Confidence level for a DataPoint value.
/// Declaration order defines comparison: Absent < Low < Medium < High.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Confidence {
    Absent,
    Low,
    Medium,
    High,
}

// ── DataPoint ─────────────────────────────────────────────────────────────────

/// One field-value from one source, tagged with source ID and confidence.
#[derive(Debug, Clone, PartialEq)]
pub struct DataPoint {
    pub field_name: String,
    pub value: Option<String>,
    pub source_id: String,
    pub confidence: Confidence,
}

impl DataPoint {
    /// Construct a DataPoint with a present value.
    pub fn present(
        field_name: impl Into<String>,
        value: impl Into<String>,
        source_id: impl Into<String>,
        confidence: Confidence,
    ) -> Self {
        Self {
            field_name: field_name.into(),
            value: Some(value.into()),
            source_id: source_id.into(),
            confidence,
        }
    }

    /// Construct an Absent DataPoint (no value, no source).
    pub fn absent(field_name: impl Into<String>) -> Self {
        Self {
            field_name: field_name.into(),
            value: None,
            source_id: String::new(),
            confidence: Confidence::Absent,
        }
    }

    /// True when confidence is Absent.
    pub fn is_absent(&self) -> bool {
        self.confidence == Confidence::Absent
    }
}

// ── BusinessProfile ───────────────────────────────────────────────────────────

/// All collected data fields for one Competitor. Every field is always
/// present as a DataPoint; missing data is Confidence::Absent.
#[derive(Debug, Clone)]
pub struct BusinessProfile {
    pub name: DataPoint,
    pub address: DataPoint,
    pub phone: DataPoint,
    pub website: DataPoint,
    pub categories: DataPoint,
    pub opening_hours: DataPoint,
    pub email: DataPoint,
    pub description: DataPoint,
    pub rating_text: DataPoint,
    pub review_count_text: DataPoint,
}

impl BusinessProfile {
    /// Total number of defined fields.
    pub const FIELD_COUNT: usize = 10;

    /// Construct a profile where every field is Absent.
    pub fn empty() -> Self {
        Self {
            name:             DataPoint::absent("name"),
            address:          DataPoint::absent("address"),
            phone:            DataPoint::absent("phone"),
            website:          DataPoint::absent("website"),
            categories:       DataPoint::absent("categories"),
            opening_hours:    DataPoint::absent("opening_hours"),
            email:            DataPoint::absent("email"),
            description:      DataPoint::absent("description"),
            rating_text:      DataPoint::absent("rating_text"),
            review_count_text: DataPoint::absent("review_count_text"),
        }
    }

    /// Fraction of fields that are not Absent (0.0–1.0).
    pub fn completeness(&self) -> f64 {
        let present = self.fields().iter().filter(|dp| !dp.is_absent()).count();
        present as f64 / Self::FIELD_COUNT as f64
    }

    /// Merge `other` into `self`. For each field, the DataPoint with
    /// the higher confidence wins. Equal confidence keeps `self`.
    pub fn merge_with(&mut self, other: BusinessProfile) {
        merge_field(&mut self.name,             other.name);
        merge_field(&mut self.address,          other.address);
        merge_field(&mut self.phone,            other.phone);
        merge_field(&mut self.website,          other.website);
        merge_field(&mut self.categories,       other.categories);
        merge_field(&mut self.opening_hours,    other.opening_hours);
        merge_field(&mut self.email,            other.email);
        merge_field(&mut self.description,      other.description);
        merge_field(&mut self.rating_text,      other.rating_text);
        merge_field(&mut self.review_count_text, other.review_count_text);
    }

    /// Iterate all 10 DataPoints by shared reference.
    pub fn fields(&self) -> [&DataPoint; 10] {
        [
            &self.name,
            &self.address,
            &self.phone,
            &self.website,
            &self.categories,
            &self.opening_hours,
            &self.email,
            &self.description,
            &self.rating_text,
            &self.review_count_text,
        ]
    }
}

// ── Competitor ────────────────────────────────────────────────────────────────

/// A business entity discovered within the search radius.
#[derive(Debug, Clone)]
pub struct Competitor {
    pub id: Uuid,
    pub profile: BusinessProfile,
    pub location: Location,
    pub distance_km: f64,
    pub keyword_score: f64,
    pub visibility_score: f64,
    pub rank: u32,
}

// ── Deduplication ─────────────────────────────────────────────────────────────

/// Haversine distance between two coordinates in metres.
fn haversine_metres(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    const R: f64 = 6_371_000.0;
    let dlat = (lat2 - lat1).to_radians();
    let dlon = (lon2 - lon1).to_radians();
    let lat1r = lat1.to_radians();
    let lat2r = lat2.to_radians();
    let a = (dlat / 2.0).sin().powi(2)
        + lat1r.cos() * lat2r.cos() * (dlon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
    R * c
}

/// Deduplicate a list of competitors.
///
/// Two competitors are the same entity when:
/// - their names match (case-insensitive, whitespace-trimmed), AND
/// - their coordinates are within 50 metres.
///
/// On match, the first-encountered entry is kept and the second is merged
/// into it (highest-confidence DataPoint per field wins).
pub fn deduplicate(competitors: Vec<Competitor>) -> Vec<Competitor> {
    let mut result: Vec<Competitor> = Vec::with_capacity(competitors.len());
    'outer: for incoming in competitors {
        let incoming_name = incoming.profile.name.value
            .as_deref()
            .unwrap_or("")
            .trim()
            .to_lowercase();
        for existing in &mut result {
            let existing_name = existing.profile.name.value
                .as_deref()
                .unwrap_or("")
                .trim()
                .to_lowercase();
            let dist = haversine_metres(
                existing.location.latitude,
                existing.location.longitude,
                incoming.location.latitude,
                incoming.location.longitude,
            );
            if existing_name == incoming_name && dist <= 50.0 {
                existing.profile.merge_with(incoming.profile);
                continue 'outer;
            }
        }
        result.push(incoming);
    }
    result
}

/// Merge one DataPoint field: higher confidence wins; equal confidence keeps `base`.
fn merge_field(base: &mut DataPoint, incoming: DataPoint) {
    if incoming.confidence > base.confidence {
        *base = incoming;
    }
}

// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::Location;

    fn make_location(lat: f64, lon: f64) -> Location {
        Location::new(lat, lon).unwrap()
    }

    fn make_competitor(name: &str, lat: f64, lon: f64) -> Competitor {
        let mut profile = BusinessProfile::empty();
        profile.name = DataPoint::present("name", name, "test", Confidence::High);
        Competitor {
            id: Uuid::new_v4(),
            profile,
            location: make_location(lat, lon),
            distance_km: 1.0,
            keyword_score: 0.5,
            visibility_score: 0.5,
            rank: 0,
        }
    }

    // ── Confidence ────────────────────────────────────────────────────────────

    #[test]
    fn confidence_ordering_correct() {
        assert!(Confidence::Absent < Confidence::Low);
        assert!(Confidence::Low < Confidence::Medium);
        assert!(Confidence::Medium < Confidence::High);
        assert!(Confidence::High > Confidence::Absent);
    }

    // ── DataPoint ─────────────────────────────────────────────────────────────

    #[test]
    fn data_point_absent_is_absent() {
        let dp = DataPoint::absent("name");
        assert!(dp.is_absent());
        assert!(dp.value.is_none());
        assert_eq!(dp.confidence, Confidence::Absent);
        assert_eq!(dp.field_name, "name");
    }

    #[test]
    fn data_point_present_not_absent() {
        let dp = DataPoint::present("name", "Iron Temple", "osm", Confidence::High);
        assert!(!dp.is_absent());
        assert_eq!(dp.value.as_deref(), Some("Iron Temple"));
        assert_eq!(dp.confidence, Confidence::High);
        assert_eq!(dp.source_id, "osm");
    }

    // ── BusinessProfile ───────────────────────────────────────────────────────

    #[test]
    fn profile_empty_all_absent() {
        let p = BusinessProfile::empty();
        assert!(p.name.is_absent());
        assert!(p.address.is_absent());
        assert!(p.phone.is_absent());
        assert!(p.website.is_absent());
        assert!(p.categories.is_absent());
        assert!(p.opening_hours.is_absent());
        assert!(p.email.is_absent());
        assert!(p.description.is_absent());
        assert!(p.rating_text.is_absent());
        assert!(p.review_count_text.is_absent());
    }

    #[test]
    fn profile_completeness_zero_when_all_absent() {
        let p = BusinessProfile::empty();
        assert_eq!(p.completeness(), 0.0);
    }

    #[test]
    fn profile_completeness_four_of_ten_is_40_percent() {
        let mut p = BusinessProfile::empty();
        p.name = DataPoint::present("name", "Gym", "osm", Confidence::High);
        p.address = DataPoint::present("address", "123 St", "osm", Confidence::Medium);
        p.phone = DataPoint::present("phone", "+31...", "yelp", Confidence::Low);
        p.website = DataPoint::present("website", "http://...", "google", Confidence::Medium);
        let ratio = p.completeness();
        assert!((ratio - 0.4).abs() < 1e-9, "expected 0.4, got {ratio}");
    }

    #[test]
    fn profile_completeness_full_is_one() {
        let mut p = BusinessProfile::empty();
        let fill = |name: &str| DataPoint::present(name, "v", "s", Confidence::Low);
        p.name = fill("name");
        p.address = fill("address");
        p.phone = fill("phone");
        p.website = fill("website");
        p.categories = fill("categories");
        p.opening_hours = fill("opening_hours");
        p.email = fill("email");
        p.description = fill("description");
        p.rating_text = fill("rating_text");
        p.review_count_text = fill("review_count_text");
        assert_eq!(p.completeness(), 1.0);
    }

    #[test]
    fn profile_merge_high_beats_low() {
        let mut base = BusinessProfile::empty();
        base.name = DataPoint::present("name", "Low Name", "src1", Confidence::Low);

        let mut other = BusinessProfile::empty();
        other.name = DataPoint::present("name", "High Name", "src2", Confidence::High);

        base.merge_with(other);
        assert_eq!(base.name.value.as_deref(), Some("High Name"));
        assert_eq!(base.name.confidence, Confidence::High);
    }

    #[test]
    fn profile_merge_equal_confidence_keeps_base() {
        let mut base = BusinessProfile::empty();
        base.name = DataPoint::present("name", "Base Name", "src1", Confidence::Medium);

        let mut other = BusinessProfile::empty();
        other.name = DataPoint::present("name", "Other Name", "src2", Confidence::Medium);

        base.merge_with(other);
        assert_eq!(base.name.value.as_deref(), Some("Base Name"));
    }

    #[test]
    fn profile_merge_absent_field_gains_present_value() {
        let mut base = BusinessProfile::empty(); // name is Absent

        let mut other = BusinessProfile::empty();
        other.name = DataPoint::present("name", "Gained Name", "src", Confidence::Low);

        base.merge_with(other);
        assert!(!base.name.is_absent());
        assert_eq!(base.name.value.as_deref(), Some("Gained Name"));
    }

    // ── Haversine ─────────────────────────────────────────────────────────────

    #[test]
    fn haversine_zero_distance_at_same_point() {
        let d = haversine_metres(52.3676, 4.9041, 52.3676, 4.9041);
        assert!(d < 0.001, "expected ~0m got {d}");
    }

    #[test]
    fn haversine_50m_threshold_detection() {
        // 0.0004° lat ≈ 44m (within 50m threshold)
        let d = haversine_metres(52.0000, 4.0000, 52.0004, 4.0000);
        assert!(d < 50.0, "expected <50m got {d}");
        // 0.001° lat ≈ 111m (beyond threshold)
        let d2 = haversine_metres(52.0000, 4.0000, 52.0010, 4.0000);
        assert!(d2 > 50.0, "expected >50m got {d2}");
    }

    // ── Deduplication ─────────────────────────────────────────────────────────

    #[test]
    fn dedup_single_competitor_unchanged() {
        let c = make_competitor("Iron Temple", 52.0, 4.0);
        let result = deduplicate(vec![c]);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn dedup_same_name_within_50m_merges_to_one() {
        let c1 = make_competitor("Iron Temple", 52.0000, 4.0000);
        let c2 = make_competitor("Iron Temple", 52.0001, 4.0000); // ~11m apart
        let result = deduplicate(vec![c1, c2]);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn dedup_same_name_beyond_50m_stays_two() {
        let c1 = make_competitor("Iron Temple", 52.0000, 4.0000);
        let c2 = make_competitor("Iron Temple", 52.0010, 4.0000); // ~111m
        let result = deduplicate(vec![c1, c2]);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn dedup_different_name_within_50m_stays_two() {
        let c1 = make_competitor("Iron Temple", 52.0000, 4.0000);
        let c2 = make_competitor("Gold Gym", 52.0000, 4.0000);
        let result = deduplicate(vec![c1, c2]);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn dedup_name_comparison_case_insensitive_and_trimmed() {
        let c1 = make_competitor("  iron temple  ", 52.0000, 4.0000);
        let c2 = make_competitor("IRON TEMPLE", 52.0001, 4.0000); // ~11m
        let result = deduplicate(vec![c1, c2]);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn dedup_merge_keeps_higher_confidence_field() {
        let mut c1 = make_competitor("Iron Temple", 52.0000, 4.0000);
        c1.profile.phone = DataPoint::present("phone", "+31-low", "osm", Confidence::Low);

        let mut c2 = make_competitor("Iron Temple", 52.0001, 4.0000); // ~11m
        c2.profile.phone = DataPoint::present("phone", "+31-high", "yelp", Confidence::High);

        let result = deduplicate(vec![c1, c2]);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].profile.phone.value.as_deref(), Some("+31-high"));
    }
}

