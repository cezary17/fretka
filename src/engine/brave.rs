use scraper::{Html, Selector};

use super::duckduckgo::SearchError;
use crate::types::SearchResult;

#[derive(Debug)]
pub struct BraveEngine {
    query: String,
    client: reqwest::Client,
}

impl BraveEngine {
    pub fn new(query: String, client: reqwest::Client) -> Result<Self, SearchError> {
        if query.trim().is_empty() {
            return Err(SearchError::EmptyQuery);
        }
        Ok(Self { query, client })
    }

    pub async fn search(&self) -> Result<String, SearchError> {
        let encoded_query: String = self
            .query
            .bytes()
            .flat_map(|b| {
                if b.is_ascii_alphanumeric() || b == b'-' || b == b'_' || b == b'.' || b == b'~' {
                    vec![b as char]
                } else if b == b' ' {
                    vec!['+']
                } else {
                    format!("%{b:02X}").chars().collect()
                }
            })
            .collect();
        let url = format!("https://search.brave.com/search?q={encoded_query}");
        Ok(self.client.get(&url).send().await?.text().await?)
    }

    pub fn parse_results(
        &self,
        html: &str,
        top_k: usize,
    ) -> Result<Vec<SearchResult>, SearchError> {
        let document = Html::parse_document(html);
        let snippet_selector = Selector::parse("#results .snippet").unwrap();

        let mut results = Vec::new();

        for element in document.select(&snippet_selector).take(top_k) {
            let title = element
                .select(&Selector::parse(".snippet-title").unwrap())
                .next()
                .map(|el| el.text().collect::<Vec<_>>().join("").trim().to_string())
                .unwrap_or_default();

            let url = element
                .select(&Selector::parse(".snippet-title").unwrap())
                .next()
                .and_then(|el| el.value().attr("href").map(|s| s.to_string()))
                .or_else(|| {
                    element
                        .select(&Selector::parse("a").unwrap())
                        .next()
                        .and_then(|el| el.value().attr("href").map(|s| s.to_string()))
                })
                .unwrap_or_default();

            let content = element
                .select(&Selector::parse(".snippet-description").unwrap())
                .next()
                .map(|el| {
                    el.text()
                        .collect::<Vec<_>>()
                        .join(" ")
                        .split_whitespace()
                        .collect::<Vec<_>>()
                        .join(" ")
                })
                .unwrap_or_default();

            if !title.is_empty() || !url.is_empty() {
                results.push(SearchResult {
                    title,
                    url,
                    content,
                });
            }
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn engine() -> BraveEngine {
        let client = reqwest::Client::new();
        BraveEngine::new("test".to_string(), client).unwrap()
    }

    fn make_html(results: &[(&str, &str, &str)]) -> String {
        let mut snippets = String::new();
        for (title, url, description) in results {
            snippets.push_str(&format!(
                r#"<div class="snippet">
                    <a class="snippet-title" href="{url}">{title}</a>
                    <div class="snippet-description">{description}</div>
                </div>"#,
            ));
        }
        format!(r#"<html><body><div id="results">{snippets}</div></body></html>"#)
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
            .parse_results(r#"<html><body><div id="results"></div></body></html>"#, 10)
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
    fn empty_query_returns_error() {
        let client = reqwest::Client::new();
        let err = BraveEngine::new("".to_string(), client).unwrap_err();
        assert_eq!(err.to_string(), "empty string passed as query");
    }

    #[test]
    fn whitespace_only_query_returns_error() {
        let client = reqwest::Client::new();
        let err = BraveEngine::new("   ".to_string(), client).unwrap_err();
        assert_eq!(err.to_string(), "empty string passed as query");
    }
}
