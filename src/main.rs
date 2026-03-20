mod engine;
mod fetcher;
mod formatter;
mod installer;
mod truncator;
mod types;

use std::time::Duration;

use clap::{Parser, Subcommand, ValueEnum};
use engine::brave::BraveEngine;
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

#[derive(Clone, ValueEnum)]
enum SearchEngine {
    DuckDuckGo,
    Brave,
}

#[derive(Subcommand)]
enum Commands {
    /// Install fretka skill into coding tool(s)
    InstallSkill,
}

#[derive(Parser)]
#[command(name = "fretka", about = "Search DuckDuckGo and extract text")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Search query: (ex: fretka "rust lang docs")
    query: Option<String>,

    /// Number of top results to display (1-100)
    #[arg(short, long, default_value_t = 5, value_parser = clap::value_parser!(u64).range(1..=100))]
    top_k: u64,

    /// Output format: markdown or json
    #[arg(short, long, value_enum, default_value_t = OutputFormat::Markdown)]
    format: OutputFormat,

    /// Search engine to use: duckduckgo or brave
    #[arg(short, long, value_enum, default_value_t = SearchEngine::DuckDuckGo)]
    engine: SearchEngine,

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

    if let Some(Commands::InstallSkill) = cli.command {
        installer::run();
        return;
    }

    let query = match cli.query {
        Some(q) => q,
        None => {
            eprintln!("error: a search query is required (ex: fretka \"rust lang docs\")");
            std::process::exit(1);
        }
    };

    let client = match build_client() {
        Ok(client) => client,
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    };

    let mut results = match cli.engine {
        SearchEngine::DuckDuckGo => {
            let engine = match DuckDuckGoEngine::new(query, client.clone()) {
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
            match engine.parse_results(&html, cli.top_k as usize) {
                Ok(results) => results,
                Err(e) => {
                    eprintln!("error: {e}");
                    std::process::exit(1);
                }
            }
        }
        SearchEngine::Brave => {
            let engine = match BraveEngine::new(query, client.clone()) {
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
            match engine.parse_results(&html, cli.top_k as usize) {
                Ok(results) => results,
                Err(e) => {
                    eprintln!("error: {e}");
                    std::process::exit(1);
                }
            }
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
