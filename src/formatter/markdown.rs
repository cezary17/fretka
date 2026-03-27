use crate::types::{
    META_AUTHORS, META_CATEGORIES, META_COMMENT, META_DOI, META_JOURNAL_REF, META_PDF_URL,
    META_PUBLISHED, META_UPDATED, SearchResult,
};

const KNOWN_KEYS: &[&str] = &[
    META_AUTHORS,
    META_CATEGORIES,
    META_PUBLISHED,
    META_UPDATED,
    META_DOI,
    META_JOURNAL_REF,
    META_COMMENT,
    META_PDF_URL,
];

fn display_key(key: &str) -> String {
    key.split('_')
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().to_string() + c.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn format_as_markdown(results: &[SearchResult]) -> String {
    results
        .iter()
        .enumerate()
        .map(|(i, result)| {
            let mut parts = vec![format!("{}. [{}]({})\n", i + 1, result.title, result.url)];

            if !result.metadata.is_empty() {
                for key in KNOWN_KEYS {
                    if let Some(value) = result.metadata.get(*key) {
                        parts.push(format!("   **{}:** {}", display_key(key), value));
                    }
                }
                for (key, value) in &result.metadata {
                    if !KNOWN_KEYS.contains(&key.as_str()) {
                        parts.push(format!("   **{key}:** {value}"));
                    }
                }
                parts.push(String::new());
            }

            parts.push(format!("   {}\n", result.content));
            parts.join("\n")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_result(title: &str, url: &str, content: &str) -> SearchResult {
        SearchResult {
            title: title.to_string(),
            url: url.to_string(),
            content: content.to_string(),
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn renders_metadata_fields() {
        let mut meta = HashMap::new();
        meta.insert("authors".to_string(), "Alice, Bob".to_string());
        meta.insert("categories".to_string(), "cs.AI".to_string());
        let results = vec![SearchResult {
            title: "Paper".to_string(),
            url: "https://arxiv.org/abs/1".to_string(),
            content: "Abstract".to_string(),
            metadata: meta,
        }];
        let output = format_as_markdown(&results);
        assert!(output.contains("**Authors:** Alice, Bob"));
        assert!(output.contains("**Categories:** cs.AI"));
    }

    #[test]
    fn empty_metadata_renders_without_metadata_section() {
        let results = vec![make_result("Title", "https://example.com", "Content")];
        let output = format_as_markdown(&results);
        assert!(!output.contains("**"));
    }

    #[test]
    fn basic_format_unchanged() {
        let results = vec![make_result("Rust", "https://rust-lang.org", "A language")];
        let output = format_as_markdown(&results);
        assert!(output.contains("[Rust](https://rust-lang.org)"));
        assert!(output.contains("A language"));
    }

    #[test]
    fn display_key_title_cases_underscored_keys() {
        assert_eq!(display_key("journal_ref"), "Journal Ref");
        assert_eq!(display_key("pdf_url"), "Pdf Url");
    }

    #[test]
    fn display_key_single_word() {
        assert_eq!(display_key("authors"), "Authors");
    }

    #[test]
    fn display_key_empty_string() {
        assert_eq!(display_key(""), "");
    }
}
