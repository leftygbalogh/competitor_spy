// Class descriptions extractor — T-030
// Spec: FORMAL_SPEC.md §13.2.8 (FR-V3-007)
//
// Extraction priority:
//  1. <p> sibling/child of heading containing lesson-type vocabulary
//  2. <section>/<article> with class/id: kurs, class, angebot, offer, leistung
//  3. <p> > 80 chars within 3 DOM levels below h2/h3 with lesson-type vocabulary
//
// Output: Vec<String>, max 8 items, each trimmed and truncated to 800 chars.
// Returns None when no qualifying element found.

use scraper::{Html, Selector, ElementRef};

const MAX_ITEMS: usize = 8;
const MAX_ITEM_CHARS: usize = 800;
const MIN_DESCRIPTION_CHARS: usize = 80;

/// Lesson-type vocabulary (same set as lesson_types extractor).
const VOCABULARY: &[&str] = &[
    "reformer", "mat", "matwork", "fusion", "pilates", "yoga", "barre", "aerial",
    "tower", "cadillac", "chair", "barrel", "clinical", "prenatal", "postnatal",
    "hot", "yin", "vinyasa", "hiit", "stretch", "mobility", "fascia",
];

fn contains_vocabulary(text: &str) -> bool {
    let lower = text.to_lowercase();
    VOCABULARY.iter().any(|&kw| {
        let kw_len = kw.len();
        let mut start = 0;
        while let Some(pos) = lower[start..].find(kw) {
            let abs_pos = start + pos;
            let before_ok = abs_pos == 0
                || !lower.as_bytes()[abs_pos - 1].is_ascii_alphabetic();
            let after_ok = abs_pos + kw_len >= lower.len()
                || !lower.as_bytes()[abs_pos + kw_len].is_ascii_alphabetic();
            if before_ok && after_ok {
                return true;
            }
            start = abs_pos + 1;
            if start >= lower.len() {
                break;
            }
        }
        false
    })
}

fn normalise(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn truncate_item(s: &str) -> String {
    let trimmed = s.trim();
    if trimmed.chars().count() <= MAX_ITEM_CHARS {
        trimmed.to_string()
    } else {
        trimmed.chars().take(MAX_ITEM_CHARS).collect()
    }
}

/// Collect <p> text from within an element reference, up to `remaining` items.
fn collect_paragraphs_from(el: ElementRef, results: &mut Vec<String>, remaining: usize) {
    if let Ok(p_sel) = Selector::parse("p") {
        for p in el.select(&p_sel) {
            let text = normalise(&p.text().collect::<String>());
            if text.chars().count() >= MIN_DESCRIPTION_CHARS {
                let item = truncate_item(&text);
                if !results.contains(&item) {
                    results.push(item);
                    if results.len() >= remaining {
                        return;
                    }
                }
            }
        }
    }
}

/// Extract class/course descriptions from raw HTML.
pub fn extract_class_descriptions(html: &str) -> Option<Vec<String>> {
    let document = Html::parse_document(html);
    let mut results: Vec<String> = Vec::new();

    // Strategy 1: <section> or <article> with class/id containing course-related keywords.
    let section_sel_str = "[class*=kurs],[class*=angebot],[class*=offer],[class*=leistung],\
                           [id*=kurs],[id*=angebot],[id*=offer],[id*=leistung]";
    if let Ok(section_sel) = Selector::parse(section_sel_str) {
        for section in document.select(&section_sel) {
            collect_paragraphs_from(section, &mut results, MAX_ITEMS);
            if results.len() >= MAX_ITEMS {
                return Some(results);
            }
        }
    }

    // Strategy 2: <p> elements that are siblings/children of a heading with vocabulary.
    if results.len() < MAX_ITEMS {
        if let Ok(heading_sel) = Selector::parse("h1, h2, h3") {
            for heading in document.select(&heading_sel) {
                let heading_text = heading.text().collect::<String>();
                if !contains_vocabulary(&heading_text) {
                    continue;
                }
                // Collect sibling <p> elements — walk next siblings.
                let mut next = heading.next_sibling();
                let mut count = 0;
                while let Some(sib) = next {
                    count += 1;
                    if count > 5 {
                        break;
                    }
                    if let Some(el) = ElementRef::wrap(sib) {
                        let tag = el.value().name();
                        if tag == "p" {
                            let text = normalise(&el.text().collect::<String>());
                            if text.chars().count() >= MIN_DESCRIPTION_CHARS {
                                let item = truncate_item(&text);
                                if !results.contains(&item) {
                                    results.push(item);
                                    if results.len() >= MAX_ITEMS {
                                        return Some(results);
                                    }
                                }
                            }
                        } else if tag.starts_with('h') {
                            // Stop at next heading.
                            break;
                        }
                    }
                    next = sib.next_sibling();
                }
            }
        }
    }

    // Strategy 3: <p> > 80 chars within sections that have vocabulary anywhere.
    if results.len() < MAX_ITEMS {
        if let Ok(sel) = Selector::parse("section, article, div") {
            for container in document.select(&sel) {
                let container_text = container.text().collect::<String>();
                if !contains_vocabulary(&container_text) {
                    continue;
                }
                collect_paragraphs_from(container, &mut results, MAX_ITEMS);
                if results.len() >= MAX_ITEMS {
                    break;
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
    fn kurs_section_with_paragraphs_extracted() {
        let html = r#"
            <html><body>
              <section class="kurs">
                <h2>Reformer Pilates</h2>
                <p>Das Reformer Pilates ist eine intensive Trainingsform, die auf dem Pilates-Gerät
                   namens Reformer durchgeführt wird. Es stärkt die Tiefenmuskulatur und verbessert
                   die Körperhaltung nachhaltig.</p>
                <p>Dieser Kurs ist für alle Niveaus geeignet und wird von erfahrenen Trainern geleitet.</p>
              </section>
            </body></html>
        "#;
        let result = extract_class_descriptions(html);
        assert!(result.is_some(), "Expected descriptions to be extracted");
        let items = result.unwrap();
        assert!(!items.is_empty());
        assert!(items.iter().any(|i| i.contains("Reformer") || i.contains("Tiefenmuskulatur")), "{items:?}");
    }

    #[test]
    fn heading_with_vocabulary_sibling_paragraphs_extracted() {
        let html = r#"
            <html><body>
              <h3>Yoga für Anfänger</h3>
              <p>In diesem Kurs lernen Sie die Grundlagen des Yoga kennen. Wir arbeiten an
                 Atemübungen, Dehnung und Entspannungstechniken für einen gesunden Alltag.</p>
            </body></html>
        "#;
        let result = extract_class_descriptions(html);
        assert!(result.is_some());
        let items = result.unwrap();
        assert!(items.iter().any(|i| i.contains("Yoga") || i.contains("Atemübungen")), "{items:?}");
    }

    #[test]
    fn capped_at_max_items() {
        let paras: String = (0..20)
            .map(|i| format!("<p>Details zu Kurs {i}: Dieser Kurs bietet eine einzigartige Erfahrung im Bereich Pilates und Wellness für alle Teilnehmer.</p>"))
            .collect();
        let html = format!("<html><body><section class=\"angebot\">{paras}</section></body></html>");
        let result = extract_class_descriptions(&html);
        assert!(result.is_some());
        let items = result.unwrap();
        assert!(items.len() <= MAX_ITEMS, "Expected ≤{MAX_ITEMS} items, got {}", items.len());
    }

    #[test]
    fn each_item_truncated_to_800_chars() {
        let long: String = "Pilates ".repeat(200); // > 800 chars
        let html = format!("<html><body><section class=\"kurs\"><p>{long}</p></section></body></html>");
        let result = extract_class_descriptions(&html);
        assert!(result.is_some());
        let items = result.unwrap();
        assert!(items[0].chars().count() <= MAX_ITEM_CHARS, "Item too long: {}", items[0].len());
    }

    #[test]
    fn short_paragraphs_below_min_length_ignored() {
        let html = r#"
            <html><body>
              <section class="kurs">
                <p>Kurz.</p>
                <p>Auch kurz.</p>
              </section>
            </body></html>
        "#;
        let result = extract_class_descriptions(html);
        // All <p> are below 80 chars, so none should be extracted.
        assert!(result.is_none(), "Expected None for short paragraphs: {result:?}");
    }

    #[test]
    fn no_vocabulary_context_returns_none() {
        let html = r#"
            <html><body>
              <h1>Willkommen</h1>
              <p>Kontaktieren Sie uns täglich von 9 bis 17 Uhr für weitere Informationen zu unserem Studio und unseren Öffnungszeiten.</p>
            </body></html>
        "#;
        let result = extract_class_descriptions(html);
        assert!(result.is_none());
    }
}
