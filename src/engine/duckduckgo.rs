use std::fmt;
use std::time::Duration;

use scraper::{Html, Selector};

use crate::types::SearchResult;

#[derive(Debug)]
pub enum SearchError {
    EmptyQuery,
    TopKTooLow,
    TopKTooHigh,
    Http(reqwest::Error),
}

impl fmt::Display for SearchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SearchError::EmptyQuery => write!(f, "empty string passed as query"),
            SearchError::TopKTooLow => write!(f, "top_k must be greater than zero"),
            SearchError::TopKTooHigh => write!(f, "top_k must be 100 or less"),
            SearchError::Http(e) => write!(f, "{e}"),
        }
    }
}

impl From<reqwest::Error> for SearchError {
    fn from(e: reqwest::Error) -> Self {
        SearchError::Http(e)
    }
}

#[derive(Debug)]
pub struct DuckDuckGoEngine {
    query: String,
}

impl DuckDuckGoEngine {
    pub fn new(query: String) -> Result<Self, SearchError> {
        if query.trim().is_empty() {
            return Err(SearchError::EmptyQuery);
        }
        Ok(Self { query })
    }

    pub async fn search(&self) -> Result<String, SearchError> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()?;
        Ok(client
            .post("https://lite.duckduckgo.com/lite/")
            .header("User-Agent", "Lynx/2.8.9rel.1")
            .form(&[("q", &self.query)])
            .send()
            .await?
            .text()
            .await?)
    }

    pub fn parse_results(
        &self,
        html: &str,
        top_k: usize,
    ) -> Result<Vec<SearchResult>, SearchError> {
        if top_k == 0 {
            return Err(SearchError::TopKTooLow);
        }
        if top_k > 100 {
            return Err(SearchError::TopKTooHigh);
        }

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
                let snippet = snippet
                    .text()
                    .collect::<Vec<_>>()
                    .join(" ")
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" ");
                SearchResult {
                    title,
                    url,
                    snippet,
                }
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn engine() -> DuckDuckGoEngine {
        DuckDuckGoEngine::new("test".to_string()).unwrap()
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
        assert_eq!(results[0].snippet, "A systems language");
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

        assert_eq!(results[0].snippet, "lots of spaces");
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
    fn top_k_zero_returns_error() {
        let html = make_html(&[("A", "https://a.com", "a")]);
        let err = engine().parse_results(&html, 0).unwrap_err();
        assert_eq!(err.to_string(), "top_k must be greater than zero");
    }

    #[test]
    fn top_k_over_100_returns_error() {
        let html = make_html(&[("A", "https://a.com", "a")]);
        let err = engine().parse_results(&html, 101).unwrap_err();
        assert_eq!(err.to_string(), "top_k must be 100 or less");
    }

    #[test]
    fn empty_query_returns_error() {
        let err = DuckDuckGoEngine::new("".to_string()).unwrap_err();
        assert_eq!(err.to_string(), "empty string passed as query");
    }

    #[test]
    fn whitespace_only_query_returns_error() {
        let err = DuckDuckGoEngine::new("   ".to_string()).unwrap_err();
        assert_eq!(err.to_string(), "empty string passed as query");
    }
}
