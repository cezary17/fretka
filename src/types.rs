use std::collections::HashMap;

pub const META_AUTHORS: &str = "authors";
pub const META_CATEGORIES: &str = "categories";
pub const META_PUBLISHED: &str = "published";
pub const META_UPDATED: &str = "updated";
pub const META_PDF_URL: &str = "pdf_url";
pub const META_DOI: &str = "doi";
pub const META_JOURNAL_REF: &str = "journal_ref";
pub const META_COMMENT: &str = "comment";

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub content: String,
    pub metadata: HashMap<String, String>,
}
