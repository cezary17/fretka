use crate::types::SearchResult;

pub fn format_as_markdown(results: &[SearchResult]) -> String {
    results
        .iter()
        .enumerate()
        .map(|(i, result)| {
            format!(
                "{}. [{}]({})\n\n   {}\n",
                i + 1,
                result.title,
                result.url,
                result.snippet
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}
