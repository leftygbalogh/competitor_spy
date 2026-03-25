// Lesson types extractor — T-027
// Spec: FORMAL_SPEC.md §13.2.5 (FR-V3-004)
//
// Scans nav links, h1–h3 headings, and list items for discipline vocabulary tokens.
// Returns deduplicated vec preserving document order. Case-insensitive match.
// Returns None when no target vocabulary token found.

use scraper::{Html, Selector};

/// Vocabulary of lesson/discipline types (lowercase; matched case-insensitively).
const VOCABULARY: &[&str] = &[
    "reformer", "mat", "matwork", "fusion", "pilates", "yoga", "barre", "aerial",
    "tower", "cadillac", "chair", "barrel", "clinical", "prenatal", "postnatal",
    "hot", "yin", "vinyasa", "hiit", "stretch", "mobility", "fascia",
];

/// Extract matched vocabulary tokens from a text string.
fn find_tokens_in_text(text: &str) -> Vec<String> {
    let lower = text.to_lowercase();
    VOCABULARY
        .iter()
        .filter(|kw| {
            let kw: &str = **kw;
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
        .map(|kw| {
            let kw: &str = *kw;
            // Return capitalised form of the matched keyword.
            let mut chars = kw.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => c.to_uppercase().to_string() + chars.as_str(),
            }
        })
        .collect()
}

/// Extract lesson/discipline types from raw HTML.
pub fn extract_lesson_types(html: &str) -> Option<Vec<String>> {
    let document = Html::parse_document(html);
    let mut seen: Vec<String> = Vec::new();

    let selectors = [
        "nav a",
        "h1", "h2", "h3",
        "li",
    ];

    for selector_str in &selectors {
        if let Ok(sel) = Selector::parse(selector_str) {
            for el in document.select(&sel) {
                let text = el.text().collect::<String>();
                for token in find_tokens_in_text(&text) {
                    if !seen.contains(&token) {
                        seen.push(token);
                    }
                }
            }
        }
    }

    if seen.is_empty() {
        None
    } else {
        Some(seen)
    }
}

// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nav_with_vocabulary_tokens_extracted() {
        let html = r#"
            <html><body>
              <nav>
                <a href="/reformer">Reformer Pilates</a>
                <a href="/mat">Mat Pilates</a>
                <a href="/barre">Barre</a>
              </nav>
            </body></html>
        "#;
        let result = extract_lesson_types(html);
        assert!(result.is_some());
        let types = result.unwrap();
        assert!(types.contains(&"Reformer".to_string()), "Expected Reformer: {types:?}");
        assert!(types.contains(&"Mat".to_string()), "Expected Mat: {types:?}");
        assert!(types.contains(&"Barre".to_string()), "Expected Barre: {types:?}");
    }

    #[test]
    fn heading_with_vocabulary_extracted() {
        let html = r#"
            <html><body>
              <h2>Yoga und Pilates Kurse</h2>
              <h3>Reformer Training</h3>
            </body></html>
        "#;
        let result = extract_lesson_types(html);
        assert!(result.is_some());
        let types = result.unwrap();
        assert!(types.contains(&"Yoga".to_string()));
        assert!(types.contains(&"Pilates".to_string()));
        assert!(types.contains(&"Reformer".to_string()));
    }

    #[test]
    fn duplicates_suppressed() {
        let html = r#"
            <html><body>
              <nav><a href="/r">Reformer</a></nav>
              <h2>Reformer Pilates Kurs</h2>
              <li>Reformer Klasse</li>
            </body></html>
        "#;
        let result = extract_lesson_types(html);
        assert!(result.is_some());
        let types = result.unwrap();
        let reformer_count = types.iter().filter(|t| t.as_str() == "Reformer").count();
        assert_eq!(reformer_count, 1, "Reformer should appear only once: {types:?}");
    }

    #[test]
    fn case_insensitive_match() {
        let html = r#"
            <html><body>
              <li>REFORMER kurs</li>
              <li>MATWORK sessions</li>
            </body></html>
        "#;
        let result = extract_lesson_types(html);
        assert!(result.is_some());
        let types = result.unwrap();
        assert!(types.contains(&"Reformer".to_string()), "{types:?}");
        assert!(types.contains(&"Matwork".to_string()), "{types:?}");
    }

    #[test]
    fn no_vocabulary_returns_none() {
        let html = r#"
            <html><body>
              <h1>Willkommen</h1>
              <p>Wir freuen uns auf Ihren Besuch.</p>
            </body></html>
        "#;
        let result = extract_lesson_types(html);
        assert!(result.is_none());
    }

    #[test]
    fn partial_word_not_matched() {
        // "matwork" should match but "automatic" should not match "mat"
        let html = r#"<html><body><p>automatic systems</p></body></html>"#;
        let result = extract_lesson_types(html);
        assert!(result.is_none(), "Partial word match should not occur: {result:?}");
    }
}
