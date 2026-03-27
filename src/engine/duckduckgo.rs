use std::collections::HashMap;

use scraper::{Html, Selector};

use super::{SearchEngine, SearchError};
use crate::types::SearchResult;

#[derive(Debug)]
pub struct DuckDuckGoEngine {
    query: String,
    client: reqwest::Client,
}

impl DuckDuckGoEngine {
    pub fn new(query: String, client: reqwest::Client) -> Result<Self, SearchError> {
        if query.trim().is_empty() {
            return Err(SearchError::EmptyQuery);
        }
        Ok(Self { query, client })
    }
}

impl SearchEngine for DuckDuckGoEngine {
    async fn search(&self) -> Result<String, SearchError> {
        Ok(self
            .client
            .post("https://lite.duckduckgo.com/lite/")
            .form(&[("q", &self.query)])
            .send()
            .await?
            .text()
            .await?)
    }

    fn parse_results(&self, html: &str, top_k: usize) -> Result<Vec<SearchResult>, SearchError> {
        let document = Html::parse_document(html);
        let link_selector = Selector::parse("a.result-link").unwrap();
        let snippet_selector = Selector::parse("td.result-snippet").unwrap();

        let titles: Vec<_> = document.select(&link_selector).collect();
        let snippets: Vec<_> = document.select(&snippet_selector).collect();

        Ok(titles
            .into_iter()
            .zip(snippets)
            .take(top_k)
            .map(|(link, snippet)| {
                let title = link.text().collect::<Vec<_>>().join(" ").trim().to_string();
                let url = link.value().attr("href").unwrap_or("").to_string();
                let content = snippet
                    .text()
                    .collect::<Vec<_>>()
                    .join(" ")
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" ");
                SearchResult {
                    title,
                    url,
                    content,
                    metadata: HashMap::new(),
                }
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn engine() -> DuckDuckGoEngine {
        let client = reqwest::Client::new();
        DuckDuckGoEngine::new("test".to_string(), client).unwrap()
    }

    fn make_html(results: &[(&str, &str, &str)]) -> String {
        let mut rows = String::new();
        for (title, url, snippet) in results {
            rows.push_str(&format!(
                r#"<tr><td><a class="result-link" href="{url}">{title}</a></td></tr>
                   <tr><td class="result-snippet">{snippet}</td></tr>"#,
            ));
        }
        format!("<html><body><table>{rows}</table></body></html>")
    }

    #[test]
    fn parse_single_result() {
        let html = make_html(&[("Rust Lang", "https://rust-lang.org", "A systems language")]);
        let results = engine().parse_results(&html, 10).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Rust Lang");
        assert_eq!(results[0].url, "https://rust-lang.org");
        assert_eq!(results[0].content, "A systems language");
    }

    #[test]
    fn parse_multiple_results() {
        let html = make_html(&[
            ("Result One", "https://one.com", "First snippet"),
            ("Result Two", "https://two.com", "Second snippet"),
            ("Result Three", "https://three.com", "Third snippet"),
        ]);
        let results = engine().parse_results(&html, 10).unwrap();

        assert_eq!(results.len(), 3);
        assert_eq!(results[0].title, "Result One");
        assert_eq!(results[2].title, "Result Three");
    }

    #[test]
    fn top_k_limits_results() {
        let html = make_html(&[
            ("A", "https://a.com", "a"),
            ("B", "https://b.com", "b"),
            ("C", "https://c.com", "c"),
        ]);
        let results = engine().parse_results(&html, 2).unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].title, "A");
        assert_eq!(results[1].title, "B");
    }

    #[test]
    fn empty_html_returns_no_results() {
        let results = engine()
            .parse_results("<html><body></body></html>", 10)
            .unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn snippet_whitespace_is_normalized() {
        let html = make_html(&[("Title", "https://x.com", "  lots   of   spaces  ")]);
        let results = engine().parse_results(&html, 10).unwrap();

        assert_eq!(results[0].content, "lots of spaces");
    }

    #[test]
    fn missing_href_defaults_to_empty() {
        let html = r#"<html><body><table>
            <tr><td><a class="result-link">No Link</a></td></tr>
            <tr><td class="result-snippet">snippet</td></tr>
        </table></body></html>"#;
        let results = engine().parse_results(html, 10).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].url, "");
    }

    #[test]
    fn mismatched_links_and_snippets_zips_to_shorter() {
        let html = r#"<html><body><table>
            <tr><td><a class="result-link" href="https://a.com">A</a></td></tr>
            <tr><td><a class="result-link" href="https://b.com">B</a></td></tr>
            <tr><td class="result-snippet">Only one snippet</td></tr>
        </table></body></html>"#;
        let results = engine().parse_results(html, 10).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "A");
    }

    #[test]
    fn empty_query_returns_error() {
        let client = reqwest::Client::new();
        let err = DuckDuckGoEngine::new("".to_string(), client).unwrap_err();
        assert_eq!(err.to_string(), "empty string passed as query");
    }

    #[test]
    fn whitespace_only_query_returns_error() {
        let client = reqwest::Client::new();
        let err = DuckDuckGoEngine::new("   ".to_string(), client).unwrap_err();
        assert_eq!(err.to_string(), "empty string passed as query");
    }
}
