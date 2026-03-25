// Testimonials extractor — T-029
// Spec: FORMAL_SPEC.md §13.2.7 (FR-V3-006)
//
// Extraction priority:
//  1. <blockquote> elements
//  2. Elements with class/id: testimonial, review, bewertung, kundenstimme, erfahrung
//  3. <p> elements beginning/ending with quote characters (", „, ") and > 40 chars
//
// Output: Vec<String>, max 10 items, each trimmed and truncated to 500 chars.
// Returns None when no qualifying element found.

use scraper::{Html, Selector};

const MAX_ITEMS: usize = 10;
const MAX_ITEM_CHARS: usize = 500;
const MIN_QUOTE_LENGTH: usize = 40;

fn truncate_item(s: &str) -> String {
    let trimmed = s.trim();
    if trimmed.chars().count() <= MAX_ITEM_CHARS {
        trimmed.to_string()
    } else {
        trimmed.chars().take(MAX_ITEM_CHARS).collect()
    }
}

fn normalise(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn starts_or_ends_with_quote(s: &str) -> bool {
    let trimmed = s.trim();
    let start = trimmed.starts_with('"')
        || trimmed.starts_with('\u{201E}')  // „
        || trimmed.starts_with('\u{201C}'); // "
    let end = trimmed.ends_with('"')
        || trimmed.ends_with('\u{201C}')    // "
        || trimmed.ends_with('\u{201D}');   // "
    start || end
}

/// Extract testimonials from raw HTML.
pub fn extract_testimonials(html: &str) -> Option<Vec<String>> {
    let document = Html::parse_document(html);
    let mut results: Vec<String> = Vec::new();

    // Strategy 1: <blockquote> elements.
    if let Ok(sel) = Selector::parse("blockquote") {
        for el in document.select(&sel) {
            let text = normalise(&el.text().collect::<String>());
            if !text.is_empty() {
                results.push(truncate_item(&text));
                if results.len() >= MAX_ITEMS {
                    return Some(results);
                }
            }
        }
    }

    // Strategy 2: class/id with testimonial-related keywords.
    if results.len() < MAX_ITEMS {
        let sel_str = "[class*=testimonial],[class*=review],[class*=bewertung],\
                       [class*=kundenstimme],[class*=erfahrung],\
                       [id*=testimonial],[id*=review],[id*=bewertung],\
                       [id*=kundenstimme],[id*=erfahrung]";
        if let Ok(sel) = Selector::parse(sel_str) {
            for el in document.select(&sel) {
                let text = normalise(&el.text().collect::<String>());
                if !text.is_empty() && !results.contains(&truncate_item(&text)) {
                    results.push(truncate_item(&text));
                    if results.len() >= MAX_ITEMS {
                        return Some(results);
                    }
                }
            }
        }
    }

    // Strategy 3: <p> with quote characters and minimum length.
    if results.len() < MAX_ITEMS {
        if let Ok(sel) = Selector::parse("p") {
            for el in document.select(&sel) {
                let text = normalise(&el.text().collect::<String>());
                if text.chars().count() >= MIN_QUOTE_LENGTH
                    && starts_or_ends_with_quote(&text)
                    && !results.contains(&truncate_item(&text))
                {
                    results.push(truncate_item(&text));
                    if results.len() >= MAX_ITEMS {
                        return Some(results);
                    }
                }
            }
        }
    }

    if results.is_empty() {
        None
    } else {
        Some(results)
    }
}

// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blockquote_elements_extracted() {
        let html = r#"
            <html><body>
              <blockquote>Ich bin seit Jahren begeisterte Teilnehmerin und kann den Kurs nur empfehlen.</blockquote>
              <blockquote>Der Unterricht ist professionell und macht großen Spaß!</blockquote>
            </body></html>
        "#;
        let result = extract_testimonials(html);
        assert!(result.is_some());
        let items = result.unwrap();
        assert_eq!(items.len(), 2);
        assert!(items[0].contains("begeisterte"));
    }

    #[test]
    fn testimonial_class_extracted() {
        let html = r#"
            <html><body>
              <div class="testimonial">Toller Kurs, sehr empfehlenswert!</div>
              <div class="kundenstimme">Ich komme gerne wieder!</div>
            </body></html>
        "#;
        let result = extract_testimonials(html);
        assert!(result.is_some());
        let items = result.unwrap();
        assert!(items.iter().any(|i| i.contains("empfehlenswert")), "{items:?}");
    }

    #[test]
    fn quoted_paragraph_extracted() {
        let html = r#"
            <html><body>
              <p>„Diese Stunden haben mein Leben verändert, ich fühle mich viel besser!"</p>
            </body></html>
        "#;
        let result = extract_testimonials(html);
        assert!(result.is_some());
        let items = result.unwrap();
        assert!(items[0].contains("verändert"), "{:?}", items);
    }

    #[test]
    fn capped_at_max_items() {
        let quotes: String = (0..20)
            .map(|i| format!("<blockquote>Testimonial {i} ist sehr gut und sehr lang zum Testen.</blockquote>"))
            .collect();
        let html = format!("<html><body>{quotes}</body></html>");
        let result = extract_testimonials(&html);
        assert!(result.is_some());
        let items = result.unwrap();
        assert!(items.len() <= MAX_ITEMS, "Expected ≤{MAX_ITEMS} items, got {}", items.len());
    }

    #[test]
    fn each_item_truncated_to_500_chars() {
        let long: String = "a".repeat(1000);
        let html = format!("<html><body><blockquote>{long}</blockquote></body></html>");
        let result = extract_testimonials(&html);
        assert!(result.is_some());
        let items = result.unwrap();
        assert!(items[0].chars().count() <= MAX_ITEM_CHARS);
    }

    #[test]
    fn no_testimonials_returns_none() {
        let html = r#"
            <html><body>
              <h1>Willkommen</h1>
              <p>Kontaktieren Sie uns für mehr Informationen.</p>
            </body></html>
        "#;
        let result = extract_testimonials(html);
        assert!(result.is_none());
    }

    #[test]
    fn short_quoted_paragraph_below_min_length_ignored() {
        let html = r#"<html><body><p>"Gut!"</p></body></html>"#;
        let result = extract_testimonials(html);
        // "Gut!" is only 5 chars, below MIN_QUOTE_LENGTH (40); strategy 3 should not match.
        // Strategies 1 and 2 also won't match.
        assert!(result.is_none());
    }
}
