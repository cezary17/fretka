mod engine;
mod formatter;
mod types;

use clap::Parser;
use engine::duckduckgo::DuckDuckGoEngine;
use formatter::markdown::format_as_markdown;

#[derive(Parser)]
#[command(name = "fretka", about = "Search DuckDuckGo and extract text")]
struct Cli {
    /// Search query: (ex: fretka "rust lang docs")
    query: String,

    /// Number of top results to display
    #[arg(short, long, default_value_t = 5)]
    top_k: usize,

    /// Show detailed error messages
    #[arg(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let engine = DuckDuckGoEngine::new(cli.query);
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
    let results = engine.parse_results(&html, cli.top_k);

    print!("{}", format_as_markdown(&results));
}
