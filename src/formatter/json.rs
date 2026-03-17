use crate::types::SearchResult;

pub fn format_as_json(results: &[SearchResult]) -> String {
    if results.is_empty() {
        return "[]\n".to_string();
    }

    let entries: Vec<String> = results
        .iter()
        .map(|result| {
            format!(
                "    {{\n      \"title\": {},\n      \"url\": {},\n      \"snippet\": {}\n    }}",
                escape_json_string(&result.title),
                escape_json_string(&result.url),
                escape_json_string(&result.snippet)
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

    fn make_result(title: &str, url: &str, snippet: &str) -> SearchResult {
        SearchResult {
            title: title.to_string(),
            url: url.to_string(),
            snippet: snippet.to_string(),
        }
    }

    #[test]
    fn empty_results() {
        assert_eq!(format_as_json(&[]), "[]\n");
    }

    #[test]
    fn single_result() {
        let results = vec![make_result("Rust", "https://rust-lang.org", "A language")];
        let json = format_as_json(&results);
        assert_eq!(
            json,
            "[\n    {\n      \"title\": \"Rust\",\n      \"url\": \"https://rust-lang.org\",\n      \"snippet\": \"A language\"\n    }\n]\n"
        );
    }

    #[test]
    fn multiple_results() {
        let results = vec![
            make_result("One", "https://one.com", "First"),
            make_result("Two", "https://two.com", "Second"),
        ];
        let json = format_as_json(&results);
        assert!(json.starts_with("[\n"));
        assert!(json.ends_with("]\n"));
        assert!(json.contains("},\n    {"));
    }

    #[test]
    fn escapes_double_quotes() {
        assert_eq!(escape_json_string(r#"say "hello""#), r#""say \"hello\"""#);
    }

    #[test]
    fn escapes_backslashes() {
        assert_eq!(escape_json_string(r"path\to\file"), r#""path\\to\\file""#);
    }

    #[test]
    fn escapes_newlines_and_tabs() {
        assert_eq!(escape_json_string("a\nb\tc"), r#""a\nb\tc""#);
    }

    #[test]
    fn escapes_carriage_return() {
        assert_eq!(escape_json_string("a\rb"), r#""a\rb""#);
    }

    #[test]
    fn escapes_control_characters() {
        let input = String::from('\u{0000}');
        assert_eq!(escape_json_string(&input), r#""\u0000""#);

        let input = String::from('\u{001f}');
        assert_eq!(escape_json_string(&input), r#""\u001f""#);
    }

    #[test]
    fn escapes_line_separators() {
        assert_eq!(escape_json_string("\u{2028}"), r#""\u2028""#);
        assert_eq!(escape_json_string("\u{2029}"), r#""\u2029""#);
    }

    #[test]
    fn passes_plain_ascii_through() {
        assert_eq!(escape_json_string("hello world"), r#""hello world""#);
    }

    #[test]
    fn passes_unicode_through() {
        assert_eq!(escape_json_string("café ☕"), r#""café ☕""#);
    }
}
