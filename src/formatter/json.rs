use crate::types::SearchResult;

pub fn format_as_json(results: &[SearchResult]) -> String {
    if results.is_empty() {
        return "[]\n".to_string();
    }

    let entries: Vec<String> = results
        .iter()
        .map(|result| {
            format!(
                "    {{\n      \"title\": {},\n      \"url\": {},\n      \"content\": {}\n    }}",
                escape_json_string(&result.title),
                escape_json_string(&result.url),
                escape_json_string(&result.content)
            )
        })
        .collect();

    format!("[\n{}\n]\n", entries.join(",\n"))
}

fn escape_json_string(s: &str) -> String {
    let mut escaped = String::with_capacity(s.len() + 2);
    escaped.push('"');
    for c in s.chars() {
        match c {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            '\u{2028}' => escaped.push_str("\\u2028"),
            '\u{2029}' => escaped.push_str("\\u2029"),
            c if (c as u32) < 0x20 => {
                escaped.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => escaped.push(c),
        }
    }
    escaped.push('"');
    escaped
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct JsonResult {
        title: String,
        url: String,
        content: String,
    }

    fn make_result(title: &str, url: &str, snippet: &str) -> SearchResult {
        SearchResult {
            title: title.to_string(),
            url: url.to_string(),
            content: snippet.to_string(),
        }
    }

    /// Parse the JSON output and verify it round-trips correctly.
    fn assert_roundtrips(results: &[SearchResult]) {
        let json = format_as_json(results);
        let parsed: Vec<JsonResult> =
            serde_json::from_str(&json).expect("output must be valid JSON");
        assert_eq!(parsed.len(), results.len());
        for (parsed, original) in parsed.iter().zip(results.iter()) {
            assert_eq!(parsed.title, original.title);
            assert_eq!(parsed.url, original.url);
            assert_eq!(parsed.content, original.content);
        }
    }

    #[test]
    fn empty_results() {
        assert_roundtrips(&[]);
    }

    #[test]
    fn single_result() {
        let results = vec![make_result("Rust", "https://rust-lang.org", "A language")];
        assert_roundtrips(&results);
    }

    #[test]
    fn multiple_results() {
        let results = vec![
            make_result("One", "https://one.com", "First"),
            make_result("Two", "https://two.com", "Second"),
        ];
        assert_roundtrips(&results);
    }

    #[test]
    fn roundtrips_special_characters() {
        let results = vec![make_result(
            "say \"hello\" \\ world",
            "https://example.com/path?a=1&b=2",
            "line1\nline2\ttab\r\nend",
        )];
        assert_roundtrips(&results);
    }

    #[test]
    fn roundtrips_control_characters() {
        let results = vec![make_result(
            "null\u{0000}char",
            "https://example.com",
            "unit-sep\u{001f}here",
        )];
        assert_roundtrips(&results);
    }

    #[test]
    fn roundtrips_unicode_and_line_separators() {
        let results = vec![make_result(
            "café ☕ \u{2028}line-sep\u{2029}para-sep",
            "https://example.com/émoji/🦀",
            "日本語テスト",
        )];
        assert_roundtrips(&results);
    }

    #[test]
    fn escapes_double_quotes() {
        assert_eq!(escape_json_string(r#"say "hello""#), r#""say \"hello\"""#);
    }

    #[test]
    fn escapes_backslashes() {
        assert_eq!(escape_json_string(r"path\to\file"), r#""path\\to\\file""#);
    }
}
