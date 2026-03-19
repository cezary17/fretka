use std::collections::HashMap;

use quick_xml::Reader;
use quick_xml::events::Event;

use super::{SearchEngine, SearchError};
use crate::types::{
    META_AUTHORS, META_CATEGORIES, META_COMMENT, META_DOI, META_JOURNAL_REF, META_PDF_URL,
    META_PUBLISHED, META_UPDATED, SearchResult,
};

#[derive(Debug)]
pub struct ArxivEngine {
    query: String,
    client: reqwest::Client,
    sort_by: String,
    max_results: usize,
}

impl ArxivEngine {
    pub fn new(
        query: String,
        client: reqwest::Client,
        sort_by: Option<String>,
        max_results: usize,
    ) -> Result<Self, SearchError> {
        if query.trim().is_empty() {
            return Err(SearchError::EmptyQuery);
        }
        Ok(Self {
            query,
            client,
            sort_by: sort_by.unwrap_or_else(|| "relevance".to_string()),
            max_results,
        })
    }
}

impl SearchEngine for ArxivEngine {
    async fn search(&self) -> Result<String, SearchError> {
        let url = format!(
            "https://export.arxiv.org/api/query?search_query={}&sortBy={}&sortOrder=descending&max_results={}",
            urlencoding::encode(&self.query),
            urlencoding::encode(&self.sort_by),
            self.max_results,
        );
        Ok(self.client.get(&url).send().await?.text().await?)
    }

    fn parse_results(&self, raw: &str, top_k: usize) -> Result<Vec<SearchResult>, SearchError> {
        let mut reader = Reader::from_str(raw);
        let mut results = Vec::new();

        let mut in_entry = false;
        let mut in_author = false;
        let mut current_tag = String::new();
        let mut title = String::new();
        let mut summary = String::new();
        let mut url = String::new();
        let mut published = String::new();
        let mut updated = String::new();
        let mut pdf_url = String::new();
        let mut doi = String::new();
        let mut journal_ref = String::new();
        let mut comment = String::new();
        let mut authors: Vec<String> = Vec::new();
        let mut categories: Vec<String> = Vec::new();

        loop {
            match reader.read_event() {
                Ok(Event::Start(ref e)) => {
                    let local_name = String::from_utf8_lossy(e.local_name().as_ref()).to_string();
                    match local_name.as_str() {
                        "entry" => {
                            in_entry = true;
                            title.clear();
                            summary.clear();
                            url.clear();
                            published.clear();
                            updated.clear();
                            pdf_url.clear();
                            doi.clear();
                            journal_ref.clear();
                            comment.clear();
                            authors.clear();
                            categories.clear();
                        }
                        "author" if in_entry => {
                            in_author = true;
                        }
                        "link" if in_entry => {
                            let mut rel = String::new();
                            let mut href = String::new();
                            let mut link_title = String::new();
                            for attr in e.attributes().flatten() {
                                match attr.key.local_name().as_ref() {
                                    b"rel" => {
                                        rel = String::from_utf8_lossy(&attr.value).to_string()
                                    }
                                    b"href" => {
                                        href = String::from_utf8_lossy(&attr.value).to_string()
                                    }
                                    b"title" => {
                                        link_title =
                                            String::from_utf8_lossy(&attr.value).to_string()
                                    }
                                    _ => {}
                                }
                            }
                            if rel == "alternate" {
                                url = href;
                            } else if link_title == "pdf" {
                                pdf_url = href;
                            }
                        }
                        "category" if in_entry => {
                            for attr in e.attributes().flatten() {
                                if attr.key.local_name().as_ref() == b"term" {
                                    categories
                                        .push(String::from_utf8_lossy(&attr.value).to_string());
                                }
                            }
                        }
                        _ if in_entry => {
                            current_tag = local_name;
                        }
                        _ => {}
                    }
                }
                Ok(Event::Text(ref e)) => {
                    if in_entry {
                        let text = e.unescape().unwrap_or_default().trim().to_string();
                        if !text.is_empty() {
                            match current_tag.as_str() {
                                "title" if !in_author => title = text,
                                "summary" => summary = text,
                                "name" if in_author => authors.push(text),
                                "published" => published = text,
                                "updated" => updated = text,
                                "doi" => doi = text,
                                "journal_ref" => journal_ref = text,
                                "comment" => comment = text,
                                _ => {}
                            }
                        }
                    }
                }
                Ok(Event::End(ref e)) => {
                    let local_name = String::from_utf8_lossy(e.local_name().as_ref()).to_string();
                    match local_name.as_str() {
                        "entry" => {
                            let mut metadata = HashMap::new();
                            if !authors.is_empty() {
                                metadata.insert(META_AUTHORS.to_string(), authors.join(", "));
                            }
                            if !categories.is_empty() {
                                metadata.insert(META_CATEGORIES.to_string(), categories.join(", "));
                            }
                            if !published.is_empty() {
                                metadata.insert(META_PUBLISHED.to_string(), published.clone());
                            }
                            if !updated.is_empty() {
                                metadata.insert(META_UPDATED.to_string(), updated.clone());
                            }
                            if !pdf_url.is_empty() {
                                metadata.insert(META_PDF_URL.to_string(), pdf_url.clone());
                            }
                            if !doi.is_empty() {
                                metadata.insert(META_DOI.to_string(), doi.clone());
                            }
                            if !journal_ref.is_empty() {
                                metadata.insert(META_JOURNAL_REF.to_string(), journal_ref.clone());
                            }
                            if !comment.is_empty() {
                                metadata.insert(META_COMMENT.to_string(), comment.clone());
                            }

                            results.push(SearchResult {
                                title: title.clone(),
                                url: url.clone(),
                                content: summary.clone(),
                                metadata,
                            });

                            in_entry = false;
                            if results.len() >= top_k {
                                break;
                            }
                        }
                        "author" => {
                            in_author = false;
                        }
                        _ => {
                            current_tag.clear();
                        }
                    }
                }
                Ok(Event::Empty(ref e)) if in_entry => {
                    let local_name = String::from_utf8_lossy(e.local_name().as_ref()).to_string();
                    match local_name.as_str() {
                        "link" => {
                            let mut rel = String::new();
                            let mut href = String::new();
                            let mut link_title = String::new();
                            for attr in e.attributes().flatten() {
                                match attr.key.local_name().as_ref() {
                                    b"rel" => {
                                        rel = String::from_utf8_lossy(&attr.value).to_string()
                                    }
                                    b"href" => {
                                        href = String::from_utf8_lossy(&attr.value).to_string()
                                    }
                                    b"title" => {
                                        link_title =
                                            String::from_utf8_lossy(&attr.value).to_string()
                                    }
                                    _ => {}
                                }
                            }
                            if rel == "alternate" {
                                url = href;
                            } else if link_title == "pdf" {
                                pdf_url = href;
                            }
                        }
                        "category" => {
                            for attr in e.attributes().flatten() {
                                if attr.key.local_name().as_ref() == b"term" {
                                    categories
                                        .push(String::from_utf8_lossy(&attr.value).to_string());
                                }
                            }
                        }
                        "primary_category" => {
                            // Ignored — categories already collected from <category>
                        }
                        _ => {}
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(SearchError::Parse(e.to_string())),
                _ => {}
            }
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn engine() -> ArxivEngine {
        let client = reqwest::Client::new();
        ArxivEngine::new("test".to_string(), client, None, 10).unwrap()
    }

    fn sample_atom_feed() -> &'static str {
        r#"<?xml version='1.0' encoding='UTF-8'?>
        <feed xmlns="http://www.w3.org/2005/Atom"
              xmlns:opensearch="http://a9.com/-/spec/opensearch/1.1/"
              xmlns:arxiv="http://arxiv.org/schemas/atom">
          <entry>
            <id>http://arxiv.org/abs/2301.00001v1</id>
            <title>Test Paper on Machine Learning</title>
            <summary>This paper presents a novel approach to machine learning.</summary>
            <published>2023-01-01T00:00:00Z</published>
            <updated>2023-01-02T00:00:00Z</updated>
            <link href="https://arxiv.org/abs/2301.00001v1" rel="alternate" type="text/html"/>
            <link href="https://arxiv.org/pdf/2301.00001v1" rel="related" type="application/pdf" title="pdf"/>
            <author><name>Alice Smith</name></author>
            <author><name>Bob Jones</name></author>
            <category term="cs.LG" scheme="http://arxiv.org/schemas/atom"/>
            <category term="cs.AI" scheme="http://arxiv.org/schemas/atom"/>
            <arxiv:primary_category term="cs.LG"/>
            <arxiv:comment>10 pages, 5 figures</arxiv:comment>
            <arxiv:journal_ref>Nature 2023</arxiv:journal_ref>
            <arxiv:doi>10.1234/example</arxiv:doi>
          </entry>
        </feed>"#
    }

    #[test]
    fn parse_single_entry() {
        let engine = engine();
        let results = engine.parse_results(sample_atom_feed(), 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Test Paper on Machine Learning");
        assert_eq!(results[0].url, "https://arxiv.org/abs/2301.00001v1");
        assert!(results[0].content.contains("novel approach"));
    }

    #[test]
    fn parse_metadata_fields() {
        let engine = engine();
        let results = engine.parse_results(sample_atom_feed(), 10).unwrap();
        let meta = &results[0].metadata;
        assert_eq!(meta.get(META_AUTHORS).unwrap(), "Alice Smith, Bob Jones");
        assert_eq!(meta.get(META_CATEGORIES).unwrap(), "cs.LG, cs.AI");
        assert_eq!(meta.get(META_PUBLISHED).unwrap(), "2023-01-01T00:00:00Z");
        assert_eq!(meta.get(META_UPDATED).unwrap(), "2023-01-02T00:00:00Z");
        assert_eq!(
            meta.get(META_PDF_URL).unwrap(),
            "https://arxiv.org/pdf/2301.00001v1"
        );
        assert_eq!(meta.get(META_DOI).unwrap(), "10.1234/example");
        assert_eq!(meta.get(META_JOURNAL_REF).unwrap(), "Nature 2023");
        assert_eq!(meta.get(META_COMMENT).unwrap(), "10 pages, 5 figures");
    }

    #[test]
    fn parse_respects_top_k() {
        let feed = r#"<?xml version='1.0' encoding='UTF-8'?>
        <feed xmlns="http://www.w3.org/2005/Atom">
          <entry>
            <id>http://arxiv.org/abs/1</id>
            <title>First</title>
            <summary>First abstract</summary>
            <link href="https://arxiv.org/abs/1" rel="alternate" type="text/html"/>
          </entry>
          <entry>
            <id>http://arxiv.org/abs/2</id>
            <title>Second</title>
            <summary>Second abstract</summary>
            <link href="https://arxiv.org/abs/2" rel="alternate" type="text/html"/>
          </entry>
          <entry>
            <id>http://arxiv.org/abs/3</id>
            <title>Third</title>
            <summary>Third abstract</summary>
            <link href="https://arxiv.org/abs/3" rel="alternate" type="text/html"/>
          </entry>
        </feed>"#;
        let engine = engine();
        let results = engine.parse_results(feed, 2).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].title, "First");
        assert_eq!(results[1].title, "Second");
    }

    #[test]
    fn parse_empty_feed() {
        let feed = r#"<?xml version='1.0' encoding='UTF-8'?>
        <feed xmlns="http://www.w3.org/2005/Atom">
        </feed>"#;
        let engine = engine();
        let results = engine.parse_results(feed, 10).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn parse_missing_optional_fields() {
        let feed = r#"<?xml version='1.0' encoding='UTF-8'?>
        <feed xmlns="http://www.w3.org/2005/Atom">
          <entry>
            <id>http://arxiv.org/abs/1</id>
            <title>Minimal Paper</title>
            <summary>Just an abstract</summary>
            <link href="https://arxiv.org/abs/1" rel="alternate" type="text/html"/>
          </entry>
        </feed>"#;
        let engine = engine();
        let results = engine.parse_results(feed, 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Minimal Paper");
        assert!(results[0].metadata.get(META_DOI).is_none());
        assert!(results[0].metadata.get(META_JOURNAL_REF).is_none());
        assert!(results[0].metadata.get(META_PDF_URL).is_none());
    }

    #[test]
    fn empty_query_returns_error() {
        let client = reqwest::Client::new();
        let err = ArxivEngine::new("".to_string(), client, None, 10).unwrap_err();
        assert_eq!(err.to_string(), "empty string passed as query");
    }

    #[test]
    fn whitespace_only_query_returns_error() {
        let client = reqwest::Client::new();
        let err = ArxivEngine::new("   ".to_string(), client, None, 10).unwrap_err();
        assert_eq!(err.to_string(), "empty string passed as query");
    }
}
