pub mod arxiv;
pub mod duckduckgo;

use crate::types::SearchResult;
use std::fmt;

#[derive(Debug)]
pub enum SearchError {
    EmptyQuery,
    Http(reqwest::Error),
    Parse(String),
}

impl fmt::Display for SearchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SearchError::EmptyQuery => write!(f, "empty string passed as query"),
            SearchError::Http(e) => write!(f, "{e}"),
            SearchError::Parse(e) => write!(f, "parse error: {e}"),
        }
    }
}

impl From<reqwest::Error> for SearchError {
    fn from(e: reqwest::Error) -> Self {
        SearchError::Http(e)
    }
}

pub trait SearchEngine {
    fn search(&self) -> impl std::future::Future<Output = Result<String, SearchError>> + Send;
    fn parse_results(&self, raw: &str, top_k: usize) -> Result<Vec<SearchResult>, SearchError>;
}
