// Schedule extractor — T-028
// Spec: FORMAL_SPEC.md §13.2.6 (FR-V3-005)
//
// Extraction priority:
//  1. <table> with day-of-week names in headers (German or English)
//  2. Elements with class/id: stundenplan, timetable, schedule, kursplan, kurse
//  3. Time pattern (HH:MM) + day-of-week token in the same parent element
//
// Output: whitespace-normalised plain text, truncated to 3 000 characters.
// Returns None when no qualifying element found.

use scraper::{Html, Selector};

const MAX_OUTPUT_CHARS: usize = 3_000;

const DAY_TOKENS: &[&str] = &[
    // German
    "mo", "di", "mi", "do", "fr", "sa", "so",
    "montag", "dienstag", "mittwoch", "donnerstag", "freitag", "samstag", "sonntag",
    // English
    "mon", "tue", "wed", "thu", "fri", "sat", "sun",
    "monday", "tuesday", "wednesday", "thursday", "friday", "saturday", "sunday",
];

/// Returns true if the text contains a day-of-week token (whole-word, case-insensitive).
fn contains_day(text: &str) -> bool {
    let lower = text.to_lowercase();
    DAY_TOKENS.iter().any(|day| {
        let day_len = day.len();
        let mut start = 0;
        while let Some(pos) = lower[start..].find(day) {
            let abs_pos = start + pos;
            let before_ok = abs_pos == 0
                || !lower.as_bytes()[abs_pos - 1].is_ascii_alphabetic();
            let after_ok = abs_pos + day_len >= lower.len()
                || !lower.as_bytes()[abs_pos + day_len].is_ascii_alphabetic();
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

/// Returns true if the text contains a HH:MM time pattern.
fn contains_time_pattern(text: &str) -> bool {
    // Look for digit(s):digit digit — simple heuristic
    let bytes = text.as_bytes();
    for i in 0..bytes.len().saturating_sub(4) {
        if bytes[i].is_ascii_digit()
            && bytes[i + 1] == b':'
            && bytes[i + 2].is_ascii_digit()
            && bytes[i + 3].is_ascii_digit()
        {
            return true;
        }
    }
    false
}

fn normalise(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn truncate(s: String, max: usize) -> String {
    if s.chars().count() <= max {
        s
    } else {
        s.chars().take(max).collect()
    }
}

/// Extract schedule/timetable information from raw HTML.
pub fn extract_schedule(html: &str) -> Option<String> {
    let document = Html::parse_document(html);
    let mut collected: Vec<String> = Vec::new();

    // Strategy 1: <table> with day-of-week headers.
    if let Ok(table_sel) = Selector::parse("table") {
        for table in document.select(&table_sel) {
            // Check first row headers for day tokens.
            let header_text: String = {
                let mut h = String::new();
                if let Ok(th_sel) = Selector::parse("th") {
                    for th in table.select(&th_sel) {
                        h.push(' ');
                        h.push_str(&th.text().collect::<String>());
                    }
                }
                h
            };
            if contains_day(&header_text) {
                let text = normalise(&table.text().collect::<String>());
                if !text.is_empty() {
                    collected.push(text);
                    break;
                }
            }
        }
    }

    // Strategy 2: elements with schedule-related class/id.
    if collected.is_empty() {
        let class_sel = Selector::parse(
            "[class*=stundenplan],[class*=timetable],[class*=schedule],\
             [class*=kursplan],[class*=kurse],\
             [id*=stundenplan],[id*=timetable],[id*=schedule],\
             [id*=kursplan],[id*=kurse]",
        );
        if let Ok(sel) = class_sel {
            for el in document.select(&sel) {
                let text = normalise(&el.text().collect::<String>());
                if !text.is_empty() {
                    collected.push(text);
                    break;
                }
            }
        }
    }

    // Strategy 3: time pattern + day token in same parent element (p or div).
    if collected.is_empty() {
        if let Ok(sel) = Selector::parse("p, div, li") {
            for el in document.select(&sel) {
                let text = el.text().collect::<String>();
                if contains_time_pattern(&text) && contains_day(&text) {
                    let t = normalise(&text);
                    if !t.is_empty() && t.len() < 800 {
                        collected.push(t);
                        if collected.len() >= 5 {
                            break;
                        }
                    }
                }
            }
        }
    }

    if collected.is_empty() {
        return None;
    }

    Some(truncate(collected.join(" | "), MAX_OUTPUT_CHARS))
}

// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn german_day_header_table_extracted() {
        let html = r#"
            <html><body>
              <table>
                <tr><th>Mo</th><th>Di</th><th>Mi</th></tr>
                <tr><td>10:00 Pilates</td><td>09:00 Yoga</td><td>-</td></tr>
              </table>
            </body></html>
        "#;
        let result = extract_schedule(html);
        assert!(result.is_some(), "Expected schedule to be extracted");
        let text = result.unwrap();
        assert!(text.contains("Mo") || text.contains("10:00"), "Unexpected: {text}");
    }

    #[test]
    fn stundenplan_class_div_extracted() {
        let html = r#"
            <html><body>
              <div class="stundenplan">
                <p>Montag 09:00 – Reformer</p>
                <p>Dienstag 17:00 – Mat</p>
              </div>
            </body></html>
        "#;
        let result = extract_schedule(html);
        assert!(result.is_some());
        let text = result.unwrap();
        assert!(text.contains("Montag") || text.contains("09:00"), "{text}");
    }

    #[test]
    fn time_pattern_plus_day_extracted() {
        let html = r#"
            <html><body>
              <ul>
                <li>Montag 10:00 - 11:00 Pilates Mat</li>
                <li>Freitag 18:00 - 19:00 Reformer</li>
              </ul>
            </body></html>
        "#;
        let result = extract_schedule(html);
        assert!(result.is_some());
    }

    #[test]
    fn no_schedule_returns_none() {
        let html = r#"
            <html><body>
              <h1>Kontakt</h1>
              <p>Tel: 01234 567890</p>
            </body></html>
        "#;
        let result = extract_schedule(html);
        assert!(result.is_none());
    }

    #[test]
    fn output_truncated_to_3000_chars() {
        let rows: String = (0..200)
            .map(|i| format!("<tr><th>Mo</th><td>0{}:00 Kurs {i}</td></tr>", i % 10))
            .collect();
        let html = format!("<html><body><table>{rows}</table></body></html>");
        let result = extract_schedule(&html);
        assert!(result.is_some());
        let text = result.unwrap();
        assert!(text.chars().count() <= MAX_OUTPUT_CHARS);
    }
}
