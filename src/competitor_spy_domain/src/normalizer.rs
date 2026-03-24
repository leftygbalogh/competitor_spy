// normalizer.rs — T-016
// Converts Vec<RawRecord> + resolved Location into Vec<Competitor>.
// Pure domain logic: no I/O, no async.

use std::collections::HashMap;

use uuid::Uuid;

use crate::profile::{BusinessProfile, Competitor, Confidence, DataPoint, PlaceReview};
use crate::query::Location;
use crate::run::RawRecord;

/// Convert raw adapter records into Competitor entities.
///
/// Records missing a parseable `lat`/`lon` are silently dropped —
/// they cannot be placed in geographic space.
/// `resolved_location` is the geocoded centre point of the search;
/// it is used to compute `distance_km` for each competitor.
pub fn normalize(records: Vec<RawRecord>, resolved_location: &Location) -> Vec<Competitor> {
    records
        .into_iter()
        .filter_map(|r| raw_to_competitor(r, resolved_location))
        .collect()
}

fn raw_to_competitor(record: RawRecord, resolved: &Location) -> Option<Competitor> {
    let lat: f64 = record.fields.get("lat")?.parse().ok()?;
    let lon: f64 = record.fields.get("lon")?.parse().ok()?;
    let location = Location::new(lat, lon).ok()?;
    let distance_km = haversine_km(resolved.latitude, resolved.longitude, lat, lon);
    let profile = build_profile(&record.fields, &record.adapter_id);

    Some(Competitor {
        id: Uuid::new_v4(),
        profile,
        location,
        distance_km,
        keyword_score: 0.0,
        visibility_score: 0.0,
        rank: 0,
    })
}

fn make_dp(
    field_name: &str,
    fields: &HashMap<String, String>,
    source_id: &str,
) -> DataPoint {
    match fields.get(field_name) {
        Some(v) if !v.is_empty() => {
            DataPoint::present(field_name, v.as_str(), source_id, Confidence::Medium)
        }
        _ => DataPoint::absent(field_name),
    }
}

fn build_profile(fields: &HashMap<String, String>, source_id: &str) -> BusinessProfile {
    BusinessProfile {
        name:              make_dp("name",              fields, source_id),
        address:           make_dp("address",           fields, source_id),
        phone:             make_dp("phone",             fields, source_id),
        website:           make_dp("website",           fields, source_id),
        categories:        make_dp("categories",        fields, source_id),
        opening_hours:     make_dp("opening_hours",     fields, source_id),
        email:             make_dp("email",             fields, source_id),
        description:       make_dp("description",       fields, source_id),
        rating_text:       make_dp("rating_text",       fields, source_id),
        review_count_text: make_dp("review_count_text", fields, source_id),
        editorial_summary: make_dp("editorial_summary", fields, source_id),
        price_level:       make_dp("price_level",       fields, source_id),
        reviews:           parse_reviews_json(fields),
    }
}

/// Parse the `reviews_json` field (a JSON array string set by the Google Places
/// adapter) into a `Vec<PlaceReview>`. Returns empty vec on any error or absence.
fn parse_reviews_json(fields: &HashMap<String, String>) -> Vec<PlaceReview> {
    let json_str = match fields.get("reviews_json") {
        Some(s) => s,
        None => return Vec::new(),
    };
    let values: Vec<serde_json::Value> = match serde_json::from_str(json_str) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    values
        .into_iter()
        .filter_map(|v| {
            let text = v["text"].as_str()?.to_string();
            let rating = v["rating"].as_u64()? as u8;
            let relative_time = v.get("relative_time")
                .and_then(|t| t.as_str())
                .unwrap_or("")
                .to_string();
            Some(PlaceReview { text, rating, relative_time })
        })
        .collect()
}

fn haversine_km(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    const R_KM: f64 = 6_371.0;
    let dlat = (lat2 - lat1).to_radians();
    let dlon = (lon2 - lon1).to_radians();
    let lat1r = lat1.to_radians();
    let lat2r = lat2.to_radians();
    let a = (dlat / 2.0).sin().powi(2)
        + lat1r.cos() * lat2r.cos() * (dlon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
    R_KM * c
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn resolved() -> Location {
        Location::new(51.5, -0.1).unwrap()
    }

    fn make_record(adapter_id: &str, extra: &[(&str, &str)]) -> RawRecord {
        let mut fields = HashMap::new();
        fields.insert("lat".to_string(), "51.501".to_string());
        fields.insert("lon".to_string(), "-0.102".to_string());
        fields.insert("name".to_string(), "Acme Ltd".to_string());
        fields.insert("address".to_string(), "1 High St".to_string());
        for (k, v) in extra {
            fields.insert(k.to_string(), v.to_string());
        }
        RawRecord {
            adapter_id: adapter_id.to_string(),
            fields,
        }
    }

    #[test]
    fn drops_record_without_lat() {
        let mut r = make_record("test", &[]);
        r.fields.remove("lat");
        let result = normalize(vec![r], &resolved());
        assert!(result.is_empty());
    }

    #[test]
    fn drops_record_without_lon() {
        let mut r = make_record("test", &[]);
        r.fields.remove("lon");
        let result = normalize(vec![r], &resolved());
        assert!(result.is_empty());
    }

    #[test]
    fn drops_record_with_invalid_lat() {
        let mut r = make_record("test", &[]);
        r.fields.insert("lat".to_string(), "not-a-float".to_string());
        let result = normalize(vec![r], &resolved());
        assert!(result.is_empty());
    }

    #[test]
    fn produces_competitor_from_valid_record() {
        let r = make_record("yelp", &[]);
        let result = normalize(vec![r], &resolved());
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn name_field_maps_to_profile_name() {
        let r = make_record("yelp", &[("name", "Best Plumber")]);
        let result = normalize(vec![r], &resolved());
        assert_eq!(
            result[0].profile.name.value.as_deref(),
            Some("Best Plumber")
        );
    }

    #[test]
    fn source_id_matches_adapter_id() {
        let r = make_record("google_places", &[]);
        let result = normalize(vec![r], &resolved());
        assert_eq!(result[0].profile.name.source_id, "google_places");
    }

    #[test]
    fn absent_field_when_key_missing() {
        let r = make_record("yelp", &[]);
        let result = normalize(vec![r], &resolved());
        assert!(result[0].profile.phone.is_absent());
    }

    #[test]
    fn absent_field_when_value_empty() {
        let r = make_record("yelp", &[("phone", "")]);
        let result = normalize(vec![r], &resolved());
        assert!(result[0].profile.phone.is_absent());
    }

    #[test]
    fn present_phone_maps_correctly() {
        let r = make_record("yelp", &[("phone", "+44 20 1234 5678")]);
        let result = normalize(vec![r], &resolved());
        assert_eq!(
            result[0].profile.phone.value.as_deref(),
            Some("+44 20 1234 5678")
        );
        assert_eq!(result[0].profile.phone.confidence, Confidence::Medium);
    }

    #[test]
    fn distance_km_is_positive_and_small() {
        let r = make_record("yelp", &[]); // lat=51.501, lon=-0.102
        let result = normalize(vec![r], &resolved()); // lat=51.5, lon=-0.1
        let dist = result[0].distance_km;
        assert!(dist > 0.0 && dist < 5.0, "expected small distance, got {dist}");
    }

    #[test]
    fn multiple_records_produce_multiple_competitors() {
        let r1 = make_record("yelp", &[("name", "A")]);
        let mut r2_fields = HashMap::new();
        r2_fields.insert("lat".to_string(), "51.502".to_string());
        r2_fields.insert("lon".to_string(), "-0.103".to_string());
        r2_fields.insert("name".to_string(), "B".to_string());
        let r2 = RawRecord {
            adapter_id: "google_places".to_string(),
            fields: r2_fields,
        };
        let result = normalize(vec![r1, r2], &resolved());
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn empty_vec_returns_empty() {
        let result = normalize(vec![], &resolved());
        assert!(result.is_empty());
    }

    #[test]
    fn keyword_score_and_visibility_score_start_at_zero() {
        let r = make_record("yelp", &[]);
        let result = normalize(vec![r], &resolved());
        assert_eq!(result[0].keyword_score, 0.0);
        assert_eq!(result[0].visibility_score, 0.0);
    }

    #[test]
    fn rank_starts_at_zero() {
        let r = make_record("yelp", &[]);
        let result = normalize(vec![r], &resolved());
        assert_eq!(result[0].rank, 0);
    }
}
