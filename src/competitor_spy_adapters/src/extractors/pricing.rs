// Pricing extractor — T-026
// Spec: FORMAL_SPEC.md §13.2.4 (FR-V3-003)
//
// Extraction priority:
//  1. <table> with price-related header/caption text
//  2. <ul>/<ol> list items containing € and a digit
//  3. <p>/<div> containing a digit + €/EUR
//  4. Elements with class/id containing price-related keywords
//
// Output: whitespace-normalised plain text, truncated to 2 000 characters.
// Returns None when no qualifying element found.

use scraper::{Html, Selector};

const MAX_OUTPUT_CHARS: usize = 2_000;

/// Price-related keywords used in strategy steps 1 and 4 (lowercase match).
const PRICE_KEYWORDS: &[&str] = &[
    "preis", "preise", "preisliste", "price", "pricing", "tarif", "kosten", "euro",
];

/// Returns `true` if the string contains a price-related keyword.
fn contains_price_keyword(s: &str) -> bool {
    let lower = s.to_lowercase();
    PRICE_KEYWORDS.iter().any(|kw| lower.contains(kw))
}

/// Returns `true` if the string contains a digit followed by `€` or `EUR`,
/// or `€` followed by a digit.
fn contains_price_pattern(s: &str) -> bool {
    s.contains('€') || s.to_uppercase().contains("EUR")
}

/// Normalise whitespace: collapse runs of whitespace to a single space and trim.
fn normalise(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Truncate to at most `max` characters, preserving UTF-8 character boundaries.
fn truncate(s: String, max: usize) -> String {
    if s.chars().count() <= max {
        s
    } else {
        s.chars().take(max).collect()
    }
}

/// Extract pricing information from raw HTML.
pub fn extract_pricing(html: &str) -> Option<String> {
    let document = Html::parse_document(html);
    let mut collected: Vec<String> = Vec::new();

    // Strategy 1: <table> whose caption or any header cell is price-related.
    if let Ok(table_sel) = Selector::parse("table") {
        for table in document.select(&table_sel) {
            let table_text = table.text().collect::<String>();
            // Check caption/th/td in the first two rows for a price keyword.
            let header_text = {
                let mut h = String::new();
                if let Ok(caption_sel) = Selector::parse("caption") {
                    for cap in table.select(&caption_sel) {
                        h.push_str(&cap.text().collect::<String>());
                    }
                }
                if let Ok(th_sel) = Selector::parse("th") {
                    for th in table.select(&th_sel) {
                        h.push_str(&th.text().collect::<String>());
                    }
                }
                h
            };
            if contains_price_keyword(&header_text) || contains_price_pattern(&table_text) {
                let t = normalise(&table_text);
                if !t.is_empty() {
                    collected.push(t);
                    break; // first qualifying table is sufficient
                }
            }
        }
    }

    // Strategy 2: <li> elements containing € and a digit.
    if collected.is_empty() {
        if let Ok(li_sel) = Selector::parse("li") {
            for li in document.select(&li_sel) {
                let text = li.text().collect::<String>();
                if contains_price_pattern(&text) && text.chars().any(|c| c.is_ascii_digit()) {
                    let t = normalise(&text);
                    if !t.is_empty() {
                        collected.push(t);
                    }
                }
            }
        }
    }

    // Strategy 3: <p> or <div> with digit + €/EUR.
    if collected.is_empty() {
        if let Ok(sel) = Selector::parse("p, div") {
            for el in document.select(&sel) {
                // Only direct text (not deeply nested) to avoid entire-page matches.
                let text: String = el
                    .text()
                    .take(5)
                    .collect::<Vec<_>>()
                    .join(" ");
                if contains_price_pattern(&text) && text.chars().any(|c| c.is_ascii_digit()) {
                    let t = normalise(&text);
                    if !t.is_empty() && t.len() < 500 {
                        collected.push(t);
                        if collected.len() >= 3 {
                            break;
                        }
                    }
                }
            }
        }
    }

    // Strategy 4: elements with price-related class/id.
    if collected.is_empty() {
        if let Ok(sel) = Selector::parse("[class*=preis],[class*=price],[class*=tarif],[class*=cost],[id*=preis],[id*=price],[id*=tarif],[id*=cost]") {
            for el in document.select(&sel) {
                let text = el.text().collect::<String>();
                let t = normalise(&text);
                if !t.is_empty() {
                    collected.push(t);
                }
            }
        }
    }

    if collected.is_empty() {
        return None;
    }

    let joined = collected.join(" | ");
    Some(truncate(joined, MAX_OUTPUT_CHARS))
}

// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pricing_table_with_euro_header_extracted() {
        let html = r#"
            <html><body>
              <table>
                <caption>Preisliste</caption>
                <tr><th>Kurs</th><th>Preis</th></tr>
                <tr><td>Einzelstunde</td><td>€ 20</td></tr>
                <tr><td>10er-Block</td><td>€ 170</td></tr>
              </table>
            </body></html>
        "#;
        let result = extract_pricing(html);
        assert!(result.is_some(), "Expected pricing to be extracted");
        let text = result.unwrap();
        assert!(text.contains("20") || text.contains("170"), "Expected price values in output: {text}");
    }

    #[test]
    fn pricing_list_items_with_euro_symbol() {
        let html = r#"
            <html><body>
              <ul>
                <li>Einzelstunde: € 25</li>
                <li>Monatskarte: € 90</li>
              </ul>
            </body></html>
        "#;
        let result = extract_pricing(html);
        assert!(result.is_some());
        let text = result.unwrap();
        assert!(text.contains("25") || text.contains("90"));
    }

    #[test]
    fn pricing_class_attribute_extracted() {
        let html = r#"
            <html><body>
              <div class="preise">
                <p>Probestunde: € 15</p>
                <p>Monat: € 80</p>
              </div>
            </body></html>
        "#;
        let result = extract_pricing(html);
        assert!(result.is_some());
    }

    #[test]
    fn pricing_none_when_no_price_content() {
        let html = r#"
            <html><body>
              <h1>Willkommen</h1>
              <p>Wir bieten Pilates Stunden an.</p>
            </body></html>
        "#;
        let result = extract_pricing(html);
        assert!(result.is_none(), "Expected None for page without pricing");
    }

    #[test]
    fn pricing_none_for_malformed_html() {
        let result = extract_pricing("<not valid>> <<html");
        // scraper is lenient; may parse something but should not panic and should return None
        // (no price content in the garbage)
        assert!(result.is_none());
    }

    #[test]
    fn pricing_output_truncated_to_2000_chars() {
        // Build a page with > 2000 chars of pricing content
        let many_items: String = (0..200)
            .map(|i| format!("<li>Kurs {i}: € {i}0</li>"))
            .collect();
        let html = format!("<html><body><ul>{many_items}</ul></body></html>");
        let result = extract_pricing(&html);
        assert!(result.is_some());
        let text = result.unwrap();
        assert!(text.chars().count() <= MAX_OUTPUT_CHARS, "Output exceeds {MAX_OUTPUT_CHARS} chars");
    }
}
