use std::time::Duration;

use scraper::{Html, Selector};

use crate::types::SearchResult;

pub struct DuckDuckGoEngine {
    query: String,
}

impl DuckDuckGoEngine {
    pub fn new(query: String) -> Self {
        Self { query }
    }

    pub async fn search(&self) -> Result<String, reqwest::Error> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()?;
        client
            .post("https://lite.duckduckgo.com/lite/")
            .header("User-Agent", "Lynx/2.8.9rel.1")
            .form(&[("q", &self.query)])
            .send()
            .await?
            .text()
            .await
    }

    pub fn parse_results(&self, html: &str, top_k: usize) -> Vec<SearchResult> {
        let document = Html::parse_document(html);
        let link_selector = Selector::parse("a.result-link").unwrap();
        let snippet_selector = Selector::parse("td.result-snippet").unwrap();

        let titles: Vec<_> = document.select(&link_selector).collect();
        let snippets: Vec<_> = document.select(&snippet_selector).collect();

        titles
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
            .collect()
    }
}
