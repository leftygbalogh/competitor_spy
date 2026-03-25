// PDF report renderer — T-006 (V2 card layout, BC-006)
// Formats a finalised SearchRun to a PDF file; card layout mirrors terminal.
// §4.6, §6.2, §6.5, §9.2

use std::io::{self, BufWriter};
use std::path::Path;

use printpdf::*;

use competitor_spy_domain::enrichment::COVERAGE_THRESHOLD;
use competitor_spy_domain::run::{AdapterResultStatus, SearchRun};

// ── A4 dimensions ─────────────────────────────────────────────────────────────

const A4_WIDTH_MM:     f32 = 210.0;
const A4_HEIGHT_MM:    f32 = 297.0;
const TOP_MARGIN_MM:   f32 = 15.0;
const BOTTOM_MARGIN_MM: f32 = 15.0;
const LEFT_MARGIN_MM:  f32 = 10.0;
const FONT_SIZE:       f32 = 10.0;
const LINE_HEIGHT:     f32 = 5.5;
/// Approx chars fitting on one line: (210-10-10)mm / (10pt * 25.4/72 * 0.55 mm/char) ≈ 97 → 125 safe.
const MAX_CHARS_PER_LINE: usize = 125;
/// Continuation indent that aligns text after a `label_line()` prefix (17 chars: 15+": ").
const LABEL_INDENT: &str = "                 ";

// ── Pagination macro ─────────────────────────────────────────────────────────

macro_rules! ensure_space {
    ($needed:expr, $y:expr, $layer:expr, $doc:expr, $font:expr, $font_bold:expr) => {
        if $y - $needed < BOTTOM_MARGIN_MM {
            let (new_layer, _page_idx, new_y) = new_page(&$doc, &$font, &$font_bold);
            $layer = new_layer;
            $y = new_y;
        }
    };
}

// ── Filename format ───────────────────────────────────────────────────────────

/// Slugify a string: lowercase, keep only alphanumeric, cap at `max_len` chars.
fn slugify(s: &str, max_len: usize) -> String {
    s.chars()
        .filter(|c| c.is_alphanumeric())
        .map(|c| c.to_ascii_lowercase())
        .take(max_len)
        .collect()
}

/// Generate the PDF filename embedding query parameters and timestamp.
/// Format: `{industry}_{location}_{radius}km_{YYYYMMDD}_{HHMM}.pdf`
/// Example: `pilates_stpoelten_10km_20260324_1746.pdf`
pub fn pdf_filename(run: &SearchRun) -> String {
    let industry = slugify(&run.query.industry, 10);
    let location = slugify(&run.query.location_input, 10);
    let radius   = run.query.radius.km_value();
    let ts = run.completed_at.unwrap_or(run.started_at);
    let stamp = ts.format("%Y%m%d_%H%M").to_string();
    format!("{industry}_{location}_{radius}km_{stamp}.pdf")
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Render `run` to a PDF file inside `output_dir`.
///
/// Returns the full path of the file written on success.
/// Returns `Err` on I/O failure; caller should downgrade to warning per spec §6.2.
pub fn render_to_dir(run: &SearchRun, detail: bool, output_dir: &Path) -> io::Result<std::path::PathBuf> {
    let filename = pdf_filename(run);
    let path = output_dir.join(&filename);
    let file = std::fs::File::create(&path)?;
    let writer = BufWriter::new(file);
    render_to_writer(run, detail, writer)?;
    Ok(path)
}

/// Render `run` to an arbitrary writer (useful for testing).
pub fn render_to_writer<W: io::Write + io::Seek>(
    run: &SearchRun,
    detail: bool,
    writer: W,
) -> io::Result<()> {
    let doc = build_document(run, detail);
    doc.save(&mut std::io::BufWriter::new(writer))
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))
}

/// Render to a `Vec<u8>` in-memory. Useful for tests.
pub fn render_to_bytes(run: &SearchRun, detail: bool) -> io::Result<Vec<u8>> {
    use std::io::Cursor;
    let mut buf = Cursor::new(Vec::new());
    render_to_writer(run, detail, &mut buf)?;
    Ok(buf.into_inner())
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn new_page(
    doc: &PdfDocumentReference,
    _font: &IndirectFontRef,
    _font_bold: &IndirectFontRef,
) -> (PdfLayerReference, PdfPageIndex, f32) {
    let (page_idx, layer_idx) = doc.add_page(Mm(A4_WIDTH_MM), Mm(A4_HEIGHT_MM), "Main");
    let layer = doc.get_page(page_idx).get_layer(layer_idx);
    (layer, page_idx, A4_HEIGHT_MM - TOP_MARGIN_MM)
}

fn label_line(label: &str, value: &str) -> String {
    format!("{:<15}: {}", label, value)
}

/// Word-wrap `text` at word boundaries so each line is at most `max_chars` long.
/// Lines after the first are prefixed with `continuation_indent`.
fn word_wrap(text: &str, max_chars: usize, continuation_indent: &str) -> Vec<String> {
    let cont_len = continuation_indent.len();
    let mut lines: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut is_first = true;

    for word in text.split_whitespace() {
        let budget = if is_first {
            max_chars
        } else {
            max_chars.saturating_sub(cont_len)
        };
        let new_len = if current.is_empty() {
            word.len()
        } else {
            current.len() + 1 + word.len()
        };
        if !current.is_empty() && new_len > budget {
            lines.push(if is_first {
                current.clone()
            } else {
                format!("{}{}", continuation_indent, &current)
            });
            current = word.to_string();
            is_first = false;
        } else {
            if !current.is_empty() {
                current.push(' ');
            }
            current.push_str(word);
        }
    }
    if !current.is_empty() || lines.is_empty() {
        lines.push(if is_first {
            current
        } else {
            format!("{}{}", continuation_indent, &current)
        });
    }
    lines
}

/// Draw a horizontal rule at `y` to represent the card separator.
fn draw_sep(layer: &PdfLayerReference, y: f32) {
    layer.add_line(Line {
        points: vec![
            (Point::new(Mm(LEFT_MARGIN_MM), Mm(y)), false),
            (Point::new(Mm(200.0), Mm(y)), false),
        ],
        is_closed: false,
    });
}

/// Approximate mm x-offset for URL value given char count of prefix and font pt size.
fn url_offset(char_count: usize, font_size: f32) -> f32 {
    char_count as f32 * font_size * (25.4 / 72.0) * 0.55
}

/// Estimate renderable line count for a competitor card (used for space check).
fn count_present_fields(c: &competitor_spy_domain::profile::Competitor, detail: bool, enrichment: Option<&competitor_spy_domain::enrichment::WebEnrichment>) -> usize {
    let mut n = 0;
    if c.profile.address.value.is_some()          { n += 1; }
    if c.profile.phone.value.is_some()             { n += 1; }
    if c.profile.website.value.is_some()           { n += 1; }
    if c.profile.categories.value.is_some()        { n += 1; }
    if let Some(v) = &c.profile.opening_hours.value { n += v.split('\n').count(); }
    if c.profile.price_level.value.is_some()       { n += 1; }
    if c.profile.editorial_summary.value.is_some() { n += 1; }
    if detail { n += c.profile.reviews.len(); }
    if let Some(e) = enrichment {
        if e.fetch_status.is_success() {
            if e.pricing.is_some()           { n += 1; }
            if e.lesson_types.is_some()      { n += 1; }
            if e.schedule.is_some()          { n += 1; }
            if let Some(items) = &e.testimonials     { n += 1; if detail { n += items.len(); } }
            if let Some(items) = &e.class_descriptions { n += 1; if detail { n += items.len(); } }
        }
    }
    n
}

// ── Document builder ──────────────────────────────────────────────────────────

fn build_document(run: &SearchRun, detail: bool) -> PdfDocumentReference {
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

    let mut layer = doc.get_page(page1).get_layer(layer1);

    // ── Header ──────────────────────────────────────────────────────────────
    let mut y = A4_HEIGHT_MM - TOP_MARGIN_MM;
    layer.use_text("Competitor Spy Report", 16.0, Mm(LEFT_MARGIN_MM), Mm(y), &font_bold);
    y -= 8.0;
    layer.use_text(
        &format!("Industry : {}", run.query.industry),
        FONT_SIZE, Mm(LEFT_MARGIN_MM), Mm(y), &font,
    );
    y -= LINE_HEIGHT;
    layer.use_text(
        &format!("Location : {}", run.query.location_input),
        FONT_SIZE, Mm(LEFT_MARGIN_MM), Mm(y), &font,
    );
    y -= LINE_HEIGHT;
    layer.use_text(
        &format!("Radius   : {} km", run.query.radius.km_value()),
        FONT_SIZE, Mm(LEFT_MARGIN_MM), Mm(y), &font,
    );
    if let Some(ts) = run.completed_at {
        y -= LINE_HEIGHT;
        layer.use_text(
            &format!("Run UTC  : {}", ts.format("%Y-%m-%d %H:%M:%S UTC")),
            FONT_SIZE, Mm(LEFT_MARGIN_MM), Mm(y), &font,
        );
    }
    y -= LINE_HEIGHT;

    // ── Competitor cards ─────────────────────────────────────────────────────
    if run.competitors.is_empty() {
        ensure_space!(3.0 * LINE_HEIGHT, y, layer, doc, font, font_bold);
        y -= LINE_HEIGHT;
        draw_sep(&layer, y);
        y -= LINE_HEIGHT;
        layer.use_text("(no competitors found)", FONT_SIZE, Mm(LEFT_MARGIN_MM), Mm(y), &font);
        y -= LINE_HEIGHT;
        draw_sep(&layer, y);
    } else {
        for c in &run.competitors {
            let enrich = run.enrichments.iter().find(|e| e.competitor_id == c.id);
            let field_count = count_present_fields(c, detail, enrich);
            let card_height = (3 + field_count) as f32 * LINE_HEIGHT;
            ensure_space!(card_height.max(20.0), y, layer, doc, font, font_bold);

            // Sep 1
            y -= LINE_HEIGHT;
            draw_sep(&layer, y);

            // Rank / name / rating header
            y -= LINE_HEIGHT;
            let name = c.profile.name.value.as_deref().unwrap_or("(unknown)");
            let rating_part = match (
                c.profile.rating_text.value.as_deref(),
                c.profile.review_count_text.value.as_deref(),
            ) {
                (Some(r), Some(cnt)) => format!(" | {}\u{2605} ({})", r, cnt),
                (Some(r), None)      => format!(" | {}\u{2605}", r),
                _                    => String::new(),
            };
            let header = format!("#{rank}  {name}{rating_part}", rank = c.rank);
            layer.use_text(&header, FONT_SIZE, Mm(LEFT_MARGIN_MM), Mm(y), &font_bold);

            // Sep 2
            y -= LINE_HEIGHT;
            draw_sep(&layer, y);

            // Fields — silent omission when absent
            if let Some(v) = &c.profile.address.value {
                for line in word_wrap(&label_line("Address", v), MAX_CHARS_PER_LINE, LABEL_INDENT) {
                    ensure_space!(LINE_HEIGHT, y, layer, doc, font, font_bold);
                    y -= LINE_HEIGHT;
                    layer.use_text(&line, FONT_SIZE, Mm(LEFT_MARGIN_MM), Mm(y), &font);
                }
            }
            if let Some(v) = &c.profile.phone.value {
                for line in word_wrap(&label_line("Phone", v), MAX_CHARS_PER_LINE, LABEL_INDENT) {
                    ensure_space!(LINE_HEIGHT, y, layer, doc, font, font_bold);
                    y -= LINE_HEIGHT;
                    layer.use_text(&line, FONT_SIZE, Mm(LEFT_MARGIN_MM), Mm(y), &font);
                }
            }
            if let Some(v) = &c.profile.website.value {
                ensure_space!(LINE_HEIGHT, y, layer, doc, font, font_bold);
                y -= LINE_HEIGHT;
                let prefix = format!("{:<15}: ", "Website");
                layer.use_text(&prefix, FONT_SIZE, Mm(LEFT_MARGIN_MM), Mm(y), &font);
                let url_x = LEFT_MARGIN_MM + url_offset(prefix.len(), FONT_SIZE);
                layer.set_fill_color(Color::Rgb(Rgb { r: 0.0, g: 0.0, b: 0.8, icc_profile: None }));
                layer.use_text(v, FONT_SIZE, Mm(url_x), Mm(y), &font);
                layer.set_fill_color(Color::Rgb(Rgb { r: 0.0, g: 0.0, b: 0.0, icc_profile: None }));
            }
            if let Some(v) = &c.profile.categories.value {
                for line in word_wrap(&label_line("Categories", v), MAX_CHARS_PER_LINE, LABEL_INDENT) {
                    ensure_space!(LINE_HEIGHT, y, layer, doc, font, font_bold);
                    y -= LINE_HEIGHT;
                    layer.use_text(&line, FONT_SIZE, Mm(LEFT_MARGIN_MM), Mm(y), &font);
                }
            }
            if let Some(v) = &c.profile.opening_hours.value {
                for (i, oh_line) in v.split('\n').enumerate() {
                    let text = if i == 0 {
                        label_line("Opening Hours", oh_line)
                    } else {
                        format!("{LABEL_INDENT}{oh_line}")
                    };
                    for wrapped in word_wrap(&text, MAX_CHARS_PER_LINE, LABEL_INDENT) {
                        ensure_space!(LINE_HEIGHT, y, layer, doc, font, font_bold);
                        y -= LINE_HEIGHT;
                        layer.use_text(&wrapped, FONT_SIZE, Mm(LEFT_MARGIN_MM), Mm(y), &font);
                    }
                }
            }
            if let Some(v) = &c.profile.price_level.value {
                for line in word_wrap(&label_line("Price Level", v), MAX_CHARS_PER_LINE, LABEL_INDENT) {
                    ensure_space!(LINE_HEIGHT, y, layer, doc, font, font_bold);
                    y -= LINE_HEIGHT;
                    layer.use_text(&line, FONT_SIZE, Mm(LEFT_MARGIN_MM), Mm(y), &font);
                }
            }
            if let Some(v) = &c.profile.editorial_summary.value {
                for line in word_wrap(&label_line("Editorial", v), MAX_CHARS_PER_LINE, LABEL_INDENT) {
                    ensure_space!(LINE_HEIGHT, y, layer, doc, font, font_bold);
                    y -= LINE_HEIGHT;
                    layer.use_text(&line, FONT_SIZE, Mm(LEFT_MARGIN_MM), Mm(y), &font);
                }
            }
            if detail {
                for (i, review) in c.profile.reviews.iter().enumerate() {
                    let header = format!(
                        "Review {} ({}\u{2605}, {}): ",
                        i + 1,
                        review.rating,
                        review.relative_time,
                    );
                    let review_indent = " ".repeat(header.len().min(MAX_CHARS_PER_LINE));
                    let full_text = format!("{}{}", header, review.text);
                    for line in word_wrap(&full_text, MAX_CHARS_PER_LINE, &review_indent) {
                        ensure_space!(LINE_HEIGHT, y, layer, doc, font, font_bold);
                        y -= LINE_HEIGHT;
                        layer.use_text(&line, FONT_SIZE, Mm(LEFT_MARGIN_MM), Mm(y), &font);
                    }
                }
            }
            // ── V3 website enrichment fields ──────────────────────────────────
            if let Some(e) = enrich {
                if e.fetch_status.is_success() {
                    if let Some(v) = &e.pricing {
                        for line in word_wrap(&label_line("Pricing", v), MAX_CHARS_PER_LINE, LABEL_INDENT) {
                            ensure_space!(LINE_HEIGHT, y, layer, doc, font, font_bold);
                            y -= LINE_HEIGHT;
                            layer.use_text(&line, FONT_SIZE, Mm(LEFT_MARGIN_MM), Mm(y), &font);
                        }
                    }
                    if let Some(types) = &e.lesson_types {
                        for line in word_wrap(&label_line("Lesson Types", &types.join(", ")), MAX_CHARS_PER_LINE, LABEL_INDENT) {
                            ensure_space!(LINE_HEIGHT, y, layer, doc, font, font_bold);
                            y -= LINE_HEIGHT;
                            layer.use_text(&line, FONT_SIZE, Mm(LEFT_MARGIN_MM), Mm(y), &font);
                        }
                    }
                    if let Some(v) = &e.schedule {
                        for line in word_wrap(&label_line("Schedule", v), MAX_CHARS_PER_LINE, LABEL_INDENT) {
                            ensure_space!(LINE_HEIGHT, y, layer, doc, font, font_bold);
                            y -= LINE_HEIGHT;
                            layer.use_text(&line, FONT_SIZE, Mm(LEFT_MARGIN_MM), Mm(y), &font);
                        }
                    }
                    if let Some(items) = &e.testimonials {
                        ensure_space!(LINE_HEIGHT, y, layer, doc, font, font_bold);
                        y -= LINE_HEIGHT;
                        layer.use_text(&label_line("Testimonials", &format!("{} found", items.len())), FONT_SIZE, Mm(LEFT_MARGIN_MM), Mm(y), &font);
                        if detail {
                            for t in items {
                                let content = format!("\"{}\"", t);
                                for (idx, wline) in word_wrap(&content, MAX_CHARS_PER_LINE - 2, "").into_iter().enumerate() {
                                    let line = if idx == 0 { format!("  {}", wline) } else { format!("   {}", wline) };
                                    ensure_space!(LINE_HEIGHT, y, layer, doc, font, font_bold);
                                    y -= LINE_HEIGHT;
                                    layer.use_text(&line, FONT_SIZE, Mm(LEFT_MARGIN_MM), Mm(y), &font);
                                }
                            }
                        }
                    }
                    if let Some(items) = &e.class_descriptions {
                        ensure_space!(LINE_HEIGHT, y, layer, doc, font, font_bold);
                        y -= LINE_HEIGHT;
                        layer.use_text(&label_line("Class Descs", &format!("{} found", items.len())), FONT_SIZE, Mm(LEFT_MARGIN_MM), Mm(y), &font);
                        if detail {
                            for d in items {
                                for (idx, wline) in word_wrap(d, MAX_CHARS_PER_LINE - 2, "").into_iter().enumerate() {
                                    let line = if idx == 0 { format!("  {}", wline) } else { format!("   {}", wline) };
                                    ensure_space!(LINE_HEIGHT, y, layer, doc, font, font_bold);
                                    y -= LINE_HEIGHT;
                                    layer.use_text(&line, FONT_SIZE, Mm(LEFT_MARGIN_MM), Mm(y), &font);
                                }
                            }
                        }
                    }
                }
            }
        }
        // Final separator
        ensure_space!(LINE_HEIGHT, y, layer, doc, font, font_bold);
        y -= LINE_HEIGHT;
        draw_sep(&layer, y);
    }

    // ── Footer: failed sources ────────────────────────────────────────────────
    let failed: Vec<_> = run
        .source_results
        .iter()
        .filter(|sr| sr.status.is_failed())
        .collect();

    if !failed.is_empty() {
        ensure_space!(2.0 * LINE_HEIGHT, y, layer, doc, font, font_bold);
        y -= LINE_HEIGHT;
        layer.use_text("Failed sources:", FONT_SIZE, Mm(LEFT_MARGIN_MM), Mm(y), &font_bold);
        for sr in &failed {
            ensure_space!(LINE_HEIGHT, y, layer, doc, font, font_bold);
            y -= LINE_HEIGHT;
            let reason = match &sr.status {
                AdapterResultStatus::Failed(code) => format!("{code}"),
                _ => String::new(),
            };
            layer.use_text(
                &format!("  - {} : {reason}", sr.adapter_id),
                FONT_SIZE, Mm(LEFT_MARGIN_MM), Mm(y), &font,
            );
        }
    }

    // ── Footer: V3 enrichment coverage ───────────────────────────────────────
    if !run.enrichments.is_empty() {
        ensure_space!(2.0 * LINE_HEIGHT, y, layer, doc, font, font_bold);
        y -= LINE_HEIGHT;
        let n = run.enrichments.len();
        let with_data = (run.enrichment_coverage * n as f64).round() as usize;
        layer.use_text(
            &format!(
                "Enrichment: {with_data}/{n} studios ({:.0}%) had at least one extractable field.",
                run.enrichment_coverage * 100.0,
            ),
            FONT_SIZE, Mm(LEFT_MARGIN_MM), Mm(y), &font,
        );
        if run.enrichment_coverage < COVERAGE_THRESHOLD {
            ensure_space!(LINE_HEIGHT, y, layer, doc, font, font_bold);
            y -= LINE_HEIGHT;
            layer.use_text(
                "  Warning: enrichment coverage is below the 60% threshold.",
                FONT_SIZE, Mm(LEFT_MARGIN_MM), Mm(y), &font,
            );
        }
    }
    let _ = y; // suppress unused warning

    doc
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
        // industry="yoga studio" → yogastudio (10), location="Amsterdam, Netherlands" → amsterdamn (10 chars), radius=10km
        assert_eq!(name, "yogastudio_amsterdamn_10km_20260321_1430.pdf");
    }

    #[test]
    fn render_produces_non_empty_bytes() {
        let run = make_run();
        let bytes = render_to_bytes(&run, false).expect("render failed");
        assert!(!bytes.is_empty(), "PDF bytes must not be empty");
    }

    #[test]
    fn render_produces_valid_pdf_header() {
        let run = make_run();
        let bytes = render_to_bytes(&run, false).expect("render failed");
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
        let bytes = render_to_bytes(&run, false).expect("render failed");
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
        let path = render_to_dir(&run, false, &dir).expect("render failed");
        assert!(path.exists(), "file was not created at {path:?}");
        assert_eq!(
            path.file_name().and_then(|n| n.to_str()).unwrap(),
            "yogastudio_amsterdamn_10km_20260321_1430.pdf"
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
        let bytes = render_to_bytes(&run, false).expect("render failed");
        assert!(bytes.starts_with(b"%PDF-"));
    }

    // T-OUT-006: detail=true produces more bytes than detail=false
    #[test]
    fn t_out_006_detail_true_produces_more_bytes() {
        let query =
            SearchQuery::new("yoga studio", "Amsterdam, Netherlands", Radius::Km10).unwrap();
        let ts = chrono::Utc.with_ymd_and_hms(2026, 3, 21, 14, 30, 22).unwrap();
        let mut run = SearchRun::new(query, ts);
        run.start_validating();
        run.start_geocoding();
        run.set_location(make_location());
        run.start_ranking();
        let mut profile = BusinessProfile::empty();
        profile.name = DataPoint::present("name", "Zen Yoga Amsterdam", "test", Confidence::High);
        profile.reviews = vec![
            PlaceReview {
                text: "Amazing classes!".into(),
                rating: 5,
                relative_time: "1 week ago".into(),
            },
            PlaceReview {
                text: "Highly recommend.".into(),
                rating: 4,
                relative_time: "2 weeks ago".into(),
            },
        ];
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
        let bytes_no_detail = render_to_bytes(&run, false).expect("render failed");
        let bytes_detail = render_to_bytes(&run, true).expect("render failed");
        assert!(
            bytes_detail.len() > bytes_no_detail.len(),
            "detail=true ({} bytes) must exceed detail=false ({} bytes)",
            bytes_detail.len(),
            bytes_no_detail.len()
        );
    }

    // T-OUT-007: already covered by render_produces_valid_pdf_header

    // T-OUT-008: 20-competitor run produces > 5000 bytes
    #[test]
    fn t_out_008_twenty_competitors_exceeds_5000_bytes() {
        let query =
            SearchQuery::new("yoga studio", "Amsterdam, Netherlands", Radius::Km10).unwrap();
        let ts = chrono::Utc.with_ymd_and_hms(2026, 3, 21, 14, 30, 22).unwrap();
        let mut run = SearchRun::new(query, ts);
        run.start_validating();
        run.start_geocoding();
        run.set_location(make_location());
        run.start_ranking();
        let competitors: Vec<_> = (1u32..=20)
            .map(|i| {
                let mut profile = BusinessProfile::empty();
                profile.name = DataPoint::present(
                    "name",
                    &format!("Studio #{i}"),
                    "test",
                    Confidence::High,
                );
                profile.address = DataPoint::present(
                    "address",
                    "123 Test St, Amsterdam",
                    "test",
                    Confidence::High,
                );
                Competitor {
                    id: Uuid::new_v4(),
                    profile,
                    location: make_location(),
                    distance_km: i as f64 * 0.5,
                    keyword_score: 0.5,
                    visibility_score: 0.5,
                    rank: i,
                }
            })
            .collect();
        run.set_competitors(competitors);
        run.complete(ts);
        let bytes = render_to_bytes(&run, false).expect("render failed");
        assert!(
            bytes.len() > 5000,
            "20-competitor PDF must exceed 5000 bytes, got {}",
            bytes.len()
        );
    }

    /// Visual wrap test — run with:
    ///   cargo test -p competitor_spy_output -- wrap_visual_test --ignored --nocapture
    /// Then open reports/wrap_visual_test.pdf to verify wrapping.
    #[test]
    #[ignore]
    fn wrap_visual_test() {
        let query =
            SearchQuery::new("Pilates", "Vienna, Austria", Radius::Km25).unwrap();
        let ts = chrono::Utc.with_ymd_and_hms(2026, 3, 25, 10, 0, 0).unwrap();
        let mut run = SearchRun::new(query, ts);
        run.start_validating();
        run.start_geocoding();
        run.set_location(Location::new(48.2082, 16.3738).unwrap());
        run.start_ranking();

        let long_review = "I've been taking weekly private Pilates sessions since 2019, \
            and for the last three years as a targeted complement to my physiotherapy. \
            Dora possesses extensive knowledge of both the Pilates method and the human body \
            in general. She is an excellent instructor who tailors each session to individual \
            needs. I highly recommend her studio to anyone looking for professional Pilates \
            instruction in a welcoming environment.";

        let long_categories = "yoga_studio, sports_complex, gym, sports_school, health, \
            sports_activity_location, fitness_center, point_of_interest, establishment";

        let mut profile = BusinessProfile::empty();
        profile.name = DataPoint::present("name", "Wrap Test Studio", "test", Confidence::High);
        profile.address = DataPoint::present("address", "Habsburgergasse 1a, 1010 Wien, Austria", "test", Confidence::High);
        profile.phone = DataPoint::present("phone", "+43 1 5322200", "test", Confidence::High);
        profile.categories = DataPoint::present("categories", long_categories, "test", Confidence::High);
        profile.reviews = vec![
            PlaceReview {
                text: long_review.into(),
                rating: 5,
                relative_time: "8 months ago".into(),
            },
            PlaceReview {
                text: "Great studio! Very professional and dedicated trainers. \
                    State-of-the-art equipment. I highly recommend it to everyone looking \
                    for a quality Pilates experience in Vienna.".into(),
                rating: 5,
                relative_time: "10 months ago".into(),
            },
        ];
        profile.editorial_summary = DataPoint::present(
            "editorial_summary",
            "A boutique Pilates studio offering private and group sessions in the heart of Vienna, \
             specialising in classical Pilates and physiotherapy-complementary movement.",
            "test",
            Confidence::High,
        );
        let c = Competitor {
            id: Uuid::new_v4(),
            profile,
            location: Location::new(48.2082, 16.3738).unwrap(),
            distance_km: 0.3,
            keyword_score: 0.90,
            visibility_score: 0.85,
            rank: 1,
        };
        run.set_competitors(vec![c]);
        run.complete(ts);

        let out_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent().unwrap().parent().unwrap()
            .join("reports");
        std::fs::create_dir_all(&out_dir).unwrap();
        let path = out_dir.join("wrap_visual_test.pdf");
        let file = std::fs::File::create(&path).unwrap();
        render_to_writer(&run, true, std::io::BufWriter::new(file)).expect("render failed");
        println!("PDF written to: {}", path.display());
    }
}
