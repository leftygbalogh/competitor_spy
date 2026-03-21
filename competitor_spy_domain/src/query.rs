// SearchQuery, Location, Radius — implemented in T-001
// TDD: tests are written first; implementation follows.

use thiserror::Error;

// ── Radius ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Radius {
    Km5,
    Km10,
    Km20,
    Km25,
    Km50,
}

#[derive(Debug, Error, PartialEq)]
pub enum RadiusError {
    #[error("invalid radius {0} km: must be one of 5, 10, 20, 25, 50")]
    InvalidValue(u32),
}

impl Radius {
    pub fn km_value(self) -> u32 {
        match self {
            Radius::Km5 => 5,
            Radius::Km10 => 10,
            Radius::Km20 => 20,
            Radius::Km25 => 25,
            Radius::Km50 => 50,
        }
    }
}

impl TryFrom<u32> for Radius {
    type Error = RadiusError;
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            5 => Ok(Radius::Km5),
            10 => Ok(Radius::Km10),
            20 => Ok(Radius::Km20),
            25 => Ok(Radius::Km25),
            50 => Ok(Radius::Km50),
            _ => Err(RadiusError::InvalidValue(value)),
        }
    }
}

// ── Location ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct Location {
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Debug, Error, PartialEq)]
pub enum LocationError {
    #[error("latitude {0} out of range [-90.0, 90.0]")]
    LatitudeOutOfRange(f64),
    #[error("longitude {0} out of range [-180.0, 180.0]")]
    LongitudeOutOfRange(f64),
}

impl Location {
    pub fn new(latitude: f64, longitude: f64) -> Result<Self, LocationError> {
        if !(-90.0..=90.0).contains(&latitude) {
            return Err(LocationError::LatitudeOutOfRange(latitude));
        }
        if !(-180.0..=180.0).contains(&longitude) {
            return Err(LocationError::LongitudeOutOfRange(longitude));
        }
        Ok(Self { latitude, longitude })
    }
}

// ── SearchQuery ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct SearchQuery {
    pub industry: String,
    pub location_input: String,
    pub radius: Radius,
}

#[derive(Debug, Error, PartialEq)]
pub enum SearchQueryError {
    #[error("industry must not be empty")]
    EmptyIndustry,
    #[error("location must not be empty")]
    EmptyLocation,
}

impl SearchQuery {
    pub fn new(
        industry: impl Into<String>,
        location_input: impl Into<String>,
        radius: Radius,
    ) -> Result<Self, SearchQueryError> {
        let industry = industry.into();
        let location_input = location_input.into();
        if industry.trim().is_empty() {
            return Err(SearchQueryError::EmptyIndustry);
        }
        if location_input.trim().is_empty() {
            return Err(SearchQueryError::EmptyLocation);
        }
        Ok(Self { industry, location_input, radius })
    }
}

// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Radius ────────────────────────────────────────────────────────────────

    #[test]
    fn radius_valid_values_accepted() {
        assert!(Radius::try_from(5u32).is_ok());
        assert!(Radius::try_from(10u32).is_ok());
        assert!(Radius::try_from(20u32).is_ok());
        assert!(Radius::try_from(25u32).is_ok());
        assert!(Radius::try_from(50u32).is_ok());
    }

    #[test]
    fn radius_invalid_value_rejected() {
        assert!(Radius::try_from(7u32).is_err());
        assert!(Radius::try_from(0u32).is_err());
        assert!(Radius::try_from(100u32).is_err());
    }

    #[test]
    fn radius_km_values_correct() {
        assert_eq!(Radius::Km5.km_value(), 5);
        assert_eq!(Radius::Km10.km_value(), 10);
        assert_eq!(Radius::Km20.km_value(), 20);
        assert_eq!(Radius::Km25.km_value(), 25);
        assert_eq!(Radius::Km50.km_value(), 50);
    }

    // ── Location ──────────────────────────────────────────────────────────────

    #[test]
    fn location_valid_coordinates_accepted() {
        let loc = Location::new(52.3676, 4.9041).unwrap();
        assert!((loc.latitude - 52.3676).abs() < 1e-6);
        assert!((loc.longitude - 4.9041).abs() < 1e-6);
    }

    #[test]
    fn location_lat_out_of_range_rejected() {
        assert!(matches!(
            Location::new(91.0, 0.0),
            Err(LocationError::LatitudeOutOfRange(_))
        ));
        assert!(matches!(
            Location::new(-91.0, 0.0),
            Err(LocationError::LatitudeOutOfRange(_))
        ));
    }

    #[test]
    fn location_lon_out_of_range_rejected() {
        assert!(matches!(
            Location::new(0.0, 181.0),
            Err(LocationError::LongitudeOutOfRange(_))
        ));
        assert!(matches!(
            Location::new(0.0, -181.0),
            Err(LocationError::LongitudeOutOfRange(_))
        ));
    }

    #[test]
    fn location_boundary_values_accepted() {
        assert!(Location::new(90.0, 180.0).is_ok());
        assert!(Location::new(-90.0, -180.0).is_ok());
    }

    // ── SearchQuery ───────────────────────────────────────────────────────────

    #[test]
    fn search_query_valid_fields_accepted() {
        let q = SearchQuery::new("yoga studio", "Amsterdam, Netherlands", Radius::Km10).unwrap();
        assert_eq!(q.industry, "yoga studio");
        assert_eq!(q.location_input, "Amsterdam, Netherlands");
        assert_eq!(q.radius, Radius::Km10);
    }

    #[test]
    fn search_query_empty_industry_rejected() {
        assert!(matches!(
            SearchQuery::new("", "Amsterdam", Radius::Km10),
            Err(SearchQueryError::EmptyIndustry)
        ));
    }

    #[test]
    fn search_query_whitespace_only_industry_rejected() {
        assert!(matches!(
            SearchQuery::new("   ", "Amsterdam", Radius::Km10),
            Err(SearchQueryError::EmptyIndustry)
        ));
    }

    #[test]
    fn search_query_empty_location_rejected() {
        assert!(matches!(
            SearchQuery::new("yoga", "", Radius::Km10),
            Err(SearchQueryError::EmptyLocation)
        ));
    }

    #[test]
    fn search_query_whitespace_only_location_rejected() {
        assert!(matches!(
            SearchQuery::new("yoga", "  ", Radius::Km10),
            Err(SearchQueryError::EmptyLocation)
        ));
    }
}

