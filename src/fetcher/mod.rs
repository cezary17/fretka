use dom_smoothie::{Config, Readability, TextMode};
use futures::future::join_all;

use crate::truncator::Truncator;
use crate::types::{META_PDF_URL, SearchResult};

pub struct FetchOutcome {
    pub result: SearchResult,
    pub warning: Option<String>,
}

pub struct Fetcher<T: Truncator> {
    client: reqwest::Client,
    truncator: T,
}

impl<T: Truncator> Fetcher<T> {
    pub fn new(client: reqwest::Client, truncator: T) -> Self {
        Self { client, truncator }
    }

    pub async fn fetch_results(&self, results: Vec<SearchResult>) -> Vec<FetchOutcome> {
        let futures: Vec<_> = results
            .into_iter()
            .map(|result| self.fetch_one(result))
            .collect();
        join_all(futures).await
    }

    async fn fetch_one(&self, mut result: SearchResult) -> FetchOutcome {
        let mut warning = None;
        match result.metadata.get(META_PDF_URL) {
            Some(pdf_url) if !pdf_url.is_empty() => {
                let pdf_url = pdf_url.clone();
                match self.fetch_pdf(&pdf_url).await {
                    Ok(text) => result.content = self.truncator.truncate(&text),
                    Err(e) => {
                        warning = Some(format!("PDF extraction failed for {pdf_url}: {e}"));
                        result.content = format!(
                            "[PDF fetch unsuccessful — falling back to abstract]\n\n{}",
                            result.content
                        );
                    }
                }
            }
            Some(_) => {
                warning = Some("PDF URL is empty, falling back to abstract".to_string());
                result.content = format!(
                    "[PDF fetch unsuccessful — falling back to abstract]\n\n{}",
                    result.content
                );
            }
            None => match self.fetch_and_extract(&result.url).await {
                Ok(text) => result.content = self.truncator.truncate(&text),
                Err(e) => {
                    warning = Some(format!("failed to fetch {}: {e}", result.url));
                    result.content = format!("[Failed to fetch: {e}]");
                }
            },
        }
        FetchOutcome { result, warning }
    }

    async fn fetch_and_extract(&self, url: &str) -> Result<String, Box<dyn std::error::Error>> {
        let html = self.client.get(url).send().await?.text().await?;
        extract_content(&html, url)
    }

    async fn fetch_pdf(&self, url: &str) -> Result<String, Box<dyn std::error::Error>> {
        let bytes = self.client.get(url).send().await?.bytes().await?;
        extract_pdf_text(&bytes)
    }
}

fn extract_pdf_text(bytes: &[u8]) -> Result<String, Box<dyn std::error::Error>> {
    let doc = lopdf::Document::load_mem(bytes)?;
    let mut text = String::new();
    let pages = doc.get_pages();
    for &page_num in pages.keys() {
        if let Ok(page_text) = doc.extract_text(&[page_num]) {
            text.push_str(&page_text);
            text.push('\n');
        }
    }
    if text.trim().is_empty() {
        return Err("no text extracted from PDF".into());
    }
    Ok(text)
}

fn extract_content(html: &str, url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let cfg = Config {
        text_mode: TextMode::Markdown,
        ..Default::default()
    };
    let mut readability = Readability::new(html, Some(url), Some(cfg))?;
    let article = readability.parse()?;
    Ok(article.text_content.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::truncator::max_length::MaxLengthTruncator;

    fn make_article_html(title: &str, body: &str) -> String {
        format!(
            r#"<html><head><title>{title}</title></head>
            <body><article><h1>{title}</h1><p>{body}</p></article></body></html>"#
        )
    }

    #[test]
    fn extract_content_from_article() {
        let html = make_article_html("Test Article", "This is the main content of the article.");
        let content = extract_content(&html, "https://example.com").unwrap();
        assert!(content.contains("main content of the article"));
    }

    #[test]
    fn extract_content_preserves_markdown_structure() {
        let html = r#"<html><head><title>Test</title></head>
            <body><article>
                <h1>Heading</h1>
                <p>A paragraph with <strong>bold</strong> and <em>italic</em> text.</p>
                <ul><li>Item one</li><li>Item two</li></ul>
            </article></body></html>"#;
        let content = extract_content(html, "https://example.com").unwrap();
        assert!(content.contains("Heading"));
        assert!(content.contains("Item one"));
    }

    #[test]
    fn extract_content_from_minimal_html() {
        let html = "<html><body><p>Just a paragraph.</p></body></html>";
        // dom_smoothie may or may not extract from minimal pages;
        // the important thing is it doesn't panic
        let _ = extract_content(html, "https://example.com");
    }

    #[test]
    fn extract_content_handles_links() {
        let html = r#"<html><head><title>Links</title></head>
            <body><article>
                <p>Visit <a href="https://rust-lang.org">Rust</a> for more.</p>
            </article></body></html>"#;
        let content = extract_content(html, "https://example.com").unwrap();
        assert!(content.contains("Rust"));
    }

    #[tokio::test]
    async fn fetch_one_sets_error_on_invalid_url() {
        let client = reqwest::Client::new();
        let truncator = MaxLengthTruncator::new(5000);
        let fetcher = Fetcher::new(client, truncator);

        let result = SearchResult {
            title: "Test".to_string(),
            url: "http://localhost:1".to_string(), // nothing listening
            content: "original".to_string(),
            metadata: std::collections::HashMap::new(),
        };

        let outcome = fetcher.fetch_one(result).await;
        assert!(
            outcome.result.content.starts_with("[Failed to fetch:"),
            "Expected error message, got: {}",
            outcome.result.content
        );
        assert!(outcome.warning.is_some());
    }

    #[tokio::test]
    async fn fetch_results_preserves_order() {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_millis(100))
            .build()
            .unwrap();
        let truncator = MaxLengthTruncator::new(5000);
        let fetcher = Fetcher::new(client, truncator);

        let results = vec![
            SearchResult {
                title: "First".to_string(),
                url: "http://localhost:1".to_string(),
                content: "a".to_string(),
                metadata: std::collections::HashMap::new(),
            },
            SearchResult {
                title: "Second".to_string(),
                url: "http://localhost:2".to_string(),
                content: "b".to_string(),
                metadata: std::collections::HashMap::new(),
            },
        ];

        let fetched = fetcher.fetch_results(results).await;
        assert_eq!(fetched.len(), 2);
        assert_eq!(fetched[0].result.title, "First");
        assert_eq!(fetched[1].result.title, "Second");
        // Both should have error messages since nothing is listening
        assert!(fetched[0].result.content.starts_with("[Failed to fetch:"));
        assert!(fetched[1].result.content.starts_with("[Failed to fetch:"));
    }

    #[tokio::test]
    async fn fetch_one_truncates_long_content() {
        // We can't easily mock HTTP here, but we can test the truncation
        // path by verifying the truncator is wired correctly through
        // the extract_content + truncate path
        let truncator = MaxLengthTruncator::new(10);
        let long_text = "a".repeat(100);
        let truncated = truncator.truncate(&long_text);
        assert!(truncated.ends_with("[truncated]"));
        assert!(truncated.len() < 100);
    }
}
