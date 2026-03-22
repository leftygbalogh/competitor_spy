// PDF report renderer — T-015
// Formats a finalised SearchRun to a PDF file.
// §4.6, §6.2, §6.5, §9.2

use std::io::{self, BufWriter};
use std::path::Path;

use printpdf::*;

use competitor_spy_domain::run::{AdapterResultStatus, SearchRun};

// ── A4 dimensions ─────────────────────────────────────────────────────────────

const A4_WIDTH_MM:  f32 = 210.0;
const A4_HEIGHT_MM: f32 = 297.0;

// ── Filename format ───────────────────────────────────────────────────────────

/// Generate the PDF filename from the run's completed_at timestamp.
/// Format: `competitor_spy_report_YYYYMMDD_HHMMSS_UTC.pdf`
pub fn pdf_filename(run: &SearchRun) -> String {
    let ts = run
        .completed_at
        .or(Some(run.started_at))
        .unwrap_or(run.started_at);
    ts.format("competitor_spy_report_%Y%m%d_%H%M%S_UTC.pdf")
        .to_string()
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Render `run` to a PDF file inside `output_dir`.
///
/// Returns the full path of the file written on success.
/// Returns `Err` on I/O failure; caller should downgrade to warning per spec §6.2.
pub fn render_to_dir(run: &SearchRun, output_dir: &Path) -> io::Result<std::path::PathBuf> {
    let filename = pdf_filename(run);
    let path = output_dir.join(&filename);
    let file = std::fs::File::create(&path)?;
    let writer = BufWriter::new(file);
    render_to_writer(run, writer)?;
    Ok(path)
}

/// Render `run` to an arbitrary writer (useful for testing).
pub fn render_to_writer<W: io::Write + io::Seek>(
    run: &SearchRun,
    writer: W,
) -> io::Result<()> {
    let doc = build_document(run);
    doc.save(&mut std::io::BufWriter::new(writer))
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))
}

/// Render to a `Vec<u8>` in-memory. Useful for tests.
pub fn render_to_bytes(run: &SearchRun) -> io::Result<Vec<u8>> {
    use std::io::Cursor;
    let mut buf = Cursor::new(Vec::new());
    render_to_writer(run, &mut buf)?;
    Ok(buf.into_inner())
}

// ── Document builder ──────────────────────────────────────────────────────────

fn build_document(run: &SearchRun) -> PdfDocumentReference {
    let (doc, page1, layer1) = PdfDocument::new(
        "Competitor Spy Report",
        Mm(A4_WIDTH_MM),
        Mm(A4_HEIGHT_MM),
        "Main",
    );

    let font = doc
        .add_builtin_font(BuiltinFont::Helvetica)
        .expect("builtin font always available");
    let font_bold = doc
        .add_builtin_font(BuiltinFont::HelveticaBold)
        .expect("builtin font always available");

    let layer = doc.get_page(page1).get_layer(layer1);

    // ── Header ──────────────────────────────────────────────────────────────
    let mut y = A4_HEIGHT_MM - 15.0;
    layer.use_text("Competitor Spy Report", 16.0, Mm(10.0), Mm(y), &font_bold);
    y -= 8.0;
    layer.use_text(
        &format!("Industry: {}", run.query.industry),
        10.0, Mm(10.0), Mm(y), &font,
    );
    y -= 5.0;
    layer.use_text(
        &format!("Location: {}", run.query.location_input),
        10.0, Mm(10.0), Mm(y), &font,
    );
    y -= 5.0;
    layer.use_text(
        &format!("Radius  : {} km", run.query.radius.km_value()),
        10.0, Mm(10.0), Mm(y), &font,
    );
    if let Some(ts) = run.completed_at {
        y -= 5.0;
        layer.use_text(
            &format!("Run UTC : {}", ts.format("%Y-%m-%d %H:%M:%S UTC")),
            10.0, Mm(10.0), Mm(y), &font,
        );
    }

    // ── Column header ────────────────────────────────────────────────────────
    y -= 10.0;
    let cols: &[(&str, f32)] = &[
        ("Rank",        10.0),
        ("Name",        30.0),
        ("Distance",    70.0),
        ("Address",     95.0),
        ("Phone",      140.0),
        ("Keyword%",   170.0),
        ("Visibility%",185.0),
    ];
    for &(hdr, x) in cols {
        layer.use_text(hdr, 8.0, Mm(x), Mm(y), &font_bold);
    }
    y -= 2.0;
    // Separator line
    layer.add_line(Line {
        points: vec![
            (Point::new(Mm(10.0), Mm(y)), false),
            (Point::new(Mm(200.0), Mm(y)), false),
        ],
        is_closed: false,
    });

    // ── Competitor rows ───────────────────────────────────────────────────────
    if run.competitors.is_empty() {
        y -= 6.0;
        layer.use_text("(no competitors found)", 9.0, Mm(10.0), Mm(y), &font);
    } else {
        for c in &run.competitors {
            y -= 6.0;
            if y < 15.0 {
                // Pagination not required for MVP; stop if out of space
                break;
            }
            let name    = field_val(&c.profile.name.value);
            let address = field_val(&c.profile.address.value);
            let phone   = field_val(&c.profile.phone.value);
            let dist    = format!("{:.2} km", c.distance_km);
            let kw      = format!("{}%", (c.keyword_score * 100.0).round() as u32);
            let vis     = format!("{}%", (c.visibility_score * 100.0).round() as u32);

            let row: &[(String, f32)] = &[
                (c.rank.to_string(), 10.0),
                (trunc(&name, 18),   30.0),
                (dist,               70.0),
                (trunc(&address, 20), 95.0),
                (trunc(&phone, 14),  140.0),
                (kw,                 170.0),
                (vis,                185.0),
            ];
            for (text, x) in row {
                layer.use_text(text.as_str(), 8.0, Mm(*x), Mm(y), &font);
            }
        }
    }

    // ── Footer: failed sources ────────────────────────────────────────────────
    let failed: Vec<_> = run
        .source_results
        .iter()
        .filter(|sr| sr.status.is_failed())
        .collect();

    if !failed.is_empty() {
        y -= 10.0;
        layer.use_text("Failed sources:", 9.0, Mm(10.0), Mm(y), &font_bold);
        for sr in failed {
            y -= 5.0;
            if y < 10.0 { break; }
            let reason = match &sr.status {
                AdapterResultStatus::Failed(code) => format!("{code}"),
                _ => String::new(),
            };
            layer.use_text(
                &format!("  - {} : {reason}", sr.adapter_id),
                8.0, Mm(10.0), Mm(y), &font,
            );
        }
    }

    doc
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn field_val(v: &Option<String>) -> String {
    v.clone().unwrap_or_else(|| "--".to_string())
}

fn trunc(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let end = s
            .char_indices()
            .nth(max - 1)
            .map(|(i, _)| i)
            .unwrap_or(s.len());
        format!("{}..", &s[..end])
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use competitor_spy_domain::{
        profile::{BusinessProfile, Competitor, Confidence, DataPoint},
        query::{Location, Radius, SearchQuery},
        run::{AdapterResultStatus, ReasonCode, SearchRun, SourceResult},
    };
    use uuid::Uuid;

    fn make_location() -> Location {
        Location::new(52.3676, 4.9041).unwrap()
    }

    fn make_run() -> SearchRun {
        let query =
            SearchQuery::new("yoga studio", "Amsterdam, Netherlands", Radius::Km10).unwrap();
        let ts = chrono::Utc.with_ymd_and_hms(2026, 3, 21, 14, 30, 22).unwrap();
        let mut run = SearchRun::new(query, ts);
        run.start_validating();
        run.start_geocoding();
        run.set_location(make_location());
        let failed = SourceResult {
            adapter_id: "yelp".to_string(),
            status: AdapterResultStatus::Failed(ReasonCode::Http4xx),
            records: vec![],
            retrieved_at: ts,
        };
        run.add_source_result(failed);
        run.start_ranking();
        let mut profile = BusinessProfile::empty();
        profile.name =
            DataPoint::present("name", "Zen Yoga Amsterdam", "test", Confidence::High);
        profile.address =
            DataPoint::present("address", "123 Test St, Amsterdam", "test", Confidence::High);
        let c = Competitor {
            id: Uuid::new_v4(),
            profile,
            location: make_location(),
            distance_km: 1.2,
            keyword_score: 0.85,
            visibility_score: 0.70,
            rank: 1,
        };
        run.set_competitors(vec![c]);
        run.complete(ts);
        run
    }

    #[test]
    fn pdf_filename_format_is_correct() {
        let run = make_run();
        let name = pdf_filename(&run);
        assert_eq!(name, "competitor_spy_report_20260321_143022_UTC.pdf");
    }

    #[test]
    fn render_produces_non_empty_bytes() {
        let run = make_run();
        let bytes = render_to_bytes(&run).expect("render failed");
        assert!(!bytes.is_empty(), "PDF bytes must not be empty");
    }

    #[test]
    fn render_produces_valid_pdf_header() {
        let run = make_run();
        let bytes = render_to_bytes(&run).expect("render failed");
        // Every PDF begins with the magic bytes %PDF-
        assert!(
            bytes.starts_with(b"%PDF-"),
            "output must start with %PDF-, got: {:?}",
            &bytes[..5.min(bytes.len())]
        );
    }

    #[test]
    fn render_bytes_exceed_500_bytes() {
        // A PDF with text content is always far larger than a trivial stub.
        let run = make_run();
        let bytes = render_to_bytes(&run).expect("render failed");
        assert!(
            bytes.len() > 500,
            "PDF should contain substantial content, got {} bytes",
            bytes.len()
        );
    }

    #[test]
    fn render_to_dir_creates_file_with_correct_name() {
        let run = make_run();
        let dir = std::env::temp_dir().join("cspy_test_pdf");
        std::fs::create_dir_all(&dir).unwrap();
        let path = render_to_dir(&run, &dir).expect("render failed");
        assert!(path.exists(), "file was not created at {path:?}");
        assert_eq!(
            path.file_name().and_then(|n| n.to_str()).unwrap(),
            "competitor_spy_report_20260321_143022_UTC.pdf"
        );
        let meta = std::fs::metadata(&path).unwrap();
        assert!(meta.len() > 0, "file must not be empty");
        // cleanup
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn render_empty_competitors_does_not_panic() {
        let query =
            SearchQuery::new("yoga studio", "Amsterdam, Netherlands", Radius::Km10).unwrap();
        let ts = chrono::Utc.with_ymd_and_hms(2026, 3, 21, 10, 0, 0).unwrap();
        let mut run = SearchRun::new(query, ts);
        run.start_validating();
        run.start_geocoding();
        run.set_location(make_location());
        run.start_ranking();
        run.set_competitors(vec![]);
        run.complete(ts);
        let bytes = render_to_bytes(&run).expect("render failed");
        assert!(bytes.starts_with(b"%PDF-"));
    }
}
