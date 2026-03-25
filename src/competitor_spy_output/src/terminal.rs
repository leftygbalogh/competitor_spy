// Terminal report renderer — T-005 (V2 card layout, BC-005)
// Formats a finalised SearchRun to stdout as a plain-text card display.

use std::fmt::Write as FmtWrite;
use std::io::{self, Write};

use competitor_spy_domain::enrichment::COVERAGE_THRESHOLD;
use competitor_spy_domain::run::{AdapterResultStatus, SearchRun};

// ── Card separator ────────────────────────────────────────────────────────────

const CARD_SEP: &str = "--------------------------------------------------------------------------------";

// ── Public API ───────────────────────────────────────────────────────────────

/// Render `run` to `out` as a plain-text table.
///
/// Returns `Err` only on I/O failure.
pub fn render<W: Write>(run: &SearchRun, detail: bool, out: &mut W) -> io::Result<()> {
    let text = format_run(run, detail);
    out.write_all(text.as_bytes())
}

/// Render to stdout.
pub fn render_stdout(run: &SearchRun, detail: bool) -> io::Result<()> {
    render(run, detail, &mut io::stdout())
}

// ── Formatting ────────────────────────────────────────────────────────────────

/// Build the full report as a String.
pub fn format_run(run: &SearchRun, detail: bool) -> String {
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

    // ── Competitor cards ─────────────────────────────────────────────────────
    if run.competitors.is_empty() {
        writeln!(buf, "{CARD_SEP}").unwrap();
        writeln!(buf, "  (no competitors found)").unwrap();
        writeln!(buf, "{CARD_SEP}").unwrap();
    } else {
        for c in &run.competitors {
            writeln!(buf, "{CARD_SEP}").unwrap();

            // Name / rank / rating header line
            let name = c.profile.name.value.as_deref().unwrap_or("(unknown)");
            let rating_part = match (
                c.profile.rating_text.value.as_deref(),
                c.profile.review_count_text.value.as_deref(),
            ) {
                (Some(r), Some(cnt)) => format!(" | {}★ ({})", r, cnt),
                (Some(r), None)      => format!(" | {}★", r),
                _                    => String::new(),
            };
            writeln!(buf, "#{rank}  {name}{rating_part}", rank = c.rank).unwrap();
            writeln!(buf, "{CARD_SEP}").unwrap();

            if let Some(v) = &c.profile.address.value {
                writeln!(buf, "{}", label_value("Address", v)).unwrap();
            }
            if let Some(v) = &c.profile.phone.value {
                writeln!(buf, "{}", label_value("Phone", v)).unwrap();
            }
            if let Some(v) = &c.profile.website.value {
                writeln!(buf, "{}", label_value("Website", v)).unwrap();
            }
            if let Some(v) = &c.profile.categories.value {
                writeln!(buf, "{}", label_value("Categories", v)).unwrap();
            }
            if let Some(v) = &c.profile.opening_hours.value {
                buf.push_str(&format_opening_hours(v));
            }
            if let Some(v) = &c.profile.price_level.value {
                writeln!(buf, "{}", label_value("Price Level", v)).unwrap();
            }
            if let Some(v) = &c.profile.editorial_summary.value {
                writeln!(buf, "{}", label_value("Editorial", v)).unwrap();
            }
            if detail {
                for (i, review) in c.profile.reviews.iter().enumerate() {
                    writeln!(
                        buf,
                        "Review {} ({}★, {}): {}",
                        i + 1,
                        review.rating,
                        review.relative_time,
                        review.text,
                    )
                    .unwrap();
                }
            }
            // ── V3 website enrichment fields ────────────────────────────────
            let enrich = run.enrichments.iter().find(|e| e.competitor_id == c.id);
            if let Some(e) = enrich {
                if e.fetch_status.is_success() {
                    if let Some(v) = &e.pricing {
                        writeln!(buf, "{}", label_value("Pricing", v)).unwrap();
                    }
                    if let Some(types) = &e.lesson_types {
                        writeln!(buf, "{}", label_value("Lesson Types", &types.join(", "))).unwrap();
                    }
                    if let Some(v) = &e.schedule {
                        writeln!(buf, "{}", label_value("Schedule", v)).unwrap();
                    }
                    if let Some(items) = &e.testimonials {
                        writeln!(buf, "{}", label_value("Testimonials", &format!("{} found", items.len()))).unwrap();
                        if detail {
                            for (i, t) in items.iter().enumerate() {
                                writeln!(buf, "  [{}] \"{}\"", i + 1, t).unwrap();
                            }
                        }
                    }
                    if let Some(items) = &e.class_descriptions {
                        writeln!(buf, "{}", label_value("Class Descs", &format!("{} found", items.len()))).unwrap();
                        if detail {
                            for (i, d) in items.iter().enumerate() {
                                writeln!(buf, "  [{}] {}", i + 1, d).unwrap();
                            }
                        }
                    }
                }
            }
        }
        writeln!(buf, "{CARD_SEP}").unwrap();
    }

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

    // ── Footer: V3 enrichment coverage ───────────────────────────────────────
    if !run.enrichments.is_empty() {
        writeln!(buf).unwrap();
        let n = run.enrichments.len();
        let with_data = (run.enrichment_coverage * n as f64).round() as usize;
        writeln!(
            buf,
            "Enrichment:  {with_data}/{n} studios ({:.0}%) had at least one extractable field.",
            run.enrichment_coverage * 100.0,
        ).unwrap();
        if run.enrichment_coverage < COVERAGE_THRESHOLD {
            writeln!(buf, "  Warning: enrichment coverage is below the 60% threshold.").unwrap();
        }
    }

    buf
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Format a label/value pair with the label left-padded to 15 chars before ": ".
fn label_value(label: &str, value: &str) -> String {
    format!("{:<15}: {}", label, value)
}

/// Format opening hours: first line inline, continuation lines indented 17 chars.
fn format_opening_hours(v: &str) -> String {
    let mut out = String::new();
    let mut lines = v.split('\n');
    if let Some(first) = lines.next() {
        writeln!(out, "{:<15}: {}", "Opening Hours", first).unwrap();
        for line in lines {
            writeln!(out, "                 {line}").unwrap();
        }
    }
    out
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use competitor_spy_domain::{
        profile::{BusinessProfile, Competitor, Confidence, DataPoint, PlaceReview},
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
        let output = format_run(&run, false);
        assert!(output.contains("Industry : yoga studio"), "got:\n{output}");
    }

    #[test]
    fn render_contains_location_line() {
        let run = make_run_done();
        let output = format_run(&run, false);
        assert!(output.contains("Location : Amsterdam, Netherlands"), "got:\n{output}");
    }

    #[test]
    fn render_contains_radius_line() {
        let run = make_run_done();
        let output = format_run(&run, false);
        assert!(output.contains("Radius   : 10 km"), "got:\n{output}");
    }

    #[test]
    fn render_contains_competitor_names() {
        let run = make_run_done();
        let output = format_run(&run, false);
        assert!(output.contains("Zen Yoga Amsterdam"), "got:\n{output}");
        assert!(output.contains("Power Flow Studio"), "got:\n{output}");
    }

    #[test]
    fn render_contains_rank_numbers() {
        let run = make_run_done();
        let output = format_run(&run, false);
        assert!(output.contains("#1  "), "no '#1  ' found in:\n{output}");
        assert!(output.contains("#2  "), "no '#2  ' found in:\n{output}");
    }

    // T-OUT-001: no truncation — 50-char name renders in full
    #[test]
    fn t_out_001_no_truncation_long_name() {
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
        let output = format_run(&run, false);
        assert!(
            output.contains(&long_name),
            "50-char name must appear in full:\n{output}"
        );
    }

    // T-OUT-002: card separator present
    #[test]
    fn t_out_002_card_sep_present() {
        let run = make_run_done();
        let output = format_run(&run, false);
        assert!(
            output.contains(CARD_SEP),
            "CARD_SEP must appear in output:\n{output}"
        );
    }

    // T-OUT-005: absent fields are silently omitted
    #[test]
    fn t_out_005_absent_fields_silently_omitted() {
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
        let output = format_run(&run, false);
        assert!(
            !output.contains("Phone          :"),
            "absent Phone label must not appear:\n{output}"
        );
        assert!(
            !output.contains("Website        :"),
            "absent Website label must not appear:\n{output}"
        );
    }

    #[test]
    fn render_footer_lists_failed_source() {
        let run = make_run_done();
        let output = format_run(&run, false);
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
        let output = format_run(&run, false);
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
        let output = format_run(&run, false);
        assert!(output.contains("(no competitors found)"), "got:\n{output}");
    }

    // T-OUT-003: detail=false omits reviews
    #[test]
    fn t_out_003_detail_false_omits_reviews() {
        let query = make_query();
        let ts = chrono::Utc.with_ymd_and_hms(2026, 3, 21, 10, 0, 0).unwrap();
        let mut run = SearchRun::new(query, ts);
        run.start_validating();
        run.start_geocoding();
        run.set_location(make_location());
        run.start_ranking();
        let mut profile = BusinessProfile::empty();
        profile.name = DataPoint::present("name", "Review Studio", "test", Confidence::High);
        profile.reviews = vec![PlaceReview {
            text: "Fantastic class!".into(),
            rating: 5,
            relative_time: "1 week ago".into(),
        }];
        let c = Competitor {
            id: Uuid::new_v4(),
            profile,
            location: make_location(),
            distance_km: 0.5,
            keyword_score: 0.8,
            visibility_score: 0.8,
            rank: 1,
        };
        run.set_competitors(vec![c]);
        run.complete(ts);
        let output = format_run(&run, false);
        assert!(
            !output.contains("Review 1"),
            "reviews must be hidden when detail=false:\n{output}"
        );
    }

    // T-OUT-004: detail=true includes reviews
    #[test]
    fn t_out_004_detail_true_includes_reviews() {
        let query = make_query();
        let ts = chrono::Utc.with_ymd_and_hms(2026, 3, 21, 10, 0, 0).unwrap();
        let mut run = SearchRun::new(query, ts);
        run.start_validating();
        run.start_geocoding();
        run.set_location(make_location());
        run.start_ranking();
        let mut profile = BusinessProfile::empty();
        profile.name = DataPoint::present("name", "Review Studio", "test", Confidence::High);
        profile.reviews = vec![PlaceReview {
            text: "Fantastic class!".into(),
            rating: 5,
            relative_time: "1 week ago".into(),
        }];
        let c = Competitor {
            id: Uuid::new_v4(),
            profile,
            location: make_location(),
            distance_km: 0.5,
            keyword_score: 0.8,
            visibility_score: 0.8,
            rank: 1,
        };
        run.set_competitors(vec![c]);
        run.complete(ts);
        let output = format_run(&run, true);
        assert!(
            output.contains("Review 1 (5\u{2605}"),
            "reviews must appear when detail=true:\n{output}"
        );
        assert!(
            output.contains("Fantastic class!"),
            "review text must appear:\n{output}"
        );
    }

    #[test]
    fn render_writes_to_provided_writer() {
        let run = make_run_done();
        let mut buf: Vec<u8> = Vec::new();
        render(&run, false, &mut buf).unwrap();
        let text = String::from_utf8(buf).unwrap();
        assert!(text.contains("yoga studio"), "got:\n{text}");
    }

    #[test]
    fn snapshot_matches_expected_output() {
        let run = make_run_done();
        let output = format_run(&run, false);
        let lines: Vec<&str> = output.lines().collect();

        // Header block
        assert!(lines[0].contains("Competitor Spy Report"));
        assert!(output.contains("Industry : yoga studio"));
        assert!(output.contains("Location : Amsterdam, Netherlands"));

        // Card separators exist
        assert!(output.contains(CARD_SEP), "card separator missing");

        // Rank header lines
        assert!(output.contains("#1  Zen Yoga Amsterdam"), "rank 1 card header");
        assert!(output.contains("#2  Power Flow Studio"), "rank 2 card header");

        // Footer
        assert!(output.contains("Failed sources:"));
        assert!(output.contains("yelp"));
    }
}
