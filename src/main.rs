mod engine;
mod fetcher;
mod formatter;
mod truncator;
mod types;

use std::time::Duration;

use clap::{Parser, ValueEnum};
use engine::duckduckgo::DuckDuckGoEngine;
use fetcher::Fetcher;
use formatter::json::format_as_json;
use formatter::markdown::format_as_markdown;
use truncator::max_length::MaxLengthTruncator;

#[derive(Clone, ValueEnum)]
enum OutputFormat {
    Markdown,
    Json,
}

#[derive(Parser)]
#[command(name = "fretka", about = "Search DuckDuckGo and extract text")]
struct Cli {
    /// Search query: (ex: fretka "rust lang docs")
    query: String,

    /// Number of top results to display
    #[arg(short, long, default_value_t = 5)]
    top_k: usize,

    /// Output format: markdown or json
    #[arg(short, long, value_enum, default_value_t = OutputFormat::Markdown)]
    format: OutputFormat,

    /// Fetch and extract content from result URLs
    #[arg(long)]
    fetch: bool,

    /// Show detailed error messages
    #[arg(short, long)]
    verbose: bool,
}

fn build_client() -> Result<reqwest::Client, reqwest::Error> {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .user_agent("Lynx/2.8.9rel.1")
        .build()
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let client = match build_client() {
        Ok(client) => client,
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    };

    let engine = match DuckDuckGoEngine::new(cli.query, client.clone()) {
        Ok(engine) => engine,
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    };
    let html = match engine.search().await {
        Ok(html) => html,
        Err(e) => {
            if cli.verbose {
                eprintln!("search failed: {e}");
            } else {
                eprintln!("search failed");
            }
            std::process::exit(1);
        }
    };
    let mut results = match engine.parse_results(&html, cli.top_k) {
        Ok(results) => results,
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    };

    if results.is_empty() {
        eprintln!("no results found for query");
        std::process::exit(1);
    }

    if cli.fetch {
        let truncator = MaxLengthTruncator::new(5000);
        let fetcher = Fetcher::new(client, truncator);
        results = fetcher.fetch_results(results).await;
    }

    let output = match cli.format {
        OutputFormat::Json => format_as_json(&results),
        OutputFormat::Markdown => format_as_markdown(&results),
    };
    print!("{output}");
}
