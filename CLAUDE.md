# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What is fretka?

A CLI tool that searches the web (DuckDuckGo or arXiv) and outputs results as markdown or JSON. Designed for use by coding agents that need web search capabilities.

## Commands

- **Build:** `cargo build`
- **Run:** `cargo run -- "search query"` (use `-t N` for top-k results, `--engine arxiv` for arXiv, `--fetch` to extract page content, `-v` for verbose errors)
- **Check:** `cargo check`
- **Lint:** `cargo clippy`
- **Format:** `cargo fmt`
- **Test:** `cargo test`

## Architecture

The codebase follows a pipeline: **search engine → parse response → fetch & extract (optional) → truncate → format output**.

- **`src/main.rs`** — CLI entry point using `clap`. Builds a shared `reqwest::Client` and dispatches to the selected engine, then wires together fetcher and formatter.
- **`src/types.rs`** — Shared `SearchResult` struct (title, url, content, metadata). Metadata is a `HashMap<String, String>` with key constants for well-known fields (authors, categories, etc.).
- **`src/engine/mod.rs`** — `SearchEngine` trait with `search()` (async, returns raw response) and `parse_results()` (sync, returns `Vec<SearchResult>`). Also defines shared `SearchError` enum.
- **`src/engine/duckduckgo.rs`** — DuckDuckGo engine. POSTs to `lite.duckduckgo.com` and scrapes HTML results using the `scraper` crate.
- **`src/engine/arxiv.rs`** — arXiv engine. GETs from `export.arxiv.org/api/query` and parses Atom XML using `quick-xml`. Populates metadata with authors, categories, dates, PDF URL, DOI, etc.
- **`src/fetcher/`** — Concurrent page fetcher. For DuckDuckGo results, fetches HTML and extracts readable content via `dom_smoothie`. For arXiv results (when `pdf_url` metadata is present), downloads and extracts PDF text via `lopdf`.
- **`src/truncator/`** — `Truncator` trait with `MaxLengthTruncator` (default 5000 chars). Applied to fetched content.
- **`src/formatter/`** — Output formatters (`markdown.rs`, `json.rs`). Both render metadata when present.

A shared `reqwest::Client` (with user-agent and timeout) is created in `main.rs` and cloned into both the engine and fetcher.

## Key dependencies

- `reqwest` — HTTP client (with `form` feature for POST body encoding)
- `scraper` — HTML parsing and CSS selector queries
- `clap` — CLI argument parsing (derive mode)
- `tokio` — Async runtime
- `dom_smoothie` — Readability-based content extraction (Mozilla Readability.js port)
- `futures` — `join_all` for concurrent fetches
- `quick-xml` — XML parsing for arXiv Atom API responses
- `lopdf` — PDF text extraction for arXiv paper fetching
- `urlencoding` — URL-encoding for arXiv query parameters

## Notes

- Uses Rust edition 2024
- The DuckDuckGo engine uses the Lynx user-agent string to get the lite HTML version
- arXiv API rate limit: 1 request per 3 seconds (satisfied by single-request-per-invocation model)
