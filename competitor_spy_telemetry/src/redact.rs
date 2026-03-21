// competitor_spy_telemetry/src/redact.rs
//
// Pre-emit secret redaction. All log strings MUST pass through redact()
// before being emitted. No credential value, API key, or token may appear
// in any log entry, stdout, or PDF output.

/// Redact known secret patterns from a log string.
///
/// Replaces the secret value portion with `[REDACTED]`. Patterns covered
/// (case-insensitive key matching):
/// - `Authorization: <value>`
/// - `Bearer <token>`
/// - `api_key=<v>`, `apikey=<v>`, `api-key=<v>`
/// - `token=<v>` or `token: <v>`
/// - `client_secret=<v>`, `secret=<v>`
/// - `password=<v>`
/// - `key=<v>` (word-boundary)
pub fn redact(s: &str) -> String {
    let mut out = s.to_owned();
    out = redact_authorization(&out);
    out = redact_bearer(&out);
    out = redact_api_key(&out);
    out = redact_token(&out);
    out = redact_secret(&out);
    out = redact_password(&out);
    out = redact_key(&out);
    out
}

fn redact_authorization(s: &str) -> String {
    // Authorization headers have the form "Authorization: <entire rest of value>".
    // The value may contain spaces (e.g. "Bearer abc123"), so we redact everything
    // after "authorization:" up to the next newline (or end of string).
    let lower = s.to_lowercase();
    let keyword = "authorization";
    let mut result = String::with_capacity(s.len());
    let mut pos = 0;
    while pos < lower.len() {
        if let Some(rel) = lower[pos..].find(keyword) {
            let abs = pos + rel;
            let after_kw = abs + keyword.len();
            // Skip optional whitespace then expect ':'.
            let ws = lower[after_kw..].chars().take_while(|c| c.is_whitespace()).count();
            let colon_pos = after_kw + ws;
            if colon_pos < lower.len() && lower.as_bytes()[colon_pos] == b':' {
                result.push_str(&s[pos..colon_pos + 1]);
                // Redact to end of line.
                let rest_start = colon_pos + 1;
                let line_end = s[rest_start..]
                    .find('\n')
                    .map(|i| rest_start + i)
                    .unwrap_or(s.len());
                // Preserve leading whitespace for readability.
                let ws2 = s[rest_start..line_end]
                    .chars()
                    .take_while(|c| c.is_whitespace())
                    .count();
                if ws2 > 0 {
                    result.push_str(&s[rest_start..rest_start + ws2]);
                }
                if line_end > rest_start + ws2 {
                    result.push_str("[REDACTED]");
                }
                pos = line_end;
                continue;
            }
            result.push_str(&s[pos..after_kw]);
            pos = after_kw;
        } else {
            result.push_str(&s[pos..]);
            break;
        }
    }
    result
}

fn redact_bearer(s: &str) -> String {
    let lower = s.to_lowercase();
    let keyword = "bearer ";
    let mut result = String::with_capacity(s.len());
    let mut pos = 0;
    while pos < lower.len() {
        if let Some(rel) = lower[pos..].find(keyword) {
            let abs = pos + rel;
            result.push_str(&s[pos..abs + keyword.len()]);
            let after = abs + keyword.len();
            let end = s[after..]
                .find(|c: char| c.is_whitespace())
                .map(|i| after + i)
                .unwrap_or(s.len());
            if end > after {
                result.push_str("[REDACTED]");
                pos = end;
            } else {
                pos = abs + keyword.len();
            }
        } else {
            result.push_str(&s[pos..]);
            break;
        }
    }
    result
}

fn redact_api_key(s: &str) -> String {
    let mut result = s.to_owned();
    for kw in &["api_key", "apikey", "api-key"] {
        result = redact_key_value(&result, kw, false);
    }
    result
}

fn redact_token(s: &str) -> String {
    redact_key_value(s, "token", true)
}

fn redact_secret(s: &str) -> String {
    let mut out = redact_key_value(s, "client_secret", false);
    out = redact_key_value(&out, "secret", true);
    out
}

fn redact_password(s: &str) -> String {
    redact_key_value(s, "password", true)
}

fn redact_key(s: &str) -> String {
    redact_key_value(s, "key", true)
}

/// Replace `<keyword><sep><value>` with `<keyword><sep>[REDACTED]`.
/// Separator is `=` or `:`. If `word_boundary` is true, keyword must not
/// be immediately preceded by alphanumeric or `_`.
fn redact_key_value(s: &str, keyword: &str, word_boundary: bool) -> String {
    let lower = s.to_lowercase();
    let kw_lower = keyword.to_lowercase();
    let kw_len = kw_lower.len();
    let mut result = String::with_capacity(s.len());
    let mut pos = 0;
    while pos < lower.len() {
        if let Some(rel) = lower[pos..].find(kw_lower.as_str()) {
            let abs = pos + rel;
            if word_boundary && abs > 0 {
                let prev_char = s[..abs].chars().next_back().unwrap_or(' ');
                if prev_char.is_alphanumeric() || prev_char == '_' {
                    result.push_str(&s[pos..abs + 1]);
                    pos = abs + 1;
                    continue;
                }
            }
            let after_kw = abs + kw_len;
            let ws_len = lower[after_kw..].chars().take_while(|c| c.is_whitespace()).count();
            let after_ws = after_kw + ws_len;
            if after_ws < lower.len() {
                let sep_char = lower.as_bytes()[after_ws] as char;
                if sep_char == '=' || sep_char == ':' {
                    result.push_str(&s[pos..after_ws + 1]);
                    let after_sep = after_ws + 1;
                    let ws2 = s[after_sep..]
                        .chars()
                        .take_while(|c| c.is_whitespace())
                        .count();
                    let value_start = after_sep + ws2;
                    if ws2 > 0 {
                        result.push_str(&s[after_sep..value_start]);
                    }
                    let value_end = s[value_start..]
                        .find(|c: char| c.is_whitespace())
                        .map(|i| value_start + i)
                        .unwrap_or(s.len());
                    if value_end > value_start {
                        result.push_str("[REDACTED]");
                        pos = value_end;
                    } else {
                        pos = value_start;
                    }
                    continue;
                }
            }
            result.push_str(&s[pos..after_kw]);
            pos = after_kw;
        } else {
            result.push_str(&s[pos..]);
            break;
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_string_unchanged() {
        let s = "geocoding_attempt adapter_id=nominatim url=https://nominatim.openstreetmap.org/search";
        assert_eq!(redact(s), s);
    }

    #[test]
    fn redacts_authorization_header() {
        let s = "Authorization: Bearer abc123token";
        let r = redact(s);
        assert!(!r.contains("abc123token"), "token must be redacted; got: {r}");
        assert!(r.contains("Authorization:"), "key must be preserved");
    }

    #[test]
    fn redacts_bearer_standalone() {
        let s = "sending request with Bearer supersecrettoken to server";
        let r = redact(s);
        assert!(!r.contains("supersecrettoken"), "got: {r}");
        assert!(r.contains("Bearer"), "prefix preserved");
    }

    #[test]
    fn redacts_api_key_equals() {
        let s = "GET /search?api_key=mysecretkey123&q=yoga+studio";
        let r = redact(s);
        assert!(!r.contains("mysecretkey123"), "got: {r}");
        assert!(r.contains("api_key="), "key preserved");
    }

    #[test]
    fn redacts_apikey_variant() {
        let s = "apikey=abc987xyz";
        let r = redact(s);
        assert!(!r.contains("abc987xyz"), "got: {r}");
    }

    #[test]
    fn redacts_token_equals() {
        let s = "result token=verysecretvar end";
        let r = redact(s);
        assert!(!r.contains("verysecretvar"), "got: {r}");
        assert!(r.contains("token="), "key preserved");
    }

    #[test]
    fn redacts_secret_equals() {
        let s = "client_secret=abc123def";
        let r = redact(s);
        assert!(!r.contains("abc123def"), "got: {r}");
    }

    #[test]
    fn redacts_password_colon() {
        let s = "password: hunter2 next_field=ok";
        let r = redact(s);
        assert!(!r.contains("hunter2"), "got: {r}");
    }

    #[test]
    fn does_not_redact_keyboard_word() {
        let s = "keyboard shortcut is ctrl+k";
        let r = redact(s);
        assert!(r.contains("keyboard"), "keyboard must not be redacted; got: {r}");
    }

    #[test]
    fn empty_string_unchanged() {
        assert_eq!(redact(""), "");
    }

    #[test]
    fn redacts_case_insensitive_key() {
        let s = "API_KEY=uppercasekey123 other=stuff";
        let r = redact(s);
        assert!(!r.contains("uppercasekey123"), "got: {r}");
    }

    #[test]
    fn multiple_secrets_in_one_string() {
        let s = "api_key=key1 token=tok2 plain=notasecret";
        let r = redact(s);
        assert!(!r.contains("key1"), "key1 not redacted; got: {r}");
        assert!(!r.contains("tok2"), "tok2 not redacted; got: {r}");
        assert!(r.contains("plain=notasecret"), "plain field preserved; got: {r}");
    }
}
