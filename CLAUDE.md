# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What is fretka?

A CLI tool that searches DuckDuckGo (via its lite/HTML interface) and outputs results as markdown. Designed for use by coding agents that need web search capabilities.

## Commands

- **Build:** `cargo build`
- **Run:** `cargo run -- "search query"` (use `-t N` for top-k results, `-v` for verbose errors)
- **Check:** `cargo check`
- **Lint:** `cargo clippy`
- **Format:** `cargo fmt`
- **Test:** `cargo test` (no tests yet)

## Architecture

The codebase follows a simple pipeline: **search engine → parse HTML → format output**.

- **`src/main.rs`** — CLI entry point using `clap`. Wires together engine and formatter.
- **`src/types.rs`** — Shared `SearchResult` struct (title, url, snippet).
- **`src/engine/`** — Search engine implementations. Currently only `duckduckgo.rs`, which POSTs to `lite.duckduckgo.com` and scrapes results using the `scraper` crate.
- **`src/formatter/`** — Output formatters. Currently only `markdown.rs`.

The engine trait pattern is implicit (not yet a formal trait) — `DuckDuckGoEngine` has `search()` (async, returns raw HTML) and `parse_results()` (sync, returns `Vec<SearchResult>`).

## Key dependencies

- `reqwest` — HTTP client (with `form` feature for POST body encoding)
- `scraper` — HTML parsing and CSS selector queries
- `clap` — CLI argument parsing (derive mode)
- `tokio` — Async runtime

## Notes

- Uses Rust edition 2024
- The DuckDuckGo engine uses the Lynx user-agent string to get the lite HTML version
