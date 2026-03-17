# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What is fretka?

A CLI tool that searches DuckDuckGo (via its lite/HTML interface) and outputs results as markdown. Designed for use by coding agents that need web search capabilities.

## Commands

- **Build:** `cargo build`
- **Run:** `cargo run -- "search query"` (use `-t N` for top-k results, `--fetch` to extract page content, `-v` for verbose errors)
- **Check:** `cargo check`
- **Lint:** `cargo clippy`
- **Format:** `cargo fmt`
- **Test:** `cargo test`

## Architecture

The codebase follows a pipeline: **search engine → parse HTML → fetch & extract (optional) → truncate → format output**.

- **`src/main.rs`** — CLI entry point using `clap`. Builds a shared `reqwest::Client` and wires together engine, fetcher, and formatter.
- **`src/types.rs`** — Shared `SearchResult` struct (title, url, content).
- **`src/engine/`** — Search engine implementations. Currently only `duckduckgo.rs`, which POSTs to `lite.duckduckgo.com` and scrapes results using the `scraper` crate.
- **`src/fetcher/`** — Concurrent page fetcher. Uses `reqwest` to GET result URLs in parallel and `dom_smoothie` to extract readable content as markdown.
- **`src/truncator/`** — `Truncator` trait with `MaxLengthTruncator` (default 5000 chars). Applied to fetched content.
- **`src/formatter/`** — Output formatters (`markdown.rs`, `json.rs`).

The engine trait pattern is implicit (not yet a formal trait) — `DuckDuckGoEngine` has `search()` (async, returns raw HTML) and `parse_results()` (sync, returns `Vec<SearchResult>`). A shared `reqwest::Client` (with user-agent and timeout) is created in `main.rs` and cloned into both the engine and fetcher.

## Key dependencies

- `reqwest` — HTTP client (with `form` feature for POST body encoding)
- `scraper` — HTML parsing and CSS selector queries
- `clap` — CLI argument parsing (derive mode)
- `tokio` — Async runtime
- `dom_smoothie` — Readability-based content extraction (Mozilla Readability.js port)
- `futures` — `join_all` for concurrent fetches

## Notes

- Uses Rust edition 2024
- The DuckDuckGo engine uses the Lynx user-agent string to get the lite HTML version
