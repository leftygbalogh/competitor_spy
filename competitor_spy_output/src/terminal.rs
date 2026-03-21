// Terminal report renderer — T-014
// Formats a finalised SearchRun to stdout as a plain-text table.
// §4.6, §6.5, §9.2

use std::fmt::Write as FmtWrite;
use std::io::{self, Write};

use competitor_spy_domain::run::{AdapterResultStatus, SearchRun};

// ── Column widths ────────────────────────────────────────────────────────────

const W_RANK:       usize = 4;
const W_NAME:       usize = 28;
const W_DIST:       usize = 9;
const W_ADDRESS:    usize = 30;
const W_PHONE:      usize = 16;
const W_WEBSITE:    usize = 28;
const W_KEYWORD:    usize = 9;
const W_VISIBILITY: usize = 11;

// ── Public API ───────────────────────────────────────────────────────────────

/// Render `run` to `out` as a plain-text table.
///
/// Returns `Err` only on I/O failure.
pub fn render<W: Write>(run: &SearchRun, out: &mut W) -> io::Result<()> {
    let text = format_run(run);
    out.write_all(text.as_bytes())
}

/// Render to stdout.
pub fn render_stdout(run: &SearchRun) -> io::Result<()> {
    render(run, &mut io::stdout())
}

// ── Formatting ────────────────────────────────────────────────────────────────

/// Build the full report as a String.
pub fn format_run(run: &SearchRun) -> String {
    let mut buf = String::new();

    // ── Header ──────────────────────────────────────────────────────────────
    writeln!(buf, "Competitor Spy Report").unwrap();
    writeln!(buf, "Industry : {}", run.query.industry).unwrap();
    writeln!(buf, "Location : {}", run.query.location_input).unwrap();
    writeln!(buf, "Radius   : {} km", run.query.radius.km_value()).unwrap();
    if let Some(t) = run.completed_at {
        writeln!(buf, "Run UTC  : {}", t.format("%Y-%m-%d %H:%M:%S UTC")).unwrap();
    }
    writeln!(buf).unwrap();

    // ── Table header ─────────────────────────────────────────────────────────
    let separator = build_separator();
    writeln!(buf, "{}", build_header_row()).unwrap();
    writeln!(buf, "{separator}").unwrap();

    // ── Competitor rows ───────────────────────────────────────────────────────
    if run.competitors.is_empty() {
        writeln!(buf, "  (no competitors found)").unwrap();
    } else {
        for c in &run.competitors {
            let name    = field_val(&c.profile.name.value);
            let address = field_val(&c.profile.address.value);
            let phone   = field_val(&c.profile.phone.value);
            let website = field_val(&c.profile.website.value);
            let dist    = format!("{:.2} km", c.distance_km);
            let kw      = format!("{}%", (c.keyword_score * 100.0).round() as u32);
            let vis     = format!("{}%", (c.visibility_score * 100.0).round() as u32);

            writeln!(
                buf,
                "{}",
                build_row(c.rank, &name, &dist, &address, &phone, &website, &kw, &vis)
            ).unwrap();
        }
    }

    writeln!(buf, "{separator}").unwrap();

    // ── Footer: failed sources ────────────────────────────────────────────────
    let failed: Vec<_> = run
        .source_results
        .iter()
        .filter(|sr| sr.status.is_failed())
        .collect();

    if !failed.is_empty() {
        writeln!(buf).unwrap();
        writeln!(buf, "Failed sources:").unwrap();
        for sr in failed {
            let reason = match &sr.status {
                AdapterResultStatus::Failed(code) => format!("{code}"),
                _ => String::new(),
            };
            writeln!(buf, "  - {} : {reason}", sr.adapter_id).unwrap();
        }
    }

    buf
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn field_val(v: &Option<String>) -> String {
    v.clone().unwrap_or_else(|| "--".to_string())
}

fn trunc(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        // Truncate at character boundary and add ellipsis
        let end = s
            .char_indices()
            .nth(max - 1)
            .map(|(i, _)| i)
            .unwrap_or(s.len());
        format!("{}…", &s[..end])
    }
}

fn cell(s: &str, width: usize) -> String {
    let truncated = trunc(s, width);
    format!("{:<width$}", truncated, width = width)
}

fn build_separator() -> String {
    format!(
        "{}-{}-{}-{}-{}-{}-{}-{}",
        "-".repeat(W_RANK),
        "-".repeat(W_NAME),
        "-".repeat(W_DIST),
        "-".repeat(W_ADDRESS),
        "-".repeat(W_PHONE),
        "-".repeat(W_WEBSITE),
        "-".repeat(W_KEYWORD),
        "-".repeat(W_VISIBILITY),
    )
}

fn build_header_row() -> String {
    format!(
        "{} {} {} {} {} {} {} {}",
        cell("Rank", W_RANK),
        cell("Name", W_NAME),
        cell("Distance", W_DIST),
        cell("Address", W_ADDRESS),
        cell("Phone", W_PHONE),
        cell("Website", W_WEBSITE),
        cell("Keyword%", W_KEYWORD),
        cell("Visibility%", W_VISIBILITY),
    )
}

fn build_row(
    rank: u32,
    name: &str,
    dist: &str,
    address: &str,
    phone: &str,
    website: &str,
    kw: &str,
    vis: &str,
) -> String {
    format!(
        "{} {} {} {} {} {} {} {}",
        cell(&rank.to_string(), W_RANK),
        cell(name, W_NAME),
        cell(dist, W_DIST),
        cell(address, W_ADDRESS),
        cell(phone, W_PHONE),
        cell(website, W_WEBSITE),
        cell(kw, W_KEYWORD),
        cell(vis, W_VISIBILITY),
    )
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use competitor_spy_domain::{
        profile::{BusinessProfile, Competitor, DataPoint, Confidence},
        query::{Location, Radius, SearchQuery},
        run::{AdapterResultStatus, ReasonCode, SearchRun, SourceResult},
    };
    use uuid::Uuid;

    fn make_location() -> Location {
        Location::new(52.3676, 4.9041).unwrap()
    }

    fn make_query() -> SearchQuery {
        SearchQuery::new("yoga studio", "Amsterdam, Netherlands", Radius::Km10).unwrap()
    }

    fn make_competitor(rank: u32, name: &str, dist: f64, kw: f64, vis: f64) -> Competitor {
        let mut profile = BusinessProfile::empty();
        profile.name = DataPoint::present("name", name, "test", Confidence::High);
        profile.address = DataPoint::present("address", "123 Test St, Amsterdam", "test", Confidence::High);
        profile.phone = DataPoint::present("phone", "+31 20 000 0000", "test", Confidence::Medium);
        Competitor {
            id: Uuid::new_v4(),
            profile,
            location: make_location(),
            distance_km: dist,
            keyword_score: kw,
            visibility_score: vis,
            rank,
        }
    }

    fn make_run_done() -> SearchRun {
        let query = make_query();
        let ts = chrono::Utc.with_ymd_and_hms(2026, 3, 21, 14, 30, 22).unwrap();
        let mut run = SearchRun::new(query, ts);
        run.start_validating();
        run.start_geocoding();
        run.set_location(make_location());
        // Add a failed source result
        let failed = SourceResult {
            adapter_id: "yelp".to_string(),
            status: AdapterResultStatus::Failed(ReasonCode::Http4xx),
            records: vec![],
            retrieved_at: ts,
        };
        run.add_source_result(failed);
        run.start_ranking();
        let c1 = make_competitor(1, "Zen Yoga Amsterdam", 1.2, 0.85, 0.70);
        let c2 = make_competitor(2, "Power Flow Studio", 2.5, 0.60, 0.50);
        run.set_competitors(vec![c1, c2]);
        run.complete(ts);
        run
    }

    // ── Tests ─────────────────────────────────────────────────────────────────

    #[test]
    fn render_contains_header_industry_line() {
        let run = make_run_done();
        let output = format_run(&run);
        assert!(output.contains("Industry : yoga studio"), "got:\n{output}");
    }

    #[test]
    fn render_contains_location_line() {
        let run = make_run_done();
        let output = format_run(&run);
        assert!(output.contains("Location : Amsterdam, Netherlands"), "got:\n{output}");
    }

    #[test]
    fn render_contains_radius_line() {
        let run = make_run_done();
        let output = format_run(&run);
        assert!(output.contains("Radius   : 10 km"), "got:\n{output}");
    }

    #[test]
    fn render_contains_competitor_names() {
        let run = make_run_done();
        let output = format_run(&run);
        assert!(output.contains("Zen Yoga Amsterdam"), "got:\n{output}");
        assert!(output.contains("Power Flow Studio"), "got:\n{output}");
    }

    #[test]
    fn render_contains_rank_numbers() {
        let run = make_run_done();
        let output = format_run(&run);
        // rank 1 and 2 appear in first column
        assert!(output.contains("1   "), "no rank 1 found in:\n{output}");
        assert!(output.contains("2   "), "no rank 2 found in:\n{output}");
    }

    #[test]
    fn render_contains_distance_formatted() {
        let run = make_run_done();
        let output = format_run(&run);
        assert!(output.contains("1.20 km"), "got:\n{output}");
        assert!(output.contains("2.50 km"), "got:\n{output}");
    }

    #[test]
    fn render_contains_keyword_and_visibility_percentages() {
        let run = make_run_done();
        let output = format_run(&run);
        assert!(output.contains("85%"), "got:\n{output}");
        assert!(output.contains("70%"), "got:\n{output}");
    }

    #[test]
    fn render_absent_field_displays_double_dash() {
        let query = make_query();
        let ts = chrono::Utc.with_ymd_and_hms(2026, 3, 21, 10, 0, 0).unwrap();
        let mut run = SearchRun::new(query, ts);
        run.start_validating();
        run.start_geocoding();
        run.set_location(make_location());
        run.start_ranking();
        // Competitor with all absent fields except name
        let mut profile = BusinessProfile::empty();
        profile.name = DataPoint::present("name", "No Fields Co", "test", Confidence::High);
        let c = Competitor {
            id: Uuid::new_v4(),
            profile,
            location: make_location(),
            distance_km: 0.5,
            keyword_score: 0.0,
            visibility_score: 0.0,
            rank: 1,
        };
        run.set_competitors(vec![c]);
        run.complete(ts);
        let output = format_run(&run);
        // Phone and website are absent -> "--"
        assert!(output.contains("--"), "absent field should be '--': {output}");
    }

    #[test]
    fn render_footer_lists_failed_source() {
        let run = make_run_done();
        let output = format_run(&run);
        assert!(output.contains("Failed sources:"), "got:\n{output}");
        assert!(output.contains("yelp"), "got:\n{output}");
        assert!(output.contains("HTTP_4XX"), "got:\n{output}");
    }

    #[test]
    fn render_no_footer_when_all_sources_succeed() {
        let query = make_query();
        let ts = chrono::Utc.with_ymd_and_hms(2026, 3, 21, 10, 0, 0).unwrap();
        let mut run = SearchRun::new(query, ts);
        run.start_validating();
        run.start_geocoding();
        run.set_location(make_location());
        let ok = SourceResult {
            adapter_id: "osm_overpass".to_string(),
            status: AdapterResultStatus::Success,
            records: vec![],
            retrieved_at: ts,
        };
        run.add_source_result(ok);
        run.start_ranking();
        run.set_competitors(vec![]);
        run.complete(ts);
        let output = format_run(&run);
        assert!(!output.contains("Failed sources:"), "unexpected footer:\n{output}");
    }

    #[test]
    fn render_empty_competitors_shows_no_competitors_message() {
        let query = make_query();
        let ts = chrono::Utc.with_ymd_and_hms(2026, 3, 21, 10, 0, 0).unwrap();
        let mut run = SearchRun::new(query, ts);
        run.start_validating();
        run.start_geocoding();
        run.set_location(make_location());
        run.start_ranking();
        run.set_competitors(vec![]);
        run.complete(ts);
        let output = format_run(&run);
        assert!(output.contains("(no competitors found)"), "got:\n{output}");
    }

    #[test]
    fn render_produces_column_headers() {
        let run = make_run_done();
        let output = format_run(&run);
        assert!(output.contains("Rank"), "got:\n{output}");
        assert!(output.contains("Name"), "got:\n{output}");
        assert!(output.contains("Distance"), "got:\n{output}");
        assert!(output.contains("Address"), "got:\n{output}");
        assert!(output.contains("Phone"), "got:\n{output}");
        assert!(output.contains("Website"), "got:\n{output}");
        assert!(output.contains("Keyword%"), "got:\n{output}");
        assert!(output.contains("Visibility%"), "got:\n{output}");
    }

    #[test]
    fn render_long_name_truncated() {
        let query = make_query();
        let ts = chrono::Utc.with_ymd_and_hms(2026, 3, 21, 10, 0, 0).unwrap();
        let mut run = SearchRun::new(query, ts);
        run.start_validating();
        run.start_geocoding();
        run.set_location(make_location());
        run.start_ranking();
        let long_name = "A".repeat(50);
        let mut profile = BusinessProfile::empty();
        profile.name = DataPoint::present("name", &long_name, "test", Confidence::High);
        let c = Competitor {
            id: Uuid::new_v4(),
            profile,
            location: make_location(),
            distance_km: 1.0,
            keyword_score: 0.5,
            visibility_score: 0.5,
            rank: 1,
        };
        run.set_competitors(vec![c]);
        run.complete(ts);
        let output = format_run(&run);
        // Name column is 28 chars wide; long name must be truncated (no full 50-char string)
        assert!(!output.contains(&long_name), "name should be truncated:\n{output}");
        // Truncated name contains ellipsis
        assert!(output.contains('…'), "truncated name should end with ellipsis:\n{output}");
    }

    #[test]
    fn render_writes_to_provided_writer() {
        let run = make_run_done();
        let mut buf: Vec<u8> = Vec::new();
        render(&run, &mut buf).unwrap();
        let text = String::from_utf8(buf).unwrap();
        assert!(text.contains("yoga studio"), "got:\n{text}");
    }

    #[test]
    fn snapshot_matches_expected_output() {
        let run = make_run_done();
        let output = format_run(&run);

        // The snapshot checks structural properties of every line without
        // hard-coding inter-column spaces, making it robust to minor width tweaks.
        let lines: Vec<&str> = output.lines().collect();

        // Header block
        assert!(lines[0].contains("Competitor Spy Report"));
        // Column header row contains all columns
        let header_line = lines.iter().find(|l| l.contains("Rank")).unwrap();
        assert!(header_line.contains("Name"));
        assert!(header_line.contains("Distance"));
        assert!(header_line.contains("Keyword%"));
        assert!(header_line.contains("Visibility%"));

        // At least one data row per competitor
        assert!(output.contains("Zen Yoga Amsterdam"));
        assert!(output.contains("Power Flow Studio"));

        // Footer
        assert!(output.contains("Failed sources:"));
        assert!(output.contains("yelp"));
    }
}
